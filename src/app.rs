use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui;

use crate::config::AppConfig;
use crate::git::{
    BranchInfo, Commit, CommitFileChange, FileStatus, GitRepo, StashEntry, TagInfo,
};
use crate::graph::CommitGraph;
use crate::i18n::{tr, Strings};
use crate::native_menu::{self, NativeMenuBar};
use crate::theme;
use crate::ui;

#[derive(Debug, Clone)]
pub struct RepoTab {
    pub path: PathBuf,
    pub name: String,
    pub branch: String,
    pub branch_filter: Option<String>,
    pub search_query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentTarget {
    None,
    File {
        path: String,
        staged: bool,
        untracked: bool,
    },
    Stash {
        index: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    None,
    Wip,
    Commit(usize),
    Compare { from: String, to: String },
    CompareWorking { hash: String },
}

#[derive(Debug, Clone)]
pub struct StatusNotice {
    pub id: u64,
    pub text: String,
    pub created_at: Instant,
}

pub const TOAST_DURATION: Duration = Duration::from_secs(5);
const MAX_TOASTS: usize = 5;

#[derive(Debug, Clone, Default)]
pub struct CreateBranchModal {
    pub open: bool,
    pub name: String,
    pub start_point: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RenameBranchModal {
    pub open: bool,
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct CreateTagModal {
    pub open: bool,
    pub name: String,
    pub message: String,
    pub target: String,
}

#[derive(Debug, Clone, Default)]
pub struct StashModal {
    pub open: bool,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ErrorModal {
    pub open: bool,
    pub raw_error: String,
    pub copied: bool,
}

pub struct GitKrakenApp {
    pub config: AppConfig,
    pub tabs: Vec<RepoTab>,
    pub active_tab: Option<usize>,
    pub repo: Option<GitRepo>,
    pub repo_path_input: String,
    pub current_branch: String,
    pub ahead_behind: Option<(usize, usize)>,
    pub remote_updates_available: bool,
    remote_check_rx: Option<mpsc::Receiver<(PathBuf, bool)>>,
    pub is_cherry_picking: bool,
    pub is_merging: bool,
    pub is_rebasing: bool,

    pub commits: Vec<Commit>,
    pub graph: CommitGraph,
    pub branches: Vec<BranchInfo>,
    pub tags: Vec<TagInfo>,
    pub stashes: Vec<StashEntry>,
    pub status_files: Vec<FileStatus>,
    pub working_stats: HashMap<String, (usize, usize)>,
    pub agent_diffs: HashMap<String, String>,
    pub agent_target: AgentTarget,
    pub agent_stash_diff: String,

    pub selection: Selection,
    pub compare_base: Option<String>,
    pub selected_commit_files: Vec<CommitFileChange>,
    pub selected_file: Option<String>,
    pub selected_file_is_staged: bool,
    pub current_diff: String,

    pub commit_message: String,
    pub amend_commit: bool,
    pub search_query: String,
    pub branch_search: String,
    pub search_match_pos: usize,
    pub focus_search: bool,
    pub branch_filter: Option<String>,

    pub toasts: Vec<StatusNotice>,
    next_toast_id: u64,
    pub branch_modal: CreateBranchModal,
    pub rename_modal: RenameBranchModal,
    pub tag_modal: CreateTagModal,
    pub stash_modal: StashModal,
    pub error_modal: ErrorModal,
    pub settings_open: bool,
    pub shortcuts_open: bool,
    pub about_open: bool,
    pub scroll_to_row: Option<usize>,
    pub logo_texture: Option<egui::TextureHandle>,
    did_initial_maximize: bool,
    _native_menu: Option<NativeMenuBar>,
}

impl GitKrakenApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_path: Option<PathBuf>) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        let config = AppConfig::load();

        let mut app = Self {
            config,
            tabs: Vec::new(),
            active_tab: None,
            repo: None,
            repo_path_input: String::new(),
            current_branch: String::new(),
            ahead_behind: None,
            remote_updates_available: false,
            remote_check_rx: None,
            is_cherry_picking: false,
            is_merging: false,
            is_rebasing: false,
            commits: Vec::new(),
            graph: CommitGraph {
                rows: Vec::new(),
                max_lanes: 1,
            },
            branches: Vec::new(),
            tags: Vec::new(),
            stashes: Vec::new(),
            status_files: Vec::new(),
            working_stats: HashMap::new(),
            agent_diffs: HashMap::new(),
            agent_target: AgentTarget::None,
            agent_stash_diff: String::new(),
            selection: Selection::None,
            compare_base: None,
            selected_commit_files: Vec::new(),
            selected_file: None,
            selected_file_is_staged: false,
            current_diff: String::new(),
            commit_message: String::new(),
            amend_commit: false,
            search_query: String::new(),
            branch_search: String::new(),
            search_match_pos: 0,
            focus_search: false,
            branch_filter: None,
            toasts: Vec::new(),
            next_toast_id: 1,
            branch_modal: CreateBranchModal::default(),
            rename_modal: RenameBranchModal::default(),
            tag_modal: CreateTagModal::default(),
            stash_modal: StashModal::default(),
            error_modal: ErrorModal::default(),
            settings_open: false,
            shortcuts_open: false,
            about_open: false,
            scroll_to_row: None,
            logo_texture: theme::load_logo_texture(&cc.egui_ctx),
            did_initial_maximize: false,
            _native_menu: NativeMenuBar::init(),
        };

        // Restore previously open tabs (lightweight metadata only)
        let saved_tabs = app.config.open_tabs.clone();
        for tab_path in saved_tabs {
            let p = PathBuf::from(&tab_path);
            if let Ok(r) = GitRepo::open(&p) {
                if !app.tabs.iter().any(|t| t.path == r.path) {
                    app.tabs.push(RepoTab {
                        name: r.repo_name(),
                        branch: r.current_branch(),
                        path: r.path,
                        branch_filter: None,
                        search_query: String::new(),
                    });
                }
            }
        }

        if let Some(path) = initial_path {
            app.open_repo(path);
        } else if !app.tabs.is_empty() {
            app.switch_tab(0);
        } else if let Ok(cwd) = std::env::current_dir() {
            if GitRepo::open(&cwd).is_ok() {
                app.open_repo(cwd);
            }
        }

        app
    }

    pub fn t(&self) -> &'static Strings {
        tr(self.config.language)
    }

    fn sync_tabs_to_config(&mut self) {
        self.config.open_tabs = self
            .tabs
            .iter()
            .map(|t| t.path.display().to_string())
            .collect();
        self.config.save();
    }

    fn save_active_tab_ui_state(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                tab.branch = self.current_branch.clone();
                tab.branch_filter = self.branch_filter.clone();
                tab.search_query = self.search_query.clone();
            }
        }
    }

    fn clear_active_repo_memory(&mut self) {
        self.commits.clear();
        self.commits.shrink_to_fit();
        self.graph = CommitGraph {
            rows: Vec::new(),
            max_lanes: 1,
        };
        self.branches.clear();
        self.tags.clear();
        self.stashes.clear();
        self.status_files.clear();
        self.working_stats.clear();
        self.agent_diffs.clear();
        self.agent_stash_diff.clear();
        self.agent_target = AgentTarget::None;
        self.selected_commit_files.clear();
        self.selected_file = None;
        self.current_diff.clear();
        self.compare_base = None;
        self.selection = Selection::None;
        self.remote_updates_available = false;
        self.remote_check_rx = None;
    }

    /// One-shot lightweight background check for remote updates (0% idle CPU, drops thread immediately).
    pub fn spawn_remote_check(&mut self, ctx: Option<egui::Context>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let (tx, rx) = mpsc::channel();
        self.remote_check_rx = Some(rx);
        std::thread::spawn(move || {
            let has_updates = repo.has_remote_updates();
            let _ = tx.send((repo.path, has_updates));
            if let Some(c) = ctx {
                c.request_repaint();
            }
        });
    }

    pub fn open_repo(&mut self, path: PathBuf) {
        match GitRepo::open(&path) {
            Ok(repo) => {
                // If already open in a tab, just switch to it
                if let Some(existing) = self.tabs.iter().position(|t| t.path == repo.path) {
                    self.switch_tab(existing);
                    return;
                }

                self.save_active_tab_ui_state();

                let path_str = repo.path.display().to_string();
                let tab = RepoTab {
                    name: repo.repo_name(),
                    branch: repo.current_branch(),
                    path: repo.path.clone(),
                    branch_filter: None,
                    search_query: String::new(),
                };
                self.tabs.push(tab);
                let new_idx = self.tabs.len() - 1;
                self.config.add_recent(&path_str);
                self.sync_tabs_to_config();
                self.switch_tab(new_idx);
            }
            Err(err) => {
                self.set_error(err);
            }
        }
    }

    pub fn switch_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        if self.active_tab == Some(idx) && self.repo.is_some() {
            return;
        }

        self.save_active_tab_ui_state();
        self.clear_active_repo_memory();

        let (tab_path, tab_filter, tab_search) = {
            let t = &self.tabs[idx];
            (t.path.clone(), t.branch_filter.clone(), t.search_query.clone())
        };

        match GitRepo::open(&tab_path) {
            Ok(repo) => {
                self.active_tab = Some(idx);
                self.repo_path_input = repo.path.display().to_string();
                self.branch_filter = tab_filter;
                self.search_query = tab_search;
                self.search_match_pos = 0;
                self.repo = Some(repo);
                self.refresh();
                self.spawn_remote_check(None);
                if !self.status_files.is_empty() {
                    self.select_wip();
                } else if !self.commits.is_empty() {
                    self.select_commit(0);
                }
            }
            Err(err) => {
                self.set_error(err);
                self.tabs.remove(idx);
                self.sync_tabs_to_config();
                if !self.tabs.is_empty() {
                    let next = idx.min(self.tabs.len() - 1);
                    self.active_tab = None;
                    self.switch_tab(next);
                } else {
                    self.active_tab = None;
                    self.repo = None;
                }
            }
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        let closing_active = self.active_tab == Some(idx);
        self.tabs.remove(idx);
        self.sync_tabs_to_config();

        if self.tabs.is_empty() {
            self.active_tab = None;
            self.repo = None;
            self.clear_active_repo_memory();
            return;
        }

        if closing_active {
            let next = idx.min(self.tabs.len() - 1);
            self.active_tab = None;
            self.switch_tab(next);
        } else if let Some(active) = self.active_tab {
            if idx < active {
                self.active_tab = Some(active - 1);
            }
        }
    }

    pub fn open_repo_dialog(&mut self) {
        let mut dialog = rfd::FileDialog::new().set_title("gity x — Open Git Repository");
        if let Some(parent) = self
            .repo
            .as_ref()
            .and_then(|r| r.path.parent())
            .filter(|p| p.exists())
        {
            dialog = dialog.set_directory(parent);
        } else if let Some(home) = std::env::var_os("HOME") {
            let code_dir = PathBuf::from(home).join("code");
            if code_dir.exists() {
                dialog = dialog.set_directory(code_dir);
            }
        }

        if let Some(folder) = dialog.pick_folder() {
            self.open_repo(folder);
        }
    }

    pub fn refresh(&mut self) {
        let Some(repo) = self.repo.clone() else {
            return;
        };

        self.current_branch = repo.current_branch();
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                tab.branch = self.current_branch.clone();
            }
        }
        self.ahead_behind = repo.ahead_behind();
        self.is_cherry_picking = repo.is_cherry_picking();
        self.is_merging = repo.is_merging();
        self.is_rebasing = repo.is_rebasing();
        self.branches = repo.load_branches();
        self.tags = repo.load_tags();
        self.stashes = repo.load_stashes();
        self.status_files = repo.load_status();
        self.reload_agent_cache(&repo);

        match repo.load_commits(
            self.config.commit_limit,
            self.branch_filter.as_deref(),
            self.config.show_remotes_in_graph,
            self.config.commit_order,
        ) {
            Ok(commits) => {
                self.graph = CommitGraph::build(&commits);
                self.commits = commits;
            }
            Err(err) => {
                self.set_error(err);
            }
        }

        match self.selection {
            Selection::Commit(idx) if idx >= self.commits.len() => {
                self.selection = if self.commits.is_empty() {
                    Selection::None
                } else {
                    Selection::Commit(0)
                };
            }
            Selection::Wip if self.status_files.is_empty() => {
                self.selection = if self.commits.is_empty() {
                    Selection::None
                } else {
                    Selection::Commit(0)
                };
            }
            _ => {}
        }

        self.reload_inspector_data();
    }

    pub fn search_matches(&self) -> Vec<usize> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        self.commits
            .iter()
            .enumerate()
            .filter_map(|(idx, commit)| {
                let hit = commit.subject.to_lowercase().contains(&query)
                    || commit.author.to_lowercase().contains(&query)
                    || commit.hash.to_lowercase().contains(&query)
                    || commit
                        .refs
                        .iter()
                        .any(|r| r.name.to_lowercase().contains(&query));
                hit.then_some(idx)
            })
            .collect()
    }

    pub fn jump_next_match(&mut self) {
        let matches = self.search_matches();
        if matches.is_empty() {
            return;
        }
        self.search_match_pos = (self.search_match_pos + 1) % matches.len();
        let commit_idx = matches[self.search_match_pos];
        self.select_commit(commit_idx);
        let wip_offset = usize::from(!self.status_files.is_empty());
        self.scroll_to_row = Some(commit_idx + wip_offset);
    }

    pub fn jump_prev_match(&mut self) {
        let matches = self.search_matches();
        if matches.is_empty() {
            return;
        }
        if self.search_match_pos == 0 {
            self.search_match_pos = matches.len() - 1;
        } else {
            self.search_match_pos -= 1;
        }
        let commit_idx = matches[self.search_match_pos];
        self.select_commit(commit_idx);
        let wip_offset = usize::from(!self.status_files.is_empty());
        self.scroll_to_row = Some(commit_idx + wip_offset);
    }

    pub fn select_wip(&mut self) {
        self.selection = Selection::Wip;
        self.config.show_right_sidebar = true;
        self.selected_commit_files.clear();
        self.selected_file = None;
        self.current_diff.clear();

        if let Some(first) = self.status_files.first().cloned() {
            let staged = first.is_staged() && !first.is_unstaged();
            let untracked = first.is_untracked();
            self.select_working_file(first.path, staged, untracked);
        }
    }

    pub fn select_commit(&mut self, idx: usize) {
        if idx >= self.commits.len() {
            return;
        }
        self.selection = Selection::Commit(idx);
        self.config.show_right_sidebar = true;
        self.selected_file = None;
        self.reload_inspector_data();
    }

    pub fn start_compare_with(&mut self, to_hash: String) {
        let Some(from_hash) = self.compare_base.clone() else {
            return;
        };
        if from_hash == to_hash {
            return;
        }
        self.selection = Selection::Compare {
            from: from_hash,
            to: to_hash,
        };
        self.config.show_right_sidebar = true;
        self.selected_file = None;
        self.reload_inspector_data();
    }

    pub fn start_compare_working(&mut self, hash: String) {
        self.selection = Selection::CompareWorking { hash };
        self.config.show_right_sidebar = true;
        self.selected_file = None;
        self.reload_inspector_data();
    }

    pub fn jump_to_hash(&mut self, hash: &str) {
        if let Some(idx) = self.commits.iter().position(|c| c.hash == hash) {
            self.select_commit(idx);
            let wip_offset = usize::from(!self.status_files.is_empty());
            self.scroll_to_row = Some(idx + wip_offset);
        }
    }

    pub fn select_commit_file(&mut self, file: Option<String>) {
        self.selected_file = file;
        let Some(repo) = self.repo.as_ref() else {
            return;
        };
        match &self.selection {
            Selection::Commit(idx) => {
                if let Some(commit) = self.commits.get(*idx) {
                    self.current_diff =
                        repo.commit_diff(&commit.hash, self.selected_file.as_deref());
                }
            }
            Selection::Compare { from, to } => {
                self.current_diff = repo.compare_diff(from, to, self.selected_file.as_deref());
            }
            Selection::CompareWorking { hash } => {
                self.current_diff =
                    repo.compare_with_working_diff(hash, self.selected_file.as_deref());
            }
            _ => {}
        }
    }

    pub fn select_working_file(&mut self, file: String, staged: bool, untracked: bool) {
        self.selected_file = Some(file.clone());
        self.selected_file_is_staged = staged;
        if let Some(repo) = self.repo.as_ref() {
            self.current_diff = repo.working_diff(&file, staged, untracked);
        }
    }

    fn reload_inspector_data(&mut self) {
        let Some(repo) = self.repo.as_ref() else {
            return;
        };
        match self.selection.clone() {
            Selection::Commit(idx) => {
                if let Some(commit) = self.commits.get(idx) {
                    let hash = commit.hash.clone();
                    self.selected_commit_files = repo.commit_files(&hash);
                    self.current_diff =
                        repo.commit_diff(&hash, self.selected_file.as_deref());
                }
            }
            Selection::Compare { from, to } => {
                self.selected_commit_files = repo.compare_files(&from, &to);
                self.current_diff = repo.compare_diff(&from, &to, self.selected_file.as_deref());
            }
            Selection::CompareWorking { hash } => {
                self.selected_commit_files = repo.compare_with_working_files(&hash);
                self.current_diff =
                    repo.compare_with_working_diff(&hash, self.selected_file.as_deref());
            }
            Selection::Wip => {
                if let Some(path) = self.selected_file.clone() {
                    if let Some(st) = self.status_files.iter().find(|f| f.path == path) {
                        self.current_diff = repo.working_diff(
                            &path,
                            self.selected_file_is_staged,
                            st.is_untracked(),
                        );
                    } else {
                        self.selected_file = None;
                        self.current_diff.clear();
                    }
                }
            }
            Selection::None => {
                self.selected_commit_files.clear();
                self.current_diff.clear();
            }
        }
    }

    pub fn set_info(&mut self, msg: impl Into<String>) {
        let id = self.next_toast_id;
        self.next_toast_id = self.next_toast_id.wrapping_add(1);
        self.toasts.push(StatusNotice {
            id,
            text: msg.into(),
            created_at: Instant::now(),
        });
        if self.toasts.len() > MAX_TOASTS {
            let overflow = self.toasts.len() - MAX_TOASTS;
            self.toasts.drain(0..overflow);
        }
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_modal.open = true;
        self.error_modal.raw_error = msg.into();
        self.error_modal.copied = false;
    }

    pub fn run_git_action<F>(&mut self, success_msg: &str, action: F)
    where
        F: FnOnce(&GitRepo) -> Result<String, String>,
    {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        match action(&repo) {
            Ok(_) => {
                self.set_info(success_msg);
                self.refresh();
            }
            Err(err) => {
                self.set_error(err);
                self.refresh();
            }
        }
    }

    pub fn has_wip(&self) -> bool {
        !self.status_files.is_empty()
    }

    pub fn open_branch_modal(&mut self, start_point: Option<String>) {
        self.branch_modal.open = true;
        self.branch_modal.name.clear();
        self.branch_modal.start_point = start_point;
    }

    pub fn open_rename_branch_modal(&mut self, old_name: String) {
        self.rename_modal.open = true;
        self.rename_modal.old_name = old_name.clone();
        self.rename_modal.new_name = old_name;
    }

    pub fn toggle_left_sidebar(&mut self) {
        self.config.show_left_sidebar = !self.config.show_left_sidebar;
        self.config.save();
    }

    pub fn toggle_right_sidebar(&mut self) {
        self.config.show_right_sidebar = !self.config.show_right_sidebar;
        self.config.save();
    }

    pub fn set_agent_mode(&mut self, enabled: bool) {
        if self.config.agent_mode != enabled {
            self.config.agent_mode = enabled;
            self.config.save();
            if enabled {
                self.refresh();
            }
        }
    }

    pub fn select_agent_file(&mut self, path: String, staged: bool, untracked: bool) {
        self.agent_target = AgentTarget::File {
            path,
            staged,
            untracked,
        };
    }

    pub fn select_agent_stash(&mut self, index: String, message: String) {
        if let Some(repo) = self.repo.as_ref() {
            self.agent_stash_diff = repo.stash_diff(&index);
        }
        self.agent_target = AgentTarget::Stash { index, message };
    }

    pub fn ensure_agent_feed_diffs(&mut self) {
        if !self.agent_diffs.is_empty() || self.status_files.is_empty() {
            return;
        }
        if let Some(repo) = self.repo.as_ref() {
            for f in &self.status_files {
                self.agent_diffs
                    .insert(f.path.clone(), repo.agent_file_diff(f));
            }
        }
    }

    fn reload_agent_cache(&mut self, repo: &GitRepo) {
        self.working_stats = repo.load_working_numstat(&self.status_files);
        self.agent_diffs.clear();
        if self.config.agent_mode && self.config.agent_all_files_feed {
            for f in &self.status_files {
                self.agent_diffs
                    .insert(f.path.clone(), repo.agent_file_diff(f));
            }
        }

        // Keep agent_target valid after refresh
        match &self.agent_target {
            AgentTarget::File { path, .. } => {
                if !self.status_files.iter().any(|f| &f.path == path) {
                    self.agent_target = AgentTarget::None;
                }
            }
            AgentTarget::Stash { index, .. } => {
                if self.stashes.iter().any(|s| &s.index == index) {
                    self.agent_stash_diff = repo.stash_diff(index);
                } else {
                    self.agent_target = AgentTarget::None;
                    self.agent_stash_diff.clear();
                }
            }
            AgentTarget::None => {}
        }

        if self.agent_target == AgentTarget::None {
            if let Some(first) = self.status_files.first() {
                self.agent_target = AgentTarget::File {
                    path: first.path.clone(),
                    staged: first.is_staged() && !first.is_unstaged(),
                    untracked: first.is_untracked(),
                };
            } else if let Some(st) = self.stashes.first() {
                self.agent_stash_diff = repo.stash_diff(&st.index);
                self.agent_target = AgentTarget::Stash {
                    index: st.index.clone(),
                    message: st.message.clone(),
                };
            }
        }
    }
}

