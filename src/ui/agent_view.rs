use eframe::egui::{
    self, Align, Color32, FontId, Layout, Margin, Pos2, Rect, RichText, Rounding, Stroke, Vec2,
};

use crate::app::{app_logo_image, AgentTarget, GitKrakenApp};
use crate::git::FileStatus;
use crate::theme;

#[derive(Debug, Clone)]
enum SplitRow {
    Hunk(String),
    Pair {
        left_ln: Option<usize>,
        left_text: Option<String>,
        left_removed: bool,
        right_ln: Option<usize>,
        right_text: Option<String>,
        right_added: bool,
    },
}

#[derive(Debug, Clone)]
enum UnifiedRow {
    Hunk(String),
    Line {
        old_ln: Option<usize>,
        new_ln: Option<usize>,
        prefix: char,
        text: String,
    },
}

pub fn show(ctx: &egui::Context, app: &mut GitKrakenApp) {
    let t = app.t();

    let q = app.search_query.trim().to_lowercase();
    let all_files = app.status_files.clone();
    let unstaged_tracked: Vec<FileStatus> = all_files
        .iter()
        .filter(|f| {
            f.is_unstaged()
                && !f.is_untracked()
                && (q.is_empty() || f.path.to_lowercase().contains(&q))
        })
        .cloned()
        .collect();
    let untracked: Vec<FileStatus> = all_files
        .iter()
        .filter(|f| f.is_untracked() && (q.is_empty() || f.path.to_lowercase().contains(&q)))
        .cloned()
        .collect();
    let staged: Vec<FileStatus> = all_files
        .iter()
        .filter(|f| f.is_staged() && (q.is_empty() || f.path.to_lowercase().contains(&q)))
        .cloned()
        .collect();
    let stashes: Vec<_> = app
        .stashes
        .iter()
        .filter(|s| {
            q.is_empty()
                || s.message.to_lowercase().contains(&q)
                || s.index.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    let total_add: usize = all_files
        .iter()
        .filter_map(|f| app.working_stats.get(&f.path).map(|s| s.0))
        .sum();
    let total_del: usize = all_files
        .iter()
        .filter_map(|f| app.working_stats.get(&f.path).map(|s| s.1))
        .sum();

    // Collapsible Left Panel: Unstaged, Untracked, Staged, Stashes + Commit Box
    if app.config.show_left_sidebar {
        let mut collapse_left = false;
        let frame = egui::Frame::none()
            .fill(theme::BG_PANEL)
            .inner_margin(Margin::symmetric(10.0, 10.0))
            .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE));

        egui::SidePanel::left("agent_changes_sidebar")
            .default_width(310.0)
            .min_width(220.0)
            .max_width(480.0)
            .frame(frame)
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("◆ {}", t.agent_review_title))
                            .strong()
                            .size(13.5)
                            .color(theme::ACCENT_CYAN),
                    );
                    if total_add > 0 {
                        ui.label(
                            RichText::new(format!("+{total_add}"))
                                .monospace()
                                .size(11.0)
                                .color(theme::ACCENT_GREEN),
                        );
                    }
                    if total_del > 0 {
                        ui.label(
                            RichText::new(format!("-{total_del}"))
                                .monospace()
                                .size(11.0)
                                .color(theme::ACCENT_ROSE),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .small_button("«")
                            .on_hover_text(t.toggle_left_sidebar)
                            .clicked()
                        {
                            collapse_left = true;
                        }
                    });
                });
                ui.add_space(6.0);

                // Search bar right above changed files & stashes
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").size(12.0).color(theme::TEXT_MUTED));
                    let w = (ui.available_width()
                        - if app.search_query.is_empty() { 4.0 } else { 26.0 })
                    .max(80.0);
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut app.search_query)
                            .desired_width(w)
                            .hint_text(format!(
                                "Filter files, stashes... ({}+F)",
                                crate::i18n::MOD_KEY
                            )),
                    );
                    if app.focus_search {
                        resp.request_focus();
                        app.focus_search = false;
                    }
                    if !app.search_query.is_empty() && ui.small_button("×").clicked() {
                        app.search_query.clear();
                    }
                });

                ui.add_space(4.0);
                ui.separator();

                // Bottom Commit Panel
                egui::TopBottomPanel::bottom("agent_commit_box")
                    .frame(
                        egui::Frame::none()
                            .fill(theme::BG_PANEL)
                            .inner_margin(Margin::symmetric(0.0, 6.0)),
                    )
                    .show_inside(ui, |ui| {
                        ui.separator();
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::multiline(&mut app.commit_message)
                                .desired_rows(2)
                                .desired_width(f32::INFINITY)
                                .hint_text(t.agent_quick_commit_hint),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let can_commit = (!staged.is_empty() || app.amend_commit)
                                && !app.commit_message.trim().is_empty();
                            let btn_label = if app.amend_commit {
                                t.amend_btn.to_string()
                            } else {
                                format!("{} ({})", t.commit_btn, staged.len())
                            };
                            if ui
                                .add_enabled(
                                    can_commit,
                                    egui::Button::new(RichText::new(btn_label).strong()),
                                )
                                .clicked()
                            {
                                let msg = app.commit_message.clone();
                                let amend = app.amend_commit;
                                let res = app
                                    .repo
                                    .as_ref()
                                    .map(|r| r.commit(&msg, amend))
                                    .unwrap_or_else(|| Err("No repository open".to_string()));
                                match res {
                                    Ok(_) => {
                                        app.commit_message.clear();
                                        app.amend_commit = false;
                                        app.set_info(t.msg_committed);
                                        app.refresh();
                                    }
                                    Err(e) => app.set_error(e),
                                }
                            }
                            ui.checkbox(&mut app.amend_commit, t.amend_last);
                        });
                    });

                // Scrollable Sections: Unstaged, Untracked, Staged, Stashes
                egui::ScrollArea::vertical()
                    .id_salt("agent_sections_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // 1. UNSTAGED (Modified / Deleted)
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} ({})",
                                    t.unstaged,
                                    unstaged_tracked.len()
                                ))
                                .strong()
                                .size(11.5)
                                .color(theme::ACCENT_AMBER),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if (!unstaged_tracked.is_empty() || !untracked.is_empty())
                                    && ui.small_button(t.stage_all).clicked()
                                {
                                    app.run_git_action("All files staged.", |r| r.stage_all());
                                }
                            });
                        });
                        if unstaged_tracked.is_empty() {
                            ui.label(
                                RichText::new(t.no_pending_files)
                                    .size(11.5)
                                    .color(theme::TEXT_MUTED)
                                    .italics(),
                            );
                        } else {
                            for file in &unstaged_tracked {
                                draw_sidebar_file_row(ui, app, file, false);
                            }
                        }

                        ui.add_space(10.0);

                        // 2. UNTRACKED
                        egui::CollapsingHeader::new(
                            RichText::new(format!("{} ({})", t.agent_untracked, untracked.len()))
                                .strong()
                                .size(11.5)
                                .color(theme::ACCENT_CYAN),
                        )
                        .default_open(!untracked.is_empty())
                        .show(ui, |ui| {
                            if untracked.is_empty() {
                                ui.label(
                                    RichText::new("—")
                                        .size(11.5)
                                        .color(theme::TEXT_MUTED),
                                );
                            } else {
                                for file in &untracked {
                                    draw_sidebar_file_row(ui, app, file, false);
                                }
                            }
                        });

                        ui.add_space(10.0);

                        // 3. STAGED
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{} ({})", t.staged, staged.len()))
                                    .strong()
                                    .size(11.5)
                                    .color(theme::ACCENT_GREEN),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if !staged.is_empty() && ui.small_button(t.unstage_all).clicked() {
                                    app.run_git_action("All files unstaged.", |r| {
                                        r.unstage_all()
                                    });
                                }
                            });
                        });
                        if staged.is_empty() {
                            ui.label(
                                RichText::new(t.no_staged_files)
                                    .size(11.5)
                                    .color(theme::TEXT_MUTED)
                                    .italics(),
                            );
                        } else {
                            for file in &staged {
                                draw_sidebar_file_row(ui, app, file, true);
                            }
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // 4. STASHES
                        egui::CollapsingHeader::new(
                            RichText::new(format!("{} ({})", t.stashes, stashes.len()))
                                .strong()
                                .size(11.5)
                                .color(theme::ACCENT_PURPLE),
                        )
                        .default_open(!stashes.is_empty())
                        .show(ui, |ui| {
                            if stashes.is_empty() {
                                ui.label(
                                    RichText::new(t.no_stashes)
                                        .size(11.5)
                                        .color(theme::TEXT_MUTED)
                                        .italics(),
                                );
                            } else {
                                for stash in &stashes {
                                    let is_sel = matches!(
                                        &app.agent_target,
                                        AgentTarget::Stash { index, .. } if index == &stash.index
                                    );
                                    ui.horizontal(|ui| {
                                        if ui
                                            .selectable_label(
                                                is_sel,
                                                RichText::new(format!(
                                                    "{}: {}",
                                                    stash.index, stash.message
                                                ))
                                                .size(12.0),
                                            )
                                            .clicked()
                                        {
                                            app.select_agent_stash(
                                                stash.index.clone(),
                                                stash.message.clone(),
                                            );
                                        }
                                    });
                                }
                            }
                        });
                    });
            });

        if collapse_left {
            app.toggle_left_sidebar();
        }
    }

    // Maximized Central Diff Area
    let center_frame = egui::Frame::none()
        .fill(theme::BG_PRIMARY)
        .inner_margin(Margin::same(0.0));

    egui::CentralPanel::default()
        .frame(center_frame)
        .show(ctx, |ui| {
            let target = app.agent_target.clone();

            if target == AgentTarget::None && all_files.is_empty() && stashes.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    if let Some(img) = app_logo_image(app.logo_texture.as_ref(), 58.0, 12.0) {
                        ui.add(img);
                    } else {
                        ui.label(RichText::new("◆").size(42.0).color(theme::ACCENT_CYAN));
                    }
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(t.agent_clean_title)
                            .size(20.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(t.agent_clean_sub)
                            .size(13.5)
                            .color(theme::TEXT_SECONDARY),
                    );
                });
                return;
            }

            // Header bar for active diff
            let bar_frame = egui::Frame::none()
                .fill(theme::BG_PANEL)
                .inner_margin(Margin::symmetric(14.0, 8.0))
                .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE));

            bar_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    if app.config.agent_all_files_feed && !all_files.is_empty() {
                        ui.label(
                            RichText::new(format!(
                                "{} ({} files)",
                                t.agent_all_files,
                                all_files.len()
                            ))
                            .strong()
                            .size(13.5)
                            .color(Color32::WHITE),
                        );
                    } else {
                        match &target {
                            AgentTarget::File {
                                path,
                                staged: is_staged,
                                untracked: is_untracked,
                            } => {
                                let badge = if *is_untracked {
                                    ("?", theme::ACCENT_CYAN)
                                } else if *is_staged {
                                    ("STAGED", theme::ACCENT_GREEN)
                                } else {
                                    ("M", theme::ACCENT_AMBER)
                                };
                                ui.label(
                                    RichText::new(badge.0)
                                        .monospace()
                                        .strong()
                                        .size(12.0)
                                        .color(badge.1),
                                );
                                ui.label(
                                    RichText::new(path)
                                        .strong()
                                        .size(13.5)
                                        .color(Color32::WHITE),
                                );
                                let (add, del) =
                                    app.working_stats.get(path).copied().unwrap_or((0, 0));
                                if add > 0 {
                                    ui.label(
                                        RichText::new(format!("+{add}"))
                                            .monospace()
                                            .size(12.0)
                                            .color(theme::ACCENT_GREEN),
                                    );
                                }
                                if del > 0 {
                                    ui.label(
                                        RichText::new(format!("-{del}"))
                                            .monospace()
                                            .size(12.0)
                                            .color(theme::ACCENT_ROSE),
                                    );
                                }
                            }
                            AgentTarget::Stash { index, message } => {
                                ui.label(
                                    RichText::new(index)
                                        .monospace()
                                        .strong()
                                        .size(12.5)
                                        .color(theme::ACCENT_PURPLE),
                                );
                                ui.label(
                                    RichText::new(message)
                                        .strong()
                                        .size(13.5)
                                        .color(Color32::WHITE),
                                );
                            }
                            AgentTarget::None => {
                                ui.label(
                                    RichText::new(t.agent_review_title)
                                        .strong()
                                        .color(theme::TEXT_SECONDARY),
                                );
                            }
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Split / Unified toggle
                        if ui
                            .selectable_label(!app.config.agent_split_diff, t.agent_unified)
                            .clicked()
                        {
                            app.config.agent_split_diff = false;
                            app.config.save();
                        }
                        if ui
                            .selectable_label(app.config.agent_split_diff, t.agent_split)
                            .clicked()
                        {
                            app.config.agent_split_diff = true;
                            app.config.save();
                        }

                        ui.separator();

                        // Focused vs All Files toggle
                        if ui
                            .selectable_label(app.config.agent_all_files_feed, t.agent_all_files)
                            .clicked()
                        {
                            app.config.agent_all_files_feed = true;
                            app.config.save();
                            app.ensure_agent_feed_diffs();
                        }
                        if ui
                            .selectable_label(!app.config.agent_all_files_feed, t.agent_single_file)
                            .clicked()
                        {
                            app.config.agent_all_files_feed = false;
                            app.config.save();
                        }

                        if !app.config.agent_all_files_feed {
                            ui.separator();
                            match &target {
                                AgentTarget::File {
                                    path,
                                    staged: is_staged,
                                    untracked: is_untracked,
                                } => {
                                    if !*is_staged {
                                        if ui
                                            .button(
                                                RichText::new(format!("↺ {}", t.agent_revert))
                                                    .color(theme::ACCENT_ROSE),
                                            )
                                            .clicked()
                                        {
                                            let p = path.clone();
                                            let u = *is_untracked;
                                            app.run_git_action(&format!("Discarded {p}."), |r| {
                                                r.discard_file(&p, u)
                                            });
                                        }
                                        if ui
                                            .button(
                                                RichText::new(format!("+ {}", t.agent_accept))
                                                    .color(theme::ACCENT_GREEN)
                                                    .strong(),
                                            )
                                            .clicked()
                                        {
                                            let p = path.clone();
                                            app.run_git_action(&format!("Staged {p}."), |r| {
                                                r.stage_file(&p)
                                            });
                                        }
                                    } else if ui
                                        .button(
                                            RichText::new(format!("- {}", t.agent_accepted))
                                                .color(theme::ACCENT_AMBER),
                                        )
                                        .clicked()
                                    {
                                        let p = path.clone();
                                        app.run_git_action(&format!("Unstaged {p}."), |r| {
                                            r.unstage_file(&p)
                                        });
                                    }
                                }
                                AgentTarget::Stash { index, .. } => {
                                    if ui
                                        .button(
                                            RichText::new(t.stash_drop).color(theme::ACCENT_ROSE),
                                        )
                                        .clicked()
                                    {
                                        let idx = index.clone();
                                        app.run_git_action(&format!("Dropped {idx}."), |r| {
                                            r.stash_drop_ref(&idx)
                                        });
                                    }
                                    if ui.button(t.stash_pop).clicked() {
                                        let idx = index.clone();
                                        app.run_git_action(&format!("Popped {idx}."), |r| {
                                            r.stash_pop_ref(&idx)
                                        });
                                    }
                                    if ui
                                        .button(
                                            RichText::new(t.stash_apply)
                                                .color(theme::ACCENT_GREEN),
                                        )
                                        .clicked()
                                    {
                                        let idx = index.clone();
                                        app.run_git_action(&format!("Applied {idx}."), |r| {
                                            r.stash_apply_ref(&idx)
                                        });
                                    }
                                }
                                AgentTarget::None => {}
                            }
                        }
                    });
                });
            });

            // Central Diff Content
            if app.config.agent_all_files_feed && !all_files.is_empty() {
                show_all_files_feed(ui, app, &all_files);
            } else {
                let diff_text = match &target {
                    AgentTarget::File {
                        path,
                        staged: is_staged,
                        untracked: is_untracked,
                    } => app
                        .repo
                        .as_ref()
                        .map(|r| r.working_diff(path, *is_staged, *is_untracked))
                        .unwrap_or_default(),
                    AgentTarget::Stash { .. } => app.agent_stash_diff.clone(),
                    AgentTarget::None => String::new(),
                };

                egui::ScrollArea::vertical()
                    .id_salt("agent_single_diff_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if app.config.agent_split_diff {
                            render_split_diff(ui, &diff_text, t.no_diff);
                        } else {
                            render_unified_diff(ui, &diff_text, t.no_diff);
                        }
                    });
            }
        });
}

