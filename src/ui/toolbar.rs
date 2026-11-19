use std::path::PathBuf;

use eframe::egui::{self, Color32, Margin, RichText, Rounding, Stroke};

use crate::app::{app_logo_image, GitKrakenApp};
use crate::config::{CommitOrder, GraphStyle};
use crate::i18n::{humanize_git_error, Language};
use crate::theme;

pub fn show_tabs(ctx: &egui::Context, app: &mut GitKrakenApp) {
    let left_pad = if cfg!(target_os = "macos") { 78.0 } else { 10.0 };

    let frame = egui::Frame::none()
        .fill(Color32::from_rgb(20, 22, 28))
        .inner_margin(Margin {
            left: left_pad,
            right: 12.0,
            top: 7.0,
            bottom: 0.0,
        });

    let mut switch_to: Option<usize> = None;
    let mut close_idx: Option<usize> = None;
    let mut open_new = false;

    egui::TopBottomPanel::top("chrome_repo_tabs")
        .exact_height(36.0)
        .frame(frame)
        .show(ctx, |ui| {
            // Allow dragging window from empty areas of the integrated title/tab bar
            let bg_rect = ui.max_rect();
            let drag_resp = ui.interact(
                bg_rect,
                ui.id().with("titlebar_drag_area"),
                egui::Sense::click_and_drag(),
            );
            if drag_resp.double_clicked() {
                let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
            } else if drag_resp.drag_started_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            if app.tabs.is_empty() {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("gity x")
                            .size(12.5)
                            .strong()
                            .color(theme::TEXT_SECONDARY),
                    );
                });
                return;
            }

            egui::ScrollArea::horizontal()
                .id_salt("repo_tabs_scroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;

                        let tab_rounding = Rounding {
                            nw: 8.0,
                            ne: 8.0,
                            sw: 0.0,
                            se: 0.0,
                        };

                        for (idx, tab) in app.tabs.iter().enumerate() {
                            let is_active = app.active_tab == Some(idx);
                            let bg = if is_active {
                                theme::BG_HEADER
                            } else {
                                Color32::from_rgb(25, 27, 34)
                            };
                            let border = if is_active {
                                theme::BORDER_ELEVATED
                            } else {
                                theme::BORDER_SUBTLE
                            };

                            let resp = egui::Frame::none()
                                .fill(bg)
                                .rounding(tab_rounding)
                                .stroke(Stroke::new(1.0_f32, border))
                                .inner_margin(Margin::symmetric(10.0, 5.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let name_color = if is_active {
                                            Color32::WHITE
                                        } else {
                                            theme::TEXT_SECONDARY
                                        };
                                        let name_resp = ui
                                            .add(
                                                egui::Label::new(
                                                    RichText::new(&tab.name)
                                                        .size(12.0)
                                                        .strong()
                                                        .color(name_color),
                                                )
                                                .sense(egui::Sense::click()),
                                            )
                                            .on_hover_text(tab.path.display().to_string())
                                            .on_hover_cursor(egui::CursorIcon::PointingHand);

                                        if name_resp.clicked() {
                                            switch_to = Some(idx);
                                        }
                                        if name_resp.middle_clicked() {
                                            close_idx = Some(idx);
                                        }

                                        if !tab.branch.is_empty() {
                                            let br_resp = ui
                                                .add(
                                                    egui::Label::new(
                                                        RichText::new(&tab.branch)
                                                            .size(10.5)
                                                            .color(if is_active {
                                                                theme::ACCENT_CYAN
                                                            } else {
                                                                theme::TEXT_MUTED
                                                            }),
                                                    )
                                                    .sense(egui::Sense::click()),
                                                )
                                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                                            if br_resp.clicked() {
                                                switch_to = Some(idx);
                                            }
                                            if br_resp.middle_clicked() {
                                                close_idx = Some(idx);
                                            }
                                        }

                                        ui.add_space(4.0);
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new("×")
                                                        .size(11.5)
                                                        .color(theme::TEXT_MUTED),
                                                )
                                                .frame(false),
                                            )
                                            .on_hover_text("Close tab (Cmd+W)")
                                            .clicked()
                                        {
                                            close_idx = Some(idx);
                                        }
                                    });
                                })
                                .response;

                            if is_active {
                                let top_accent = egui::Rect::from_min_max(
                                    egui::Pos2::new(resp.rect.left() + 6.0, resp.rect.top()),
                                    egui::Pos2::new(
                                        resp.rect.right() - 6.0,
                                        resp.rect.top() + 2.0,
                                    ),
                                );
                                ui.painter().rect_filled(
                                    top_accent,
                                    Rounding::same(1.0),
                                    theme::ACCENT_CYAN,
                                );
                            }
                        }

                        // Chrome-style "+" new tab button
                        if ui
                            .add(
                                egui::Button::new(RichText::new("+").strong().size(13.0))
                                    .min_size(egui::vec2(24.0, 22.0)),
                            )
                            .on_hover_text("Open repository in new tab (Cmd+N / Cmd+O)")
                            .clicked()
                        {
                            open_new = true;
                        }
                    });
                });
        });

    if let Some(idx) = close_idx {
        app.close_tab(idx);
    } else if let Some(idx) = switch_to {
        app.switch_tab(idx);
    } else if open_new {
        app.open_repo_dialog();
    }
}

