use std::collections::BTreeMap;

use eframe::egui::{self, Color32, Margin, RichText, Stroke};

use crate::app::{GitKrakenApp, Selection};
use crate::git::CommitFileChange;
use crate::i18n::format_human_date;
use crate::theme;
use crate::ui::diff_view;

pub fn show(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.config.show_right_sidebar {
        return;
    }

    let t = app.t();
    let frame = egui::Frame::none()
        .fill(theme::BG_PANEL)
        .inner_margin(Margin::symmetric(12.0, 8.0))
        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE));

    let mut collapse = false;

    let screen_rect = ctx.screen_rect();
    let max_w = (screen_rect.width() - 420.0).clamp(320.0, 960.0);
    let resize_id = egui::Id::new("right_inspector_custom_resize");

    // Read previous frame's drag before rendering so SidePanel & CentralPanel stay in lockstep
    if let Some(prev) = ctx.read_response(resize_id) {
        if prev.dragged() {
            if let Some(pos) = prev.interact_pointer_pos() {
                app.config.inspector_width = (screen_rect.right() - pos.x).clamp(280.0, max_w);
            }
        }
        if prev.drag_stopped() {
            app.config.save();
        }
    }

    let width = app.config.inspector_width.clamp(280.0, max_w);

    let panel_resp = egui::SidePanel::right("right_inspector")
        .resizable(false)
        .exact_width(width)
        .frame(frame)
        .show(ctx, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.horizontal(|ui| {
                if ui
                    .small_button("»")
                    .on_hover_text(t.toggle_right_sidebar)
                    .clicked()
                {
                    collapse = true;
                }
                ui.label(
                    RichText::new("INSPECTOR")
                        .strong()
                        .size(11.0)
                        .color(theme::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(RichText::new("⛶").color(theme::ACCENT_CYAN))
                        .on_hover_text(t.maximize_changes)
                        .clicked()
                    {
                        app.set_agent_mode(true);
                    }
                });
            });
            ui.separator();

            match app.selection.clone() {
                Selection::None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(120.0);
                        ui.label(
                            RichText::new(t.select_prompt)
                                .color(theme::TEXT_MUTED)
                                .size(13.5),
                        );
                    });
                }
                Selection::Wip => show_wip_inspector(ui, app),
                Selection::Commit(idx) => show_commit_inspector(ui, app, idx),
                Selection::Compare { from, to } => show_compare_inspector(ui, app, &from, &to),
                Selection::CompareWorking { hash } => {
                    show_compare_working_inspector(ui, app, &hash)
                }
            }
        });

    // Dedicated foreground 6px splitter handle along the left border of the inspector
    let panel_rect = panel_resp.response.rect;
    egui::Area::new(egui::Id::new("right_inspector_resize_area"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(panel_rect.left() - 3.0, panel_rect.top()))
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(6.0, panel_rect.height()),
                egui::Sense::hover(),
            );
            let resp = ui.interact(rect, resize_id, egui::Sense::drag());
            if resp.hovered() || resp.dragged() {
                ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                let col = if resp.dragged() {
                    theme::ACCENT_CYAN
                } else {
                    theme::BORDER_ELEVATED
                };
                ui.painter().line_segment(
                    [
                        egui::pos2(panel_rect.left(), panel_rect.top()),
                        egui::pos2(panel_rect.left(), panel_rect.bottom()),
                    ],
                    Stroke::new(2.0_f32, col),
                );
            }
        });

    if collapse {
        app.toggle_right_sidebar();
    }
}

