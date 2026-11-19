use eframe::egui::{
    self, Align2, Color32, FontId, Margin, Pos2, Rect, RichText, Rounding, Sense, Stroke, Vec2,
};
use eframe::epaint::CubicBezierShape;

use crate::app::{GitKrakenApp, Selection};
use crate::config::GraphStyle;
use crate::git::{Commit, RefKind, ResetMode};
use crate::graph::{GraphEdge, GraphRow};
use crate::i18n::{format_human_date, Strings};
use crate::theme;

const LANE_SPACING: f32 = 18.0;
const GRAPH_LEFT_PAD: f32 = 16.0;
const AUTHOR_COL_WIDTH: f32 = 128.0;
const DATE_COL_WIDTH: f32 = 102.0;
const HASH_COL_WIDTH: f32 = 72.0;

enum GraphAction {
    Checkout(String),
    CherryPick(String),
    Revert(String),
    Merge(String),
    Rebase(String),
    Reset(String, ResetMode),
    CreateBranch(String),
    CreateTag(String),
    SelectCompareBase(String),
    CompareWithBase(String),
    CompareWorking(String),
    ClearCompare,
    CopyHash(String),
    CopyMessage(String),
}

pub fn show(ctx: &egui::Context, app: &mut GitKrakenApp) {
    let t = app.t();
    let row_height = if app.config.compact_rows { 28.0 } else { 34.0 };
    let graph_style = app.config.graph_style;

    let right_gap = if app.config.show_right_sidebar { 6.0 } else { 0.0 };
    let frame = egui::Frame::none()
        .fill(theme::BG_PRIMARY)
        .inner_margin(Margin {
            left: 0.0,
            right: right_gap,
            top: 0.0,
            bottom: 0.0,
        });

    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        // Sleek Search & Branch Filter bar directly above git history
        draw_search_and_filter_bar(ui, app);

        let has_wip = app.has_wip();
        let wip_offset = usize::from(has_wip);
        let total_rows = app.commits.len() + wip_offset;

        let graph_col_width =
            ((app.graph.max_lanes.clamp(2, 16) as f32) * LANE_SPACING + GRAPH_LEFT_PAD * 2.0)
                .clamp(76.0, 320.0);

        draw_header_bar(ui, graph_col_width, t);

        if total_rows == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.label(
                    RichText::new(t.no_commits)
                        .color(theme::TEXT_MUTED)
                        .size(15.0),
                );
            });
            return;
        }

        let head_lane = app
            .commits
            .iter()
            .enumerate()
            .find(|(_, c)| c.is_head)
            .and_then(|(i, _)| app.graph.rows.get(i))
            .map(|r| (r.lane, r.color_idx))
            .unwrap_or((0, 0));

        let query = app.search_query.trim().to_lowercase();
        let compare_base = app.compare_base.clone();

        let mut scroll_area = egui::ScrollArea::vertical()
            .id_salt("commit_graph_scroll")
            .auto_shrink([false, false]);

        if let Some(target_row) = app.scroll_to_row.take() {
            let target_y = (target_row as f32 * row_height - 120.0).max(0.0);
            scroll_area = scroll_area.vertical_scroll_offset(target_y);
        }

        let mut clicked_wip = false;
        let mut clicked_commit: Option<usize> = None;
        let mut pending_action: Option<GraphAction> = None;

        ui.spacing_mut().item_spacing.y = 0.0;

        scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;

            for row_idx in row_range {
                let row_width = ui.available_width();
                let (_, rect) = ui.allocate_space(Vec2::new(row_width, row_height));
                let response = ui
                    .interact(
                        rect,
                        ui.id().with(("graph_row", row_idx)),
                        Sense::click(),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                if has_wip && row_idx == 0 {
                    let selected = app.selection == Selection::Wip;
                    draw_row_bg(ui, rect, 0, selected, false, response.hovered(), false);

                    let staged_count = app.status_files.iter().filter(|f| f.is_staged()).count();
                    let unstaged_count =
                        app.status_files.iter().filter(|f| f.is_unstaged()).count();

                    draw_wip_row(
                        ui,
                        rect,
                        graph_col_width,
                        head_lane.0,
                        staged_count,
                        unstaged_count,
                    );

                    if response.clicked() {
                        clicked_wip = true;
                    }
                    continue;
                }

                let commit_idx = row_idx - wip_offset;
                let Some(commit) = app.commits.get(commit_idx) else {
                    continue;
                };
                let Some(graph_row) = app.graph.rows.get(commit_idx) else {
                    continue;
                };

                let is_selected = app.selection == Selection::Commit(commit_idx);
                let is_compare_base = compare_base.as_deref() == Some(commit.hash.as_str());
                let matches_search = if query.is_empty() {
                    true
                } else {
                    commit.subject.to_lowercase().contains(&query)
                        || commit.author.to_lowercase().contains(&query)
                        || commit.hash.to_lowercase().contains(&query)
                        || commit
                            .refs
                            .iter()
                            .any(|r| r.name.to_lowercase().contains(&query))
                };
                let highlight_match = !query.is_empty() && matches_search;

                draw_row_bg(
                    ui,
                    rect,
                    row_idx,
                    is_selected,
                    is_compare_base,
                    response.hovered(),
                    highlight_match,
                );

                let (subject_truncated, show_meta_cols, author_x, date_x, hash_x) =
                    draw_commit_row(
                        ui,
                        rect,
                        graph_col_width,
                        commit,
                        graph_row,
                        !query.is_empty() && !matches_search,
                        is_compare_base,
                        graph_style,
                    );

                if let Some(pointer) = ctx.pointer_hover_pos() {
                    if rect.contains(pointer) {
                        let graph_end_x = rect.left() + graph_col_width;
                        let px = pointer.x;

                        if px < graph_end_x {
                            // Graph / Topology column tooltip
                            response.clone().on_hover_ui_at_pointer(|ui| {
                                ui.set_max_width(340.0);
                                let kind = if commit.is_head {
                                    "HEAD Commit"
                                } else if graph_row.is_merge {
                                    "Merge Commit"
                                } else {
                                    "Commit"
                                };
                                ui.label(
                                    RichText::new(format!("{kind} • {}", commit.short_hash))
                                        .strong()
                                        .size(12.5)
                                        .color(theme::ACCENT_CYAN),
                                );
                                if !commit.parents.is_empty() {
                                    let p_list: Vec<&str> = commit
                                        .parents
                                        .iter()
                                        .map(|p| &p[..p.len().min(7)])
                                        .collect();
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {}",
                                            t.parents,
                                            p_list.join(", ")
                                        ))
                                        .monospace()
                                        .size(11.5)
                                        .color(theme::TEXT_SECONDARY),
                                    );
                                }
                            });
                        } else if !show_meta_cols || px < author_x {
                            // Commit / Refs column tooltip
                            if subject_truncated || !commit.body.is_empty() || !commit.refs.is_empty()
                            {
                                response.clone().on_hover_ui_at_pointer(|ui| {
                                    ui.set_max_width(480.0);
                                    ui.label(
                                        RichText::new(&commit.subject)
                                            .strong()
                                            .color(Color32::WHITE)
                                            .size(13.0),
                                    );
                                    if !commit.body.is_empty() {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(&commit.body)
                                                .color(theme::TEXT_SECONDARY)
                                                .size(12.0),
                                        );
                                    }
                                    if !commit.refs.is_empty() {
                                        ui.add_space(4.0);
                                        let ref_names: Vec<&str> =
                                            commit.refs.iter().map(|r| r.name.as_str()).collect();
                                        ui.label(
                                            RichText::new(ref_names.join("  •  "))
                                                .size(11.0)
                                                .color(theme::ACCENT_CYAN),
                                        );
                                    }
                                });
                            }
                        } else if px < date_x {
                            // Rich Author column tooltip
                            let author_commits = app
                                .commits
                                .iter()
                                .filter(|c| c.email == commit.email && c.author == commit.author)
                                .count();
                            let co_authors: Vec<&str> = commit
                                .body
                                .lines()
                                .filter_map(|l| {
                                    let trimmed = l.trim();
                                    trimmed
                                        .strip_prefix("Co-authored-by:")
                                        .or_else(|| trimmed.strip_prefix("Co-Authored-By:"))
                                        .map(str::trim)
                                })
                                .collect();
                            let initials = author_initials(&commit.author);
                            let avatar_color = author_avatar_color(&commit.author);

                            response.clone().on_hover_ui_at_pointer(|ui| {
                                ui.set_max_width(380.0);
                                ui.horizontal(|ui| {
                                    // Circular initials avatar
                                    let (av_rect, _) = ui.allocate_exact_size(
                                        Vec2::new(30.0, 30.0),
                                        Sense::hover(),
                                    );
                                    ui.painter().circle_filled(
                                        av_rect.center(),
                                        15.0,
                                        avatar_color.linear_multiply(0.28),
                                    );
                                    ui.painter().circle_stroke(
                                        av_rect.center(),
                                        15.0,
                                        Stroke::new(1.0_f32, avatar_color),
                                    );
                                    ui.painter().text(
                                        av_rect.center(),
                                        Align2::CENTER_CENTER,
                                        &initials,
                                        FontId::proportional(11.5),
                                        Color32::WHITE,
                                    );

                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing.y = 1.0;
                                        ui.label(
                                            RichText::new(&commit.author)
                                                .strong()
                                                .size(13.0)
                                                .color(Color32::WHITE),
                                        );
                                        if !commit.email.is_empty() {
                                            ui.label(
                                                RichText::new(&commit.email)
                                                    .size(11.5)
                                                    .color(theme::TEXT_SECONDARY),
                                            );
                                        }
                                    });
                                });

                                ui.add_space(6.0);
                                ui.separator();

                                // Stats & signature row
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "{author_commits} commits in loaded history"
                                        ))
                                        .size(11.0)
                                        .color(theme::TEXT_MUTED),
                                    );
                                    let sig_info = match commit.sig_status {
                                        'G' => Some(("✓ Signed", theme::ACCENT_GREEN)),
                                        'U' => Some(("✓ Signed (untrusted)", theme::ACCENT_AMBER)),
                                        'B' | 'R' | 'X' | 'Y' => {
                                            Some(("✗ Invalid signature", theme::ACCENT_ROSE))
                                        }
                                        _ => None,
                                    };
                                    if let Some((sig_lbl, sig_col)) = sig_info {
                                        ui.label(RichText::new("•").color(theme::TEXT_MUTED));
                                        ui.label(
                                            RichText::new(sig_lbl).size(11.0).color(sig_col),
                                        );
                                    }
                                });

                                // Committer if different from author
                                if !commit.committer.is_empty()
                                    && (commit.committer != commit.author
                                        || commit.committer_email != commit.email)
                                {
                                    ui.add_space(3.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "Committed by: {} <{}>",
                                            commit.committer, commit.committer_email
                                        ))
                                        .size(11.0)
                                        .color(theme::TEXT_SECONDARY),
                                    );
                                }

                                // Co-authors if present in commit trailers
                                if !co_authors.is_empty() {
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new("Co-authored by:")
                                            .size(10.5)
                                            .color(theme::TEXT_MUTED),
                                    );
                                    for co in &co_authors {
                                        ui.label(
                                            RichText::new(format!("• {co}"))
                                                .size(11.0)
                                                .color(theme::ACCENT_PURPLE),
                                        );
                                    }
                                }
                            });
                        } else if px < hash_x {
                            // Date column tooltip
                            let pretty_date = format_human_date(&commit.date, app.config.language);
                            response.clone().on_hover_ui_at_pointer(|ui| {
                                ui.label(
                                    RichText::new(&pretty_date)
                                        .strong()
                                        .size(12.5)
                                        .color(Color32::WHITE),
                                );
                                ui.label(
                                    RichText::new(&commit.relative_date)
                                        .size(11.5)
                                        .color(theme::TEXT_SECONDARY),
                                );
                            });
                        } else {
                            // Hash column tooltip
                            response.clone().on_hover_ui_at_pointer(|ui| {
                                ui.label(
                                    RichText::new(&commit.hash)
                                        .monospace()
                                        .size(12.0)
                                        .color(theme::ACCENT_CYAN),
                                );
                            });
                        }
                    }
                }

                if response.clicked() {
                    clicked_commit = Some(commit_idx);
                }

                response.context_menu(|ui| {
                    if ui.button(t.cherry_pick).clicked() {
                        pending_action = Some(GraphAction::CherryPick(commit.hash.clone()));
                        ui.close_menu();
                    }
                    if ui.button(t.checkout).clicked() {
                        pending_action = Some(GraphAction::Checkout(commit.hash.clone()));
                        ui.close_menu();
                    }
                    if ui.button(t.revert).clicked() {
                        pending_action = Some(GraphAction::Revert(commit.hash.clone()));
                        ui.close_menu();
                    }
                    if ui.button(t.merge_into_current).clicked() {
                        pending_action = Some(GraphAction::Merge(commit.hash.clone()));
                        ui.close_menu();
                    }
                    if ui.button(t.rebase_onto).clicked() {
                        pending_action = Some(GraphAction::Rebase(commit.hash.clone()));
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(t.create_branch_from).clicked() {
                        pending_action = Some(GraphAction::CreateBranch(commit.short_hash.clone()));
                        ui.close_menu();
                    }
                    if ui.button(t.tag_here).clicked() {
                        pending_action = Some(GraphAction::CreateTag(commit.short_hash.clone()));
                        ui.close_menu();
                    }
                    ui.separator();

                    // Git Graph style compare options
                    if let Some(base) = compare_base.as_deref() {
                        if base != commit.hash.as_str()
                            && ui.button(t.compare_with_selected).clicked()
                        {
                            pending_action =
                                Some(GraphAction::CompareWithBase(commit.hash.clone()));
                            ui.close_menu();
                        }
                        if ui.button(t.clear_compare).clicked() {
                            pending_action = Some(GraphAction::ClearCompare);
                            ui.close_menu();
                        }
                    } else if ui.button(t.compare_select).clicked() {
                        pending_action = Some(GraphAction::SelectCompareBase(commit.hash.clone()));
                        ui.close_menu();
                    }
                    if ui.button(t.compare_with_working).clicked() {
                        pending_action = Some(GraphAction::CompareWorking(commit.hash.clone()));
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.menu_button("Reset", |ui| {
                        if ui.button(t.reset_soft).clicked() {
                            pending_action =
                                Some(GraphAction::Reset(commit.hash.clone(), ResetMode::Soft));
                            ui.close_menu();
                        }
                        if ui.button(t.reset_mixed).clicked() {
                            pending_action =
                                Some(GraphAction::Reset(commit.hash.clone(), ResetMode::Mixed));
                            ui.close_menu();
                        }
                        if ui
                            .button(RichText::new(t.reset_hard).color(theme::ACCENT_ROSE))
                            .clicked()
                        {
                            pending_action =
                                Some(GraphAction::Reset(commit.hash.clone(), ResetMode::Hard));
                            ui.close_menu();
                        }
                    });

                    ui.separator();
                    if ui.button(t.copy_hash).clicked() {
                        pending_action = Some(GraphAction::CopyHash(commit.hash.clone()));
                        ui.close_menu();
                    }
                    if ui.button(t.copy_message).clicked() {
                        pending_action = Some(GraphAction::CopyMessage(commit.subject.clone()));
                        ui.close_menu();
                    }
                });
            }
        });

        if clicked_wip {
            app.select_wip();
        }
        if let Some(idx) = clicked_commit {
            app.select_commit(idx);
        }
        if let Some(action) = pending_action {
            match action {
                GraphAction::Checkout(hash) => {
                    app.run_git_action(&format!("Checked out {hash}."), |r| r.checkout(&hash));
                }
                GraphAction::CherryPick(hash) => {
                    app.run_git_action(t.msg_cherry_picked, |r| r.cherry_pick(&hash));
                }
                GraphAction::Revert(hash) => {
                    app.run_git_action(t.msg_reverted, |r| r.revert_commit(&hash));
                }
                GraphAction::Merge(hash) => {
                    app.run_git_action(&format!("Merged {hash} into current branch."), |r| {
                        r.merge_branch(&hash)
                    });
                }
                GraphAction::Rebase(hash) => {
                    app.run_git_action(&format!("Rebased current branch onto {hash}."), |r| {
                        r.rebase_onto(&hash)
                    });
                }
                GraphAction::Reset(hash, mode) => {
                    app.run_git_action(&format!("Reset ({mode:?}) to {hash}."), |r| {
                        r.reset_to(&hash, mode)
                    });
                }
                GraphAction::CreateBranch(short) => {
                    app.open_branch_modal(Some(short));
                }
                GraphAction::CreateTag(short) => {
                    app.tag_modal.open = true;
                    app.tag_modal.name.clear();
                    app.tag_modal.message.clear();
                    app.tag_modal.target = short;
                }
                GraphAction::SelectCompareBase(hash) => {
                    app.compare_base = Some(hash);
                }
                GraphAction::CompareWithBase(hash) => {
                    app.start_compare_with(hash);
                }
                GraphAction::CompareWorking(hash) => {
                    app.start_compare_working(hash);
                }
                GraphAction::ClearCompare => {
                    app.compare_base = None;
                }
                GraphAction::CopyHash(hash) => {
                    ctx.copy_text(hash);
                    app.set_info(t.msg_hash_copied);
                }
                GraphAction::CopyMessage(msg) => {
                    ctx.copy_text(msg);
                    app.set_info(t.msg_message_copied);
                }
            }
        }
    });
}