/// Custom vector sidebar toggle icon (Antigravity style — never shows missing glyph boxes).
fn draw_sidebar_toggle_btn(
    ui: &mut egui::Ui,
    is_left: bool,
    active: bool,
    tooltip: &str,
) -> egui::Response {
    let size = egui::vec2(26.0, 26.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let resp = resp
        .on_hover_text(tooltip)
        .on_hover_cursor(egui::CursorIcon::PointingHand);

    let bg = if resp.hovered() {
        theme::BG_ROW_HOVER
    } else {
        theme::BG_SURFACE
    };
    let border = if resp.hovered() {
        theme::BORDER_ELEVATED
    } else {
        theme::BORDER_SUBTLE
    };
    let icon_col = if active {
        theme::ACCENT_CYAN
    } else {
        theme::TEXT_MUTED
    };

    let p = ui.painter();
    p.rect_filled(rect, Rounding::same(6.0), bg);
    p.rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0_f32, border));

    let icon_rect = rect.shrink2(egui::vec2(6.0, 7.0));
    p.rect_stroke(
        icon_rect,
        Rounding::same(2.0),
        Stroke::new(1.2_f32, icon_col),
    );
    let div_x = if is_left {
        icon_rect.left() + 4.5
    } else {
        icon_rect.right() - 4.5
    };
    p.line_segment(
        [
            egui::Pos2::new(div_x, icon_rect.top()),
            egui::Pos2::new(div_x, icon_rect.bottom()),
        ],
        Stroke::new(1.2_f32, icon_col),
    );

    resp
}