fn show_wip_inspector(ui: &mut egui::Ui, app: &mut GitKrakenApp) {
    let t = app.t();

    ui.label(
        RichText::new(t.wip_title)
            .strong()
            .size(15.0)
            .color(theme::ACCENT_AMBER),
    );
    ui.separator();

    let unstaged: Vec<_> = app
        .status_files
        .iter()
        .filter(|f| f.is_unstaged())
        .cloned()
        .collect();
    let staged: Vec<_> = app
        .status_files
        .iter()
        .filter(|f| f.is_staged())
        .cloned()
        .collect();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{} ({})", t.unstaged, unstaged.len()))
                .strong()
                .size(11.5)
                .color(theme::TEXT_SECONDARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !unstaged.is_empty() && ui.small_button(t.stage_all).clicked() {
                app.run_git_action("All files staged.", |r| r.stage_all());
            }
        });
    });

    let files_max_w = ui.available_width();
    egui::ScrollArea::vertical()
        .id_salt("unstaged_files_scroll")
        .max_width(files_max_w)
        .max_height(115.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.set_max_width(files_max_w);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            if unstaged.is_empty() {
                ui.label(
                    RichText::new(t.no_pending_files)
                        .size(12.0)
                        .color(theme::TEXT_MUTED)
                        .italics(),
                );
            }
            for file in &unstaged {
                let is_sel = app.selected_file.as_deref() == Some(file.path.as_str())
                    && !app.selected_file_is_staged;
                ui.horizontal(|ui| {
                    let badge = if file.is_untracked() {
                        '?'
                    } else {
                        file.worktree_status
                    };
                    ui.label(
                        RichText::new(badge.to_string())
                            .monospace()
                            .strong()
                            .color(status_char_color(badge)),
                    );

                    if ui
                        .selectable_label(is_sel, RichText::new(&file.path).size(12.0))
                        .clicked()
                    {
                        app.select_working_file(file.path.clone(), false, file.is_untracked());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("×")
                            .on_hover_text(t.discard_hover)
                            .clicked()
                        {
                            let path = file.path.clone();
                            let untracked = file.is_untracked();
                            app.run_git_action(&format!("Discarded: {path}"), |r| {
                                r.discard_file(&path, untracked)
                            });
                        }
                        if ui.small_button("+").on_hover_text(t.stage_hover).clicked() {
                            let path = file.path.clone();
                            app.run_git_action(&format!("Staged: {path}"), |r| r.stage_file(&path));
                        }
                    });
                });
            }
        });

    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{} ({})", t.staged, staged.len()))
                .strong()
                .size(11.5)
                .color(theme::ACCENT_GREEN),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !staged.is_empty() && ui.small_button(t.unstage_all).clicked() {
                app.run_git_action("All files unstaged.", |r| r.unstage_all());
            }
        });
    });

    let staged_max_w = ui.available_width();
    egui::ScrollArea::vertical()
        .id_salt("staged_files_scroll")
        .max_width(staged_max_w)
        .max_height(115.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.set_max_width(staged_max_w);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            if staged.is_empty() {
                ui.label(
                    RichText::new(t.no_staged_files)
                        .size(12.0)
                        .color(theme::TEXT_MUTED)
                        .italics(),
                );
            }
            for file in &staged {
                let is_sel = app.selected_file.as_deref() == Some(file.path.as_str())
                    && app.selected_file_is_staged;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(file.index_status.to_string())
                            .monospace()
                            .strong()
                            .color(status_char_color(file.index_status)),
                    );

                    if ui
                        .selectable_label(is_sel, RichText::new(&file.path).size(12.0))
                        .clicked()
                    {
                        app.select_working_file(file.path.clone(), true, false);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("-")
                            .on_hover_text(t.unstage_hover)
                            .clicked()
                        {
                            let path = file.path.clone();
                            app.run_git_action(&format!("Unstaged: {path}"), |r| {
                                r.unstage_file(&path)
                            });
                        }
                    });
                });
            }
        });

    ui.add_space(6.0);
    ui.separator();

    ui.label(
        RichText::new(t.commit_msg_label)
            .strong()
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add(
        egui::TextEdit::multiline(&mut app.commit_message)
            .desired_rows(3)
            .desired_width(f32::INFINITY)
            .hint_text(t.commit_msg_hint),
    );

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let can_commit =
            (!staged.is_empty() || app.amend_commit) && !app.commit_message.trim().is_empty();
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
            let ok = app
                .repo
                .as_ref()
                .map(|r| r.commit(&msg, amend))
                .unwrap_or_else(|| Err("No repository open".to_string()));
            match ok {
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

    ui.add_space(6.0);
    ui.separator();
    ui.label(
        RichText::new(
            app.selected_file
                .as_deref()
                .map(|f| format!("DIFF — {f}"))
                .unwrap_or_else(|| "DIFF".to_string()),
        )
        .strong()
        .size(11.5)
        .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(4.0);
    diff_view::show(ui, &app.current_diff, t.no_diff);
}

fn show_commit_inspector(ui: &mut egui::Ui, app: &mut GitKrakenApp, idx: usize) {
    let t = app.t();
    let Some(commit) = app.commits.get(idx).cloned() else {
        return;
    };

    ui.label(
        RichText::new(&commit.subject)
            .strong()
            .size(15.0)
            .color(Color32::WHITE),
    );

    if !commit.body.is_empty() {
        ui.add_space(4.0);
        let body_w = ui.available_width();
        egui::ScrollArea::vertical()
            .id_salt("commit_body_scroll")
            .max_width(body_w)
            .max_height(150.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_max_width(body_w);
                ui.label(
                    RichText::new(&commit.body)
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                );
            });
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(&commit.short_hash)
                .monospace()
                .color(theme::ACCENT_CYAN)
                .strong(),
        );
        ui.label(RichText::new("•").color(theme::TEXT_MUTED));
        ui.add(
            egui::Label::new(
                RichText::new(&commit.author)
                    .size(12.0)
                    .color(theme::TEXT_PRIMARY),
            )
            .truncate(),
        );
    });
    if !commit.email.is_empty() {
        ui.add(
            egui::Label::new(
                RichText::new(&commit.email)
                    .size(11.5)
                    .color(theme::TEXT_SECONDARY),
            )
            .truncate(),
        );
    }
    let pretty_date = format_human_date(&commit.date, app.config.language);
    ui.add(
        egui::Label::new(
            RichText::new(format!("{pretty_date} ({})", commit.relative_date))
                .size(11.5)
                .color(theme::TEXT_MUTED),
        )
        .truncate(),
    );

    if !commit.parents.is_empty() {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(t.parents)
                    .size(11.5)
                    .color(theme::TEXT_MUTED),
            );
            for p in &commit.parents {
                let short = &p[..p.len().min(7)];
                if ui
                    .small_button(RichText::new(short).monospace().color(theme::ACCENT_CYAN))
                    .clicked()
                {
                    app.jump_to_hash(p);
                }
            }
        });
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui
            .small_button(
                RichText::new(format!("+ {}", t.cherry_pick))
                    .color(theme::ACCENT_GREEN)
                    .strong(),
            )
            .clicked()
        {
            let hash = commit.hash.clone();
            app.run_git_action(t.msg_cherry_picked, |r| r.cherry_pick(&hash));
        }
        if ui.small_button(t.checkout).clicked() {
            let hash = commit.hash.clone();
            app.run_git_action(&format!("Checked out {}.", commit.short_hash), |r| {
                r.checkout(&hash)
            });
        }
        if ui.small_button(t.revert).clicked() {
            let hash = commit.hash.clone();
            app.run_git_action(t.msg_reverted, |r| r.revert_commit(&hash));
        }
    });
    ui.horizontal(|ui| {
        if ui.small_button(t.branch_here).clicked() {
            app.open_branch_modal(Some(commit.short_hash.clone()));
        }
        if ui.small_button(t.tag_here).clicked() {
            app.tag_modal.open = true;
            app.tag_modal.name.clear();
            app.tag_modal.message.clear();
            app.tag_modal.target = commit.short_hash.clone();
        }
        if let Some(base) = app.compare_base.as_deref() {
            if base != commit.hash.as_str() {
                if ui
                    .small_button(
                        RichText::new(t.compare_with_selected).color(theme::ACCENT_PURPLE),
                    )
                    .clicked()
                {
                    app.start_compare_with(commit.hash.clone());
                }
            } else if ui.small_button(t.clear_compare).clicked() {
                app.compare_base = None;
            }
        } else if ui.small_button(t.compare_select).clicked() {
            app.compare_base = Some(commit.hash.clone());
        }
        if ui.small_button("vs WIP").clicked() {
            app.start_compare_working(commit.hash.clone());
        }
    });

    ui.add_space(6.0);
    ui.separator();

    show_files_and_diff(ui, app);
}