fn draw_search_and_filter_bar(ui: &mut egui::Ui, app: &mut GitKrakenApp) {
    let t = app.t();
    let matches = app.search_matches();
    let match_count = matches.len();
    let mut go_next = false;
    let mut go_prev = false;
    let mut filter_changed = false;

    let branch_names: Vec<String> = app
        .branches
        .iter()
        .filter(|b| !b.is_remote)
        .map(|b| b.name.clone())
        .collect();
    let current_filter_label = app
        .branch_filter
        .as_deref()
        .unwrap_or(t.all_branches)
        .to_string();

    egui::Frame::none()
        .fill(theme::BG_PRIMARY)
        .inner_margin(Margin::symmetric(12.0, 8.0))
        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Antigravity-style rounded search pill
                egui::Frame::none()
                    .fill(theme::BG_SURFACE)
                    .rounding(Rounding::same(9.0))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                    .inner_margin(Margin::symmetric(10.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("🔍")
                                    .size(12.0)
                                    .color(theme::TEXT_MUTED),
                            );

                            let search_w = (ui.available_width() - 250.0).clamp(160.0, 380.0);
                            let search_resp = ui.add(
                                egui::TextEdit::singleline(&mut app.search_query)
                                    .frame(false)
                                    .desired_width(search_w)
                                    .hint_text(t.search_hint),
                            );
                            if app.focus_search {
                                search_resp.request_focus();
                                app.focus_search = false;
                            }
                            if search_resp.changed() {
                                app.search_match_pos = 0;
                            }
                            if search_resp.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                go_next = true;
                            }
                        });
                    });

                if !app.search_query.is_empty() {
                    let counter = if match_count == 0 {
                        "0/0".to_string()
                    } else {
                        format!(
                            "{}/{}",
                            (app.search_match_pos % match_count) + 1,
                            match_count
                        )
                    };
                    ui.label(
                        RichText::new(counter)
                            .size(11.0)
                            .monospace()
                            .color(if match_count == 0 {
                                theme::ACCENT_ROSE
                            } else {
                                theme::ACCENT_AMBER
                            }),
                    );
                    if ui.small_button("▲").clicked() {
                        go_prev = true;
                    }
                    if ui.small_button("▼").clicked() {
                        go_next = true;
                    }
                    if ui.small_button("×").clicked() {
                        app.search_query.clear();
                        app.search_match_pos = 0;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("graph_branch_filter_combo")
                        .selected_text(
                            RichText::new(format!("● {current_filter_label}"))
                                .size(11.5)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .width(135.0)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(app.branch_filter.is_none(), t.all_branches)
                                .clicked()
                            {
                                app.branch_filter = None;
                                filter_changed = true;
                            }
                            for b in &branch_names {
                                let selected = app.branch_filter.as_deref() == Some(b.as_str());
                                if ui.selectable_label(selected, b).clicked() {
                                    app.branch_filter = Some(b.clone());
                                    filter_changed = true;
                                }
                            }
                        });
                });
            });
        });

    if filter_changed {
        app.refresh();
    }
    if go_next {
        app.jump_next_match();
    } else if go_prev {
        app.jump_prev_match();
    }
}

