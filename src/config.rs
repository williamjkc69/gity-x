use std::path::PathBuf;

use crate::i18n::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphStyle {
    Rounded,
    Angular,
}

impl GraphStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rounded => "rounded",
            Self::Angular => "angular",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "angular" => Self::Angular,
            _ => Self::Rounded,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitOrder {
    Topo,
    Date,
    AuthorDate,
}

impl CommitOrder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Topo => "topo",
            Self::Date => "date",
            Self::AuthorDate => "author_date",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "date" => Self::Date,
            "author_date" => Self::AuthorDate,
            _ => Self::Topo,
        }
    }

    pub fn git_flag(self) -> &'static str {
        match self {
            Self::Topo => "--topo-order",
            Self::Date => "--date-order",
            Self::AuthorDate => "--author-date-order",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub language: Language,
    pub commit_limit: usize,
    pub show_remotes_in_graph: bool,
    pub compact_rows: bool,
    pub show_left_sidebar: bool,
    pub show_right_sidebar: bool,
    pub inspector_width: f32,
    pub graph_style: GraphStyle,
    pub commit_order: CommitOrder,
    pub file_view_tree: bool,
    pub agent_mode: bool,
    pub agent_split_diff: bool,
    pub agent_all_files_feed: bool,
    pub open_tabs: Vec<String>,
    pub recent_repos: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::En,
            commit_limit: 500,
            show_remotes_in_graph: true,
            compact_rows: false,
            show_left_sidebar: true,
            show_right_sidebar: true,
            inspector_width: 440.0,
            graph_style: GraphStyle::Rounded,
            commit_order: CommitOrder::Topo,
            file_view_tree: true,
            agent_mode: false,
            agent_split_diff: true,
            agent_all_files_feed: false,
            open_tabs: Vec::new(),
            recent_repos: Vec::new(),
        }
    }
}

impl AppConfig {
    fn config_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return Some(PathBuf::from(appdata).join("gity-x").join("config.txt"));
        }

        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(|h| {
                PathBuf::from(h)
                    .join(".config")
                    .join("gity-x")
                    .join("config.txt")
            })
    }

    pub fn load() -> Self {
        let mut cfg = Self::default();
        let Some(path) = Self::config_path() else {
            return cfg;
        };
        let Ok(content) = std::fs::read_to_string(path) else {
            return cfg;
        };

        for line in content.lines() {
            if let Some((key, val)) = line.split_once('=') {
                match key.trim() {
                    "language" => cfg.language = Language::from_code(val),
                    "commit_limit" => {
                        if let Ok(n) = val.trim().parse::<usize>() {
                            cfg.commit_limit = n.clamp(50, 5000);
                        }
                    }
                    "show_remotes_in_graph" => {
                        cfg.show_remotes_in_graph = val.trim() == "true";
                    }
                    "compact_rows" => {
                        cfg.compact_rows = val.trim() == "true";
                    }
                    "show_left_sidebar" => {
                        cfg.show_left_sidebar = val.trim() == "true";
                    }
                    "show_right_sidebar" => {
                        cfg.show_right_sidebar = val.trim() == "true";
                    }
                    "inspector_width" => {
                        if let Ok(w) = val.trim().parse::<f32>() {
                            cfg.inspector_width = w.clamp(280.0, 960.0);
                        }
                    }
                    "graph_style" => {
                        cfg.graph_style = GraphStyle::from_str(val);
                    }
                    "commit_order" => {
                        cfg.commit_order = CommitOrder::from_str(val);
                    }
                    "file_view_tree" => {
                        cfg.file_view_tree = val.trim() == "true";
                    }
                    "agent_mode" => {
                        cfg.agent_mode = val.trim() == "true";
                    }
                    "agent_split_diff" => {
                        cfg.agent_split_diff = val.trim() == "true";
                    }
                    "agent_all_files_feed" => {
                        cfg.agent_all_files_feed = val.trim() == "true";
                    }
                    "tab" => {
                        let p = val.trim().to_string();
                        if !p.is_empty() && !cfg.open_tabs.contains(&p) {
                            cfg.open_tabs.push(p);
                        }
                    }
                    "recent" => {
                        let p = val.trim().to_string();
                        if !p.is_empty() && !cfg.recent_repos.contains(&p) {
                            cfg.recent_repos.push(p);
                        }
                    }
                    _ => {}
                }
            }
        }
        cfg
    }

    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut out = String::new();
        out.push_str(&format!("language={}\n", self.language.code()));
        out.push_str(&format!("commit_limit={}\n", self.commit_limit));
        out.push_str(&format!(
            "show_remotes_in_graph={}\n",
            self.show_remotes_in_graph
        ));
        out.push_str(&format!("compact_rows={}\n", self.compact_rows));
        out.push_str(&format!("show_left_sidebar={}\n", self.show_left_sidebar));
        out.push_str(&format!("show_right_sidebar={}\n", self.show_right_sidebar));
        out.push_str(&format!("inspector_width={:.0}\n", self.inspector_width));
        out.push_str(&format!("graph_style={}\n", self.graph_style.as_str()));
        out.push_str(&format!("commit_order={}\n", self.commit_order.as_str()));
        out.push_str(&format!("file_view_tree={}\n", self.file_view_tree));
        out.push_str(&format!("agent_mode={}\n", self.agent_mode));
        out.push_str(&format!("agent_split_diff={}\n", self.agent_split_diff));
        out.push_str(&format!(
            "agent_all_files_feed={}\n",
            self.agent_all_files_feed
        ));
        for t in self.open_tabs.iter().take(12) {
            out.push_str(&format!("tab={t}\n"));
        }
        for r in self.recent_repos.iter().take(8) {
            out.push_str(&format!("recent={r}\n"));
        }
        let _ = std::fs::write(path, out);
    }

    pub fn add_recent(&mut self, repo_path: &str) {
        self.recent_repos.retain(|r| r != repo_path);
        self.recent_repos.insert(0, repo_path.to_string());
        self.recent_repos.truncate(8);
        self.save();
    }
}
