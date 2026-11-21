use std::collections::HashMap;

use serde_json::{json, Value};

use super::{
    basic_auth_header, format_http_error, normalize_iso_for_display, CommentKind, MergeStrategy,
    PrComment, PrCommit, PrFileChange, PrProvider, PrState, PrStateFilter, PullRequestDetail,
    PullRequestSummary, RemoteSpec,
};

pub struct BitbucketProvider {
    spec: RemoteSpec,
    /// `(username, app_password_or_token)` — if username is empty, uses Bearer auth.
    creds: Option<(String, String)>,
}

impl BitbucketProvider {
    pub fn new(spec: RemoteSpec, creds: Option<(String, String)>) -> Self {
        Self { spec, creds }
    }

    fn api_url(&self, path: &str) -> String {
        format!("https://api.bitbucket.org/2.0{path}")
    }

    fn req(&self, method: &str, path: &str) -> (String, ureq::Request) {
        let url = self.api_url(path);
        let mut r = ureq::request(method, &url)
            .timeout(std::time::Duration::from_secs(20))
            .set("User-Agent", "gity-x");
        if let Some((user, token)) = &self.creds {
            // Bitbucket Repo/Workspace Access Tokens (ATCTT3x...) always require Bearer auth
            if user.is_empty() || token.starts_with("ATCTT3x") {
                r = r.set("Authorization", &format!("Bearer {token}"));
            } else {
                r = r.set("Authorization", &basic_auth_header(user, token));
            }
        }
        (url, r)
    }

    fn get_json(&self, path: &str) -> Result<Value, String> {
        let (url, r) = self.req("GET", path);
        let resp = r
            .call()
            .map_err(|e| format_http_error("Bitbucket", &url, self.creds.is_some(), e))?;
        resp.into_json::<Value>()
            .map_err(|e| format!("Bitbucket JSON parse error: {e}"))
    }

    fn get_text(&self, path: &str) -> Result<String, String> {
        let (url, r) = self.req("GET", path);
        let resp = r
            .call()
            .map_err(|e| format_http_error("Bitbucket", &url, self.creds.is_some(), e))?;
        resp.into_string()
            .map_err(|e| format!("Bitbucket text read error: {e}"))
    }

    fn send_json(&self, method: &str, path: &str, payload: Value) -> Result<Value, String> {
        let (url, r) = self.req(method, path);
        let resp = r
            .send_json(payload)
            .map_err(|e| format_http_error("Bitbucket", &url, self.creds.is_some(), e))?;
        resp.into_json::<Value>().or_else(|_| Ok(Value::Null))
    }

    fn parse_summary(v: &Value) -> PullRequestSummary {
        let state_str = v.get("state").and_then(Value::as_str).unwrap_or("OPEN");
        let is_draft = v.get("draft").and_then(Value::as_bool).unwrap_or(false);
        let state = match state_str {
            "MERGED" => PrState::Merged,
            "DECLINED" | "SUPERSEDED" => PrState::Closed,
            _ if is_draft => PrState::Draft,
            _ => PrState::Open,
        };

        PullRequestSummary {
            number: v.get("id").and_then(Value::as_u64).unwrap_or(0),
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
                .pointer("/author/display_name")
                .or_else(|| v.pointer("/author/nickname"))
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            source_branch: v
                .pointer("/source/branch/name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            target_branch: v
                .pointer("/destination/branch/name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            created_at: normalize_iso_for_display(
                v.get("created_on").and_then(Value::as_str).unwrap_or(""),
            ),
            updated_at: normalize_iso_for_display(
                v.get("updated_on").and_then(Value::as_str).unwrap_or(""),
            ),
            html_url: v
                .pointer("/links/html/href")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            additions: None,
            deletions: None,
            changed_files: None,
            comments_count: v.get("comment_count").and_then(Value::as_u64).unwrap_or(0) as usize,
            mergeable: None,
        }
    }
}

impl PrProvider for BitbucketProvider {
    fn list_prs(&self, filter: PrStateFilter) -> Result<Vec<PullRequestSummary>, String> {
        let query = match filter {
            PrStateFilter::Open => "state=OPEN",
            PrStateFilter::Merged => "state=MERGED",
            PrStateFilter::Closed => "state=DECLINED",
            PrStateFilter::All => "state=OPEN&state=MERGED&state=DECLINED",
        };
        let path = format!(
            "/repositories/{}/{}/pullrequests?{query}&pagelen=50",
            self.spec.owner, self.spec.repo
        );
        let data = self.get_json(&path)?;
        let Some(arr) = data.get("values").and_then(Value::as_array) else {
            return Ok(Vec::new());
        };
        Ok(arr.iter().map(Self::parse_summary).collect())
    }