fn draw_header_bar(ui: &mut egui::Ui, graph_col_width: f32, t: &Strings) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 26.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::ZERO, theme::BG_HEADER);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0_f32, theme::BORDER_SUBTLE),
    );

    let font = FontId::proportional(11.0);
    let cy = rect.center().y;

    painter.text(
        Pos2::new(rect.left() + 12.0, cy),
        Align2::LEFT_CENTER,
        t.col_graph,
        font.clone(),
        theme::TEXT_MUTED,
    );
    painter.text(
        Pos2::new(rect.left() + graph_col_width + 8.0, cy),
        Align2::LEFT_CENTER,
        t.col_commit,
        font.clone(),
        theme::TEXT_MUTED,
    );

    let hash_x = rect.right() - HASH_COL_WIDTH;
    let date_x = hash_x - DATE_COL_WIDTH;
    let author_x = date_x - AUTHOR_COL_WIDTH;

    if author_x > rect.left() + graph_col_width + 140.0 {
        painter.text(
            Pos2::new(author_x, cy),
            Align2::LEFT_CENTER,
            t.col_author,
            font.clone(),
            theme::TEXT_MUTED,
        );
        painter.text(
            Pos2::new(date_x, cy),
            Align2::LEFT_CENTER,
            t.col_date,
            font.clone(),
            theme::TEXT_MUTED,
        );
        painter.text(
            Pos2::new(hash_x, cy),
            Align2::LEFT_CENTER,
            t.col_hash,
            font,
            theme::TEXT_MUTED,
        );
    }
}

