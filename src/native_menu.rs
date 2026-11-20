#[cfg(target_os = "macos")]
use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};

pub const ID_ABOUT: &str = "app_about";
pub const ID_SETTINGS: &str = "app_settings";
pub const ID_SHORTCUTS: &str = "app_shortcuts";

pub const ID_OPEN_REPO: &str = "file_open_repo";
pub const ID_NEW_TAB: &str = "file_new_tab";
pub const ID_CLOSE_TAB: &str = "file_close_tab";
pub const ID_REFRESH: &str = "file_refresh";

pub const ID_MODE_GRAPH: &str = "view_mode_graph";
pub const ID_MODE_AGENT: &str = "view_mode_agent";
pub const ID_MODE_PRS: &str = "view_mode_prs";
pub const ID_TOGGLE_LEFT: &str = "view_toggle_left";
pub const ID_TOGGLE_RIGHT: &str = "view_toggle_right";
pub const ID_FOCUS_SEARCH: &str = "view_focus_search";

pub const ID_GIT_FETCH: &str = "git_fetch";
pub const ID_GIT_PULL: &str = "git_pull";
pub const ID_GIT_PUSH: &str = "git_push";
pub const ID_GIT_NEW_BRANCH: &str = "git_new_branch";
pub const ID_GIT_STASH: &str = "git_stash";
pub const ID_GIT_POP: &str = "git_pop";

pub struct NativeMenuBar {
    #[cfg(target_os = "macos")]
    _menu: Menu,
}

impl NativeMenuBar {
    #[cfg(not(target_os = "macos"))]
    pub fn init() -> Option<Self> {
        None
    }

    #[cfg(target_os = "macos")]
    pub fn init() -> Option<Self> {
        let menu = Menu::new();
        let cmd = Modifiers::SUPER;

        let app_menu = Submenu::new("gity x", true);
        let _ = app_menu.append_items(&[
            &MenuItem::with_id(ID_ABOUT, "About gity x", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(
                ID_SETTINGS,
                "Settings...",
                true,
                Some(Accelerator::new(Some(cmd), Code::Comma)),
            ),
            &MenuItem::with_id(
                ID_SHORTCUTS,
                "Keyboard Shortcuts",
                true,
                Some(Accelerator::new(Some(cmd), Code::Slash)),
            ),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(Some("Quit gity x")),
        ]);

        let file_menu = Submenu::new("File", true);
        let _ = file_menu.append_items(&[
            &MenuItem::with_id(
                ID_OPEN_REPO,
                "Open Repository...",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyO)),
            ),
            &MenuItem::with_id(
                ID_NEW_TAB,
                "New Tab...",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyN)),
            ),
            &MenuItem::with_id(
                ID_CLOSE_TAB,
                "Close Tab",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyW)),
            ),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(
                ID_REFRESH,
                "Refresh Repository",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyR)),
            ),
        ]);

        let edit_menu = Submenu::new("Edit", true);
        let _ = edit_menu.append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::select_all(None),
        ]);

        let view_menu = Submenu::new("View", true);
        let _ = view_menu.append_items(&[
            &MenuItem::with_id(ID_MODE_GRAPH, "Git Graph Mode", true, None),
            &MenuItem::with_id(
                ID_MODE_AGENT,
                "Agent Mode",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyE)),
            ),
            &MenuItem::with_id(
                ID_MODE_PRS,
                "Pull Requests",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyP)),
            ),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(
                ID_TOGGLE_LEFT,
                "Toggle Left Sidebar",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyB)),
            ),
            &MenuItem::with_id(
                ID_TOGGLE_RIGHT,
                "Toggle Right Inspector",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyI)),
            ),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(
                ID_FOCUS_SEARCH,
                "Find / Filter",
                true,
                Some(Accelerator::new(Some(cmd), Code::KeyF)),
            ),
        ]);

        let git_menu = Submenu::new("Git", true);
        let _ = git_menu.append_items(&[
            &MenuItem::with_id(ID_GIT_FETCH, "Fetch", true, None),
            &MenuItem::with_id(ID_GIT_PULL, "Pull", true, None),
            &MenuItem::with_id(ID_GIT_PUSH, "Push", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(ID_GIT_NEW_BRANCH, "New Branch From...", true, None),
            &MenuItem::with_id(ID_GIT_STASH, "Stash Changes...", true, None),
            &MenuItem::with_id(ID_GIT_POP, "Pop Stash", true, None),
        ]);

        let help_menu = Submenu::new("Help", true);
        let _ = help_menu.append_items(&[
            &MenuItem::with_id(ID_SHORTCUTS, "Keyboard Shortcuts", true, None),
            &MenuItem::with_id(ID_ABOUT, "About gity x", true, None),
        ]);

        let _ = menu.append_items(&[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &git_menu,
            &help_menu,
        ]);

        menu.init_for_nsapp();

        Some(Self { _menu: menu })
    }

    pub fn poll_events() -> Vec<String> {
        #[cfg(target_os = "macos")]
        {
            let mut events = Vec::new();
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                events.push(event.id.0);
            }
            events
        }
        #[cfg(not(target_os = "macos"))]
        {
            Vec::new()
        }
    }
}
