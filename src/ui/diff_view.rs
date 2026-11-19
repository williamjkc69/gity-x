use eframe::egui::{self, Color32, FontId, RichText};

use crate::theme;

pub fn show(ui: &mut egui::Ui, diff_text: &str, empty_msg: &str) {
    if diff_text.trim().is_empty() {
        ui.label(
            RichText::new(empty_msg)
                .color(theme::TEXT_MUTED)
                .italics(),
        );
        return;
    }

    let max_w = ui.available_width();
    egui::ScrollArea::both()
        .id_salt("diff_scroll_area")
        .max_width(max_w)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

            for (idx, line) in diff_text.lines().enumerate() {
                if idx >= 1500 {
                    ui.label(
                        RichText::new("... (diff truncated at 1,500 lines — select a single file to view full diff)")
                            .color(theme::ACCENT_AMBER)
                            .italics(),
                    );
                    break;
                }
                let (fg, bg) = classify_diff_line(line);
                let mut rt = RichText::new(line)
                    .font(FontId::monospace(12.0))
                    .color(fg);
                if let Some(bg_col) = bg {
                    rt = rt.background_color(bg_col);
                }
                ui.label(rt);
            }
        });
}

fn classify_diff_line(line: &str) -> (Color32, Option<Color32>) {
    if line.starts_with("+++") || line.starts_with("---") || line.starts_with("diff --git") {
        (theme::TEXT_SECONDARY, None)
    } else if line.starts_with("@@") {
        (theme::DIFF_HUNK_FG, None)
    } else if line.starts_with('+') {
        (theme::DIFF_ADD_FG, Some(theme::DIFF_ADD_BG))
    } else if line.starts_with('-') {
        (theme::DIFF_DEL_FG, Some(theme::DIFF_DEL_BG))
    } else {
        (theme::TEXT_PRIMARY, None)
    }
}
