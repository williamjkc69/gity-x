mod app;
mod config;
mod git;
mod graph;
mod i18n;
mod native_menu;
mod theme;
mod ui;

use std::path::PathBuf;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let initial_path = std::env::args().nth(1).map(PathBuf::from);

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1460.0, 900.0])
        .with_min_inner_size([980.0, 620.0])
        .with_maximized(true)
        .with_title("gity x");

    if let Some(icon) = theme::load_window_icon() {
        viewport = viewport.with_icon(icon);
    }

    #[cfg(target_os = "macos")]
    {
        viewport = viewport
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "gity x",
        options,
        Box::new(|cc| Ok(Box::new(app::GitKrakenApp::new(cc, initial_path)))),
    )
}