fn draw_sidebar_file_row(
    ui: &mut egui::Ui,
    app: &mut GitKrakenApp,
    file: &FileStatus,
    in_staged_section: bool,
) {
    let t = app.t();
    let is_sel = matches!(
        &app.agent_target,
        AgentTarget::File { path, staged, .. } if path == &file.path && *staged == in_staged_section
    );
    let (add, del) = app
        .working_stats
        .get(&file.path)
        .copied()
        .unwrap_or((0, 0));

    ui.horizontal(|ui| {
        let badge = if file.is_untracked() {
            '?'
        } else if in_staged_section {
            file.index_status
        } else {
            file.worktree_status
        };
        let badge_col = match badge {
            'A' => theme::ACCENT_GREEN,
            '?' => theme::ACCENT_CYAN,
            'D' => theme::ACCENT_ROSE,
            'R' => theme::ACCENT_PURPLE,
            _ => theme::ACCENT_AMBER,
        };

        ui.label(
            RichText::new(badge.to_string())
                .monospace()
                .size(11.5)
                .strong()
                .color(badge_col),
        );

        let name = file
            .path
            .rsplit_once('/')
            .map(|(_, f)| f)
            .unwrap_or(&file.path);

        if ui
            .selectable_label(is_sel, RichText::new(name).size(12.0))
            .on_hover_text(&file.path)
            .clicked()
        {
            app.select_agent_file(file.path.clone(), in_staged_section, file.is_untracked());
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if in_staged_section {
                if ui
                    .small_button("-")
                    .on_hover_text(t.unstage_hover)
                    .clicked()
                {
                    let p = file.path.clone();
                    app.run_git_action(&format!("Unstaged {p}."), |r| r.unstage_file(&p));
                }
            } else {
                if ui
                    .small_button("×")
                    .on_hover_text(t.discard_hover)
                    .clicked()
                {
                    let p = file.path.clone();
                    let u = file.is_untracked();
                    app.run_git_action(&format!("Discarded {p}."), |r| r.discard_file(&p, u));
                }
                if ui.small_button("+").on_hover_text(t.stage_hover).clicked() {
                    let p = file.path.clone();
                    app.run_git_action(&format!("Staged {p}."), |r| r.stage_file(&p));
                }
            }

            if del > 0 {
                ui.label(
                    RichText::new(format!("-{del}"))
                        .monospace()
                        .size(10.5)
                        .color(theme::ACCENT_ROSE),
                );
            }
            if add > 0 {
                ui.label(
                    RichText::new(format!("+{add}"))
                        .monospace()
                        .size(10.5)
                        .color(theme::ACCENT_GREEN),
                );
            }
        });
    });
}