fn show_compare_inspector(ui: &mut egui::Ui, app: &mut GitKrakenApp, from: &str, to: &str) {
    let t = app.t();
    let short_from = &from[..from.len().min(7)];
    let short_to = &to[..to.len().min(7)];

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{}: {short_from} → {short_to}", t.comparing_commits))
                .strong()
                .size(14.0)
                .color(theme::ACCENT_PURPLE),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(t.clear_compare).clicked() {
                app.compare_base = None;
                if !app.commits.is_empty() {
                    app.select_commit(0);
                } else {
                    app.selection = Selection::None;
                }
            }
        });
    });
    ui.separator();

    show_files_and_diff(ui, app);
}

fn show_compare_working_inspector(ui: &mut egui::Ui, app: &mut GitKrakenApp, hash: &str) {
    let t = app.t();
    let short = &hash[..hash.len().min(7)];

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{}: {short} → WIP", t.comparing_working))
                .strong()
                .size(14.0)
                .color(theme::ACCENT_AMBER),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(t.clear_compare).clicked() {
                if !app.commits.is_empty() {
                    app.select_commit(0);
                } else {
                    app.selection = Selection::None;
                }
            }
        });
    });
    ui.separator();

    show_files_and_diff(ui, app);
}

fn show_files_and_diff(ui: &mut egui::Ui, app: &mut GitKrakenApp) {
    let t = app.t();
    let files = app.selected_commit_files.clone();
    let total_add: usize = files.iter().filter_map(|f| f.additions).sum();
    let total_del: usize = files.iter().filter_map(|f| f.deletions).sum();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{} ({})", t.changed_files, files.len()))
                .strong()
                .size(11.5)
                .color(theme::TEXT_SECONDARY),
        );
        if total_add > 0 || total_del > 0 {
            ui.label(
                RichText::new(format!("+{total_add}"))
                    .monospace()
                    .size(11.0)
                    .color(theme::ACCENT_GREEN),
            );
            ui.label(
                RichText::new(format!("-{total_del}"))
                    .monospace()
                    .size(11.0)
                    .color(theme::ACCENT_ROSE),
            );
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let toggle_lbl = if app.config.file_view_tree {
                t.view_list
            } else {
                t.view_tree
            };
            if ui.small_button(toggle_lbl).clicked() {
                app.config.file_view_tree = !app.config.file_view_tree;
                app.config.save();
            }
            if app.selected_file.is_some() && ui.small_button(t.view_all).clicked() {
                app.select_commit_file(None);
            }
        });
    });

    let mut picked_file: Option<String> = None;

    let commit_files_max_w = ui.available_width();
    egui::ScrollArea::vertical()
        .id_salt("commit_files_scroll")
        .max_width(commit_files_max_w)
        .max_height(155.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.set_max_width(commit_files_max_w);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            if app.config.file_view_tree {
                // Group changed files by parent directory (Git Graph tree style)
                let mut groups: BTreeMap<String, Vec<&CommitFileChange>> = BTreeMap::new();
                for change in &files {
                    let (dir, _) = match change.path.rsplit_once('/') {
                        Some((d, f)) => (d.to_string(), f),
                        None => (String::new(), change.path.as_str()),
                    };
                    groups.entry(dir).or_default().push(change);
                }

                for (dir, dir_files) in groups {
                    if dir.is_empty() {
                        for change in dir_files {
                            if draw_file_change_row(
                                ui,
                                change,
                                &change.path,
                                app.selected_file.as_deref() == Some(change.path.as_str()),
                            ) {
                                picked_file = Some(change.path.clone());
                            }
                        }
                    } else {
                        egui::CollapsingHeader::new(
                            RichText::new(format!("{dir}/"))
                                .size(11.5)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .id_salt(format!("dir_{dir}"))
                        .default_open(true)
                        .show(ui, |ui| {
                            for change in dir_files {
                                let fname = change
                                    .path
                                    .rsplit_once('/')
                                    .map(|(_, f)| f)
                                    .unwrap_or(&change.path);
                                if draw_file_change_row(
                                    ui,
                                    change,
                                    fname,
                                    app.selected_file.as_deref() == Some(change.path.as_str()),
                                ) {
                                    picked_file = Some(change.path.clone());
                                }
                            }
                        });
                    }
                }
            } else {
                for change in &files {
                    if draw_file_change_row(
                        ui,
                        change,
                        &change.path,
                        app.selected_file.as_deref() == Some(change.path.as_str()),
                    ) {
                        picked_file = Some(change.path.clone());
                    }
                }
            }
        });

    if let Some(f) = picked_file {
        app.select_commit_file(Some(f));
    }

    ui.add_space(6.0);
    ui.separator();
    ui.label(
        RichText::new(
            app.selected_file
                .as_deref()
                .map(|f| format!("DIFF — {f}"))
                .unwrap_or_else(|| t.full_diff.to_string()),
        )
        .strong()
        .size(11.5)
        .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(4.0);
    diff_view::show(ui, &app.current_diff, t.no_diff);
}

