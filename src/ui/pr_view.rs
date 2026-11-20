use eframe::egui::{
    self, Align, Color32, FontId, Layout, Margin, RichText, Rounding, Sense, Stroke, Vec2,
};

use crate::app::{GitKrakenApp, PrActionKind, PrDetailTab};
use crate::i18n::format_human_date;
use crate::providers::{CommentKind, MergeStrategy, PrState, PrStateFilter, PullRequestDetail};
use crate::theme;

pub fn show(ctx: &egui::Context, app: &mut GitKrakenApp) {
    let t = app.t();

    let Some(spec) = app.remote_spec.clone() else {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::BG_PRIMARY))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.label(
                        RichText::new(t.mode_prs)
                            .size(20.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(t.pr_no_provider)
                            .size(13.5)
                            .color(theme::TEXT_SECONDARY),
                    );
                });
            });
        return;
    };

    // Collapsible Left PR List Panel
    if app.config.show_left_sidebar {
        let mut collapse_left = false;
        let mut trigger_reload_list = false;
        let mut picked_pr: Option<u64> = None;

        let frame = egui::Frame::none()
            .fill(theme::BG_PANEL)
            .inner_margin(Margin::symmetric(10.0, 10.0))
            .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE));

        egui::SidePanel::left("pr_list_sidebar")
            .default_width(330.0)
            .min_width(250.0)
            .max_width(480.0)
            .frame(frame)
            .show(ctx, |ui| {
                // Header: Provider pill + repo slug + refresh + collapse
                ui.horizontal(|ui| {
                    egui::Frame::none()
                        .fill(theme::BG_SURFACE)
                        .rounding(Rounding::same(6.0))
                        .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                        .inner_margin(Margin::symmetric(7.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(spec.kind.label())
                                    .size(11.0)
                                    .strong()
                                    .color(theme::ACCENT_CYAN),
                            );
                        });

                    ui.add(
                        egui::Label::new(
                            RichText::new(&spec.full_path)
                                .size(12.0)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .truncate(),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .small_button("«")
                            .on_hover_text(t.toggle_left_sidebar)
                            .clicked()
                        {
                            collapse_left = true;
                        }
                        if ui
                            .small_button("↻")
                            .on_hover_text(t.refresh)
                            .clicked()
                        {
                            trigger_reload_list = true;
                        }
                    });
                });

                ui.add_space(6.0);

                // State filter tabs: Open | Merged | Closed | All
                ui.horizontal(|ui| {
                    for (flt, lbl) in [
                        (PrStateFilter::Open, t.pr_open),
                        (PrStateFilter::Merged, t.pr_merged),
                        (PrStateFilter::Closed, t.pr_closed),
                        (PrStateFilter::All, t.pr_all),
                    ] {
                        if ui
                            .selectable_label(app.pr_filter == flt, RichText::new(lbl).size(11.5))
                            .clicked()
                            && app.pr_filter != flt
                        {
                            app.pr_filter = flt;
                            trigger_reload_list = true;
                        }
                    }
                });

                ui.add_space(6.0);

                // Search filter
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").size(12.0).color(theme::TEXT_MUTED));
                    let w = (ui.available_width()
                        - if app.pr_search.is_empty() { 4.0 } else { 26.0 })
                    .max(90.0);
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut app.pr_search)
                            .desired_width(w)
                            .hint_text(t.pr_filter_hint),
                    );
                    if app.focus_search {
                        resp.request_focus();
                        app.focus_search = false;
                    }
                    if !app.pr_search.is_empty() && ui.small_button("×").clicked() {
                        app.pr_search.clear();
                    }
                });

                ui.add_space(4.0);
                ui.separator();

                if app.pr_loading_list {
                    ui.add_space(24.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(t.pr_loading)
                                .color(theme::ACCENT_CYAN)
                                .size(12.5),
                        );
                    });
                    return;
                }

                let q = app.pr_search.trim().to_lowercase();
                let filtered: Vec<_> = app
                    .pr_list
                    .iter()
                    .filter(|pr| {
                        if q.is_empty() {
                            return true;
                        }
                        pr.title.to_lowercase().contains(&q)
                            || pr.author.to_lowercase().contains(&q)
                            || pr.source_branch.to_lowercase().contains(&q)
                            || format!("#{}", pr.number).contains(&q)
                    })
                    .cloned()
                    .collect();

                if filtered.is_empty() {
                    ui.add_space(24.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(t.pr_none_found)
                                .color(theme::TEXT_MUTED)
                                .italics(),
                        );
                        ui.add_space(10.0);
                        if ui.button(t.pr_configure_token).clicked() {
                            app.settings_open = true;
                        }
                    });
                    return;
                }

                egui::ScrollArea::vertical()
                    .id_salt("pr_list_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for pr in &filtered {
                            let is_sel = app.pr_selected == Some(pr.number);
                            let card_bg = if is_sel {
                                theme::BG_ROW_SELECTED
                            } else {
                                theme::BG_SURFACE
                            };
                            let card_border = if is_sel {
                                theme::ACCENT_CYAN
                            } else {
                                theme::BORDER_SUBTLE
                            };

                            let card_resp = egui::Frame::none()
                                .fill(card_bg)
                                .rounding(Rounding::same(8.0))
                                .stroke(Stroke::new(1.0_f32, card_border))
                                .inner_margin(Margin::symmetric(10.0, 8.0))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    // Top meta: state dot + #number + comments
                                    ui.horizontal(|ui| {
                                        let (st_lbl, st_col) = state_badge(pr.state, t);
                                        ui.label(
                                            RichText::new(format!("● {st_lbl}"))
                                                .size(10.5)
                                                .strong()
                                                .color(st_col),
                                        );
                                        ui.label(
                                            RichText::new(format!("#{}", pr.number))
                                                .monospace()
                                                .size(11.0)
                                                .color(theme::TEXT_MUTED),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if pr.comments_count > 0 {
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{} msg",
                                                            pr.comments_count
                                                        ))
                                                        .size(10.5)
                                                        .color(theme::TEXT_MUTED),
                                                    );
                                                }
                                            },
                                        );
                                    });

                                    ui.add_space(3.0);
                                    ui.label(
                                        RichText::new(&pr.title)
                                            .size(12.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                    ui.add_space(3.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "{} • {} → {}",
                                            pr.author, pr.source_branch, pr.target_branch
                                        ))
                                        .size(11.0)
                                        .color(theme::TEXT_SECONDARY),
                                    )
                                    .on_hover_text(format_human_date(
                                        if pr.updated_at.is_empty() {
                                            &pr.created_at
                                        } else {
                                            &pr.updated_at
                                        },
                                        app.config.language,
                                    ));
                                })
                                .response
                                .interact(Sense::click())
                                .on_hover_cursor(egui::CursorIcon::PointingHand);

                            if card_resp.clicked() {
                                picked_pr = Some(pr.number);
                            }
                            ui.add_space(5.0);
                        }
                    });
            });

        if collapse_left {
            app.toggle_left_sidebar();
        }
        if trigger_reload_list {
            app.fetch_pr_list(Some(ctx.clone()));
        }
        if let Some(num) = picked_pr {
            app.fetch_pr_detail(num, Some(ctx.clone()));
        }
    }

    // Main Right/Center GitHub-Style PR Detail Panel
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(theme::BG_PRIMARY)
                .inner_margin(Margin::symmetric(18.0, 14.0)),
        )
        .show(ctx, |ui| {
            if app.pr_loading_detail {
                ui.vertical_centered(|ui| {
                    ui.add_space(140.0);
                    ui.label(
                        RichText::new(t.pr_loading)
                            .size(15.0)
                            .color(theme::ACCENT_CYAN),
                    );
                });
                return;
            }

            let Some(detail) = app.pr_detail.clone() else {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.label(
                        RichText::new(t.pr_select_prompt)
                            .size(14.5)
                            .color(theme::TEXT_MUTED),
                    );
                });
                return;
            };

            show_pr_detail(ctx, ui, app, &detail);
        });
}