pub fn show(ctx: &egui::Context, app: &mut GitKrakenApp) {
    let t = app.t();
    let frame = egui::Frame::none()
        .fill(theme::BG_HEADER)
        .inner_margin(Margin::symmetric(12.0, 6.0))
        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE));

    egui::TopBottomPanel::top("top_toolbar")
        .frame(frame)
        .show(ctx, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 28.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    if app.repo.is_some()
                        && draw_sidebar_toggle_btn(
                            ui,
                            true,
                            app.config.show_left_sidebar,
                            t.toggle_left_sidebar,
                        )
                        .clicked()
                    {
                        app.toggle_left_sidebar();
                    }

                    if let Some(img) = app_logo_image(app.logo_texture.as_ref(), 24.0, 5.0) {
                        ui.add(img).on_hover_text("gity x");
                    }
                    ui.separator();

                    if app.repo.is_none()
                        && ui
                            .add(
                                egui::Button::new(t.open_repo)
                                    .min_size(egui::vec2(0.0, 26.0)),
                            )
                            .clicked()
                    {
                        app.open_repo_dialog();
                    }

                    if let Some(repo) = app.repo.as_ref() {
                        let repo_name = repo.repo_name();
                        let recents = app.config.recent_repos.clone();
                        let mut open_recent: Option<PathBuf> = None;

                        ui.menu_button(
                            RichText::new(format!("{repo_name}  ▼"))
                                .size(12.5)
                                .color(theme::TEXT_PRIMARY),
                            |ui| {
                                if ui.button(format!("+ {}", t.open_repo)).clicked() {
                                    app.open_repo_dialog();
                                    ui.close_menu();
                                }
                                if !recents.is_empty() {
                                    ui.separator();
                                    for r in &recents {
                                        if ui.button(r).clicked() {
                                            open_recent = Some(PathBuf::from(r));
                                            ui.close_menu();
                                        }
                                    }
                                }
                            },
                        );
                        if let Some(path) = open_recent {
                            app.open_repo(path);
                            return;
                        }

                        // Current branch switcher dropdown (aligned 26px height)
                        let mut branch_text = format!("● {}", app.current_branch);
                        if let Some((ahead, behind)) = app.ahead_behind {
                            if ahead > 0 || behind > 0 {
                                branch_text.push_str(&format!("  ↑{ahead} ↓{behind}"));
                            }
                        }
                        branch_text.push_str("  ▼");

                        let local_branches: Vec<(String, bool)> = app
                            .branches
                            .iter()
                            .filter(|b| !b.is_remote)
                            .map(|b| (b.name.clone(), b.is_current))
                            .collect();
                        let remote_branches: Vec<String> = app
                            .branches
                            .iter()
                            .filter(|b| b.is_remote)
                            .map(|b| b.name.clone())
                            .collect();
                        let mut checkout_target: Option<String> = None;
                        let mut open_new_branch = false;

                        ui.menu_button(
                            RichText::new(branch_text)
                                .color(theme::ACCENT_CYAN)
                                .size(12.0),
                            |ui| {
                                ui.set_min_width(240.0);

                                // Search input inside branch dropdown
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("🔍")
                                            .size(11.5)
                                            .color(theme::TEXT_MUTED),
                                    );
                                    let w = if app.branch_search.is_empty() {
                                        200.0
                                    } else {
                                        178.0
                                    };
                                    ui.add(
                                        egui::TextEdit::singleline(&mut app.branch_search)
                                            .desired_width(w)
                                            .hint_text("Filter branches..."),
                                    );
                                    if !app.branch_search.is_empty() && ui.small_button("×").clicked()
                                    {
                                        app.branch_search.clear();
                                    }
                                });

                                ui.add_space(4.0);
                                if ui.button(format!("+ {}", t.create_branch_title)).clicked() {
                                    open_new_branch = true;
                                    app.branch_search.clear();
                                    ui.close_menu();
                                }
                                ui.separator();

                                let q = app.branch_search.trim().to_lowercase();
                                let filtered_local: Vec<&(String, bool)> = local_branches
                                    .iter()
                                    .filter(|(n, _)| q.is_empty() || n.to_lowercase().contains(&q))
                                    .collect();
                                let filtered_remote: Vec<&String> = remote_branches
                                    .iter()
                                    .filter(|n| q.is_empty() || n.to_lowercase().contains(&q))
                                    .collect();

                                egui::ScrollArea::vertical()
                                    .id_salt("toolbar_branch_switcher_scroll")
                                    .max_height(320.0)
                                    .show(ui, |ui| {
                                        if filtered_local.is_empty() && filtered_remote.is_empty() {
                                            ui.label(
                                                RichText::new("No matching branches")
                                                    .size(12.0)
                                                    .color(theme::TEXT_MUTED)
                                                    .italics(),
                                            );
                                            return;
                                        }

                                        if !filtered_local.is_empty() {
                                            ui.label(
                                                RichText::new(format!(
                                                    "{} ({})",
                                                    t.local_branches,
                                                    filtered_local.len()
                                                ))
                                                .size(10.5)
                                                .color(theme::TEXT_MUTED),
                                            );
                                            for (name, is_cur) in filtered_local {
                                                let lbl = if *is_cur {
                                                    RichText::new(format!("● {name}"))
                                                        .color(theme::ACCENT_CYAN)
                                                        .strong()
                                                } else {
                                                    RichText::new(format!("  {name}"))
                                                        .color(theme::TEXT_PRIMARY)
                                                };
                                                if ui.selectable_label(*is_cur, lbl).clicked() {
                                                    if !*is_cur {
                                                        checkout_target = Some(name.clone());
                                                    }
                                                    app.branch_search.clear();
                                                    ui.close_menu();
                                                }
                                            }
                                        }

                                        if !filtered_remote.is_empty() {
                                            ui.add_space(4.0);
                                            ui.separator();
                                            ui.label(
                                                RichText::new(format!(
                                                    "{} ({})",
                                                    t.remote_branches,
                                                    filtered_remote.len()
                                                ))
                                                .size(10.5)
                                                .color(theme::TEXT_MUTED),
                                            );
                                            for name in filtered_remote {
                                                if ui
                                                    .selectable_label(
                                                        false,
                                                        RichText::new(format!("☁ {name}"))
                                                            .color(theme::ACCENT_PURPLE),
                                                    )
                                                    .clicked()
                                                {
                                                    checkout_target = Some(name.clone());
                                                    app.branch_search.clear();
                                                    ui.close_menu();
                                                }
                                            }
                                        }
                                    });
                            },
                        );

                        if open_new_branch {
                            app.open_branch_modal(None);
                        } else if let Some(target) = checkout_target {
                            app.run_git_action(&format!("Checked out '{target}'."), |r| {
                                r.checkout(&target)
                            });
                        }

                        ui.separator();

                        // Mode switcher pill: Git Graph vs Agent Mode (aligned 26px height)
                        let graph_active = !app.config.agent_mode;
                        let graph_btn = egui::Button::new(
                            RichText::new(format!("● {}", t.mode_graph))
                                .size(12.0)
                                .color(if graph_active {
                                    Color32::WHITE
                                } else {
                                    theme::TEXT_SECONDARY
                                }),
                        )
                        .fill(if graph_active {
                            theme::BG_ROW_SELECTED
                        } else {
                            theme::BG_PRIMARY
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if graph_active {
                                theme::ACCENT_CYAN
                            } else {
                                theme::BORDER_SUBTLE
                            },
                        ))
                        .rounding(Rounding::same(7.0))
                        .min_size(egui::vec2(0.0, 26.0));

                        if ui.add(graph_btn).clicked() {
                            app.set_agent_mode(false);
                        }

                        let agent_active = app.config.agent_mode;
                        let agent_text = if !app.status_files.is_empty() {
                            format!("◆ {} ({})", t.mode_agent, app.status_files.len())
                        } else {
                            format!("◆ {}", t.mode_agent)
                        };
                        let agent_btn = egui::Button::new(
                            RichText::new(agent_text).size(12.0).color(if agent_active {
                                Color32::WHITE
                            } else {
                                theme::TEXT_SECONDARY
                            }),
                        )
                        .fill(if agent_active {
                            theme::BG_ROW_SELECTED
                        } else {
                            theme::BG_PRIMARY
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if agent_active {
                                theme::ACCENT_CYAN
                            } else {
                                theme::BORDER_SUBTLE
                            },
                        ))
                        .rounding(Rounding::same(7.0))
                        .min_size(egui::vec2(0.0, 26.0));

                        if ui.add(agent_btn).on_hover_text("Cmd+E").clicked() {
                            app.set_agent_mode(true);
                        }

                        ui.separator();

                        // Sync group with lightweight Fetch / Pull / Push status indicators
                        let btn_h = egui::vec2(0.0, 26.0);
                        let (ahead, behind) = app.ahead_behind.unwrap_or((0, 0));

                        let fetch_has = app.remote_updates_available;
                        let fetch_btn = egui::Button::new(
                            RichText::new(if fetch_has {
                                format!("↓ {} ●", t.fetch)
                            } else {
                                format!("↓ {}", t.fetch)
                            })
                            .color(if fetch_has {
                                theme::ACCENT_AMBER
                            } else {
                                theme::TEXT_PRIMARY
                            }),
                        )
                        .fill(if fetch_has {
                            Color32::from_rgb(56, 46, 30)
                        } else {
                            theme::BG_SURFACE
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if fetch_has {
                                theme::ACCENT_AMBER
                            } else {
                                theme::BORDER_ELEVATED
                            },
                        ))
                        .min_size(btn_h);

                        let fetch_resp = ui.add(fetch_btn);
                        let fetch_resp = if fetch_has {
                            fetch_resp.on_hover_text("Remote has new commits — click to fetch")
                        } else {
                            fetch_resp
                        };
                        if fetch_resp.clicked() {
                            app.remote_updates_available = false;
                            app.run_git_action(t.msg_fetched, |r| r.fetch());
                        }

                        let pull_has = behind > 0;
                        let pull_btn = egui::Button::new(
                            RichText::new(if pull_has {
                                format!("↓ {} ({behind})", t.pull)
                            } else {
                                format!("↓ {}", t.pull)
                            })
                            .color(if pull_has {
                                Color32::WHITE
                            } else {
                                theme::TEXT_PRIMARY
                            }),
                        )
                        .fill(if pull_has {
                            Color32::from_rgb(44, 54, 86)
                        } else {
                            theme::BG_SURFACE
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if pull_has {
                                theme::ACCENT_CYAN
                            } else {
                                theme::BORDER_ELEVATED
                            },
                        ))
                        .min_size(btn_h);

                        let pull_resp = ui.add(pull_btn);
                        let pull_resp = if pull_has {
                            pull_resp
                                .on_hover_text(format!("{behind} incoming commit(s) ready to pull"))
                        } else {
                            pull_resp
                        };
                        if pull_resp.clicked() {
                            app.remote_updates_available = false;
                            app.run_git_action(t.msg_pulled, |r| r.pull());
                        }

                        let push_has = ahead > 0;
                        let push_btn = egui::Button::new(
                            RichText::new(if push_has {
                                format!("↑ {} ({ahead})", t.push)
                            } else {
                                format!("↑ {}", t.push)
                            })
                            .color(if push_has {
                                Color32::WHITE
                            } else {
                                theme::TEXT_PRIMARY
                            }),
                        )
                        .fill(if push_has {
                            Color32::from_rgb(26, 56, 44)
                        } else {
                            theme::BG_SURFACE
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if push_has {
                                theme::ACCENT_GREEN
                            } else {
                                theme::BORDER_ELEVATED
                            },
                        ))
                        .min_size(btn_h);

                        let push_resp = ui.add(push_btn);
                        let push_resp = if push_has {
                            push_resp
                                .on_hover_text(format!("{ahead} local commit(s) ready to push"))
                        } else {
                            push_resp
                        };
                        if push_resp.clicked() {
                            app.run_git_action(t.msg_pushed, |r| r.push());
                        }

                        ui.separator();

                        // Workspace actions group
                        if ui
                            .add(egui::Button::new(t.new_branch).min_size(btn_h))
                            .clicked()
                        {
                            app.open_branch_modal(None);
                        }
                        if ui
                            .add(egui::Button::new(t.stash).min_size(btn_h))
                            .clicked()
                        {
                            app.stash_modal.open = true;
                            app.stash_modal.message.clear();
                        }
                        if ui.add(egui::Button::new(t.pop).min_size(btn_h)).clicked() {
                            app.run_git_action(t.msg_stash_popped, |r| r.stash_pop());
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !app.config.agent_mode
                                && draw_sidebar_toggle_btn(
                                    ui,
                                    false,
                                    app.config.show_right_sidebar,
                                    t.toggle_right_sidebar,
                                )
                                .clicked()
                            {
                                app.toggle_right_sidebar();
                            }
                            if ui
                                .add(
                                    egui::Button::new("⚙")
                                        .min_size(egui::vec2(26.0, 26.0)),
                                )
                                .on_hover_text(t.settings)
                                .clicked()
                            {
                                app.settings_open = !app.settings_open;
                            }
                            if ui
                                .add(
                                    egui::Button::new("↻")
                                        .min_size(egui::vec2(26.0, 26.0)),
                                )
                                .on_hover_text(format!("{} (Cmd+R)", t.refresh))
                                .clicked()
                            {
                                app.refresh();
                                app.spawn_remote_check(Some(ctx.clone()));
                                app.set_info(t.msg_refreshed);
                            }
                        });
                    } else {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(format!("⚙ {}", t.settings))
                                        .min_size(egui::vec2(0.0, 26.0)),
                                )
                                .clicked()
                            {
                                app.settings_open = !app.settings_open;
                            }
                        });
                    }
                },
            );

            // Active Git operation banner (Cherry-pick / Merge / Rebase in progress)
            if app.is_cherry_picking || app.is_merging || app.is_rebasing {
                ui.add_space(5.0);
                let state_text = if app.is_rebasing {
                    t.state_rebase
                } else if app.is_cherry_picking {
                    t.state_cherry_pick
                } else {
                    t.state_merge
                };

                egui::Frame::none()
                    .fill(Color32::from_rgb(74, 52, 22))
                    .rounding(Rounding::same(5.0))
                    .inner_margin(Margin::symmetric(10.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("⚠ {state_text}"))
                                    .color(theme::ACCENT_AMBER)
                                    .strong()
                                    .size(12.0),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button(
                                            RichText::new(t.abort_op).color(theme::ACCENT_ROSE),
                                        )
                                        .clicked()
                                    {
                                        if app.is_rebasing {
                                            app.run_git_action("Rebase aborted.", |r| {
                                                r.abort_rebase()
                                            });
                                        } else if app.is_cherry_picking {
                                            app.run_git_action("Cherry-pick aborted.", |r| {
                                                r.abort_cherry_pick()
                                            });
                                        } else {
                                            app.run_git_action("Merge aborted.", |r| {
                                                r.abort_merge()
                                            });
                                        }
                                    }
                                    if ui
                                        .small_button(
                                            RichText::new(t.continue_op)
                                                .color(theme::ACCENT_GREEN),
                                        )
                                        .clicked()
                                    {
                                        if app.is_rebasing {
                                            app.run_git_action("Rebase continued.", |r| {
                                                r.continue_rebase()
                                            });
                                        } else if app.is_cherry_picking {
                                            app.run_git_action("Cherry-pick continued.", |r| {
                                                r.continue_cherry_pick()
                                            });
                                        } else {
                                            app.run_git_action("Merge continued.", |r| {
                                                r.continue_merge()
                                            });
                                        }
                                    }
                                },
                            );
                        });
                    });
            }
        });
}

