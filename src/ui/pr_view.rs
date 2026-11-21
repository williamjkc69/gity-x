use eframe::egui::{
    self,
    text::{LayoutJob, TextFormat},
    Align, Color32, FontId, Layout, Margin, RichText, Rounding, Sense, Stroke, Vec2,
};

use crate::app::{GitKrakenApp, PrActionKind, PrDetailTab, SettingsTab};
use crate::i18n::format_human_date;
use crate::providers::{
    CommentKind, MergeStrategy, PrState, PrStateFilter, ProviderKind, PullRequestDetail,
};
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
                    ui.add_space(12.0);
                    if ui.button(format!("?  {}", t.help_btn)).clicked() {
                        app.help_search.clear();
                        app.help_open = true;
                    }
                });
            });
        return;
    };

    let has_token = match spec.kind {
        ProviderKind::GitHub => !app.config.github_token.trim().is_empty() || app.gh_cli_detected,
        ProviderKind::GitLab => !app.config.gitlab_token.trim().is_empty(),
        ProviderKind::Bitbucket => !app.config.bitbucket_token.trim().is_empty(),
    };

    let mut trigger_reload_list = false;
    let mut toggled_pr: Option<u64> = None;
    let mut pending_action: Option<(u64, PrActionKind)> = None;
    let mut open_provider_settings = false;
    let mut open_provider_help = false;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(theme::BG_PRIMARY)
                .inner_margin(Margin::symmetric(20.0, 14.0)),
        )
        .show(ctx, |ui| {
            // Top PR Controls Bar
            egui::Frame::none()
                .fill(theme::BG_PANEL)
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                .inner_margin(Margin::symmetric(14.0, 10.0))
                .show(ui, |ui| {
                    // Row 1: Provider badge + repo path + filters + Help / Settings / Refresh
                    ui.horizontal(|ui| {
                        egui::Frame::none()
                            .fill(theme::BG_SURFACE)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                            .inner_margin(Margin::symmetric(8.0, 3.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(spec.kind.label())
                                        .size(11.5)
                                        .strong()
                                        .color(theme::ACCENT_CYAN),
                                );
                            });

                        ui.label(
                            RichText::new(&spec.full_path)
                                .size(13.0)
                                .strong()
                                .color(Color32::WHITE),
                        );

                        ui.separator();

                        for (flt, lbl) in [
                            (PrStateFilter::Open, t.pr_open),
                            (PrStateFilter::Merged, t.pr_merged),
                            (PrStateFilter::Closed, t.pr_closed),
                            (PrStateFilter::All, t.pr_all),
                        ] {
                            if ui
                                .selectable_label(app.pr_filter == flt, RichText::new(lbl).size(12.0))
                                .clicked()
                                && app.pr_filter != flt
                            {
                                app.pr_filter = flt;
                                trigger_reload_list = true;
                            }
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new("↻")
                                        .min_size(Vec2::new(26.0, 24.0)),
                                )
                                .on_hover_text(t.refresh)
                                .clicked()
                            {
                                trigger_reload_list = true;
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(format!("? {}", t.help_btn))
                                            .size(11.5)
                                            .color(theme::ACCENT_CYAN),
                                    )
                                    .min_size(Vec2::new(0.0, 24.0)),
                                )
                                .clicked()
                            {
                                open_provider_help = true;
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(if has_token {
                                            "✓ Token"
                                        } else {
                                            t.pr_configure_token
                                        })
                                        .size(11.5)
                                        .color(if has_token {
                                            theme::ACCENT_GREEN
                                        } else {
                                            theme::ACCENT_AMBER
                                        }),
                                    )
                                    .min_size(Vec2::new(0.0, 24.0)),
                                )
                                .clicked()
                            {
                                open_provider_settings = true;
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Row 2: Full-width search + subtle hint
                    ui.horizontal(|ui| {
                        let w = (ui.available_width()
                            - if app.pr_search.is_empty() { 4.0 } else { 28.0 })
                        .max(160.0);
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
                });

            // Gentle warning banner when no token is detected for this provider
            if !has_token {
                ui.add_space(8.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(56, 44, 24))
                    .rounding(Rounding::same(8.0))
                    .stroke(Stroke::new(1.0_f32, theme::ACCENT_AMBER.linear_multiply(0.7)))
                    .inner_margin(Margin::symmetric(12.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("!  {}", t.pr_token_missing_banner))
                                    .size(12.0)
                                    .color(theme::ACCENT_AMBER),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.small_button(t.pr_configure_token).clicked() {
                                    open_provider_settings = true;
                                }
                                if ui.small_button(t.help_btn).clicked() {
                                    open_provider_help = true;
                                }
                            });
                        });
                    });
            }

            ui.add_space(10.0);

            if app.pr_loading_list {
                ui.add_space(48.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(t.pr_loading)
                            .size(14.5)
                            .color(theme::ACCENT_CYAN),
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
                        || pr.target_branch.to_lowercase().contains(&q)
                        || format!("#{}", pr.number).contains(&q)
                })
                .cloned()
                .collect();

            if filtered.is_empty() {
                ui.add_space(48.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(t.pr_none_found)
                            .size(14.0)
                            .color(theme::TEXT_MUTED)
                            .italics(),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 260.0).max(0.0) * 0.5);
                        if ui.button(t.pr_configure_token).clicked() {
                            open_provider_settings = true;
                        }
                        if ui.button(format!("? {}", t.help_btn)).clicked() {
                            open_provider_help = true;
                        }
                    });
                });
                return;
            }

            ui.label(
                RichText::new(format!("{} ({})", t.pr_click_to_expand, filtered.len()))
                    .size(11.5)
                    .color(theme::TEXT_MUTED),
            );
            ui.add_space(6.0);

            // Accordion List of Pull Requests
            egui::ScrollArea::vertical()
                .id_salt("pr_accordion_list_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for pr in &filtered {
                        let is_expanded = app.pr_selected == Some(pr.number);
                        let card_bg = if is_expanded {
                            theme::BG_PANEL
                        } else {
                            theme::BG_SURFACE
                        };
                        let card_border = if is_expanded {
                            theme::ACCENT_CYAN
                        } else {
                            theme::BORDER_SUBTLE
                        };

                        egui::Frame::none()
                            .fill(card_bg)
                            .rounding(Rounding::same(9.0))
                            .stroke(Stroke::new(1.0_f32, card_border))
                            .inner_margin(Margin::symmetric(14.0, 10.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                // Clickable summary header row
                                let header_resp = ui
                                    .scope(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(if is_expanded { "▾" } else { "▸" })
                                                    .size(13.0)
                                                    .color(if is_expanded {
                                                        theme::ACCENT_CYAN
                                                    } else {
                                                        theme::TEXT_MUTED
                                                    }),
                                            );

                                            let (st_lbl, st_col) = state_badge(pr.state, t);
                                            egui::Frame::none()
                                                .fill(st_col.linear_multiply(0.18))
                                                .rounding(Rounding::same(9.0))
                                                .stroke(Stroke::new(
                                                    1.0_f32,
                                                    st_col.linear_multiply(0.65),
                                                ))
                                                .inner_margin(Margin::symmetric(7.0, 2.0))
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        RichText::new(format!("● {st_lbl}"))
                                                            .size(10.5)
                                                            .strong()
                                                            .color(st_col),
                                                    );
                                                });

                                            ui.label(
                                                RichText::new(format!("#{}", pr.number))
                                                    .monospace()
                                                    .size(12.0)
                                                    .color(theme::TEXT_MUTED),
                                            );

                                            ui.label(
                                                RichText::new(&pr.title)
                                                    .size(13.5)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );

                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    let date_raw = if pr.updated_at.is_empty() {
                                                        &pr.created_at
                                                    } else {
                                                        &pr.updated_at
                                                    };
                                                    if !date_raw.is_empty() {
                                                        ui.label(
                                                            RichText::new(format_human_date(
                                                                date_raw,
                                                                app.config.language,
                                                            ))
                                                            .size(11.0)
                                                            .color(theme::TEXT_MUTED),
                                                        );
                                                    }
                                                    if pr.comments_count > 0 {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "{} msg",
                                                                pr.comments_count
                                                            ))
                                                            .size(11.0)
                                                            .color(theme::TEXT_SECONDARY),
                                                        );
                                                    }
                                                },
                                            );
                                        });

                                        ui.add_space(4.0);
                                        ui.horizontal(|ui| {
                                            ui.add_space(18.0);
                                            ui.label(
                                                RichText::new(&pr.author)
                                                    .size(11.5)
                                                    .strong()
                                                    .color(theme::TEXT_SECONDARY),
                                            );
                                            ui.label(RichText::new("•").color(theme::TEXT_MUTED));
                                            draw_branch_chip(ui, &pr.source_branch);
                                            ui.label(
                                                RichText::new("→").color(theme::TEXT_MUTED),
                                            );
                                            draw_branch_chip(ui, &pr.target_branch);
                                        });
                                    })
                                    .response
                                    .interact(Sense::click())
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                                if header_resp.clicked() {
                                    toggled_pr = Some(pr.number);
                                }

                                // Expanded PR Detail Drawer (only shown for clicked PR)
                                if is_expanded {
                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.add_space(8.0);

                                    if app.pr_loading_detail {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(16.0);
                                            ui.label(
                                                RichText::new(t.pr_loading)
                                                    .size(13.0)
                                                    .color(theme::ACCENT_CYAN),
                                            );
                                            ui.add_space(16.0);
                                        });
                                    } else if let Some(detail) = app
                                        .pr_detail
                                        .clone()
                                        .filter(|d| d.summary.number == pr.number)
                                    {
                                        if let Some(act) =
                                            show_expanded_pr_detail(ctx, ui, app, &detail)
                                        {
                                            pending_action = Some((pr.number, act));
                                        }
                                    }
                                }
                            });

                        ui.add_space(7.0);
                    }
                });
        });

    if open_provider_settings {
        app.settings_tab = SettingsTab::Providers;
        app.settings_open = true;
    }
    if open_provider_help {
        app.help_search = spec.kind.label().to_string();
        app.help_open = true;
    }
    if trigger_reload_list {
        app.fetch_pr_list(Some(ctx.clone()));
    }
    if let Some(num) = toggled_pr {
        app.toggle_pr_expand(num, ctx.clone());
    }
    if let Some((num, act)) = pending_action {
        app.run_pr_action(num, act, ctx.clone());
    }
}