fn show_all_files_feed(ui: &mut egui::Ui, app: &mut GitKrakenApp, files: &[FileStatus]) {
    let t = app.t();
    let mut stage_path: Option<String> = None;
    let mut unstage_path: Option<String> = None;
    let mut discard_target: Option<(String, bool)> = None;

    egui::ScrollArea::vertical()
        .id_salt("agent_feed_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::none()
                .inner_margin(Margin::symmetric(14.0, 12.0))
                .show(ui, |ui| {
                    for file in files {
                        let (add, del) = app
                            .working_stats
                            .get(&file.path)
                            .copied()
                            .unwrap_or((0, 0));
                        let is_accepted = file.is_staged() && !file.is_unstaged();
                        let (badge, badge_col) = file_status_badge(file);

                        egui::Frame::none()
                            .fill(theme::BG_PANEL)
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                            .inner_margin(Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(badge)
                                            .monospace()
                                            .strong()
                                            .color(badge_col),
                                    );
                                    ui.label(
                                        RichText::new(&file.path)
                                            .strong()
                                            .size(13.0)
                                            .color(Color32::WHITE),
                                    );
                                    if add > 0 {
                                        ui.label(
                                            RichText::new(format!("+{add}"))
                                                .monospace()
                                                .size(11.5)
                                                .color(theme::ACCENT_GREEN),
                                        );
                                    }
                                    if del > 0 {
                                        ui.label(
                                            RichText::new(format!("-{del}"))
                                                .monospace()
                                                .size(11.5)
                                                .color(theme::ACCENT_ROSE),
                                        );
                                    }

                                    ui.with_layout(
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            if ui
                                                .small_button(
                                                    RichText::new(format!("↺ {}", t.agent_revert))
                                                        .color(theme::ACCENT_ROSE),
                                                )
                                                .clicked()
                                            {
                                                discard_target = Some((
                                                    file.path.clone(),
                                                    file.is_untracked(),
                                                ));
                                            }
                                            if is_accepted {
                                                if ui
                                                    .small_button(
                                                        RichText::new(format!(
                                                            "- {}",
                                                            t.agent_accepted
                                                        ))
                                                        .color(theme::ACCENT_AMBER),
                                                    )
                                                    .clicked()
                                                {
                                                    unstage_path = Some(file.path.clone());
                                                }
                                            } else if ui
                                                .small_button(
                                                    RichText::new(format!("+ {}", t.agent_accept))
                                                        .color(theme::ACCENT_GREEN),
                                                )
                                                .clicked()
                                            {
                                                stage_path = Some(file.path.clone());
                                            }
                                        },
                                    );
                                });

                                ui.separator();

                                let diff_text = app
                                    .agent_diffs
                                    .get(&file.path)
                                    .map(String::as_str)
                                    .unwrap_or("");
                                if app.config.agent_split_diff {
                                    render_split_diff(ui, diff_text, t.no_diff);
                                } else {
                                    render_unified_diff(ui, diff_text, t.no_diff);
                                }
                            });

                        ui.add_space(12.0);
                    }
                });
        });

    if let Some(p) = stage_path {
        app.run_git_action(&format!("Staged {p}."), |r| r.stage_file(&p));
    }
    if let Some(p) = unstage_path {
        app.run_git_action(&format!("Unstaged {p}."), |r| r.unstage_file(&p));
    }
    if let Some((p, untracked)) = discard_target {
        app.run_git_action(&format!("Discarded {p}."), |r| r.discard_file(&p, untracked));
    }
}