pub fn show_modals(ctx: &egui::Context, app: &mut GitKrakenApp) {
    show_branch_modal(ctx, app);
    show_rename_modal(ctx, app);
    show_tag_modal(ctx, app);
    show_stash_modal(ctx, app);
    show_settings_modal(ctx, app);
    show_shortcuts_modal(ctx, app);
    show_about_modal(ctx, app);
    show_error_modal(ctx, app);
    show_success_toast(ctx, app);
}

fn show_branch_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.branch_modal.open {
        return;
    }
    let t = app.t();
    let mut close_modal = false;
    let mut create_now = false;

    let local_branches: Vec<String> = app
        .branches
        .iter()
        .filter(|b| !b.is_remote)
        .map(|b| b.name.clone())
        .collect();
    let remote_branches: Vec<String> = app
        .branches
        .iter()
        .filter(|b| b.is_remote)
        .map(|b| b.name.clone())
        .collect();
    let tag_names: Vec<String> = app.tags.iter().map(|tg| tg.name.clone()).collect();
    let current_head_label = format!("{} ({})", t.current_head, app.current_branch);

    egui::Window::new(t.create_branch_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(310.0);

            ui.label(t.base_ref_label);
            let selected_label = app
                .branch_modal
                .start_point
                .as_deref()
                .unwrap_or(&current_head_label)
                .to_string();

            egui::ComboBox::from_id_salt("branch_base_combo")
                .selected_text(selected_label)
                .width(290.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            app.branch_modal.start_point.is_none(),
                            &current_head_label,
                        )
                        .clicked()
                    {
                        app.branch_modal.start_point = None;
                    }
                    if !local_branches.is_empty() {
                        ui.separator();
                        ui.label(
                            RichText::new(t.local_branches)
                                .size(10.5)
                                .color(theme::TEXT_MUTED),
                        );
                        for b in &local_branches {
                            let sel = app.branch_modal.start_point.as_deref() == Some(b.as_str());
                            if ui.selectable_label(sel, format!("● {b}")).clicked() {
                                app.branch_modal.start_point = Some(b.clone());
                            }
                        }
                    }
                    if !remote_branches.is_empty() {
                        ui.separator();
                        ui.label(
                            RichText::new(t.remote_branches)
                                .size(10.5)
                                .color(theme::TEXT_MUTED),
                        );
                        for b in &remote_branches {
                            let sel = app.branch_modal.start_point.as_deref() == Some(b.as_str());
                            if ui.selectable_label(sel, format!("☁ {b}")).clicked() {
                                app.branch_modal.start_point = Some(b.clone());
                            }
                        }
                    }
                    if !tag_names.is_empty() {
                        ui.separator();
                        ui.label(RichText::new(t.tags).size(10.5).color(theme::TEXT_MUTED));
                        for tg in &tag_names {
                            let sel = app.branch_modal.start_point.as_deref() == Some(tg.as_str());
                            if ui.selectable_label(sel, format!("# {tg}")).clicked() {
                                app.branch_modal.start_point = Some(tg.clone());
                            }
                        }
                    }
                });

            ui.add_space(8.0);
            ui.label(t.branch_name);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.branch_modal.name)
                    .desired_width(290.0)
                    .hint_text("feature/my-branch"),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                create_now = true;
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(t.cancel).clicked() {
                    close_modal = true;
                }
                if ui.button(t.create_and_checkout).clicked() {
                    create_now = true;
                }
            });
        });

    if create_now {
        let name = app.branch_modal.name.clone();
        let start = app.branch_modal.start_point.clone();
        let ok_msg = match start.as_deref() {
            Some(sp) => format!("Branch '{name}' created from '{sp}'."),
            None => format!("Branch '{name}' created."),
        };
        app.run_git_action(&ok_msg, |r| r.create_branch(&name, start.as_deref()));
        close_modal = true;
    }

    if close_modal {
        app.branch_modal.open = false;
        app.branch_modal.name.clear();
        app.branch_modal.start_point = None;
    }
}