fn show_expanded_pr_detail(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    app: &mut GitKrakenApp,
    detail: &PullRequestDetail,
) -> Option<PrActionKind> {
    let t = app.t();
    let pr = &detail.summary;
    let busy = app.pr_action_busy;
    let is_open_like = matches!(pr.state, PrState::Open | PrState::Draft);
    let mut pending_action: Option<PrActionKind> = None;

    // 1. Metadata + Checkout / Open in browser
    ui.horizontal(|ui| {
        if let (Some(add), Some(del)) = (pr.additions, pr.deletions) {
            ui.label(
                RichText::new(format!("+{add}"))
                    .monospace()
                    .size(12.0)
                    .color(theme::ACCENT_GREEN),
            );
            ui.label(
                RichText::new(format!("-{del}"))
                    .monospace()
                    .size(12.0)
                    .color(theme::ACCENT_ROSE),
            );
            ui.label(RichText::new("•").color(theme::TEXT_MUTED));
        }
        ui.label(
            RichText::new(format!(
                "{} {} • {} {}",
                detail.commits.len(),
                t.pr_tab_commits,
                detail.files.len(),
                t.pr_tab_files
            ))
            .size(11.5)
            .color(theme::TEXT_SECONDARY),
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

    ui.add_space(8.0);

    // 2. Action Bar (Merge strategy + Merge / Approve / Close-Decline / Reopen)
    egui::Frame::none()
        .fill(theme::BG_SURFACE)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
        .inner_margin(Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if is_open_like {
                    let strat_label = match app.pr_merge_strategy {
                        MergeStrategy::Merge => t.pr_merge_commit,
                        MergeStrategy::Squash => t.pr_squash_merge,
                        MergeStrategy::Rebase => t.pr_rebase_merge,
                    };

                    egui::ComboBox::from_id_salt(("pr_merge_strategy_combo", pr.number))
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
                            RichText::new("× Conflicts")
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
                        RichText::new(format!(
                            "● {} — {} → {}",
                            t.pr_merged, pr.source_branch, pr.target_branch
                        ))
                        .color(theme::ACCENT_PURPLE)
                        .strong(),
                    );
                }
            });
        });

    ui.add_space(8.0);

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
            let btn = egui::Button::new(RichText::new(label).size(12.0).color(if active {
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
            .rounding(Rounding::same(6.0))
            .min_size(Vec2::new(0.0, 26.0));

            if ui.add(btn).clicked() {
                app.pr_detail_tab = tab;
            }
        }
    });

    ui.add_space(8.0);

    // 4. Tab Content
    match app.pr_detail_tab {
        PrDetailTab::Conversation => {
            // Description card
            egui::Frame::none()
                .fill(theme::BG_PRIMARY)
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&pr.author)
                                .strong()
                                .size(12.0)
                                .color(theme::ACCENT_CYAN),
                        );
                        if !pr.created_at.is_empty() {
                            let pretty = format_human_date(&pr.created_at, app.config.language);
                            ui.label(
                                RichText::new(pretty)
                                    .size(11.0)
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
                        render_pr_body(ui, &pr.body, 12.5);
                    }
                });

            ui.add_space(8.0);

            for comment in &detail.comments {
                let border = match comment.kind {
                    CommentKind::Approved => theme::ACCENT_GREEN.linear_multiply(0.6),
                    CommentKind::ChangesRequested => theme::ACCENT_ROSE.linear_multiply(0.6),
                    CommentKind::Comment => theme::BORDER_SUBTLE,
                };

                egui::Frame::none()
                    .fill(theme::BG_PRIMARY)
                    .rounding(Rounding::same(7.0))
                    .stroke(Stroke::new(1.0_f32, border))
                    .inner_margin(Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&comment.author)
                                    .strong()
                                    .size(12.0)
                                    .color(Color32::WHITE),
                            );
                            match comment.kind {
                                CommentKind::Approved => {
                                    ui.label(
                                        RichText::new(t.pr_approved_badge)
                                            .size(11.0)
                                            .strong()
                                            .color(theme::ACCENT_GREEN),
                                    );
                                }
                                CommentKind::ChangesRequested => {
                                    ui.label(
                                        RichText::new(t.pr_changes_requested_badge)
                                            .size(11.0)
                                            .strong()
                                            .color(theme::ACCENT_ROSE),
                                    );
                                }
                                CommentKind::Comment => {}
                            }
                            if !comment.created_at.is_empty() {
                                let pretty =
                                    format_human_date(&comment.created_at, app.config.language);
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(pretty)
                                            .size(11.0)
                                            .color(theme::TEXT_MUTED),
                                    );
                                });
                            }
                        });
                        if !comment.body.trim().is_empty() {
                            ui.add_space(4.0);
                            render_pr_body(ui, &comment.body, 12.0);
                        }
                    });
                ui.add_space(5.0);
            }

            ui.add_space(6.0);

            // Comment / Review composer
            egui::Frame::none()
                .fill(theme::BG_PRIMARY)
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut app.pr_comment_draft)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .hint_text(t.pr_comment_hint),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !busy && !app.pr_comment_draft.trim().is_empty(),
                                egui::Button::new(t.pr_comment_btn)
                                    .min_size(Vec2::new(86.0, 26.0)),
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
                    .fill(theme::BG_PRIMARY)
                    .rounding(Rounding::same(7.0))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(12.0, 7.0))
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
                                    .size(12.5)
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
                    .fill(theme::BG_PRIMARY)
                    .rounding(Rounding::same(7.0))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(10.0, 7.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        egui::CollapsingHeader::new(
                            RichText::new(format!(
                                "[{}]  {}   +{} -{}",
                                file.status, file.path, file.additions, file.deletions
                            ))
                            .monospace()
                            .size(11.5)
                            .color(st_col),
                        )
                        .id_salt(("pr_file", pr.number, &file.path))
                        .default_open(detail.files.len() <= 6)
                        .show(ui, |ui| {
                            if file.patch.trim().is_empty() {
                                ui.label(
                                    RichText::new(t.no_diff)
                                        .color(theme::TEXT_MUTED)
                                        .italics(),
                                );
                            } else {
                                egui::ScrollArea::horizontal()
                                    .id_salt(("pr_patch", pr.number, &file.path))
                                    .show(ui, |ui| {
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
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
                                                .font(FontId::monospace(11.5))
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
                ui.add_space(6.0);
            }
        }
    }

    pending_action
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
        .fill(theme::BG_PRIMARY)
        .rounding(Rounding::same(5.0))
        .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
        .inner_margin(Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(name)
                    .monospace()
                    .size(11.0)
                    .color(theme::ACCENT_CYAN),
            );
        });
}