fn draw_file_change_row(
    ui: &mut egui::Ui,
    change: &CommitFileChange,
    display_name: &str,
    is_sel: bool,
) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        let first_ch = change.status.chars().next().unwrap_or('M');
        ui.label(
            RichText::new(&change.status)
                .monospace()
                .size(11.5)
                .strong()
                .color(status_char_color(first_ch)),
        );
        if ui
            .selectable_label(is_sel, RichText::new(display_name).size(12.0))
            .clicked()
        {
            clicked = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(del) = change.deletions {
                if del > 0 {
                    ui.label(
                        RichText::new(format!("-{del}"))
                            .monospace()
                            .size(11.0)
                            .color(theme::ACCENT_ROSE),
                    );
                }
            }
            if let Some(add) = change.additions {
                if add > 0 {
                    ui.label(
                        RichText::new(format!("+{add}"))
                            .monospace()
                            .size(11.0)
                            .color(theme::ACCENT_GREEN),
                    );
                }
            }
        });
    });
    clicked
}

fn status_char_color(ch: char) -> Color32 {
    match ch {
        'A' | '?' => theme::ACCENT_GREEN,
        'D' => theme::ACCENT_ROSE,
        'R' | 'C' => theme::ACCENT_PURPLE,
        _ => theme::ACCENT_AMBER,
    }
}