fn render_split_diff(ui: &mut egui::Ui, diff_text: &str, empty_msg: &str) {
    let rows = parse_split_rows(diff_text);
    if rows.is_empty() {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(empty_msg).color(theme::TEXT_MUTED).italics());
        });
        return;
    }

    let total_w = ui.available_width().max(500.0);
    let half_w = total_w * 0.5;
    let gutter_w = 46.0;
    let row_h = 20.0;
    let font = FontId::monospace(12.0);
    let ln_font = FontId::monospace(11.0);

    ui.spacing_mut().item_spacing.y = 0.0;

    for (idx, row) in rows.iter().enumerate() {
        if idx >= 1800 {
            ui.add_space(6.0);
            ui.label(
                RichText::new("... (truncated at 1,800 diff rows)")
                    .color(theme::ACCENT_AMBER)
                    .italics(),
            );
            break;
        }

        let (_, rect) = ui.allocate_space(Vec2::new(total_w, row_h));
        let painter = ui.painter();
        let cy = rect.center().y;

        match row {
            SplitRow::Hunk(header) => {
                painter.rect_filled(rect, Rounding::ZERO, Color32::from_rgb(28, 38, 56));
                painter.text(
                    Pos2::new(rect.left() + 12.0, cy),
                    egui::Align2::LEFT_CENTER,
                    header,
                    FontId::monospace(11.5),
                    theme::DIFF_HUNK_FG,
                );
            }
            SplitRow::Pair {
                left_ln,
                left_text,
                left_removed,
                right_ln,
                right_text,
                right_added,
            } => {
                let left_rect =
                    Rect::from_min_max(rect.min, Pos2::new(rect.left() + half_w, rect.max.y));
                let right_rect =
                    Rect::from_min_max(Pos2::new(rect.left() + half_w, rect.top()), rect.max);

                let left_bg = if *left_removed {
                    Color32::from_rgb(56, 24, 34)
                } else if left_text.is_none() {
                    Color32::from_rgb(17, 20, 29)
                } else {
                    theme::BG_PRIMARY
                };
                let right_bg = if *right_added {
                    Color32::from_rgb(20, 52, 40)
                } else if right_text.is_none() {
                    Color32::from_rgb(17, 20, 29)
                } else {
                    theme::BG_PRIMARY
                };

                painter.rect_filled(left_rect, Rounding::ZERO, left_bg);
                painter.rect_filled(right_rect, Rounding::ZERO, right_bg);

                painter.line_segment(
                    [
                        Pos2::new(rect.left() + half_w, rect.top()),
                        Pos2::new(rect.left() + half_w, rect.bottom()),
                    ],
                    Stroke::new(1.0_f32, theme::BORDER_SUBTLE),
                );

                if let Some(ln) = left_ln {
                    painter.text(
                        Pos2::new(left_rect.left() + gutter_w - 8.0, cy),
                        egui::Align2::RIGHT_CENTER,
                        ln.to_string(),
                        ln_font.clone(),
                        theme::TEXT_MUTED,
                    );
                }
                if let Some(text) = left_text {
                    let clip = Rect::from_min_max(
                        Pos2::new(left_rect.left() + gutter_w, left_rect.top()),
                        Pos2::new(left_rect.right() - 6.0, left_rect.bottom()),
                    );
                    let fg = if *left_removed {
                        theme::DIFF_DEL_FG
                    } else {
                        theme::TEXT_PRIMARY
                    };
                    let prefix = if *left_removed { "- " } else { "  " };
                    painter.with_clip_rect(clip).text(
                        Pos2::new(clip.left() + 4.0, cy),
                        egui::Align2::LEFT_CENTER,
                        format!("{prefix}{text}"),
                        font.clone(),
                        fg,
                    );
                }

                if let Some(ln) = right_ln {
                    painter.text(
                        Pos2::new(right_rect.left() + gutter_w - 8.0, cy),
                        egui::Align2::RIGHT_CENTER,
                        ln.to_string(),
                        ln_font.clone(),
                        theme::TEXT_MUTED,
                    );
                }
                if let Some(text) = right_text {
                    let clip = Rect::from_min_max(
                        Pos2::new(right_rect.left() + gutter_w, right_rect.top()),
                        Pos2::new(right_rect.right() - 6.0, right_rect.bottom()),
                    );
                    let fg = if *right_added {
                        theme::DIFF_ADD_FG
                    } else {
                        theme::TEXT_PRIMARY
                    };
                    let prefix = if *right_added { "+ " } else { "  " };
                    painter.with_clip_rect(clip).text(
                        Pos2::new(clip.left() + 4.0, cy),
                        egui::Align2::LEFT_CENTER,
                        format!("{prefix}{text}"),
                        font.clone(),
                        fg,
                    );
                }
            }
        }
    }
}

