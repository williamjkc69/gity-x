use serde_json::{json, Value};

use super::{
    format_http_error, normalize_iso_for_display, CommentKind, MergeStrategy, PrComment, PrCommit,
    PrFileChange, PrProvider, PrState, PrStateFilter, PullRequestDetail, PullRequestSummary,
    RemoteSpec,
};

pub struct GitHubProvider {
    spec: RemoteSpec,
    token: Option<String>,
}

impl GitHubProvider {
    pub fn new(spec: RemoteSpec, token: Option<String>) -> Self {
        Self { spec, token }
    }

    fn api_base(&self) -> String {
        if self.spec.host == "github.com" {
            "https://api.github.com".to_string()
        } else {
            format!("https://{}/api/v3", self.spec.host)
        }
    }

    fn req(&self, method: &str, path: &str) -> (String, ureq::Request) {
        let url = format!("{}{path}", self.api_base());
        let mut r = ureq::request(method, &url)
            .timeout(std::time::Duration::from_secs(20))
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set("User-Agent", "gity-x");
        if let Some(tok) = &self.token {
            r = r.set("Authorization", &format!("Bearer {tok}"));
        }
        (url, r)
    }

    fn get_json(&self, path: &str) -> Result<Value, String> {
        let (url, r) = self.req("GET", path);
        let resp = r
            .call()
            .map_err(|e| format_http_error("GitHub", &url, self.token.is_some(), e))?;
        resp.into_json::<Value>()
            .map_err(|e| format!("GitHub JSON parse error: {e}"))
    }

    fn send_json(&self, method: &str, path: &str, payload: Value) -> Result<Value, String> {
        let (url, r) = self.req(method, path);
        let resp = r
            .send_json(payload)
            .map_err(|e| format_http_error("GitHub", &url, self.token.is_some(), e))?;
        resp.into_json::<Value>()
            .or_else(|_| Ok(Value::Null))
    }

    fn parse_summary(v: &Value) -> PullRequestSummary {
        let is_draft = v.get("draft").and_then(Value::as_bool).unwrap_or(false);
        let merged_at = v.get("merged_at").and_then(Value::as_str);
        let state_str = v.get("state").and_then(Value::as_str).unwrap_or("open");

        let state = if merged_at.is_some() {
            PrState::Merged
        } else if state_str == "closed" {
            PrState::Closed
        } else if is_draft {
            PrState::Draft
        } else {
            PrState::Open
        };

        let comments = v.get("comments").and_then(Value::as_u64).unwrap_or(0) as usize
            + v.get("review_comments").and_then(Value::as_u64).unwrap_or(0) as usize;

        PullRequestSummary {
            number: v.get("number").and_then(Value::as_u64).unwrap_or(0),
            title: v
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            body: v
                .get("body")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            state,
            author: v
                .pointer("/user/login")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            source_branch: v
                .pointer("/head/ref")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            target_branch: v
                .pointer("/base/ref")
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
                .get("html_url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            additions: v.get("additions").and_then(Value::as_u64).map(|n| n as usize),
            deletions: v.get("deletions").and_then(Value::as_u64).map(|n| n as usize),
            changed_files: v
                .get("changed_files")
                .and_then(Value::as_u64)
                .map(|n| n as usize),
            comments_count: comments,
            mergeable: v.get("mergeable").and_then(Value::as_bool),
        }
    }
}

impl PrProvider for GitHubProvider {
    fn list_prs(&self, filter: PrStateFilter) -> Result<Vec<PullRequestSummary>, String> {
        let state_param = match filter {
            PrStateFilter::Open => "open",
            PrStateFilter::Closed | PrStateFilter::Merged => "closed",
            PrStateFilter::All => "all",
        };
        let path = format!(
            "/repos/{}/{}/pulls?state={state_param}&per_page=50&sort=updated&direction=desc",
            self.spec.owner, self.spec.repo
        );
        let data = self.get_json(&path)?;
        let Some(arr) = data.as_array() else {
            return Ok(Vec::new());
        };

        let mut list: Vec<PullRequestSummary> = arr.iter().map(Self::parse_summary).collect();
        match filter {
            PrStateFilter::Merged => list.retain(|p| p.state == PrState::Merged),
            PrStateFilter::Closed => list.retain(|p| p.state == PrState::Closed),
            _ => {}
        }
        Ok(list)
    }