    fn get_pr_detail(&self, number: u64) -> Result<PullRequestDetail, String> {
        let base = format!(
            "/repositories/{}/{}/pullrequests/{number}",
            self.spec.owner, self.spec.repo
        );
        let pr_json = self.get_json(&base)?;
        let mut summary = Self::parse_summary(&pr_json);

        // Commits
        let commits_json = self
            .get_json(&format!("{base}/commits?pagelen=100"))
            .unwrap_or(Value::Null);
        let commits = commits_json
            .get("values")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .map(|c| {
                        let hash = c.get("hash").and_then(Value::as_str).unwrap_or("").to_string();
                        let short = hash[..hash.len().min(7)].to_string();
                        let msg = c
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .lines()
                            .next()
                            .unwrap_or("")
                            .to_string();
                        let author = c
                            .pointer("/author/user/display_name")
                            .or_else(|| c.pointer("/author/raw"))
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                            .to_string();
                        PrCommit {
                            hash,
                            short_hash: short,
                            message: msg,
                            author,
                            date: normalize_iso_for_display(
                                c.get("date").and_then(Value::as_str).unwrap_or(""),
                            ),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Diffstat + split unified diff per file
        let raw_diff = self.get_text(&format!("{base}/diff")).unwrap_or_default();
        let patches_by_file = split_unified_diff_by_file(&raw_diff);

        let diffstat_json = self
            .get_json(&format!("{base}/diffstat?pagelen=100"))
            .unwrap_or(Value::Null);
        let mut total_add = 0usize;
        let mut total_del = 0usize;
        let files = diffstat_json
            .get("values")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .map(|d| {
                        let status_raw = d.get("status").and_then(Value::as_str).unwrap_or("modified");
                        let status = match status_raw {
                            "added" => "A",
                            "removed" => "D",
                            "renamed" => "R",
                            _ => "M",
                        }
                        .to_string();
                        let path = d
                            .pointer("/new/path")
                            .or_else(|| d.pointer("/old/path"))
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let add = d.get("lines_added").and_then(Value::as_u64).unwrap_or(0) as usize;
                        let del =
                            d.get("lines_removed").and_then(Value::as_u64).unwrap_or(0) as usize;
                        total_add += add;
                        total_del += del;
                        let patch = patches_by_file.get(&path).cloned().unwrap_or_default();
                        PrFileChange {
                            path,
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

        // Participants (approvals / changes requested) + comments
        let mut comments = Vec::new();
        if let Some(participants) = pr_json.get("participants").and_then(Value::as_array) {
            for p in participants {
                let state = p.get("state").and_then(Value::as_str).unwrap_or("");
                let approved = p.get("approved").and_then(Value::as_bool).unwrap_or(false);
                let kind = if approved || state == "approved" {
                    Some(CommentKind::Approved)
                } else if state == "changes_requested" {
                    Some(CommentKind::ChangesRequested)
                } else {
                    None
                };
                if let Some(k) = kind {
                    comments.push(PrComment {
                        author: p
                            .pointer("/user/display_name")
                            .or_else(|| p.pointer("/user/nickname"))
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                            .to_string(),
                        body: String::new(),
                        created_at: normalize_iso_for_display(
                            p.get("participated_on").and_then(Value::as_str).unwrap_or(""),
                        ),
                        kind: k,
                    });
                }
            }
        }

        if let Ok(c_json) = self.get_json(&format!("{base}/comments?pagelen=100")) {
            if let Some(vals) = c_json.get("values").and_then(Value::as_array) {
                for c in vals {
                    if c.get("deleted").and_then(Value::as_bool).unwrap_or(false) {
                        continue;
                    }
                    comments.push(PrComment {
                        author: c
                            .pointer("/user/display_name")
                            .or_else(|| c.pointer("/user/nickname"))
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                            .to_string(),
                        body: c
                            .pointer("/content/raw")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        created_at: normalize_iso_for_display(
                            c.get("created_on").and_then(Value::as_str).unwrap_or(""),
                        ),
                        kind: CommentKind::Comment,
                    });
                }
            }
        }

        comments.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(PullRequestDetail {
            summary,
            commits,
            files,
            comments,
        })
    }

    fn approve_pr(&self, number: u64) -> Result<String, String> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{number}/approve",
            self.spec.owner, self.spec.repo
        );
        self.send_json("POST", &path, json!({}))?;
        Ok(format!("PR #{number} approved."))
    }

    fn request_changes(&self, number: u64, comment: &str) -> Result<String, String> {
        let base = format!(
            "/repositories/{}/{}/pullrequests/{number}",
            self.spec.owner, self.spec.repo
        );
        self.send_json("POST", &format!("{base}/request-changes"), json!({}))?;
        if !comment.trim().is_empty() {
            let _ = self.add_comment(number, comment);
        }
        Ok(format!("Changes requested on PR #{number}."))
    }

    fn add_comment(&self, number: u64, body: &str) -> Result<String, String> {
        if body.trim().is_empty() {
            return Err("Comment cannot be empty.".to_string());
        }
        let path = format!(
            "/repositories/{}/{}/pullrequests/{number}/comments",
            self.spec.owner, self.spec.repo
        );
        self.send_json("POST", &path, json!({ "content": { "raw": body.trim() } }))?;
        Ok(format!("Comment posted on PR #{number}."))
    }

    fn merge_pr(&self, number: u64, strategy: MergeStrategy) -> Result<String, String> {
        let strat = match strategy {
            MergeStrategy::Merge => "merge_commit",
            MergeStrategy::Squash => "squash",
            MergeStrategy::Rebase => "fast_forward",
        };
        let path = format!(
            "/repositories/{}/{}/pullrequests/{number}/merge",
            self.spec.owner, self.spec.repo
        );
        self.send_json("POST", &path, json!({ "merge_strategy": strat }))?;
        Ok(format!("PR #{number} merged ({strat})."))
    }

    fn close_or_decline_pr(&self, number: u64) -> Result<String, String> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{number}/decline",
            self.spec.owner, self.spec.repo
        );
        self.send_json("POST", &path, json!({}))?;
        Ok(format!("PR #{number} declined."))
    }

    fn reopen_pr(&self, _number: u64) -> Result<String, String> {
        Err("Bitbucket Cloud does not support reopening declined pull requests via API. Please create a new pull request from the branch.".to_string())
    }
}

fn split_unified_diff_by_file(raw: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut current_file: Option<String> = None;
    let mut buf = String::new();

    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            if let Some(path) = current_file.take() {
                map.insert(path, std::mem::take(&mut buf));
            }
            // Extract b/<path>
            let b_path = rest
                .split(" b/")
                .nth(1)
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
            current_file = Some(b_path);
        }
        buf.push_str(line);
        buf.push('\n');
    }
    if let Some(path) = current_file {
        map.insert(path, buf);
    }
    map
}