fn render_unified_diff(ui: &mut egui::Ui, diff_text: &str, empty_msg: &str) {
    let rows = parse_unified_rows(diff_text);
    if rows.is_empty() {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(empty_msg).color(theme::TEXT_MUTED).italics());
        });
        return;
    }

    let total_w = ui.available_width().max(400.0);
    let row_h = 20.0;
    let font = FontId::monospace(12.0);
    let ln_font = FontId::monospace(11.0);

    ui.spacing_mut().item_spacing.y = 0.0;

    for (idx, row) in rows.iter().enumerate() {
        if idx >= 1800 {
            ui.add_space(6.0);
            ui.label(
                RichText::new("... (truncated at 1,800 diff lines)")
                    .color(theme::ACCENT_AMBER)
                    .italics(),
            );
            break;
        }

        let (_, rect) = ui.allocate_space(Vec2::new(total_w, row_h));
        let painter = ui.painter();
        let cy = rect.center().y;

        match row {
            UnifiedRow::Hunk(header) => {
                painter.rect_filled(rect, Rounding::ZERO, Color32::from_rgb(28, 38, 56));
                painter.text(
                    Pos2::new(rect.left() + 12.0, cy),
                    egui::Align2::LEFT_CENTER,
                    header,
                    FontId::monospace(11.5),
                    theme::DIFF_HUNK_FG,
                );
            }
            UnifiedRow::Line {
                old_ln,
                new_ln,
                prefix,
                text,
            } => {
                let (bg, fg) = match prefix {
                    '+' => (Color32::from_rgb(20, 52, 40), theme::DIFF_ADD_FG),
                    '-' => (Color32::from_rgb(56, 24, 34), theme::DIFF_DEL_FG),
                    _ => (theme::BG_PRIMARY, theme::TEXT_PRIMARY),
                };
                painter.rect_filled(rect, Rounding::ZERO, bg);

                if let Some(ln) = old_ln {
                    painter.text(
                        Pos2::new(rect.left() + 42.0, cy),
                        egui::Align2::RIGHT_CENTER,
                        ln.to_string(),
                        ln_font.clone(),
                        theme::TEXT_MUTED,
                    );
                }
                if let Some(ln) = new_ln {
                    painter.text(
                        Pos2::new(rect.left() + 86.0, cy),
                        egui::Align2::RIGHT_CENTER,
                        ln.to_string(),
                        ln_font.clone(),
                        theme::TEXT_MUTED,
                    );
                }

                let code_clip = Rect::from_min_max(
                    Pos2::new(rect.left() + 96.0, rect.top()),
                    Pos2::new(rect.right() - 8.0, rect.bottom()),
                );
                painter.with_clip_rect(code_clip).text(
                    Pos2::new(code_clip.left() + 4.0, cy),
                    egui::Align2::LEFT_CENTER,
                    format!("{prefix} {text}"),
                    font.clone(),
                    fg,
                );
            }
        }
    }
}