fn show_rename_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.rename_modal.open {
        return;
    }
    let t = app.t();
    let mut close_modal = false;
    let mut rename_now = false;

    egui::Window::new(t.rename_branch_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                RichText::new(format!("{} →", app.rename_modal.old_name))
                    .color(theme::TEXT_SECONDARY)
                    .size(12.0),
            );
            ui.add_space(4.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.rename_modal.new_name)
                    .desired_width(270.0),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                rename_now = true;
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(t.cancel).clicked() {
                    close_modal = true;
                }
                if ui.button(t.rename_btn).clicked() {
                    rename_now = true;
                }
            });
        });

    if rename_now {
        let old = app.rename_modal.old_name.clone();
        let new = app.rename_modal.new_name.clone();
        app.run_git_action(&format!("Renamed '{old}' to '{new}'."), |r| {
            r.rename_branch(&old, &new)
        });
        close_modal = true;
    }

    if close_modal {
        app.rename_modal.open = false;
        app.rename_modal.old_name.clear();
        app.rename_modal.new_name.clear();
    }
}

fn show_tag_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.tag_modal.open {
        return;
    }
    let t = app.t();
    let mut close_modal = false;
    let mut create_now = false;

    egui::Window::new(t.create_tag_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                RichText::new(format!("{} {}", t.from_commit, app.tag_modal.target))
                    .color(theme::TEXT_SECONDARY)
                    .size(12.0),
            );
            ui.add_space(6.0);

            ui.label(t.tag_name);
            ui.add(
                egui::TextEdit::singleline(&mut app.tag_modal.name)
                    .desired_width(270.0)
                    .hint_text("v1.0.0"),
            );
            ui.add_space(6.0);

            ui.label(t.tag_message_opt);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.tag_modal.message)
                    .desired_width(270.0)
                    .hint_text("Release notes..."),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                create_now = true;
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(t.cancel).clicked() {
                    close_modal = true;
                }
                if ui.button(t.create).clicked() {
                    create_now = true;
                }
            });
        });

    if create_now {
        let name = app.tag_modal.name.clone();
        let target = app.tag_modal.target.clone();
        let msg = app.tag_modal.message.clone();
        app.run_git_action(&format!("Tag '{name}' created."), |r| {
            r.create_tag(&name, &target, Some(&msg))
        });
        close_modal = true;
    }

    if close_modal {
        app.tag_modal.open = false;
        app.tag_modal.name.clear();
        app.tag_modal.message.clear();
        app.tag_modal.target.clear();
    }
}