fn draw_row_bg(
    ui: &egui::Ui,
    rect: Rect,
    row_idx: usize,
    selected: bool,
    is_compare_base: bool,
    hovered: bool,
    highlight_match: bool,
) {
    let bg = if selected {
        theme::BG_ROW_SELECTED
    } else if is_compare_base {
        theme::BG_ROW_COMPARE
    } else if hovered {
        theme::BG_ROW_HOVER
    } else if highlight_match {
        Color32::from_rgb(54, 48, 30)
    } else if row_idx % 2 == 1 {
        theme::BG_ROW_ALT
    } else {
        theme::BG_PRIMARY
    };
    ui.painter().rect_filled(rect, Rounding::ZERO, bg);
    if selected {
        let bar = Rect::from_min_max(rect.min, Pos2::new(rect.left() + 3.0, rect.max.y));
        ui.painter()
            .rect_filled(bar, Rounding::ZERO, theme::ACCENT_CYAN);
    } else if is_compare_base {
        let bar = Rect::from_min_max(rect.min, Pos2::new(rect.left() + 3.0, rect.max.y));
        ui.painter()
            .rect_filled(bar, Rounding::ZERO, theme::ACCENT_PURPLE);
    }
}

fn lane_x(rect: Rect, lane: usize) -> f32 {
    rect.left() + GRAPH_LEFT_PAD + (lane as f32) * LANE_SPACING
}