fn parse_hunk_starts(header: &str) -> (usize, usize) {
    let mut old_start = 1;
    let mut new_start = 1;
    for part in header.split_whitespace() {
        if let Some(rest) = part.strip_prefix('-') {
            if let Some(n) = rest.split(',').next().and_then(|s| s.parse::<usize>().ok()) {
                old_start = n;
            }
        } else if let Some(rest) = part.strip_prefix('+') {
            if let Some(n) = rest.split(',').next().and_then(|s| s.parse::<usize>().ok()) {
                new_start = n;
            }
        }
    }
    (old_start, new_start)
}

fn parse_split_rows(diff_text: &str) -> Vec<SplitRow> {
    let mut rows = Vec::new();
    let mut old_ln = 1usize;
    let mut new_ln = 1usize;
    let mut in_hunk = false;

    let mut del_buf: Vec<(usize, String)> = Vec::new();
    let mut add_buf: Vec<(usize, String)> = Vec::new();

    let flush_buffers = |rows: &mut Vec<SplitRow>,
                         del_buf: &mut Vec<(usize, String)>,
                         add_buf: &mut Vec<(usize, String)>| {
        let max_len = del_buf.len().max(add_buf.len());
        for i in 0..max_len {
            let left = del_buf.get(i);
            let right = add_buf.get(i);
            rows.push(SplitRow::Pair {
                left_ln: left.map(|x| x.0),
                left_text: left.map(|x| x.1.clone()),
                left_removed: left.is_some(),
                right_ln: right.map(|x| x.0),
                right_text: right.map(|x| x.1.clone()),
                right_added: right.is_some(),
            });
        }
        del_buf.clear();
        add_buf.clear();
    };

    for line in diff_text.lines() {
        if line.starts_with("@@") {
            flush_buffers(&mut rows, &mut del_buf, &mut add_buf);
            let (o, n) = parse_hunk_starts(line);
            old_ln = o;
            new_ln = n;
            in_hunk = true;
            rows.push(SplitRow::Hunk(line.to_string()));
            continue;
        }
        if !in_hunk {
            continue;
        }
        if let Some(rest) = line.strip_prefix('-') {
            del_buf.push((old_ln, rest.to_string()));
            old_ln += 1;
        } else if let Some(rest) = line.strip_prefix('+') {
            add_buf.push((new_ln, rest.to_string()));
            new_ln += 1;
        } else if let Some(rest) = line.strip_prefix(' ') {
            flush_buffers(&mut rows, &mut del_buf, &mut add_buf);
            rows.push(SplitRow::Pair {
                left_ln: Some(old_ln),
                left_text: Some(rest.to_string()),
                left_removed: false,
                right_ln: Some(new_ln),
                right_text: Some(rest.to_string()),
                right_added: false,
            });
            old_ln += 1;
            new_ln += 1;
        }
    }
    flush_buffers(&mut rows, &mut del_buf, &mut add_buf);
    rows
}

