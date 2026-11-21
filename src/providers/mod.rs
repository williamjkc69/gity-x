pub mod bitbucket;
pub mod github;
pub mod gitlab;

use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    GitHub,
    GitLab,
    Bitbucket,
}

impl ProviderKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::GitLab => "GitLab",
            Self::Bitbucket => "Bitbucket",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSpec {
    pub kind: ProviderKind,
    pub host: String,
    /// Owner/org/workspace (for GitLab, may contain nested groups e.g. "group/subgroup")
    pub owner: String,
    pub repo: String,
    /// Full slug e.g. "owner/repo" or "group/sub/repo"
    pub full_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderAuth {
    pub github_token: String,
    pub gitlab_token: String,
    pub bitbucket_user: String,
    pub bitbucket_token: String,
}

impl ProviderAuth {
    pub fn resolve_github_token(&self) -> Option<(String, &'static str)> {
        if !self.github_token.trim().is_empty() {
            return Some((self.github_token.trim().to_string(), "Settings"));
        }
        if let Ok(tok) = std::env::var("GITHUB_TOKEN") {
            if !tok.trim().is_empty() {
                return Some((tok.trim().to_string(), "GITHUB_TOKEN"));
            }
        }
        if let Some(tok) = run_cli_token("gh", &["auth", "token"]) {
            return Some((tok, "gh CLI"));
        }
        None
    }

    pub fn resolve_gitlab_token(&self) -> Option<(String, &'static str)> {
        if !self.gitlab_token.trim().is_empty() {
            return Some((self.gitlab_token.trim().to_string(), "Settings"));
        }
        if let Ok(tok) = std::env::var("GITLAB_TOKEN") {
            if !tok.trim().is_empty() {
                return Some((tok.trim().to_string(), "GITLAB_TOKEN"));
            }
        }
        None
    }

    pub fn resolve_bitbucket_auth(&self) -> Option<(String, String, &'static str)> {
        let user = if !self.bitbucket_user.trim().is_empty() {
            self.bitbucket_user.trim().to_string()
        } else {
            std::env::var("BITBUCKET_USER").unwrap_or_default()
        };
        let tok = if !self.bitbucket_token.trim().is_empty() {
            self.bitbucket_token.trim().to_string()
        } else {
            std::env::var("BITBUCKET_TOKEN").unwrap_or_default()
        };
        if !tok.is_empty() {
            let src = if !self.bitbucket_token.trim().is_empty() {
                "Settings"
            } else {
                "BITBUCKET_TOKEN"
            };
            return Some((user, tok, src));
        }
        None
    }
}

fn cli_candidate_paths(program: &str) -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from(program)];

    #[cfg(unix)]
    {
        for dir in [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/opt/local/bin",
            "/home/linuxbrew/.linuxbrew/bin",
            "/snap/bin",
        ] {
            let p = PathBuf::from(dir).join(program);
            if p.is_file() {
                candidates.push(p);
            }
        }
        if let Some(home) = std::env::var_os("HOME") {
            for rel in [".local/bin", "bin"] {
                let p = PathBuf::from(&home).join(rel).join(program);
                if p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let exe_name = format!("{program}.exe");
        if let Some(pf) = std::env::var_os("ProgramFiles") {
            let p = PathBuf::from(pf).join("GitHub CLI").join(&exe_name);
            if p.is_file() {
                candidates.push(p);
            }
        }
        if let Some(local) = std::env::var_os("LocalAppData") {
            let p = PathBuf::from(local)
                .join("Programs")
                .join("GitHub CLI")
                .join(&exe_name);
            if p.is_file() {
                candidates.push(p);
            }
        }
        if let Some(home) = std::env::var_os("USERPROFILE") {
            let p = PathBuf::from(home).join("scoop").join("shims").join(&exe_name);
            if p.is_file() {
                candidates.push(p);
            }
        }
    }

    candidates
}

#[allow(unused_mut)]
fn run_cli_token(program: &str, args: &[&str]) -> Option<String> {
    for bin in cli_candidate_paths(program) {
        let mut cmd = Command::new(&bin);
        cmd.args(args)
            .env("GH_PROMPT_DISABLED", "1")
            .stdin(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x0800_0000);
        }
        let Ok(out) = cmd.output() else {
            continue;
        };
        if !out.status.success() {
            // Binary ran and explicitly failed (e.g. not logged in); no need to retry
            return None;
        }
        let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !token.is_empty() {
            return Some(token);
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Draft,
    Merged,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrStateFilter {
    Open,
    Merged,
    Closed,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    Merge,
    Squash,
    Rebase,
}

#[derive(Debug, Clone)]
pub struct PullRequestSummary {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: PrState,
    pub author: String,
    pub source_branch: String,
    pub target_branch: String,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub additions: Option<usize>,
    pub deletions: Option<usize>,
    pub changed_files: Option<usize>,
    pub comments_count: usize,
    pub mergeable: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct PrCommit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct PrFileChange {
    pub path: String,
    pub status: String,
    pub additions: usize,
    pub deletions: usize,
    pub patch: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Comment,
    Approved,
    ChangesRequested,
}

#[derive(Debug, Clone)]
pub struct PrComment {
    pub author: String,
    pub body: String,
    pub created_at: String,
    pub kind: CommentKind,
}

#[derive(Debug, Clone)]
pub struct PullRequestDetail {
    pub summary: PullRequestSummary,
    pub commits: Vec<PrCommit>,
    pub files: Vec<PrFileChange>,
    pub comments: Vec<PrComment>,
}

pub trait PrProvider: Send {
    fn list_prs(&self, filter: PrStateFilter) -> Result<Vec<PullRequestSummary>, String>;
    fn get_pr_detail(&self, number: u64) -> Result<PullRequestDetail, String>;
    fn approve_pr(&self, number: u64) -> Result<String, String>;
    fn request_changes(&self, number: u64, comment: &str) -> Result<String, String>;
    fn add_comment(&self, number: u64, body: &str) -> Result<String, String>;
    fn merge_pr(&self, number: u64, strategy: MergeStrategy) -> Result<String, String>;
    fn close_or_decline_pr(&self, number: u64) -> Result<String, String>;
    fn reopen_pr(&self, number: u64) -> Result<String, String>;
}

pub fn create_provider(
    spec: RemoteSpec,
    auth: &ProviderAuth,
) -> Result<Box<dyn PrProvider>, String> {
    match spec.kind {
        ProviderKind::GitHub => {
            let token = auth.resolve_github_token().map(|(t, _)| t);
            Ok(Box::new(github::GitHubProvider::new(spec, token)))
        }
        ProviderKind::GitLab => {
            let token = auth.resolve_gitlab_token().map(|(t, _)| t);
            Ok(Box::new(gitlab::GitLabProvider::new(spec, token)))
        }
        ProviderKind::Bitbucket => {
            let creds = auth.resolve_bitbucket_auth().map(|(u, t, _)| (u, t));
            Ok(Box::new(bitbucket::BitbucketProvider::new(spec, creds)))
        }
    }
}

/// Parses SSH and HTTPS git remote URLs into a unified `RemoteSpec`.
pub fn parse_remote_url(raw: &str) -> Option<RemoteSpec> {
    let url = raw.trim().trim_end_matches('/');

    let (host_raw, path_raw) = if let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("ssh://"))
    {
        // Strip optional user@ prefix
        let no_user = rest.split_once('@').map(|(_, r)| r).unwrap_or(rest);
        let (h, p) = no_user.split_once('/')?;
        // Strip optional :port
        let host = h.split_once(':').map(|(hh, _)| hh).unwrap_or(h);
        (host, p)
    } else if let Some((_, after_at)) = url.split_once('@') {
        // SCP-style: git@github.com:owner/repo.git
        let (h, p) = after_at.split_once(':')?;
        (h, p)
    } else {
        return None;
    };

    let host = host_raw.to_lowercase();
    let clean_path = path_raw.trim_start_matches('/').trim_end_matches(".git");
    let (owner, repo) = clean_path.rsplit_once('/')?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    let kind = if host.contains("github") {
        ProviderKind::GitHub
    } else if host.contains("gitlab") {
        ProviderKind::GitLab
    } else if host.contains("bitbucket") {
        ProviderKind::Bitbucket
    } else {
        // Self-hosted instances default to GitLab API v4 if host is unknown
        return None;
    };

    Some(RemoteSpec {
        kind,
        host,
        owner: owner.to_string(),
        repo: repo.to_string(),
        full_path: clean_path.to_string(),
    })
}

pub fn format_http_error(provider: &str, url: &str, err: ureq::Error) -> String {
    match err {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            let friendly = extract_api_error_msg(&body);
            if friendly.is_empty() {
                format!("{provider} API error (HTTP {code})\nURL: {url}")
            } else {
                format!("{provider} API error (HTTP {code}): {friendly}\nURL: {url}")
            }
        }
        ureq::Error::Transport(t) => {
            format!("{provider} network error: {t}\nURL: {url}")
        }
    }
}

fn extract_api_error_msg(body: &str) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
            let mut out = msg.to_string();
            if let Some(errs) = val.get("errors").and_then(|v| v.as_array()) {
                for e in errs {
                    if let Some(em) = e.get("message").and_then(|v| v.as_str()) {
                        out.push_str(&format!("\n- {em}"));
                    }
                }
            }
            return out;
        }
        if let Some(msg) = val
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|v| v.as_str())
        {
            return msg.to_string();
        }
        if let Some(msg) = val.get("error").and_then(|v| v.as_str()) {
            return msg.to_string();
        }
    }
    body.chars().take(400).collect()
}

