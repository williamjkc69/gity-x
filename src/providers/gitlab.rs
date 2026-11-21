use serde_json::{json, Value};

use super::{
    format_http_error, normalize_iso_for_display, url_encode_component, CommentKind, MergeStrategy,
    PrComment, PrCommit, PrFileChange, PrProvider, PrState, PrStateFilter, PullRequestDetail,
    PullRequestSummary, RemoteSpec,
};

pub struct GitLabProvider {
    spec: RemoteSpec,
    token: Option<String>,
}

impl GitLabProvider {
    pub fn new(spec: RemoteSpec, token: Option<String>) -> Self {
        Self { spec, token }
    }

    fn project_encoded(&self) -> String {
        url_encode_component(&self.spec.full_path)
    }

    fn api_url(&self, path: &str) -> String {
        format!("https://{}/api/v4{path}", self.spec.host)
    }

    fn req(&self, method: &str, path: &str) -> (String, ureq::Request) {
        let url = self.api_url(path);
        let mut r = ureq::request(method, &url)
            .timeout(std::time::Duration::from_secs(20))
            .set("User-Agent", "gity-x");
        if let Some(tok) = &self.token {
            r = r.set("PRIVATE-TOKEN", tok);
        }
        (url, r)
    }

    fn get_json(&self, path: &str) -> Result<Value, String> {
        let (url, r) = self.req("GET", path);
        let resp = r
            .call()
            .map_err(|e| format_http_error("GitLab", &url, self.token.is_some(), e))?;
        resp.into_json::<Value>()
            .map_err(|e| format!("GitLab JSON parse error: {e}"))
    }

    fn send_json(&self, method: &str, path: &str, payload: Value) -> Result<Value, String> {
        let (url, r) = self.req(method, path);
        let resp = r
            .send_json(payload)
            .map_err(|e| format_http_error("GitLab", &url, self.token.is_some(), e))?;
        resp.into_json::<Value>().or_else(|_| Ok(Value::Null))
    }