fn show_stash_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.stash_modal.open {
        return;
    }
    let t = app.t();
    let mut close_modal = false;
    let mut save_now = false;

    egui::Window::new(t.stash_modal_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(t.stash_msg_opt);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.stash_modal.message)
                    .desired_width(280.0)
                    .hint_text("WIP: refactor..."),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                save_now = true;
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(t.cancel).clicked() {
                    close_modal = true;
                }
                if ui.button(t.save_stash).clicked() {
                    save_now = true;
                }
            });
        });

    if save_now {
        let msg = app.stash_modal.message.clone();
        app.run_git_action(t.msg_stashed, |r| r.stash_save(&msg));
        close_modal = true;
    }

    if close_modal {
        app.stash_modal.open = false;
        app.stash_modal.message.clear();
    }
}

fn show_settings_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.settings_open {
        return;
    }
    let t = app.t();
    let mut need_refresh = false;
    let mut need_save = false;
    let mut close_now = false;

    egui::Window::new(t.settings_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(350.0);

            ui.horizontal(|ui| {
                ui.label(t.language_label);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(app.config.language == Language::Es, Language::Es.label())
                        .clicked()
                    {
                        app.config.language = Language::Es;
                        need_save = true;
                    }
                    if ui
                        .selectable_label(app.config.language == Language::En, Language::En.label())
                        .clicked()
                    {
                        app.config.language = Language::En;
                        need_save = true;
                    }
                });
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(t.graph_style_label);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(
                            app.config.graph_style == GraphStyle::Angular,
                            t.graph_angular,
                        )
                        .clicked()
                    {
                        app.config.graph_style = GraphStyle::Angular;
                        need_save = true;
                    }
                    if ui
                        .selectable_label(
                            app.config.graph_style == GraphStyle::Rounded,
                            t.graph_rounded,
                        )
                        .clicked()
                    {
                        app.config.graph_style = GraphStyle::Rounded;
                        need_save = true;
                    }
                });
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(t.commit_order_label);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for (ord, lbl) in [
                        (CommitOrder::AuthorDate, t.order_author),
                        (CommitOrder::Date, t.order_date),
                        (CommitOrder::Topo, t.order_topo),
                    ] {
                        if ui
                            .selectable_label(app.config.commit_order == ord, lbl)
                            .clicked()
                        {
                            app.config.commit_order = ord;
                            need_save = true;
                            need_refresh = true;
                        }
                    }
                });
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(t.commit_limit_label);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for &limit in &[2000_usize, 1000, 500, 200] {
                        if ui
                            .selectable_label(app.config.commit_limit == limit, limit.to_string())
                            .clicked()
                        {
                            app.config.commit_limit = limit;
                            need_save = true;
                            need_refresh = true;
                        }
                    }
                });
            });

            ui.add_space(8.0);
            if ui
                .checkbox(&mut app.config.show_remotes_in_graph, t.show_remotes_label)
                .changed()
            {
                need_save = true;
                need_refresh = true;
            }

            ui.add_space(4.0);
            if ui
                .checkbox(&mut app.config.compact_rows, t.compact_rows_label)
                .changed()
            {
                need_save = true;
            }

            ui.add_space(4.0);
            if ui
                .checkbox(
                    &mut app.config.auto_check_remote,
                    t.auto_check_remote_label,
                )
                .changed()
            {
                need_save = true;
                if app.config.auto_check_remote {
                    app.spawn_remote_check(Some(ctx.clone()));
                } else {
                    app.remote_updates_available = false;
                }
            }

            ui.add_space(12.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t.close).clicked() {
                        close_now = true;
                    }
                });
            });
        });

    if need_save {
        app.config.save();
    }
    if need_refresh {
        app.refresh();
    }
    if close_now {
        app.settings_open = false;
    }
}