impl GitKrakenApp {
    fn handle_native_menu_events(&mut self) {
        let t = self.t();
        for id in NativeMenuBar::poll_events() {
            match id.as_str() {
                native_menu::ID_ABOUT => {
                    self.about_open = true;
                }
                native_menu::ID_SETTINGS => {
                    self.settings_open = true;
                }
                native_menu::ID_SHORTCUTS => {
                    self.shortcuts_open = true;
                }
                native_menu::ID_OPEN_REPO | native_menu::ID_NEW_TAB => {
                    self.open_repo_dialog();
                }
                native_menu::ID_CLOSE_TAB => {
                    if let Some(idx) = self.active_tab {
                        self.close_tab(idx);
                    }
                }
                native_menu::ID_REFRESH => {
                    if self.repo.is_some() {
                        self.refresh();
                        self.spawn_remote_check(None);
                        self.set_info(t.msg_refreshed);
                    }
                }
                native_menu::ID_MODE_GRAPH => {
                    self.set_agent_mode(false);
                }
                native_menu::ID_MODE_AGENT => {
                    self.set_agent_mode(true);
                }
                native_menu::ID_TOGGLE_LEFT => {
                    self.toggle_left_sidebar();
                }
                native_menu::ID_TOGGLE_RIGHT => {
                    self.toggle_right_sidebar();
                }
                native_menu::ID_FOCUS_SEARCH => {
                    self.focus_search = true;
                }
                native_menu::ID_GIT_FETCH => {
                    self.remote_updates_available = false;
                    self.run_git_action(t.msg_fetched, |r| r.fetch());
                }
                native_menu::ID_GIT_PULL => {
                    self.remote_updates_available = false;
                    self.run_git_action(t.msg_pulled, |r| r.pull());
                }
                native_menu::ID_GIT_PUSH => {
                    self.run_git_action(t.msg_pushed, |r| r.push());
                }
                native_menu::ID_GIT_NEW_BRANCH => {
                    if self.repo.is_some() {
                        self.open_branch_modal(None);
                    }
                }
                native_menu::ID_GIT_STASH => {
                    if self.repo.is_some() {
                        self.stash_modal.open = true;
                        self.stash_modal.message.clear();
                    }
                }
                native_menu::ID_GIT_POP => {
                    self.run_git_action(t.msg_stash_popped, |r| r.stash_pop());
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for GitKrakenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.did_initial_maximize {
            self.did_initial_maximize = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }

        self.handle_native_menu_events();

        // Non-blocking O(1) receive of one-shot remote check result
        if let Some(rx) = &self.remote_check_rx {
            match rx.try_recv() {
                Ok((checked_path, has_updates)) => {
                    if self.repo.as_ref().map(|r| &r.path) == Some(&checked_path) {
                        self.remote_updates_available = has_updates;
                    }
                    self.remote_check_rx = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.remote_check_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        // Global shortcuts
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Comma)) {
            self.settings_open = !self.settings_open;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Slash)) {
            self.shortcuts_open = !self.shortcuts_open;
        }
        if ctx.input(|i| {
            i.modifiers.command
                && (i.key_pressed(egui::Key::O) || i.key_pressed(egui::Key::N))
        }) {
            self.open_repo_dialog();
        }
        if !self.tabs.is_empty() {
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::W)) {
                if let Some(idx) = self.active_tab {
                    self.close_tab(idx);
                }
            }
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Tab)) {
                if let Some(idx) = self.active_tab {
                    let next = (idx + 1) % self.tabs.len();
                    self.switch_tab(next);
                }
            }
            let num_keys = [
                egui::Key::Num1,
                egui::Key::Num2,
                egui::Key::Num3,
                egui::Key::Num4,
                egui::Key::Num5,
                egui::Key::Num6,
                egui::Key::Num7,
                egui::Key::Num8,
                egui::Key::Num9,
            ];
            for (i, key) in num_keys.iter().enumerate() {
                if i < self.tabs.len()
                    && ctx.input(|inp| inp.modifiers.command && inp.key_pressed(*key))
                {
                    self.switch_tab(i);
                    break;
                }
            }
        }

