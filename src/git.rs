use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::CommitOrder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefKind {
    Head,
    LocalBranch,
    RemoteBranch,
    Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}

impl ResetMode {
    fn flag(self) -> &'static str {
        match self {
            Self::Soft => "--soft",
            Self::Mixed => "--mixed",
            Self::Hard => "--hard",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GitRef {
    pub name: String,
    pub kind: RefKind,
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub email: String,
    pub committer: String,
    pub committer_email: String,
    pub sig_status: char,
    pub date: String,
    pub relative_date: String,
    pub subject: String,
    pub body: String,
    pub refs: Vec<GitRef>,
    pub is_head: bool,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub is_current: bool,
    pub target_hash: String,
}

#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub target_hash: String,
}

#[derive(Debug, Clone)]
pub struct StashEntry {
    pub index: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub index_status: char,
    pub worktree_status: char,
}

impl FileStatus {
    pub fn is_staged(&self) -> bool {
        self.index_status != ' ' && self.index_status != '?'
    }

    pub fn is_unstaged(&self) -> bool {
        self.worktree_status != ' ' || self.index_status == '?'
    }

    pub fn is_untracked(&self) -> bool {
        self.index_status == '?' && self.worktree_status == '?'
    }
}

#[derive(Debug, Clone)]
pub struct CommitFileChange {
    pub status: String,
    pub path: String,
    pub additions: Option<usize>,
    pub deletions: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct GitRepo {
    pub path: PathBuf,
}

impl GitRepo {
    pub fn normalize_path(raw: &Path) -> PathBuf {
        let s = raw.to_string_lossy();
        let trimmed = s.trim().trim_matches('"').trim_matches('\'');
        if let Some(rest) = trimmed.strip_prefix("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                return PathBuf::from(home).join(rest);
            }
        } else if trimmed == "~" {
            if let Some(home) = std::env::var_os("HOME") {
                return PathBuf::from(home);
            }
        }
        PathBuf::from(trimmed)
    }