fn show_shortcuts_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.shortcuts_open {
        return;
    }
    let t = app.t();
    let mut close_now = false;

    let rows = [
        ("Cmd + O / Cmd + N", "Open repository in new tab"),
        ("Cmd + W", "Close active tab"),
        ("Cmd + 1..9", "Switch to tab 1–9"),
        ("Ctrl + Tab", "Next repository tab"),
        ("Cmd + E", "Switch Git Graph / Agent Mode"),
        ("Cmd + R", "Refresh repository"),
        ("Cmd + F", "Focus search / filter"),
        ("Cmd + B", "Toggle left sidebar"),
        ("Cmd + I", "Toggle right inspector"),
        ("Cmd + ,", "Open Settings"),
        ("Cmd + /", "Open Keyboard Shortcuts"),
        ("Esc", "Close active modal"),
    ];

    egui::Window::new("Keyboard Shortcuts — gity x")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(360.0);
            for (keys, desc) in rows {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(desc)
                            .size(12.5)
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::Frame::none()
                            .fill(theme::BG_SURFACE)
                            .rounding(Rounding::same(5.0))
                            .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                            .inner_margin(Margin::symmetric(7.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(keys)
                                        .monospace()
                                        .size(11.0)
                                        .color(theme::ACCENT_CYAN),
                                );
                            });
                    });
                });
                ui.add_space(2.0);
            }

            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t.close).clicked() {
                        close_now = true;
                    }
                });
            });
        });

    if close_now {
        app.shortcuts_open = false;
    }
}