pub fn basic_auth_header(user: &str, pass: &str) -> String {
    let raw = format!("{user}:{pass}");
    format!("Basic {}", base64_encode(raw.as_bytes()))
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

pub fn url_encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

pub fn normalize_iso_for_display(iso: &str) -> String {
    // Converts RFC3339 "2026-11-19T14:30:00Z" into "2026-11-19 14:30:00 UTC" compatible with format_human_date
    let cleaned = iso.replace('T', " ");
    if let Some(idx) = cleaned.find('.') {
        format!("{} UTC", &cleaned[..idx])
    } else if let Some(stripped) = cleaned.strip_suffix('Z') {
        format!("{stripped} UTC")
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_urls() {
        let ssh = parse_remote_url("git@github.com:williamjkc69/gity-x.git").unwrap();
        assert_eq!(ssh.kind, ProviderKind::GitHub);
        assert_eq!(ssh.owner, "williamjkc69");
        assert_eq!(ssh.repo, "gity-x");
        assert_eq!(ssh.full_path, "williamjkc69/gity-x");

        let https = parse_remote_url("https://github.com/williamjkc69/gity-x").unwrap();
        assert_eq!(https, ssh);
    }

    #[test]
    fn parses_gitlab_nested_group_urls() {
        let spec = parse_remote_url("git@gitlab.company.com:org/team/sub/repo.git").unwrap();
        assert_eq!(spec.kind, ProviderKind::GitLab);
        assert_eq!(spec.host, "gitlab.company.com");
        assert_eq!(spec.owner, "org/team/sub");
        assert_eq!(spec.repo, "repo");
        assert_eq!(spec.full_path, "org/team/sub/repo");
        assert_eq!(
            url_encode_component(&spec.full_path),
            "org%2Fteam%2Fsub%2Frepo"
        );
    }

    #[test]
    fn parses_bitbucket_urls() {
        let https =
            parse_remote_url("https://user@bitbucket.org/myworkspace/my-repo.git").unwrap();
        assert_eq!(https.kind, ProviderKind::Bitbucket);
        assert_eq!(https.owner, "myworkspace");
        assert_eq!(https.repo, "my-repo");

        let ssh = parse_remote_url("git@bitbucket.org:myworkspace/my-repo.git").unwrap();
        assert_eq!(ssh, https);
    }

    #[test]
    fn encodes_basic_auth_and_iso() {
        assert_eq!(basic_auth_header("user", "pass"), "Basic dXNlcjpwYXNz");
        assert_eq!(
            normalize_iso_for_display("2026-11-19T14:30:00.123456Z"),
            "2026-11-19 14:30:00 UTC"
        );
        assert_eq!(
            normalize_iso_for_display("2026-11-19T14:30:00Z"),
            "2026-11-19 14:30:00 UTC"
        );
    }

    #[test]
    #[ignore = "live network smoke test"]
    fn live_github_list_and_detail() {
        let spec = parse_remote_url("https://github.com/emilk/egui.git").unwrap();
        let auth = ProviderAuth::default();
        let provider = create_provider(spec, &auth).unwrap();
        let prs = provider.list_prs(PrStateFilter::Open).unwrap();
        assert!(!prs.is_empty(), "expected open PRs on emilk/egui");
        let detail = provider.get_pr_detail(prs[0].number).unwrap();
        assert_eq!(detail.summary.number, prs[0].number);
        assert!(!detail.commits.is_empty());
        println!(
            "GitHub OK: #{} '{}' ({} commits, {} files, {} comments)",
            detail.summary.number,
            detail.summary.title,
            detail.commits.len(),
            detail.files.len(),
            detail.comments.len()
        );
    }

    #[test]
    #[ignore = "live network smoke test"]
    fn live_gitlab_list_and_detail() {
        let spec = parse_remote_url("https://gitlab.com/gitlab-org/gitlab-runner.git").unwrap();
        let auth = ProviderAuth::default();
        let provider = create_provider(spec, &auth).unwrap();
        let mrs = provider.list_prs(PrStateFilter::Open).unwrap();
        assert!(!mrs.is_empty(), "expected open MRs on gitlab-org/gitlab-runner");
        let detail = provider.get_pr_detail(mrs[0].number).unwrap();
        assert_eq!(detail.summary.number, mrs[0].number);
        println!(
            "GitLab OK: !{} '{}' ({} commits, {} files, {} comments)",
            detail.summary.number,
            detail.summary.title,
            detail.commits.len(),
            detail.files.len(),
            detail.comments.len()
        );
    }

    #[test]
    #[ignore = "live network smoke test"]
    fn live_bitbucket_list_and_detail() {
        let spec =
            parse_remote_url("https://bitbucket.org/atlassian/atlassian-event.git").unwrap();
        let auth = ProviderAuth::default();
        let provider = create_provider(spec, &auth).unwrap();
        let prs = provider.list_prs(PrStateFilter::All).unwrap();
        assert!(
            !prs.is_empty(),
            "expected PRs on atlassian/atlassian-event"
        );
        let detail = provider.get_pr_detail(prs[0].number).unwrap();
        assert_eq!(detail.summary.number, prs[0].number);
        println!(
            "Bitbucket OK: #{} '{}' ({} commits, {} files, {} comments)",
            detail.summary.number,
            detail.summary.title,
            detail.commits.len(),
            detail.files.len(),
            detail.comments.len()
        );
    }
}