    pub fn open(dir: &Path) -> Result<Self, String> {
        let normalized = Self::normalize_path(dir);
        let target = if normalized.is_file() {
            normalized
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or(normalized)
        } else {
            normalized
        };

        if !target.exists() {
            return Err(format!("Folder does not exist: {}", target.display()));
        }

        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .current_dir(&target)
            .output()
            .map_err(|e| format!("Failed to run git: {e}"))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if err.is_empty() {
                format!(
                    "Not a Git repository: {}",
                    target.display()
                )
            } else {
                format!("{} ({})", err, target.display())
            });
        }

        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Self {
            path: PathBuf::from(root),
        })
    }

    pub fn repo_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string()
    }

    fn run_git(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .map_err(|e| format!("Error running git: {e}"))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let details = match (stderr.is_empty(), stdout.is_empty()) {
                (false, false) => format!("{stderr}\n{stdout}"),
                (false, true) => stderr,
                (true, false) => stdout,
                (true, true) => format!("git exited with status {}", output.status),
            };
            Err(format!("$ git {}\n\n{details}", args.join(" ")))
        }
    }

    fn git_dir(&self) -> Option<PathBuf> {
        self.run_git(&["rev-parse", "--git-dir"])
            .ok()
            .map(|s| {
                let p = PathBuf::from(s.trim());
                if p.is_absolute() {
                    p
                } else {
                    self.path.join(p)
                }
            })
    }

    pub fn is_cherry_picking(&self) -> bool {
        self.git_dir()
            .map(|d| d.join("CHERRY_PICK_HEAD").exists())
            .unwrap_or(false)
    }

    pub fn is_merging(&self) -> bool {
        self.git_dir()
            .map(|d| d.join("MERGE_HEAD").exists())
            .unwrap_or(false)
    }

    pub fn is_rebasing(&self) -> bool {
        self.git_dir()
            .map(|d| d.join("rebase-merge").exists() || d.join("rebase-apply").exists())
            .unwrap_or(false)
    }

    pub fn current_branch(&self) -> String {
        if let Ok(branch) = self.run_git(&["symbolic-ref", "--short", "HEAD"]) {
            let b = branch.trim();
            if !b.is_empty() {
                return b.to_string();
            }
        }
        if let Ok(hash) = self.run_git(&["rev-parse", "--short", "HEAD"]) {
            let h = hash.trim();
            if !h.is_empty() {
                return format!("HEAD ({h})");
            }
        }
        "HEAD".to_string()
    }

    pub fn head_hash(&self) -> Option<String> {
        self.run_git(&["rev-parse", "HEAD"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn ahead_behind(&self) -> Option<(usize, usize)> {
        let out = self
            .run_git(&["rev-list", "--left-right", "--count", "HEAD...@{u}"])
            .ok()?;
        let mut parts = out.split_whitespace();
        let ahead = parts.next()?.parse::<usize>().ok()?;
        let behind = parts.next()?.parse::<usize>().ok()?;
        Some((ahead, behind))
    }

    /// Lightweight ~60-byte remote check using `git ls-remote` (no objects downloaded).
    /// Returns true if the remote branch tip differs from our local remote-tracking ref.
    pub fn has_remote_updates(&self) -> bool {
        let Ok(upstream) = self.run_git(&["rev-parse", "--abbrev-ref", "@{u}"]) else {
            return false;
        };
        let upstream = upstream.trim();
        let Some((remote, branch)) = upstream.split_once('/') else {
            return false;
        };
        let Ok(local_remote_sha) = self.run_git(&["rev-parse", "@{u}"]) else {
            return false;
        };
        let local_remote_sha = local_remote_sha.trim();
        let ref_spec = format!("refs/heads/{branch}");

        let Ok(out) = Command::new("git")
            .args(["ls-remote", remote, &ref_spec])
            .env("GIT_TERMINAL_PROMPT", "0")
            .current_dir(&self.path)
            .output()
        else {
            return false;
        };
        if !out.status.success() {
            return false;
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        let Some(remote_sha) = stdout.split_whitespace().next() else {
            return false;
        };
        !remote_sha.is_empty() && remote_sha != local_remote_sha
    }

    pub fn load_commits(
        &self,
        limit: usize,
        branch_filter: Option<&str>,
        show_remotes: bool,
        order: CommitOrder,
    ) -> Result<Vec<Commit>, String> {
        let limit_arg = format!("--max-count={limit}");
        let format_arg =
            "--format=%x1e%H%x1f%h%x1f%P%x1f%an%x1f%ae%x1f%cn%x1f%ce%x1f%G?%x1f%ai%x1f%ar%x1f%D%x1f%s%x1f%b";

        let mut args: Vec<&str> = vec![
            "log",
            order.git_flag(),
            "--decorate=full",
            &limit_arg,
            format_arg,
        ];
        if let Some(branch) = branch_filter {
            args.push(branch);
        } else if show_remotes {
            args.push("--all");
        } else {
            args.push("--branches");
            args.push("--tags");
            args.push("HEAD");
        }

        let raw = match self.run_git(&args) {
            Ok(out) => out,
            Err(e) if e.contains("does not have any commits yet") => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };

        let head_hash = self.head_hash();
        let mut commits = Vec::new();

        for record in raw.split('\x1e') {
            let record = record.trim_start_matches('\n');
            if record.is_empty() {
                continue;
            }
            let fields: Vec<&str> = record.split('\x1f').collect();
            if fields.len() < 13 {
                continue;
            }

            let hash = fields[0].trim().to_string();
            let short_hash = fields[1].trim().to_string();
            let parents: Vec<String> = fields[2]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            let author = fields[3].trim().to_string();
            let email = fields[4].trim().to_string();
            let committer = fields[5].trim().to_string();
            let committer_email = fields[6].trim().to_string();
            let sig_status = fields[7].trim().chars().next().unwrap_or('N');
            let date = fields[8].trim().to_string();
            let relative_date = fields[9].trim().to_string();
            let refs = parse_refs(fields[10]);
            let subject = fields[11].trim().to_string();
            let body = fields[12].trim().to_string();
            let is_head = head_hash.as_deref() == Some(hash.as_str())
                || refs.iter().any(|r| r.kind == RefKind::Head);

            commits.push(Commit {
                hash,
                short_hash,
                parents,
                author,
                email,
                committer,
                committer_email,
                sig_status,
                date,
                relative_date,
                subject,
                body,
                refs,
                is_head,
            });
        }

        Ok(commits)
    }

    pub fn load_branches(&self) -> Vec<BranchInfo> {
        let mut branches = Vec::new();
        let current = self.current_branch();

        if let Ok(out) = self.run_git(&[
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname:short)%09%(objectname)",
            "refs/heads",
        ]) {
            for line in out.lines() {
                let mut parts = line.split('\t');
                if let (Some(name), Some(hash)) = (parts.next(), parts.next()) {
                    let name = name.trim().to_string();
                    if !name.is_empty() {
                        branches.push(BranchInfo {
                            is_current: name == current,
                            name,
                            is_remote: false,
                            target_hash: hash.trim().to_string(),
                        });
                    }
                }
            }
        }

        if let Ok(out) = self.run_git(&[
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname:short)%09%(objectname)",
            "refs/remotes",
        ]) {
            for line in out.lines() {
                let mut parts = line.split('\t');
                if let (Some(name), Some(hash)) = (parts.next(), parts.next()) {
                    let name = name.trim().to_string();
                    if !name.is_empty() && !name.ends_with("/HEAD") {
                        branches.push(BranchInfo {
                            name,
                            is_remote: true,
                            is_current: false,
                            target_hash: hash.trim().to_string(),
                        });
                    }
                }
            }
        }

        branches
    }

    pub fn load_tags(&self) -> Vec<TagInfo> {
        let mut tags = Vec::new();
        if let Ok(out) = self.run_git(&[
            "for-each-ref",
            "--sort=-creatordate",
            "--format=%(refname:short)%09%(*objectname)%09%(objectname)",
            "refs/tags",
        ]) {
            for line in out.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 {
                    let name = parts[0].trim().to_string();
                    let target = if !parts[1].trim().is_empty() {
                        parts[1].trim().to_string()
                    } else {
                        parts[2].trim().to_string()
                    };
                    if !name.is_empty() {
                        tags.push(TagInfo {
                            name,
                            target_hash: target,
                        });
                    }
                }
            }
        }
        tags
    }

    pub fn load_stashes(&self) -> Vec<StashEntry> {
        let mut stashes = Vec::new();
        if let Ok(out) = self.run_git(&["stash", "list", "--format=%gd%x09%gs"]) {
            for line in out.lines() {
                let mut parts = line.splitn(2, '\t');
                if let (Some(index), Some(message)) = (parts.next(), parts.next()) {
                    stashes.push(StashEntry {
                        index: index.trim().to_string(),
                        message: message.trim().to_string(),
                    });
                }
            }
        }
        stashes
    }

    pub fn load_status(&self) -> Vec<FileStatus> {
        let mut files = Vec::new();
        if let Ok(out) = self.run_git(&["status", "--porcelain=v1", "-uall"]) {
            for line in out.lines() {
                if line.len() < 4 {
                    continue;
                }
                let mut chars = line.chars();
                let index_status = chars.next().unwrap_or(' ');
                let worktree_status = chars.next().unwrap_or(' ');
                let raw_path = line[3..].trim();
                let path = if let Some((_, new_path)) = raw_path.split_once(" -> ") {
                    new_path.trim_matches('"').to_string()
                } else {
                    raw_path.trim_matches('"').to_string()
                };
                files.push(FileStatus {
                    path,
                    index_status,
                    worktree_status,
                });
            }
        }
        files
    }

    pub fn commit_files(&self, hash: &str) -> Vec<CommitFileChange> {
        let numstat_map = parse_numstat(
            &self
                .run_git(&[
                    "diff-tree",
                    "--no-commit-id",
                    "--numstat",
                    "-r",
                    "--root",
                    hash,
                ])
                .unwrap_or_default(),
        );
        let name_status = self
            .run_git(&[
                "diff-tree",
                "--no-commit-id",
                "--name-status",
                "-r",
                "--root",
                hash,
            ])
            .unwrap_or_default();
        parse_file_changes(&name_status, &numstat_map)
    }

    pub fn commit_diff(&self, hash: &str, file: Option<&str>) -> String {
        let res = match file {
            Some(path) => self.run_git(&["show", "--format=", hash, "--", path]),
            None => self.run_git(&["show", "--stat", "--patch", "--format=", hash]),
        };
        res.unwrap_or_else(|e| format!("Error loading diff: {e}"))
    }

    pub fn compare_files(&self, from_hash: &str, to_hash: &str) -> Vec<CommitFileChange> {
        let numstat_map = parse_numstat(
            &self
                .run_git(&["diff", "--numstat", from_hash, to_hash])
                .unwrap_or_default(),
        );
        let name_status = self
            .run_git(&["diff", "--name-status", from_hash, to_hash])
            .unwrap_or_default();
        parse_file_changes(&name_status, &numstat_map)
    }

    pub fn compare_diff(&self, from_hash: &str, to_hash: &str, file: Option<&str>) -> String {
        let res = match file {
            Some(path) => self.run_git(&["diff", from_hash, to_hash, "--", path]),
            None => self.run_git(&["diff", "--stat", "--patch", from_hash, to_hash]),
        };
        res.unwrap_or_else(|e| format!("Error loading diff: {e}"))
    }

    pub fn compare_with_working_files(&self, hash: &str) -> Vec<CommitFileChange> {
        let numstat_map = parse_numstat(
            &self
                .run_git(&["diff", "--numstat", hash])
                .unwrap_or_default(),
        );
        let name_status = self
            .run_git(&["diff", "--name-status", hash])
            .unwrap_or_default();
        parse_file_changes(&name_status, &numstat_map)
    }

    pub fn compare_with_working_diff(&self, hash: &str, file: Option<&str>) -> String {
        let res = match file {
            Some(path) => self.run_git(&["diff", hash, "--", path]),
            None => self.run_git(&["diff", "--stat", "--patch", hash]),
        };
        res.unwrap_or_else(|e| format!("Error loading diff: {e}"))
    }

    pub fn working_diff(&self, file: &str, staged: bool, untracked: bool) -> String {
        if untracked {
            let full = self.path.join(file);
            return match std::fs::read_to_string(&full) {
                Ok(content) => {
                    let mut out = format!(
                        "--- /dev/null\n+++ b/{file}\n@@ -0,0 +1,{} @@\n",
                        content.lines().count()
                    );
                    for line in content.lines() {
                        out.push('+');
                        out.push_str(line);
                        out.push('\n');
                    }
                    out
                }
                Err(_) => "(Binary or unreadable file)".to_string(),
            };
        }

        let res = if staged {
            self.run_git(&["diff", "--cached", "--", file])
        } else {
            self.run_git(&["diff", "--", file])
        };
        res.unwrap_or_else(|e| format!("Error loading diff: {e}"))
    }

    /// Full diff of a working tree file vs HEAD (covers staged + unstaged + untracked).
    pub fn agent_file_diff(&self, file: &FileStatus) -> String {
        if file.is_untracked() {
            return self.working_diff(&file.path, false, true);
        }
        // Diff against HEAD if HEAD exists; fallback to cached/unstaged diff
        if let Ok(diff) = self.run_git(&["diff", "HEAD", "--", &file.path]) {
            if !diff.trim().is_empty() {
                return diff;
            }
        }
        if file.is_staged() {
            let d = self.working_diff(&file.path, true, false);
            if !d.trim().is_empty() {
                return d;
            }
        }
        self.working_diff(&file.path, false, false)
    }

    /// Per-file (+additions, -deletions) for all current working tree changes.
    pub fn load_working_numstat(&self, files: &[FileStatus]) -> HashMap<String, (usize, usize)> {
        let mut stats: HashMap<String, (usize, usize)> = HashMap::new();

        if let Ok(out) = self.run_git(&["diff", "HEAD", "--numstat"]) {
            for (path, (a, d)) in parse_numstat(&out) {
                stats.insert(path, (a.unwrap_or(0), d.unwrap_or(0)));
            }
        } else {
            for cmd in [&["diff", "--numstat"][..], &["diff", "--cached", "--numstat"][..]] {
                if let Ok(out) = self.run_git(cmd) {
                    for (path, (a, d)) in parse_numstat(&out) {
                        let entry = stats.entry(path).or_insert((0, 0));
                        entry.0 += a.unwrap_or(0);
                        entry.1 += d.unwrap_or(0);
                    }
                }
            }
        }

        for f in files {
            if f.is_untracked() {
                let count = std::fs::read_to_string(self.path.join(&f.path))
                    .map(|c| c.lines().count())
                    .unwrap_or(0);
                stats.insert(f.path.clone(), (count, 0));
            }
        }

        stats
    }

    pub fn stage_file(&self, path: &str) -> Result<String, String> {
        self.run_git(&["add", "--", path])
    }

    pub fn unstage_file(&self, path: &str) -> Result<String, String> {
        self.run_git(&["reset", "HEAD", "--", path])
            .or_else(|_| self.run_git(&["rm", "--cached", "--", path]))
    }

    pub fn stage_all(&self) -> Result<String, String> {
        self.run_git(&["add", "-A"])
    }

    pub fn unstage_all(&self) -> Result<String, String> {
        self.run_git(&["reset", "HEAD"])
    }

    pub fn discard_file(&self, path: &str, untracked: bool) -> Result<String, String> {
        if untracked {
            let full = self.path.join(path);
            std::fs::remove_file(&full).map_err(|e| format!("Could not remove {path}: {e}"))?;
            Ok(String::new())
        } else {
            self.run_git(&["checkout", "--", path])
        }
    }

    pub fn commit(&self, message: &str, amend: bool) -> Result<String, String> {
        if message.trim().is_empty() {
            return Err("Commit message cannot be empty.".to_string());
        }
        if amend {
            self.run_git(&["commit", "--amend", "-m", message.trim()])
        } else {
            self.run_git(&["commit", "-m", message.trim()])
        }
    }

    pub fn checkout(&self, target: &str) -> Result<String, String> {
        self.run_git(&["checkout", target])
    }

    pub fn cherry_pick(&self, hash: &str) -> Result<String, String> {
        self.run_git(&["cherry-pick", hash])
    }

    pub fn continue_cherry_pick(&self) -> Result<String, String> {
        self.run_git(&["cherry-pick", "--continue"])
    }

    pub fn abort_cherry_pick(&self) -> Result<String, String> {
        self.run_git(&["cherry-pick", "--abort"])
    }

    pub fn revert_commit(&self, hash: &str) -> Result<String, String> {
        self.run_git(&["revert", "--no-edit", hash])
    }

    pub fn rebase_onto(&self, target: &str) -> Result<String, String> {
        self.run_git(&["rebase", target])
    }

    pub fn continue_rebase(&self) -> Result<String, String> {
        self.run_git(&["rebase", "--continue"])
    }

    pub fn abort_rebase(&self) -> Result<String, String> {
        self.run_git(&["rebase", "--abort"])
    }

    pub fn reset_to(&self, hash: &str, mode: ResetMode) -> Result<String, String> {
        self.run_git(&["reset", mode.flag(), hash])
    }

    pub fn create_branch(&self, name: &str, start_point: Option<&str>) -> Result<String, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Branch name cannot be empty.".to_string());
        }
        match start_point {
            Some(sp) => self.run_git(&["checkout", "-b", name, sp]),
            None => self.run_git(&["checkout", "-b", name]),
        }
    }

    pub fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<String, String> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err("New branch name cannot be empty.".to_string());
        }
        self.run_git(&["branch", "-m", old_name, new_name])
    }

    pub fn delete_branch(&self, name: &str) -> Result<String, String> {
        self.run_git(&["branch", "-d", name])
    }

    pub fn force_delete_branch(&self, name: &str) -> Result<String, String> {
        self.run_git(&["branch", "-D", name])
    }

    pub fn push_branch(&self, name: &str) -> Result<String, String> {
        self.run_git(&["push", "-u", "origin", name])
    }

    pub fn delete_remote_branch(&self, remote_ref: &str) -> Result<String, String> {
        if let Some((remote, branch)) = remote_ref.split_once('/') {
            self.run_git(&["push", remote, "--delete", branch])
        } else {
            Err(format!("Invalid remote branch reference: {remote_ref}"))
        }
    }

    pub fn merge_branch(&self, name: &str) -> Result<String, String> {
        self.run_git(&["merge", name])
    }

    pub fn merge_branch_no_ff(&self, name: &str) -> Result<String, String> {
        self.run_git(&["merge", "--no-ff", name])
    }

    pub fn continue_merge(&self) -> Result<String, String> {
        self.run_git(&["merge", "--continue"])
    }

    pub fn abort_merge(&self) -> Result<String, String> {
        self.run_git(&["merge", "--abort"])
    }

    pub fn create_tag(
        &self,
        name: &str,
        target: &str,
        message: Option<&str>,
    ) -> Result<String, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Tag name cannot be empty.".to_string());
        }
        match message.map(str::trim).filter(|m| !m.is_empty()) {
            Some(msg) => self.run_git(&["tag", "-a", name, target, "-m", msg]),
            None => self.run_git(&["tag", name, target]),
        }
    }

    pub fn push_tag(&self, name: &str) -> Result<String, String> {
        self.run_git(&["push", "origin", name])
    }

    pub fn delete_tag(&self, name: &str) -> Result<String, String> {
        self.run_git(&["tag", "-d", name])
    }

    pub fn fetch(&self) -> Result<String, String> {
        self.run_git(&["fetch", "--all", "--prune"])
    }

    pub fn pull(&self) -> Result<String, String> {
        self.run_git(&["pull"])
    }

    pub fn push(&self) -> Result<String, String> {
        self.run_git(&["push"])
    }

    pub fn stash_save(&self, message: &str) -> Result<String, String> {
        if message.trim().is_empty() {
            self.run_git(&["stash", "push", "--include-untracked"])
        } else {
            self.run_git(&["stash", "push", "--include-untracked", "-m", message.trim()])
        }
    }

    pub fn stash_pop(&self) -> Result<String, String> {
        self.run_git(&["stash", "pop"])
    }

    pub fn stash_apply_ref(&self, index: &str) -> Result<String, String> {
        self.run_git(&["stash", "apply", index])
    }

    pub fn stash_pop_ref(&self, index: &str) -> Result<String, String> {
        self.run_git(&["stash", "pop", index])
    }

    pub fn stash_drop_ref(&self, index: &str) -> Result<String, String> {
        self.run_git(&["stash", "drop", index])
    }

    pub fn stash_diff(&self, index: &str) -> String {
        self.run_git(&["stash", "show", "-p", "--include-untracked", index])
            .or_else(|_| self.run_git(&["stash", "show", "-p", index]))
            .unwrap_or_else(|e| format!("Error loading stash diff: {e}"))
    }
}