fn show_about_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.about_open {
        return;
    }
    let t = app.t();
    let mut close_now = false;

    egui::Window::new("About gity x")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(320.0);
            ui.vertical_centered(|ui| {
                ui.add_space(6.0);
                if let Some(img) = app_logo_image(app.logo_texture.as_ref(), 68.0, 14.0) {
                    ui.add(img);
                    ui.add_space(8.0);
                }
                ui.label(
                    RichText::new("gity x")
                        .size(26.0)
                        .strong()
                        .color(theme::ACCENT_CYAN),
                );
                ui.label(
                    RichText::new(format!(
                        "v{} • Native Rust Git Client",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Created by williamjkc69")
                        .size(12.0)
                        .strong()
                        .color(theme::ACCENT_CYAN),
                );
                ui.add_space(10.0);
                ui.label(
                    RichText::new(
                        "Fast, lightweight Git Graph & working-tree reviewer built with Rust + egui.",
                    )
                    .size(12.5)
                    .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Config: ~/.config/gity-x/config.txt")
                        .monospace()
                        .size(11.0)
                        .color(theme::TEXT_MUTED),
                );
                ui.add_space(12.0);
                if ui.button(t.close).clicked() {
                    close_now = true;
                }
            });
        });

    if close_now {
        app.about_open = false;
    }
}

fn show_error_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.error_modal.open {
        return;
    }

    let t = app.t();
    let (summary, hint) = humanize_git_error(&app.error_modal.raw_error, app.config.language);
    let mut close_now = false;
    let mut copy_now = false;

    egui::Window::new(
        RichText::new(format!("✕  {}", t.error_modal_title))
            .color(theme::ACCENT_ROSE)
            .strong(),
    )
    .collapsible(false)
    .resizable(true)
    .default_width(520.0)
    .min_width(380.0)
    .max_width(740.0)
    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
    .show(ctx, |ui| {
        ui.label(
            RichText::new(summary)
                .strong()
                .size(14.0)
                .color(Color32::WHITE),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(hint)
                .size(12.5)
                .color(theme::TEXT_SECONDARY),
        );

        ui.add_space(10.0);
        ui.label(
            RichText::new(t.error_details_label)
                .size(11.0)
                .color(theme::TEXT_MUTED),
        );
        ui.add_space(4.0);

        egui::Frame::none()
            .fill(theme::BG_PRIMARY)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("error_modal_details_scroll")
                    .max_height(220.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&app.error_modal.raw_error)
                                .monospace()
                                .size(11.5)
                                .color(theme::DIFF_DEL_FG),
                        );
                    });
            });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let copy_lbl = if app.error_modal.copied {
                RichText::new(t.copied_btn).color(theme::ACCENT_GREEN).strong()
            } else {
                RichText::new(t.copy_btn).color(Color32::WHITE)
            };
            if ui
                .add(
                    egui::Button::new(copy_lbl)
                        .fill(theme::BG_ROW_SELECTED)
                        .stroke(Stroke::new(
                            1.0_f32,
                            if app.error_modal.copied {
                                theme::ACCENT_GREEN
                            } else {
                                theme::ACCENT_CYAN
                            },
                        ))
                        .min_size(egui::vec2(96.0, 28.0)),
                )
                .clicked()
            {
                copy_now = true;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(egui::Button::new(t.close).min_size(egui::vec2(84.0, 28.0)))
                    .clicked()
                {
                    close_now = true;
                }
            });
        });
    });

    if copy_now {
        ctx.copy_text(app.error_modal.raw_error.clone());
        app.error_modal.copied = true;
    }
    if close_now {
        app.error_modal.open = false;
        app.error_modal.copied = false;
    }
}

fn show_success_toast(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if app.toasts.is_empty() {
        return;
    }

    let toasts = app.toasts.clone();
    let mut dismiss_id: Option<u64> = None;

    egui::Area::new(egui::Id::new("gity_x_success_toast_stack"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-18.0, -18.0))
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                for notice in &toasts {
                    egui::Frame::none()
                        .fill(theme::BG_SURFACE)
                        .rounding(Rounding::same(10.0))
                        .stroke(Stroke::new(
                            1.0_f32,
                            theme::ACCENT_GREEN.linear_multiply(0.65),
                        ))
                        .shadow(egui::epaint::Shadow {
                            offset: egui::vec2(0.0, 6.0),
                            blur: 18.0,
                            spread: 0.0,
                            color: Color32::from_black_alpha(120),
                        })
                        .inner_margin(Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("✓")
                                        .size(13.5)
                                        .strong()
                                        .color(theme::ACCENT_GREEN),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new(&notice.text)
                                        .size(12.5)
                                        .color(Color32::WHITE),
                                );
                                ui.add_space(10.0);
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("×")
                                                .size(12.0)
                                                .color(theme::TEXT_MUTED),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    dismiss_id = Some(notice.id);
                                }
                            });
                        });
                }
            });
        });

    if let Some(id) = dismiss_id {
        app.toasts.retain(|t| t.id != id);
    }
}