fn show_pr_detail(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    app: &mut GitKrakenApp,
    detail: &PullRequestDetail,
) {
    let t = app.t();
    let pr = &detail.summary;
    let busy = app.pr_action_busy;

    // 1. PR Header
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(&pr.title)
                .size(18.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.label(
            RichText::new(format!("#{}", pr.number))
                .size(17.0)
                .monospace()
                .color(theme::TEXT_MUTED),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if !pr.html_url.is_empty() && ui.button(t.pr_open_browser).clicked() {
                let _ = webbrowser::open(&pr.html_url);
            }
            if !pr.source_branch.is_empty() && ui.button(t.pr_checkout_branch).clicked() {
                let br = pr.source_branch.clone();
                app.run_git_action(&format!("Checked out branch '{br}'."), |r| {
                    let _ = r.fetch();
                    r.checkout(&br)
                });
            }
        });
    });

    ui.add_space(6.0);

    // State pill + branch flow + stats
    ui.horizontal_wrapped(|ui| {
        let (st_lbl, st_col) = state_badge(pr.state, t);
        egui::Frame::none()
            .fill(st_col.linear_multiply(0.22))
            .rounding(Rounding::same(10.0))
            .stroke(Stroke::new(1.0_f32, st_col.linear_multiply(0.7)))
            .inner_margin(Margin::symmetric(9.0, 3.0))
            .show(ui, |ui| {
                ui.label(RichText::new(format!("● {st_lbl}")).size(11.5).strong().color(st_col));
            });

        ui.label(
            RichText::new(&pr.author)
                .strong()
                .size(12.5)
                .color(theme::TEXT_PRIMARY),
        );
        ui.label(
            RichText::new(t.pr_wants_to_merge)
                .size(12.0)
                .color(theme::TEXT_SECONDARY),
        );

        draw_branch_chip(ui, &pr.source_branch);
        ui.label(RichText::new("→").color(theme::TEXT_MUTED));
        draw_branch_chip(ui, &pr.target_branch);

        if !pr.created_at.is_empty() {
            let pretty = format_human_date(&pr.created_at, app.config.language);
            ui.label(RichText::new("•").color(theme::TEXT_MUTED));
            ui.label(RichText::new(pretty).size(11.5).color(theme::TEXT_MUTED));
        }

        if let (Some(add), Some(del)) = (pr.additions, pr.deletions) {
            ui.label(RichText::new("•").color(theme::TEXT_MUTED));
            ui.label(
                RichText::new(format!("+{add}"))
                    .monospace()
                    .size(11.5)
                    .color(theme::ACCENT_GREEN),
            );
            ui.label(
                RichText::new(format!("-{del}"))
                    .monospace()
                    .size(11.5)
                    .color(theme::ACCENT_ROSE),
            );
        }
    });

    ui.add_space(10.0);

    // 2. Action Bar (Merge strategy + Merge / Approve / Close-Decline / Reopen)
    let is_open_like = matches!(pr.state, PrState::Open | PrState::Draft);
    let mut pending_action: Option<PrActionKind> = None;

    egui::Frame::none()
        .fill(theme::BG_PANEL)
        .rounding(Rounding::same(9.0))
        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
        .inner_margin(Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if is_open_like {
                    let strat_label = match app.pr_merge_strategy {
                        MergeStrategy::Merge => t.pr_merge_commit,
                        MergeStrategy::Squash => t.pr_squash_merge,
                        MergeStrategy::Rebase => t.pr_rebase_merge,
                    };

                    egui::ComboBox::from_id_salt("pr_merge_strategy_combo")
                        .selected_text(RichText::new(strat_label).size(12.0))
                        .width(175.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.pr_merge_strategy,
                                MergeStrategy::Merge,
                                t.pr_merge_commit,
                            );
                            ui.selectable_value(
                                &mut app.pr_merge_strategy,
                                MergeStrategy::Squash,
                                t.pr_squash_merge,
                            );
                            ui.selectable_value(
                                &mut app.pr_merge_strategy,
                                MergeStrategy::Rebase,
                                t.pr_rebase_merge,
                            );
                        });

                    if ui
                        .add_enabled(
                            !busy,
                            egui::Button::new(
                                RichText::new(format!("✓ {}", t.pr_merge_btn))
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(36, 98, 66))
                            .stroke(Stroke::new(1.0_f32, theme::ACCENT_GREEN))
                            .min_size(Vec2::new(82.0, 26.0)),
                        )
                        .clicked()
                    {
                        pending_action = Some(PrActionKind::Merge(app.pr_merge_strategy));
                    }

                    ui.separator();

                    if ui
                        .add_enabled(
                            !busy,
                            egui::Button::new(
                                RichText::new(t.pr_approve_btn).color(theme::ACCENT_GREEN),
                            )
                            .min_size(Vec2::new(0.0, 26.0)),
                        )
                        .clicked()
                    {
                        pending_action = Some(PrActionKind::Approve);
                    }

                    if pr.mergeable == Some(false) {
                        ui.label(
                            RichText::new("✕ Conflicts")
                                .size(11.5)
                                .strong()
                                .color(theme::ACCENT_ROSE),
                        );
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !busy,
                                egui::Button::new(
                                    RichText::new(t.pr_close_decline_btn)
                                        .color(theme::ACCENT_ROSE),
                                )
                                .min_size(Vec2::new(0.0, 26.0)),
                            )
                            .clicked()
                        {
                            pending_action = Some(PrActionKind::CloseOrDecline);
                        }
                    });
                } else if pr.state == PrState::Closed {
                    ui.label(
                        RichText::new(t.pr_closed)
                            .color(theme::ACCENT_ROSE)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !busy,
                                egui::Button::new(
                                    RichText::new(t.pr_reopen_btn).color(theme::ACCENT_GREEN),
                                )
                                .min_size(Vec2::new(0.0, 26.0)),
                            )
                            .clicked()
                        {
                            pending_action = Some(PrActionKind::Reopen);
                        }
                    });
                } else {
                    ui.label(
                        RichText::new(format!("● {} — {} → {}", t.pr_merged, pr.source_branch, pr.target_branch))
                            .color(theme::ACCENT_PURPLE)
                            .strong(),
                    );
                }
            });
        });

    ui.add_space(10.0);

    // 3. GitHub-style Sub-Tabs: Conversation | Commits | Files changed
    ui.horizontal(|ui| {
        let tabs = [
            (
                PrDetailTab::Conversation,
                format!("{} ({})", t.pr_tab_conversation, detail.comments.len()),
            ),
            (
                PrDetailTab::Commits,
                format!("{} ({})", t.pr_tab_commits, detail.commits.len()),
            ),
            (
                PrDetailTab::Files,
                format!("{} ({})", t.pr_tab_files, detail.files.len()),
            ),
        ];
        for (tab, label) in tabs {
            let active = app.pr_detail_tab == tab;
            let btn = egui::Button::new(RichText::new(label).size(12.5).color(if active {
                Color32::WHITE
            } else {
                theme::TEXT_SECONDARY
            }))
            .fill(if active {
                theme::BG_ROW_SELECTED
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if active {
                Stroke::new(1.0_f32, theme::ACCENT_CYAN)
            } else {
                Stroke::new(1.0_f32, theme::BORDER_SUBTLE)
            })
            .rounding(Rounding::same(7.0))
            .min_size(Vec2::new(0.0, 28.0));

            if ui.add(btn).clicked() {
                app.pr_detail_tab = tab;
            }
        }
    });

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    // 4. Tab Content
    egui::ScrollArea::vertical()
        .id_salt("pr_detail_body_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| match app.pr_detail_tab {
            PrDetailTab::Conversation => {
                // Description card
                egui::Frame::none()
                    .fill(theme::BG_PANEL)
                    .rounding(Rounding::same(9.0))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&pr.author)
                                    .strong()
                                    .size(12.5)
                                    .color(theme::ACCENT_CYAN),
                            );
                            if !pr.created_at.is_empty() {
                                let pretty =
                                    format_human_date(&pr.created_at, app.config.language);
                                ui.label(
                                    RichText::new(pretty)
                                        .size(11.5)
                                        .color(theme::TEXT_MUTED),
                                );
                            }
                        });
                        ui.separator();
                        ui.add_space(4.0);
                        if pr.body.trim().is_empty() {
                            ui.label(
                                RichText::new(t.pr_no_description)
                                    .color(theme::TEXT_MUTED)
                                    .italics(),
                            );
                        } else {
                            ui.label(
                                RichText::new(&pr.body)
                                    .size(13.0)
                                    .color(theme::TEXT_PRIMARY),
                            );
                        }
                    });

                ui.add_space(10.0);

                // Reviews & Comments timeline
                for comment in &detail.comments {
                    let border = match comment.kind {
                        CommentKind::Approved => theme::ACCENT_GREEN.linear_multiply(0.6),
                        CommentKind::ChangesRequested => theme::ACCENT_ROSE.linear_multiply(0.6),
                        CommentKind::Comment => theme::BORDER_SUBTLE,
                    };

                    egui::Frame::none()
                        .fill(theme::BG_PANEL)
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(1.0_f32, border))
                        .inner_margin(Margin::symmetric(12.0, 9.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&comment.author)
                                        .strong()
                                        .size(12.5)
                                        .color(Color32::WHITE),
                                );
                                match comment.kind {
                                    CommentKind::Approved => {
                                        ui.label(
                                            RichText::new(t.pr_approved_badge)
                                                .size(11.5)
                                                .strong()
                                                .color(theme::ACCENT_GREEN),
                                        );
                                    }
                                    CommentKind::ChangesRequested => {
                                        ui.label(
                                            RichText::new(t.pr_changes_requested_badge)
                                                .size(11.5)
                                                .strong()
                                                .color(theme::ACCENT_ROSE),
                                        );
                                    }
                                    CommentKind::Comment => {}
                                }
                                if !comment.created_at.is_empty() {
                                    let pretty = format_human_date(
                                        &comment.created_at,
                                        app.config.language,
                                    );
                                    ui.with_layout(
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(pretty)
                                                    .size(11.0)
                                                    .color(theme::TEXT_MUTED),
                                            );
                                        },
                                    );
                                }
                            });
                            if !comment.body.trim().is_empty() {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(&comment.body)
                                        .size(12.5)
                                        .color(theme::TEXT_PRIMARY),
                                );
                            }
                        });
                    ui.add_space(6.0);
                }

                ui.add_space(8.0);

                // Comment / Review composer box
                egui::Frame::none()
                    .fill(theme::BG_PANEL)
                    .rounding(Rounding::same(9.0))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut app.pr_comment_draft)
                                .desired_rows(3)
                                .desired_width(f32::INFINITY)
                                .hint_text(t.pr_comment_hint),
                        );
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(
                                    !busy && !app.pr_comment_draft.trim().is_empty(),
                                    egui::Button::new(t.pr_comment_btn)
                                        .min_size(Vec2::new(90.0, 26.0)),
                                )
                                .clicked()
                            {
                                pending_action =
                                    Some(PrActionKind::Comment(app.pr_comment_draft.clone()));
                            }
                            if is_open_like {
                                if ui
                                    .add_enabled(
                                        !busy,
                                        egui::Button::new(
                                            RichText::new(t.pr_approve_btn)
                                                .color(theme::ACCENT_GREEN),
                                        )
                                        .min_size(Vec2::new(0.0, 26.0)),
                                    )
                                    .clicked()
                                {
                                    pending_action = Some(PrActionKind::Approve);
                                }
                                if ui
                                    .add_enabled(
                                        !busy,
                                        egui::Button::new(
                                            RichText::new(t.pr_request_changes_btn)
                                                .color(theme::ACCENT_AMBER),
                                        )
                                        .min_size(Vec2::new(0.0, 26.0)),
                                    )
                                    .clicked()
                                {
                                    pending_action = Some(PrActionKind::RequestChanges(
                                        app.pr_comment_draft.clone(),
                                    ));
                                }
                            }
                        });
                    });
            }

            PrDetailTab::Commits => {
                for c in &detail.commits {
                    egui::Frame::none()
                        .fill(theme::BG_PANEL)
                        .rounding(Rounding::same(7.0))
                        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                if ui
                                    .small_button(
                                        RichText::new(&c.short_hash)
                                            .monospace()
                                            .color(theme::ACCENT_CYAN),
                                    )
                                    .on_hover_text(&c.hash)
                                    .clicked()
                                {
                                    ctx.copy_text(c.hash.clone());
                                    app.set_info(t.msg_hash_copied);
                                }
                                ui.label(
                                    RichText::new(&c.message)
                                        .size(13.0)
                                        .color(Color32::WHITE),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let pretty = format_human_date(&c.date, app.config.language);
                                    ui.label(
                                        RichText::new(pretty)
                                            .size(11.0)
                                            .color(theme::TEXT_MUTED),
                                    );
                                    ui.label(
                                        RichText::new(&c.author)
                                            .size(11.5)
                                            .color(theme::TEXT_SECONDARY),
                                    );
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
            }

            PrDetailTab::Files => {
                for file in &detail.files {
                    let st_col = match file.status.as_str() {
                        "A" => theme::ACCENT_GREEN,
                        "D" => theme::ACCENT_ROSE,
                        "R" => theme::ACCENT_PURPLE,
                        _ => theme::ACCENT_AMBER,
                    };

                    egui::Frame::none()
                        .fill(theme::BG_PANEL)
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            egui::CollapsingHeader::new(
                                RichText::new(format!(
                                    "[{}]  {}   +{} -{}",
                                    file.status, file.path, file.additions, file.deletions
                                ))
                                .monospace()
                                .size(12.0)
                                .color(st_col),
                            )
                            .id_salt(format!("pr_file_{}", file.path))
                            .default_open(detail.files.len() <= 8)
                            .show(ui, |ui| {
                                if file.patch.trim().is_empty() {
                                    ui.label(
                                        RichText::new(t.no_diff)
                                            .color(theme::TEXT_MUTED)
                                            .italics(),
                                    );
                                } else {
                                    egui::ScrollArea::horizontal()
                                        .id_salt(format!("pr_patch_{}", file.path))
                                        .show(ui, |ui| {
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Extend);
                                            for (idx, line) in file.patch.lines().enumerate() {
                                                if idx >= 800 {
                                                    ui.label(
                                                        RichText::new(
                                                            "... (patch truncated at 800 lines)",
                                                        )
                                                        .color(theme::ACCENT_AMBER)
                                                        .italics(),
                                                    );
                                                    break;
                                                }
                                                let (fg, bg) = if line.starts_with("@@") {
                                                    (theme::DIFF_HUNK_FG, None)
                                                } else if line.starts_with('+')
                                                    && !line.starts_with("+++")
                                                {
                                                    (theme::DIFF_ADD_FG, Some(theme::DIFF_ADD_BG))
                                                } else if line.starts_with('-')
                                                    && !line.starts_with("---")
                                                {
                                                    (theme::DIFF_DEL_FG, Some(theme::DIFF_DEL_BG))
                                                } else {
                                                    (theme::TEXT_PRIMARY, None)
                                                };
                                                let mut rt = RichText::new(line)
                                                    .font(FontId::monospace(12.0))
                                                    .color(fg);
                                                if let Some(bgc) = bg {
                                                    rt = rt.background_color(bgc);
                                                }
                                                ui.label(rt);
                                            }
                                        });
                                }
                            });
                        });
                    ui.add_space(8.0);
                }
            }
        });

    if let Some(action) = pending_action {
        app.run_pr_action(pr.number, action, ctx.clone());
    }
}

fn state_badge(state: PrState, t: &crate::i18n::Strings) -> (&'static str, Color32) {
    match state {
        PrState::Open => (t.pr_open, theme::ACCENT_GREEN),
        PrState::Draft => (t.pr_draft, theme::TEXT_MUTED),
        PrState::Merged => (t.pr_merged, theme::ACCENT_PURPLE),
        PrState::Closed => (t.pr_closed, theme::ACCENT_ROSE),
    }
}

fn draw_branch_chip(ui: &mut egui::Ui, name: &str) {
    egui::Frame::none()
        .fill(theme::BG_SURFACE)
        .rounding(Rounding::same(5.0))
        .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
        .inner_margin(Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(name)
                    .monospace()
                    .size(11.5)
                    .color(theme::ACCENT_CYAN),
            );
        });
}