    fn parse_summary(v: &Value) -> PullRequestSummary {
        let is_draft = v.get("draft").and_then(Value::as_bool).unwrap_or(false)
            || v.get("work_in_progress")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        let state_str = v.get("state").and_then(Value::as_str).unwrap_or("opened");
        let state = match state_str {
            "merged" => PrState::Merged,
            "closed" => PrState::Closed,
            _ if is_draft => PrState::Draft,
            _ => PrState::Open,
        };

        let merge_status = v
            .get("detailed_merge_status")
            .or_else(|| v.get("merge_status"))
            .and_then(Value::as_str);
        let mergeable = merge_status.map(|s| s == "mergeable" || s == "can_be_merged");

        PullRequestSummary {
            number: v.get("iid").and_then(Value::as_u64).unwrap_or(0),
            title: v
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            body: v
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            state,
            author: v
                .pointer("/author/username")
                .or_else(|| v.pointer("/author/name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            source_branch: v
                .get("source_branch")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            target_branch: v
                .get("target_branch")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            created_at: normalize_iso_for_display(
                v.get("created_at").and_then(Value::as_str).unwrap_or(""),
            ),
            updated_at: normalize_iso_for_display(
                v.get("updated_at").and_then(Value::as_str).unwrap_or(""),
            ),
            html_url: v
                .get("web_url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            additions: None,
            deletions: None,
            changed_files: v
                .get("changes_count")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<usize>().ok()),
            comments_count: v
                .get("user_notes_count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize,
            mergeable,
        }
    }
}

impl PrProvider for GitLabProvider {
    fn list_prs(&self, filter: PrStateFilter) -> Result<Vec<PullRequestSummary>, String> {
        let state_param = match filter {
            PrStateFilter::Open => "opened",
            PrStateFilter::Merged => "merged",
            PrStateFilter::Closed => "closed",
            PrStateFilter::All => "all",
        };
        let path = format!(
            "/projects/{}/merge_requests?state={state_param}&per_page=50&order_by=updated_at&sort=desc",
            self.project_encoded()
        );
        let data = self.get_json(&path)?;
        let Some(arr) = data.as_array() else {
            return Ok(Vec::new());
        };
        Ok(arr.iter().map(Self::parse_summary).collect())
    }

    fn get_pr_detail(&self, number: u64) -> Result<PullRequestDetail, String> {
        let proj = self.project_encoded();
        let mr_json = self.get_json(&format!("/projects/{proj}/merge_requests/{number}"))?;
        let mut summary = Self::parse_summary(&mr_json);

        // Commits
        let commits_json = self
            .get_json(&format!(
                "/projects/{proj}/merge_requests/{number}/commits?per_page=100"
            ))
            .unwrap_or(Value::Array(Vec::new()));
        let commits = commits_json
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|c| {
                        let hash = c.get("id").and_then(Value::as_str).unwrap_or("").to_string();
                        let short = c
                            .get("short_id")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                            .unwrap_or_else(|| hash[..hash.len().min(7)].to_string());
                        PrCommit {
                            hash,
                            short_hash: short,
                            message: c
                                .get("title")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                            author: c
                                .get("author_name")
                                .and_then(Value::as_str)
                                .unwrap_or("unknown")
                                .to_string(),
                            date: normalize_iso_for_display(
                                c.get("created_at").and_then(Value::as_str).unwrap_or(""),
                            ),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // File diffs
        let diffs_json = self
            .get_json(&format!(
                "/projects/{proj}/merge_requests/{number}/diffs?per_page=100"
            ))
            .unwrap_or(Value::Array(Vec::new()));
        let mut total_add = 0usize;
        let mut total_del = 0usize;
        let files = diffs_json
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|d| {
                        let status = if d.get("new_file").and_then(Value::as_bool).unwrap_or(false)
                        {
                            "A"
                        } else if d
                            .get("deleted_file")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                        {
                            "D"
                        } else if d
                            .get("renamed_file")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                        {
                            "R"
                        } else {
                            "M"
                        }
                        .to_string();
                        let patch = d
                            .get("diff")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let (add, del) = count_patch_lines(&patch);
                        total_add += add;
                        total_del += del;
                        PrFileChange {
                            path: d
                                .get("new_path")
                                .or_else(|| d.get("old_path"))
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                            status,
                            additions: add,
                            deletions: del,
                            patch,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        summary.additions = Some(total_add);
        summary.deletions = Some(total_del);
        summary.changed_files = Some(files.len());

        // Notes + approvals
        let mut comments = Vec::new();
        if let Ok(Value::Array(notes)) = self.get_json(&format!(
            "/projects/{proj}/merge_requests/{number}/notes?per_page=100&sort=asc"
        )) {
            for n in notes {
                let is_system = n.get("system").and_then(Value::as_bool).unwrap_or(false);
                let body = n.get("body").and_then(Value::as_str).unwrap_or("").trim();
                let kind = if is_system {
                    if body.to_lowercase().contains("approved this merge request") {
                        CommentKind::Approved
                    } else {
                        continue;
                    }
                } else {
                    CommentKind::Comment
                };
                comments.push(PrComment {
                    author: n
                        .pointer("/author/username")
                        .or_else(|| n.pointer("/author/name"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string(),
                    body: body.to_string(),
                    created_at: normalize_iso_for_display(
                        n.get("created_at").and_then(Value::as_str).unwrap_or(""),
                    ),
                    kind,
                });
            }
        }

        Ok(PullRequestDetail {
            summary,
            commits,
            files,
            comments,
        })
    }

    fn approve_pr(&self, number: u64) -> Result<String, String> {
        let path = format!(
            "/projects/{}/merge_requests/{number}/approve",
            self.project_encoded()
        );
        self.send_json("POST", &path, json!({}))?;
        Ok(format!("MR !{number} approved."))
    }

    fn request_changes(&self, number: u64, comment: &str) -> Result<String, String> {
        let proj = self.project_encoded();
        // Unapprove if previously approved (ignore error if user hadn't approved)
        let _ = self.send_json(
            "POST",
            &format!("/projects/{proj}/merge_requests/{number}/unapprove"),
            json!({}),
        );
        let body = if comment.trim().is_empty() {
            "**Changes requested** via gity x.".to_string()
        } else {
            format!("**Changes requested:**\n\n{}", comment.trim())
        };
        self.send_json(
            "POST",
            &format!("/projects/{proj}/merge_requests/{number}/notes"),
            json!({ "body": body }),
        )?;
        Ok(format!("Changes requested on MR !{number}."))
    }

    fn add_comment(&self, number: u64, body: &str) -> Result<String, String> {
        if body.trim().is_empty() {
            return Err("Comment cannot be empty.".to_string());
        }
        let path = format!(
            "/projects/{}/merge_requests/{number}/notes",
            self.project_encoded()
        );
        self.send_json("POST", &path, json!({ "body": body.trim() }))?;
        Ok(format!("Comment posted on MR !{number}."))
    }

    fn merge_pr(&self, number: u64, strategy: MergeStrategy) -> Result<String, String> {
        let proj = self.project_encoded();
        match strategy {
            MergeStrategy::Merge => {
                self.send_json(
                    "PUT",
                    &format!("/projects/{proj}/merge_requests/{number}/merge"),
                    json!({ "squash": false }),
                )?;
                Ok(format!("MR !{number} merged."))
            }
            MergeStrategy::Squash => {
                self.send_json(
                    "PUT",
                    &format!("/projects/{proj}/merge_requests/{number}/merge"),
                    json!({ "squash": true }),
                )?;
                Ok(format!("MR !{number} squashed and merged."))
            }
            MergeStrategy::Rebase => {
                // Rebase MR onto target branch first (async on GitLab), wait briefly, then merge
                self.send_json(
                    "PUT",
                    &format!("/projects/{proj}/merge_requests/{number}/rebase"),
                    json!({}),
                )?;
                for _ in 0..8 {
                    std::thread::sleep(std::time::Duration::from_millis(600));
                    if let Ok(mr) = self.get_json(&format!(
                        "/projects/{proj}/merge_requests/{number}?include_rebase_in_progress=true"
                    )) {
                        let rebasing = mr
                            .get("rebase_in_progress")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        if !rebasing {
                            break;
                        }
                    }
                }
                self.send_json(
                    "PUT",
                    &format!("/projects/{proj}/merge_requests/{number}/merge"),
                    json!({ "squash": false }),
                )?;
                Ok(format!("MR !{number} rebased and merged."))
            }
        }
    }

    fn close_or_decline_pr(&self, number: u64) -> Result<String, String> {
        let path = format!(
            "/projects/{}/merge_requests/{number}",
            self.project_encoded()
        );
        self.send_json("PUT", &path, json!({ "state_event": "close" }))?;
        Ok(format!("MR !{number} closed."))
    }

    fn reopen_pr(&self, number: u64) -> Result<String, String> {
        let path = format!(
            "/projects/{}/merge_requests/{number}",
            self.project_encoded()
        );
        self.send_json("PUT", &path, json!({ "state_event": "reopen" }))?;
        Ok(format!("MR !{number} reopened."))
    }
}

fn count_patch_lines(patch: &str) -> (usize, usize) {
    let mut add = 0usize;
    let mut del = 0usize;
    for line in patch.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            add += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            del += 1;
        }
    }
    (add, del)
}