fn parse_numstat(raw: &str) -> HashMap<String, (Option<usize>, Option<usize>)> {
    let mut map = HashMap::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let add = parts[0].trim().parse::<usize>().ok();
            let del = parts[1].trim().parse::<usize>().ok();
            let path = parts[2].trim().to_string();
            map.insert(path, (add, del));
        }
    }
    map
}

fn parse_file_changes(
    name_status: &str,
    numstat: &HashMap<String, (Option<usize>, Option<usize>)>,
) -> Vec<CommitFileChange> {
    let mut changes = Vec::new();
    for line in name_status.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let status = parts[0].trim().to_string();
            let path = parts.last().unwrap_or(&"").trim().to_string();
            let (additions, deletions) = numstat.get(&path).copied().unwrap_or((None, None));
            changes.push(CommitFileChange {
                status,
                path,
                additions,
                deletions,
            });
        }
    }
    changes
}

fn parse_refs(raw: &str) -> Vec<GitRef> {
    let mut refs = Vec::new();
    for item in raw.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if let Some(rest) = item.strip_prefix("HEAD -> ") {
            let name = rest.strip_prefix("refs/heads/").unwrap_or(rest).to_string();
            refs.push(GitRef {
                name,
                kind: RefKind::Head,
            });
        } else if item == "HEAD" {
            refs.push(GitRef {
                name: "HEAD".to_string(),
                kind: RefKind::Head,
            });
        } else if let Some(tag) = item.strip_prefix("tag: ") {
            let name = tag.strip_prefix("refs/tags/").unwrap_or(tag).to_string();
            refs.push(GitRef {
                name,
                kind: RefKind::Tag,
            });
        } else if let Some(local) = item.strip_prefix("refs/heads/") {
            refs.push(GitRef {
                name: local.to_string(),
                kind: RefKind::LocalBranch,
            });
        } else if let Some(remote) = item.strip_prefix("refs/remotes/") {
            if !remote.ends_with("/HEAD") {
                refs.push(GitRef {
                    name: remote.to_string(),
                    kind: RefKind::RemoteBranch,
                });
            }
        }
    }
    refs
}