fn strip_html_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        if let Some(end) = rest[start + 4..].find("-->") {
            rest = &rest[start + 4 + end + 3..];
        } else {
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn looks_like_markdown(text: &str) -> bool {
    if text.contains("```")
        || text.contains("**")
        || text.contains("~~")
        || text.contains('`')
        || (text.contains("](") && text.contains('['))
    {
        return true;
    }
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("# ")
            || t.starts_with("## ")
            || t.starts_with("### ")
            || t.starts_with("#### ")
            || t.starts_with("- ")
            || t.starts_with("* ")
            || t.starts_with("> ")
            || t == "---"
            || t == "***"
            || t == "___"
            || (t.starts_with('|') && t.ends_with('|'))
            || parse_ordered_list_prefix(t).is_some()
        {
            return true;
        }
    }
    false
}

fn render_pr_body(ui: &mut egui::Ui, raw: &str, base_size: f32) {
    let cleaned = strip_html_comments(raw);
    let body = cleaned.trim();
    if body.is_empty() {
        return;
    }

    if !looks_like_markdown(body) {
        ui.label(
            RichText::new(body)
                .size(base_size)
                .color(theme::TEXT_PRIMARY),
        );
        return;
    }

    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Blank line spacing
        if trimmed.is_empty() {
            ui.add_space(5.0);
            i += 1;
            continue;
        }

        // Fenced code block (``` or ~~~)
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fence = &trimmed[..3];
            let lang = trimmed[3..].trim();
            i += 1;
            let mut code = String::new();
            while i < lines.len() && !lines[i].trim().starts_with(fence) {
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1; // skip closing fence
            }
            egui::Frame::none()
                .fill(theme::BG_SURFACE)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                .inner_margin(Margin::symmetric(10.0, 7.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if !lang.is_empty() {
                        ui.label(
                            RichText::new(lang)
                                .monospace()
                                .size(10.0)
                                .color(theme::TEXT_MUTED),
                        );
                        ui.add_space(2.0);
                    }
                    egui::ScrollArea::horizontal()
                        .id_salt(("md_code", i, code.len()))
                        .show(ui, |ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                            ui.label(
                                RichText::new(&code)
                                    .font(FontId::monospace(11.5))
                                    .color(theme::TEXT_PRIMARY),
                            );
                        });
                });
            ui.add_space(5.0);
            continue;
        }

        // Horizontal rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            ui.add_space(3.0);
            ui.separator();
            ui.add_space(3.0);
            i += 1;
            continue;
        }

        // Headings (# .. ####)
        if let Some((level, heading)) = parse_heading(trimmed) {
            let (size, color) = match level {
                1 => (base_size + 4.5, Color32::WHITE),
                2 => (base_size + 2.5, Color32::WHITE),
                3 => (base_size + 1.0, theme::ACCENT_CYAN),
                _ => (base_size + 0.5, theme::TEXT_PRIMARY),
            };
            ui.add_space(4.0);
            ui.label(RichText::new(heading).size(size).strong().color(color));
            if level <= 2 {
                ui.separator();
            }
            ui.add_space(2.0);
            i += 1;
            continue;
        }

        // Markdown table block
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let mut rows: Vec<Vec<String>> = Vec::new();
            while i < lines.len() {
                let row_t = lines[i].trim();
                if !(row_t.starts_with('|') && row_t.ends_with('|')) {
                    break;
                }
                let cells: Vec<String> = row_t
                    .trim_matches('|')
                    .split('|')
                    .map(|c| c.trim().to_string())
                    .collect();
                let is_sep = !cells.is_empty()
                    && cells
                        .iter()
                        .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')));
                if !is_sep {
                    rows.push(cells);
                }
                i += 1;
            }
            if !rows.is_empty() {
                egui::Frame::none()
                    .fill(theme::BG_SURFACE)
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        egui::Grid::new(("md_table", i, rows.len()))
                            .striped(true)
                            .spacing([14.0, 4.0])
                            .show(ui, |ui| {
                                for (r_idx, row) in rows.iter().enumerate() {
                                    for cell in row {
                                        if r_idx == 0 {
                                            ui.label(
                                                RichText::new(cell)
                                                    .size(base_size - 0.5)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );
                                        } else {
                                            render_inline_md(ui, cell, base_size - 0.5);
                                        }
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                ui.add_space(5.0);
            }
            continue;
        }

        // Blockquote (> ...)
        if let Some(quoted) = trimmed.strip_prefix("> ").or_else(|| trimmed.strip_prefix('>')) {
            let mut quote_lines = vec![quoted.trim()];
            i += 1;
            while i < lines.len() {
                let qt = lines[i].trim();
                if let Some(q) = qt.strip_prefix("> ").or_else(|| qt.strip_prefix('>')) {
                    quote_lines.push(q.trim());
                    i += 1;
                } else {
                    break;
                }
            }
            egui::Frame::none()
                .fill(theme::BG_SURFACE)
                .rounding(Rounding::same(5.0))
                .stroke(Stroke::new(1.0_f32, theme::ACCENT_CYAN.linear_multiply(0.5)))
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for ql in quote_lines {
                        render_inline_md(ui, ql, base_size - 0.5);
                    }
                });
            ui.add_space(4.0);
            continue;
        }

        // Task list (- [x] / - [ ])
        let indent_spaces = line.len() - line.trim_start().len();
        let indent_px = ((indent_spaces / 2) as f32 * 14.0).min(42.0);
        if let Some((checked, rest)) = parse_task_item(trimmed) {
            ui.horizontal(|ui| {
                ui.add_space(indent_px);
                if checked {
                    ui.label(
                        RichText::new("✓")
                            .size(base_size)
                            .strong()
                            .color(theme::ACCENT_GREEN),
                    );
                } else {
                    ui.label(
                        RichText::new("□")
                            .size(base_size)
                            .color(theme::TEXT_MUTED),
                    );
                }
                render_inline_md(ui, rest, base_size);
            });
            i += 1;
            continue;
        }

        // Unordered bullet list (- / * / +)
        if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
        {
            ui.horizontal(|ui| {
                ui.add_space(indent_px + 4.0);
                ui.label(
                    RichText::new("•")
                        .size(base_size)
                        .color(theme::ACCENT_CYAN),
                );
                render_inline_md(ui, rest, base_size);
            });
            i += 1;
            continue;
        }

        // Ordered numbered list (1. ...)
        if let Some((num_lbl, rest)) = parse_ordered_list_prefix(trimmed) {
            ui.horizontal(|ui| {
                ui.add_space(indent_px + 4.0);
                ui.label(
                    RichText::new(num_lbl)
                        .size(base_size)
                        .color(theme::ACCENT_CYAN),
                );
                render_inline_md(ui, rest, base_size);
            });
            i += 1;
            continue;
        }

        // Normal paragraph line with inline markdown
        render_inline_md(ui, trimmed, base_size);
        i += 1;
    }
}

fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if (1..=6).contains(&hashes) {
        let rest = line[hashes..].strip_prefix(' ')?;
        Some((hashes, rest.trim()))
    } else {
        None
    }
}

fn parse_task_item(trimmed: &str) -> Option<(bool, &str)> {
    for prefix in ["- [x] ", "- [X] ", "* [x] ", "* [X] "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some((true, rest));
        }
    }
    for prefix in ["- [ ] ", "* [ ] "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some((false, rest));
        }
    }
    None
}

fn parse_ordered_list_prefix(trimmed: &str) -> Option<(&str, &str)> {
    let digits = trimmed.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 || digits > 3 {
        return None;
    }
    let after = &trimmed[digits..];
    let rest = after.strip_prefix(". ")?;
    Some((&trimmed[..digits + 1], rest))
}

/// Renders inline Markdown (`**bold**`, `*italic*`, `` `code` ``, `~~strike~~`, `[text](url)`).
fn render_inline_md(ui: &mut egui::Ui, text: &str, base_size: f32) {
    // Extract clickable markdown links so clicking opens them in the browser
    let mut job = LayoutJob::default();
    let mut links: Vec<(String, String)> = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    let normal_fmt = TextFormat {
        font_id: FontId::proportional(base_size),
        color: theme::TEXT_PRIMARY,
        ..Default::default()
    };
    let bold_fmt = TextFormat {
        font_id: FontId::proportional(base_size),
        color: Color32::WHITE,
        ..Default::default()
    };
    let italic_fmt = TextFormat {
        font_id: FontId::proportional(base_size),
        color: theme::TEXT_SECONDARY,
        italics: true,
        ..Default::default()
    };
    let strike_fmt = TextFormat {
        font_id: FontId::proportional(base_size),
        color: theme::TEXT_MUTED,
        strikethrough: Stroke::new(1.0_f32, theme::TEXT_MUTED),
        ..Default::default()
    };
    let code_fmt = TextFormat {
        font_id: FontId::monospace((base_size - 0.5).max(10.5)),
        color: theme::ACCENT_CYAN,
        background: theme::BG_SURFACE,
        ..Default::default()
    };
    let link_fmt = TextFormat {
        font_id: FontId::proportional(base_size),
        color: theme::ACCENT_CYAN,
        underline: Stroke::new(1.0_f32, theme::ACCENT_CYAN),
        ..Default::default()
    };

    let mut plain_start = 0;
    let flush_plain = |job: &mut LayoutJob, start: usize, end: usize| {
        if end > start {
            job.append(&text[start..end], 0.0, normal_fmt.clone());
        }
    };

    while i < bytes.len() {
        // Inline code `...`
        if bytes[i] == b'`' {
            if let Some(rel_end) = text[i + 1..].find('`') {
                flush_plain(&mut job, plain_start, i);
                let code = &text[i + 1..i + 1 + rel_end];
                job.append(&format!(" {code} "), 0.0, code_fmt.clone());
                i = i + 2 + rel_end;
                plain_start = i;
                continue;
            }
        }
        // Bold **...**
        if text[i..].starts_with("**") {
            if let Some(rel_end) = text[i + 2..].find("**") {
                flush_plain(&mut job, plain_start, i);
                job.append(&text[i + 2..i + 2 + rel_end], 0.0, bold_fmt.clone());
                i = i + 4 + rel_end;
                plain_start = i;
                continue;
            }
        }
        // Strikethrough ~~...~~
        if text[i..].starts_with("~~") {
            if let Some(rel_end) = text[i + 2..].find("~~") {
                flush_plain(&mut job, plain_start, i);
                job.append(&text[i + 2..i + 2 + rel_end], 0.0, strike_fmt.clone());
                i = i + 4 + rel_end;
                plain_start = i;
                continue;
            }
        }
        // Markdown link [label](url) or image ![alt](url)
        let is_img = bytes[i] == b'!' && text[i..].starts_with("![");
        if bytes[i] == b'[' || is_img {
            let open_br = if is_img { i + 1 } else { i };
            if let Some(close_br_rel) = text[open_br + 1..].find("](") {
                let url_start = open_br + 1 + close_br_rel + 2;
                if let Some(url_end_rel) = text[url_start..].find(')') {
                    let label = &text[open_br + 1..open_br + 1 + close_br_rel];
                    let url = &text[url_start..url_start + url_end_rel];
                    if url.starts_with("http://") || url.starts_with("https://") {
                        flush_plain(&mut job, plain_start, i);
                        let shown = if label.is_empty() { url } else { label };
                        job.append(shown, 0.0, link_fmt.clone());
                        links.push((shown.to_string(), url.to_string()));
                        i = url_start + url_end_rel + 1;
                        plain_start = i;
                        continue;
                    }
                }
            }
        }
        // Italic *...* (single asterisk, not followed by space)
        if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] != b' ' && bytes[i + 1] != b'*' {
            if let Some(rel_end) = text[i + 1..].find('*') {
                if rel_end > 0 {
                    flush_plain(&mut job, plain_start, i);
                    job.append(&text[i + 1..i + 1 + rel_end], 0.0, italic_fmt.clone());
                    i = i + 2 + rel_end;
                    plain_start = i;
                    continue;
                }
            }
        }
        // Advance to next UTF-8 char boundary
        let ch_len = text[i..].chars().next().map_or(1, char::len_utf8);
        i += ch_len;
    }
    flush_plain(&mut job, plain_start, bytes.len());

    let resp = ui.label(job);
    if links.len() == 1 {
        let url = &links[0].1;
        if resp
            .on_hover_text(url)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            let _ = webbrowser::open(url);
        }
    } else if links.len() > 1 {
        ui.horizontal_wrapped(|ui| {
            for (lbl, url) in links {
                if ui
                    .small_button(RichText::new(format!("→ {lbl}")).color(theme::ACCENT_CYAN))
                    .on_hover_text(&url)
                    .clicked()
                {
                    let _ = webbrowser::open(&url);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_markdown_vs_plain_text() {
        assert!(looks_like_markdown("## Summary\n- item 1"));
        assert!(looks_like_markdown("Fixes **auth** bug with `token`"));
        assert!(looks_like_markdown("```rust\nfn main() {}\n```"));
        assert!(!looks_like_markdown("Simple plain text description."));
    }

    #[test]
    fn strips_html_template_comments() {
        let src = "<!-- fill this out -->\n## Summary\nHello";
        assert_eq!(strip_html_comments(src).trim(), "## Summary\nHello");
    }
}