    fn get_pr_detail(&self, number: u64) -> Result<PullRequestDetail, String> {
        let base = format!("/repos/{}/{}", self.spec.owner, self.spec.repo);

        let pr_json = self.get_json(&format!("{base}/pulls/{number}"))?;
        let summary = Self::parse_summary(&pr_json);

        // Commits
        let commits_json = self
            .get_json(&format!("{base}/pulls/{number}/commits?per_page=100"))
            .unwrap_or(Value::Array(Vec::new()));
        let commits = commits_json
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|c| {
                        let sha = c.get("sha").and_then(Value::as_str).unwrap_or("").to_string();
                        let short = sha[..sha.len().min(7)].to_string();
                        let msg = c
                            .pointer("/commit/message")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .lines()
                            .next()
                            .unwrap_or("")
                            .to_string();
                        let author = c
                            .pointer("/commit/author/name")
                            .or_else(|| c.pointer("/author/login"))
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                            .to_string();
                        let date = normalize_iso_for_display(
                            c.pointer("/commit/author/date")
                                .and_then(Value::as_str)
                                .unwrap_or(""),
                        );
                        PrCommit {
                            hash: sha,
                            short_hash: short,
                            message: msg,
                            author,
                            date,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Changed files + patches
        let files_json = self
            .get_json(&format!("{base}/pulls/{number}/files?per_page=100"))
            .unwrap_or(Value::Array(Vec::new()));
        let files = files_json
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|f| {
                        let status_raw = f.get("status").and_then(Value::as_str).unwrap_or("modified");
                        let status = match status_raw {
                            "added" => "A",
                            "removed" => "D",
                            "renamed" => "R",
                            _ => "M",
                        }
                        .to_string();
                        PrFileChange {
                            path: f
                                .get("filename")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                            status,
                            additions: f.get("additions").and_then(Value::as_u64).unwrap_or(0)
                                as usize,
                            deletions: f.get("deletions").and_then(Value::as_u64).unwrap_or(0)
                                as usize,
                            patch: f
                                .get("patch")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Comments + Reviews merged into one timeline
        let mut comments = Vec::new();
        if let Ok(Value::Array(reviews)) =
            self.get_json(&format!("{base}/pulls/{number}/reviews?per_page=100"))
        {
            for r in reviews {
                let state = r.get("state").and_then(Value::as_str).unwrap_or("");
                let body = r.get("body").and_then(Value::as_str).unwrap_or("").trim();
                let kind = match state {
                    "APPROVED" => CommentKind::Approved,
                    "CHANGES_REQUESTED" => CommentKind::ChangesRequested,
                    _ => {
                        if body.is_empty() {
                            continue;
                        }
                        CommentKind::Comment
                    }
                };
                comments.push(PrComment {
                    author: r
                        .pointer("/user/login")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string(),
                    body: body.to_string(),
                    created_at: normalize_iso_for_display(
                        r.get("submitted_at").and_then(Value::as_str).unwrap_or(""),
                    ),
                    kind,
                });
            }
        }

        if let Ok(Value::Array(issue_comments)) =
            self.get_json(&format!("{base}/issues/{number}/comments?per_page=100"))
        {
            for c in issue_comments {
                comments.push(PrComment {
                    author: c
                        .pointer("/user/login")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string(),
                    body: c
                        .get("body")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    created_at: normalize_iso_for_display(
                        c.get("created_at").and_then(Value::as_str).unwrap_or(""),
                    ),
                    kind: CommentKind::Comment,
                });
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
            "/repos/{}/{}/pulls/{number}/reviews",
            self.spec.owner, self.spec.repo
        );
        self.send_json("POST", &path, json!({ "event": "APPROVE" }))?;
        Ok(format!("PR #{number} approved."))
    }

    fn request_changes(&self, number: u64, comment: &str) -> Result<String, String> {
        let body = if comment.trim().is_empty() {
            "Changes requested via gity x."
        } else {
            comment.trim()
        };
        let path = format!(
            "/repos/{}/{}/pulls/{number}/reviews",
            self.spec.owner, self.spec.repo
        );
        self.send_json(
            "POST",
            &path,
            json!({ "event": "REQUEST_CHANGES", "body": body }),
        )?;
        Ok(format!("Changes requested on PR #{number}."))
    }

    fn add_comment(&self, number: u64, body: &str) -> Result<String, String> {
        if body.trim().is_empty() {
            return Err("Comment cannot be empty.".to_string());
        }
        let path = format!(
            "/repos/{}/{}/issues/{number}/comments",
            self.spec.owner, self.spec.repo
        );
        self.send_json("POST", &path, json!({ "body": body.trim() }))?;
        Ok(format!("Comment posted on PR #{number}."))
    }

    fn merge_pr(&self, number: u64, strategy: MergeStrategy) -> Result<String, String> {
        let method = match strategy {
            MergeStrategy::Merge => "merge",
            MergeStrategy::Squash => "squash",
            MergeStrategy::Rebase => "rebase",
        };
        let path = format!(
            "/repos/{}/{}/pulls/{number}/merge",
            self.spec.owner, self.spec.repo
        );
        self.send_json("PUT", &path, json!({ "merge_method": method }))?;
        Ok(format!("PR #{number} merged ({method})."))
    }

    fn close_or_decline_pr(&self, number: u64) -> Result<String, String> {
        let path = format!(
            "/repos/{}/{}/pulls/{number}",
            self.spec.owner, self.spec.repo
        );
        self.send_json("PATCH", &path, json!({ "state": "closed" }))?;
        Ok(format!("PR #{number} closed."))
    }

    fn reopen_pr(&self, number: u64) -> Result<String, String> {
        let path = format!(
            "/repos/{}/{}/pulls/{number}",
            self.spec.owner, self.spec.repo
        );
        self.send_json("PATCH", &path, json!({ "state": "open" }))?;
        Ok(format!("PR #{number} reopened."))
    }
}