fn parse_unified_rows(diff_text: &str) -> Vec<UnifiedRow> {
    let mut rows = Vec::new();
    let mut old_ln = 1usize;
    let mut new_ln = 1usize;
    let mut in_hunk = false;

    for line in diff_text.lines() {
        if line.starts_with("@@") {
            let (o, n) = parse_hunk_starts(line);
            old_ln = o;
            new_ln = n;
            in_hunk = true;
            rows.push(UnifiedRow::Hunk(line.to_string()));
            continue;
        }
        if !in_hunk {
            continue;
        }
        if let Some(rest) = line.strip_prefix('-') {
            rows.push(UnifiedRow::Line {
                old_ln: Some(old_ln),
                new_ln: None,
                prefix: '-',
                text: rest.to_string(),
            });
            old_ln += 1;
        } else if let Some(rest) = line.strip_prefix('+') {
            rows.push(UnifiedRow::Line {
                old_ln: None,
                new_ln: Some(new_ln),
                prefix: '+',
                text: rest.to_string(),
            });
            new_ln += 1;
        } else if let Some(rest) = line.strip_prefix(' ') {
            rows.push(UnifiedRow::Line {
                old_ln: Some(old_ln),
                new_ln: Some(new_ln),
                prefix: ' ',
                text: rest.to_string(),
            });
            old_ln += 1;
            new_ln += 1;
        }
    }
    rows
}

fn file_status_badge(file: &FileStatus) -> (&'static str, Color32) {
    if file.is_untracked() {
        ("?", theme::ACCENT_CYAN)
    } else if file.index_status == 'A' {
        ("A", theme::ACCENT_GREEN)
    } else if file.worktree_status == 'D' || file.index_status == 'D' {
        ("D", theme::ACCENT_ROSE)
    } else if file.index_status == 'R' {
        ("R", theme::ACCENT_PURPLE)
    } else {
        ("M", theme::ACCENT_AMBER)
    }
}