        if self.repo.is_some() {
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::R)) {
                self.refresh();
                self.spawn_remote_check(Some(ctx.clone()));
                let msg = self.t().msg_refreshed;
                self.set_info(msg);
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::B)) {
                self.toggle_left_sidebar();
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::I)) {
                self.toggle_right_sidebar();
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
                self.focus_search = true;
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::E)) {
                self.set_agent_mode(!self.config.agent_mode);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.branch_modal.open = false;
            self.rename_modal.open = false;
            self.tag_modal.open = false;
            self.stash_modal.open = false;
            self.error_modal.open = false;
            self.settings_open = false;
            self.shortcuts_open = false;
            self.about_open = false;
        }

        // Auto-dismiss expired floating toasts independently (single wakeup for soonest)
        self.toasts.retain(|t| t.created_at.elapsed() < TOAST_DURATION);
        if let Some(min_remaining) = self
            .toasts
            .iter()
            .filter_map(|t| TOAST_DURATION.checked_sub(t.created_at.elapsed()))
            .min()
        {
            ctx.request_repaint_after(min_remaining);
        }

        ui::toolbar::show_tabs(ctx, self);
        ui::toolbar::show(ctx, self);

        if self.repo.is_none() {
            let t = self.t();
            let recents = self.config.recent_repos.clone();

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(90.0);
                    if let Some(tex) = app_logo_image(self.logo_texture.as_ref(), 78.0, 16.0) {
                        ui.add(tex);
                        ui.add_space(12.0);
                    }
                    ui.heading(
                        egui::RichText::new("gity x")
                            .size(34.0)
                            .color(theme::ACCENT_CYAN)
                            .strong(),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(t.app_subtitle)
                            .size(15.0)
                            .color(theme::TEXT_SECONDARY),
                    );
                    ui.add_space(26.0);
                    if ui
                        .add_sized(
                            [240.0, 40.0],
                            egui::Button::new(
                                egui::RichText::new(t.open_repo_btn).size(15.0).strong(),
                            ),
                        )
                        .clicked()
                    {
                        self.open_repo_dialog();
                    }
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 420.0).max(0.0) * 0.5);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.repo_path_input)
                                .desired_width(330.0)
                                .hint_text(t.paste_repo_hint),
                        );
                        if ui.button(t.open).clicked() && !self.repo_path_input.trim().is_empty() {
                            let path = PathBuf::from(self.repo_path_input.trim());
                            self.open_repo(path);
                        }
                    });

                    if !recents.is_empty() {
                        ui.add_space(32.0);
                        ui.label(
                            egui::RichText::new(t.recent_repos)
                                .size(11.5)
                                .color(theme::TEXT_MUTED)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        for recent in recents {
                            if ui
                                .button(
                                    egui::RichText::new(&recent)
                                        .size(12.5)
                                        .color(theme::TEXT_PRIMARY),
                                )
                                .clicked()
                            {
                                self.open_repo(PathBuf::from(recent));
                            }
                        }
                    }
                });
            });
            ui::toolbar::show_modals(ctx, self);
            return;
        }

        if self.config.agent_mode {
            ui::agent_view::show(ctx, self);
        } else {
            ui::sidebar::show(ctx, self);
            ui::inspector::show(ctx, self);
            ui::graph_view::show(ctx, self);
        }
        ui::toolbar::show_modals(ctx, self);
    }
}

pub fn app_logo_image<'a>(
    tex: Option<&'a egui::TextureHandle>,
    size: f32,
    rounding: f32,
) -> Option<egui::Image<'a>> {
    tex.map(|t| {
        egui::Image::new((t.id(), egui::vec2(size, size)))
            .rounding(egui::Rounding::same(rounding))
            .uv(egui::Rect::from_min_max(
                egui::pos2(0.11, 0.11),
                egui::pos2(0.89, 0.89),
            ))
    })
}