fn draw_wip_row(
    ui: &egui::Ui,
    rect: Rect,
    graph_col_width: f32,
    head_lane: usize,
    staged_count: usize,
    unstaged_count: usize,
) {
    let painter = ui.painter();
    let cx = lane_x(rect, head_lane);
    let cy = rect.center().y;

    painter.line_segment(
        [Pos2::new(cx, cy + 6.0), Pos2::new(cx, rect.bottom())],
        Stroke::new(2.0_f32, theme::ACCENT_AMBER),
    );

    painter.circle_filled(Pos2::new(cx, cy), 6.0, theme::BG_PRIMARY);
    painter.circle_stroke(
        Pos2::new(cx, cy),
        5.5,
        Stroke::new(2.2_f32, theme::ACCENT_AMBER),
    );

    let text_x = rect.left() + graph_col_width + 8.0;
    let label = format!("// WIP — {staged_count} staged, {unstaged_count} unstaged");
    painter.text(
        Pos2::new(text_x, cy),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        theme::ACCENT_AMBER,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_commit_row(
    ui: &egui::Ui,
    rect: Rect,
    graph_col_width: f32,
    commit: &Commit,
    graph_row: &GraphRow,
    dimmed: bool,
    is_compare_base: bool,
    graph_style: GraphStyle,
) -> (bool, bool, f32, f32, f32) {
    let painter = ui.painter();
    let cy = rect.center().y;

    let graph_clip = Rect::from_min_max(
        rect.min,
        Pos2::new(rect.left() + graph_col_width, rect.max.y),
    );
    let gp = painter.with_clip_rect(graph_clip);

    for edge in &graph_row.top_edges {
        draw_edge(&gp, rect, edge, rect.top(), cy, graph_style);
    }
    for edge in &graph_row.bottom_edges {
        draw_edge(&gp, rect, edge, cy, rect.bottom(), graph_style);
    }

    let node_pos = Pos2::new(lane_x(rect, graph_row.lane), cy);
    let node_color = theme::lane_color(graph_row.color_idx);

    gp.circle_filled(node_pos, 6.8, theme::BG_PRIMARY);

    if graph_row.is_merge {
        gp.circle_filled(node_pos, 5.5, node_color);
        gp.circle_filled(node_pos, 2.4, theme::BG_PRIMARY);
    } else {
        gp.circle_filled(node_pos, 5.0, node_color);
    }

    if commit.is_head {
        gp.circle_stroke(node_pos, 8.2, Stroke::new(1.8_f32, Color32::WHITE));
    } else if is_compare_base {
        gp.circle_stroke(node_pos, 8.2, Stroke::new(1.8_f32, theme::ACCENT_PURPLE));
    }

    let hash_x = rect.right() - HASH_COL_WIDTH;
    let date_x = hash_x - DATE_COL_WIDTH;
    let author_x = date_x - AUTHOR_COL_WIDTH;
    let show_meta_cols = author_x > rect.left() + graph_col_width + 140.0;

    let subject_right_limit = if show_meta_cols {
        author_x - 10.0
    } else {
        rect.right() - 8.0
    };

    let msg_clip = Rect::from_min_max(
        Pos2::new(rect.left() + graph_col_width + 4.0, rect.top()),
        Pos2::new(subject_right_limit, rect.bottom()),
    );
    let mp = painter.with_clip_rect(msg_clip);

    let mut cursor_x = rect.left() + graph_col_width + 8.0;

    for git_ref in &commit.refs {
        let (pill_bg, pill_fg, prefix) = match git_ref.kind {
            RefKind::Head => (Color32::from_rgb(44, 52, 78), theme::ACCENT_CYAN, "★ "),
            RefKind::LocalBranch => (Color32::from_rgb(30, 48, 42), theme::ACCENT_GREEN, "● "),
            RefKind::RemoteBranch => (Color32::from_rgb(44, 38, 62), theme::ACCENT_PURPLE, "☁ "),
            RefKind::Tag => (Color32::from_rgb(56, 46, 30), theme::ACCENT_AMBER, "# "),
        };

        let text = format!("{prefix}{}", git_ref.name);
        let galley = mp.layout_no_wrap(text, FontId::proportional(11.0), pill_fg);
        let pill_w = galley.size().x + 12.0;
        let pill_h = 20.0;
        let pill_rect = Rect::from_min_size(
            Pos2::new(cursor_x, cy - pill_h * 0.5),
            Vec2::new(pill_w, pill_h),
        );

        mp.rect_filled(pill_rect, Rounding::same(6.0), pill_bg);
        mp.rect_stroke(
            pill_rect,
            Rounding::same(6.0),
            Stroke::new(1.0_f32, pill_fg.linear_multiply(0.35)),
        );
        mp.galley(
            Pos2::new(pill_rect.left() + 6.0, cy - galley.size().y * 0.5),
            galley,
            pill_fg,
        );

        cursor_x += pill_w + 5.0;
    }

    let subject_color = if dimmed {
        theme::TEXT_MUTED
    } else if commit.is_head {
        Color32::WHITE
    } else {
        theme::TEXT_PRIMARY
    };

    let subject_galley =
        mp.layout_no_wrap(commit.subject.clone(), FontId::proportional(13.0), subject_color);
    let subject_truncated = cursor_x + subject_galley.size().x > subject_right_limit;
    mp.galley(
        Pos2::new(cursor_x, cy - subject_galley.size().y * 0.5),
        subject_galley,
        subject_color,
    );

    if show_meta_cols {
        let meta_color = if dimmed {
            theme::TEXT_MUTED
        } else {
            theme::TEXT_SECONDARY
        };

        let author_clip = Rect::from_min_max(
            Pos2::new(author_x, rect.top()),
            Pos2::new(date_x - 6.0, rect.bottom()),
        );
        painter.with_clip_rect(author_clip).text(
            Pos2::new(author_x, cy),
            Align2::LEFT_CENTER,
            &commit.author,
            FontId::proportional(12.0),
            meta_color,
        );

        let date_clip = Rect::from_min_max(
            Pos2::new(date_x, rect.top()),
            Pos2::new(hash_x - 6.0, rect.bottom()),
        );
        painter.with_clip_rect(date_clip).text(
            Pos2::new(date_x, cy),
            Align2::LEFT_CENTER,
            &commit.relative_date,
            FontId::proportional(11.5),
            theme::TEXT_MUTED,
        );

        painter.text(
            Pos2::new(hash_x, cy),
            Align2::LEFT_CENTER,
            &commit.short_hash,
            FontId::monospace(11.5),
            theme::ACCENT_CYAN.linear_multiply(if dimmed { 0.45 } else { 0.85 }),
        );
    }

    (subject_truncated, show_meta_cols, author_x, date_x, hash_x)
}

fn draw_edge(
    painter: &egui::Painter,
    rect: Rect,
    edge: &GraphEdge,
    y_start: f32,
    y_end: f32,
    style: GraphStyle,
) {
    let x0 = lane_x(rect, edge.from_lane);
    let x1 = lane_x(rect, edge.to_lane);
    let color = theme::lane_color(edge.color_idx);
    let stroke = Stroke::new(2.2_f32, color);

    if edge.from_lane == edge.to_lane {
        painter.line_segment([Pos2::new(x0, y_start), Pos2::new(x1, y_end)], stroke);
        return;
    }

    match style {
        GraphStyle::Rounded => {
            let mid_y = (y_start + y_end) * 0.5;
            let bezier = CubicBezierShape::from_points_stroke(
                [
                    Pos2::new(x0, y_start),
                    Pos2::new(x0, mid_y),
                    Pos2::new(x1, mid_y),
                    Pos2::new(x1, y_end),
                ],
                false,
                Color32::TRANSPARENT,
                stroke,
            );
            painter.add(bezier);
        }
        GraphStyle::Angular => {
            let mid_y = y_start + (y_end - y_start) * 0.35;
            painter.line_segment([Pos2::new(x0, y_start), Pos2::new(x0, mid_y)], stroke);
            painter.line_segment([Pos2::new(x0, mid_y), Pos2::new(x1, y_end)], stroke);
        }
    }
}

fn author_initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.len() {
        0 => "?".to_string(),
        1 => words[0]
            .chars()
            .take(2)
            .collect::<String>()
            .to_uppercase(),
        _ => {
            let first = words[0].chars().next().unwrap_or('?');
            let last = words[words.len() - 1].chars().next().unwrap_or('?');
            format!("{first}{last}").to_uppercase()
        }
    }
}

fn author_avatar_color(name: &str) -> Color32 {
    let hash: usize = name
        .bytes()
        .fold(0usize, |acc, b| acc.wrapping_mul(31).wrapping_add(b as usize));
    theme::lane_color(hash)
}
