use std::path::PathBuf;

use eframe::egui::{self, Color32, Margin, RichText, Rounding, Stroke};

use crate::app::{app_logo_image, GitKrakenApp, ProviderTestState, SettingsTab, ViewMode};
use crate::config::{CommitOrder, GraphStyle};
use crate::i18n::{humanize_git_error, Language, MOD_KEY};
use crate::providers::ProviderKind;
use crate::theme;
use crate::ui::help_modal;
use crate::updater::{self, UpdateState};

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
                                            .on_hover_text(format!("Close tab ({MOD_KEY}+W)"))
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
                            .on_hover_text(format!(
                                "Open repository in new tab ({MOD_KEY}+N / {MOD_KEY}+O)"
                            ))
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

fn draw_update_pill(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut GitKrakenApp) {
    let t = app.t();
    match app.update_state.clone() {
        UpdateState::Downloading { version, progress } => {
            let pct = (progress * 100.0).round() as u32;
            egui::Frame::none()
                .fill(Color32::from_rgb(38, 46, 68))
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0_f32, theme::ACCENT_CYAN))
                .inner_margin(Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("↓ {} v{version} ({pct}%)", t.update_downloading))
                            .size(11.5)
                            .color(theme::ACCENT_CYAN),
                    );
                });
        }
        UpdateState::ReadyToRestart {
            version,
            launch_path,
        } => {
            let btn = egui::Button::new(
                RichText::new(format!("✓ {} (v{version})", t.update_restart_btn))
                    .size(11.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(34, 92, 62))
            .stroke(Stroke::new(1.0_f32, theme::ACCENT_GREEN))
            .rounding(Rounding::same(6.0))
            .min_size(egui::vec2(0.0, 26.0));
            if ui.add(btn).clicked() {
                if let Err(err) = updater::relaunch_and_exit(&launch_path) {
                    app.set_error(err);
                }
            }
        }
        UpdateState::Available(rel) => {
            let btn = egui::Button::new(
                RichText::new(format!("↓ {} v{}", t.update_available_btn, rel.version))
                    .size(11.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(44, 54, 86))
            .stroke(Stroke::new(1.0_f32, theme::ACCENT_CYAN))
            .rounding(Rounding::same(6.0))
            .min_size(egui::vec2(0.0, 26.0));
            let hover = if rel.can_self_install {
                format!("Download and install v{} automatically", rel.version)
            } else {
                format!("Open release page for v{}", rel.version)
            };
            if ui.add(btn).on_hover_text(hover).clicked() {
                if rel.can_self_install {
                    app.start_update_download(rel, ctx.clone());
                } else {
                    let _ = webbrowser::open(&rel.html_url);
                }
            }
        }
        _ => {}
    }
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
                        && app.view_mode != ViewMode::PullRequests
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

                        // Mode switcher pills: Git Graph | Agent Mode | Pull Requests (aligned 26px height)
                        let graph_active = app.view_mode == ViewMode::Graph;
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
                            app.set_view_mode(ViewMode::Graph, Some(ctx.clone()));
                        }

                        let agent_active = app.view_mode == ViewMode::Agent;
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

                        if ui.add(agent_btn).on_hover_text(format!("{MOD_KEY}+E")).clicked() {
                            app.set_view_mode(ViewMode::Agent, Some(ctx.clone()));
                        }

                        let pr_active = app.view_mode == ViewMode::PullRequests;
                        let pr_text = if !app.pr_list.is_empty() {
                            format!("→ {} ({})", t.mode_prs, app.pr_list.len())
                        } else {
                            format!("→ {}", t.mode_prs)
                        };
                        let pr_btn = egui::Button::new(
                            RichText::new(pr_text).size(12.0).color(if pr_active {
                                Color32::WHITE
                            } else {
                                theme::TEXT_SECONDARY
                            }),
                        )
                        .fill(if pr_active {
                            theme::BG_ROW_SELECTED
                        } else {
                            theme::BG_PRIMARY
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if pr_active {
                                theme::ACCENT_CYAN
                            } else {
                                theme::BORDER_SUBTLE
                            },
                        ))
                        .rounding(Rounding::same(7.0))
                        .min_size(egui::vec2(0.0, 26.0));

                        if ui.add(pr_btn).on_hover_text(format!("{MOD_KEY}+P")).clicked() {
                            app.set_view_mode(ViewMode::PullRequests, Some(ctx.clone()));
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
                            if app.view_mode == ViewMode::Graph
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
                                    egui::Button::new(
                                        RichText::new("?").strong().color(theme::ACCENT_CYAN),
                                    )
                                    .min_size(egui::vec2(26.0, 26.0)),
                                )
                                .on_hover_text(t.help_center_title)
                                .clicked()
                            {
                                app.help_open = !app.help_open;
                            }
                            if ui
                                .add(
                                    egui::Button::new("↻")
                                        .min_size(egui::vec2(26.0, 26.0)),
                                )
                                .on_hover_text(format!("{} ({MOD_KEY}+R)", t.refresh))
                                .clicked()
                            {
                                app.refresh();
                                app.spawn_remote_check(Some(ctx.clone()));
                                if app.view_mode == ViewMode::PullRequests {
                                    app.fetch_pr_list(Some(ctx.clone()));
                                }
                                app.set_info(t.msg_refreshed);
                            }
                            draw_update_pill(ui, ctx, app);
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
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(format!("? {}", t.help_btn))
                                            .color(theme::ACCENT_CYAN),
                                    )
                                    .min_size(egui::vec2(0.0, 26.0)),
                                )
                                .on_hover_text(t.help_center_title)
                                .clicked()
                            {
                                app.help_open = !app.help_open;
                            }
                            draw_update_pill(ui, ctx, app);
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
    help_modal::show(ctx, app);
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

fn render_test_status(ui: &mut egui::Ui, state: &ProviderTestState, testing_lbl: &str) {
    match state {
        ProviderTestState::Idle => {}
        ProviderTestState::Testing => {
            ui.label(
                RichText::new(testing_lbl)
                    .size(11.0)
                    .color(theme::ACCENT_CYAN),
            );
        }
        ProviderTestState::Ok(msg) => {
            ui.label(
                RichText::new(format!("✓ {msg}"))
                    .size(11.0)
                    .color(theme::ACCENT_GREEN),
            );
        }
        ProviderTestState::Err(err) => {
            ui.label(
                RichText::new(format!("× {err}"))
                    .size(11.0)
                    .color(theme::ACCENT_ROSE),
            );
        }
    }
}

fn show_settings_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.settings_open {
        return;
    }
    let t = app.t();
    let es = app.config.language == Language::Es;
    let mut need_refresh = false;
    let mut need_save = false;
    let mut close_now = false;
    let mut open_help = false;
    let mut test_provider: Option<ProviderKind> = None;

    egui::Window::new(t.settings_title)
        .collapsible(false)
        .resizable(true)
        .default_width(580.0)
        .min_width(480.0)
        .max_height(640.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            // Top segmented tab bar: General & Graph | Git Providers & Tokens
            ui.horizontal(|ui| {
                for (tab, label) in [
                    (SettingsTab::General, t.settings_tab_general),
                    (SettingsTab::Providers, t.settings_tab_providers),
                ] {
                    let active = app.settings_tab == tab;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .size(12.5)
                            .strong()
                            .color(if active {
                                Color32::WHITE
                            } else {
                                theme::TEXT_SECONDARY
                            }),
                    )
                    .fill(if active {
                        theme::BG_ROW_SELECTED
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(Stroke::new(
                        1.0_f32,
                        if active {
                            theme::ACCENT_CYAN
                        } else {
                            theme::BORDER_SUBTLE
                        },
                    ))
                    .rounding(Rounding::same(7.0))
                    .min_size(egui::vec2(0.0, 28.0));

                    if ui.add(btn).clicked() {
                        app.settings_tab = tab;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new(format!("? {}", t.help_btn))
                                .size(11.5)
                                .color(theme::ACCENT_CYAN),
                        )
                        .clicked()
                    {
                        open_help = true;
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            egui::ScrollArea::vertical()
                .id_salt("settings_modal_body_scroll")
                .auto_shrink([false, true])
                .max_height(480.0)
                .show(ui, |ui| match app.settings_tab {
                    SettingsTab::General => {
                        // Card 1: Appearance & Graph
                        egui::Frame::none()
                            .fill(theme::BG_SURFACE)
                            .rounding(Rounding::same(9.0))
                            .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                            .inner_margin(Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new(if es {
                                        "Apariencia y Grafo Git"
                                    } else {
                                        "Appearance & Git Graph"
                                    })
                                    .size(12.5)
                                    .strong()
                                    .color(theme::ACCENT_CYAN),
                                );
                                ui.label(
                                    RichText::new(if es {
                                        "Personaliza idioma, estilo visual de ramas y cuántos commits cargar en memoria."
                                    } else {
                                        "Customize language, commit branch line style, and how many commits are loaded."
                                    })
                                    .size(11.0)
                                    .color(theme::TEXT_MUTED),
                                );
                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    ui.label(t.language_label);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .selectable_label(
                                                    app.config.language == Language::Es,
                                                    Language::Es.label(),
                                                )
                                                .clicked()
                                            {
                                                app.config.language = Language::Es;
                                                need_save = true;
                                            }
                                            if ui
                                                .selectable_label(
                                                    app.config.language == Language::En,
                                                    Language::En.label(),
                                                )
                                                .clicked()
                                            {
                                                app.config.language = Language::En;
                                                need_save = true;
                                            }
                                        },
                                    );
                                });

                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.label(t.graph_style_label);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
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
                                        },
                                    );
                                });

                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.label(t.commit_order_label);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            for (ord, lbl) in [
                                                (CommitOrder::AuthorDate, t.order_author),
                                                (CommitOrder::Date, t.order_date),
                                                (CommitOrder::Topo, t.order_topo),
                                            ] {
                                                if ui
                                                    .selectable_label(
                                                        app.config.commit_order == ord,
                                                        lbl,
                                                    )
                                                    .clicked()
                                                {
                                                    app.config.commit_order = ord;
                                                    need_save = true;
                                                    need_refresh = true;
                                                }
                                            }
                                        },
                                    );
                                });

                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.label(t.commit_limit_label);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            for &limit in &[2000_usize, 1000, 500, 200] {
                                                if ui
                                                    .selectable_label(
                                                        app.config.commit_limit == limit,
                                                        limit.to_string(),
                                                    )
                                                    .clicked()
                                                {
                                                    app.config.commit_limit = limit;
                                                    need_save = true;
                                                    need_refresh = true;
                                                }
                                            }
                                        },
                                    );
                                });

                                ui.add_space(8.0);
                                if ui
                                    .checkbox(
                                        &mut app.config.show_remotes_in_graph,
                                        t.show_remotes_label,
                                    )
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
                            });

                        ui.add_space(10.0);

                        // Card 2: Sync & Auto-Updates
                        egui::Frame::none()
                            .fill(theme::BG_SURFACE)
                            .rounding(Rounding::same(9.0))
                            .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                            .inner_margin(Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new(if es {
                                        "Sincronización y Actualizaciones"
                                    } else {
                                        "Sync & Automatic Updates"
                                    })
                                    .size(12.5)
                                    .strong()
                                    .color(theme::ACCENT_CYAN),
                                );
                                ui.add_space(6.0);

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
                                ui.label(
                                    RichText::new(if es {
                                        "Desactivado por defecto para evitar avisos del Llavero (Keychain) en repos HTTPS."
                                    } else {
                                        "Off by default to prevent macOS Keychain password prompts on HTTPS remotes."
                                    })
                                    .size(11.0)
                                    .color(theme::TEXT_MUTED),
                                );

                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    if ui
                                        .checkbox(&mut app.config.auto_update, t.auto_update_label)
                                        .changed()
                                    {
                                        need_save = true;
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let checking = matches!(
                                                app.update_state,
                                                UpdateState::Checking
                                                    | UpdateState::Downloading { .. }
                                            );
                                            if ui
                                                .add_enabled(
                                                    !checking,
                                                    egui::Button::new(t.check_updates_btn),
                                                )
                                                .clicked()
                                            {
                                                app.check_for_updates(ctx.clone(), true);
                                            }
                                        },
                                    );
                                });
                                ui.label(
                                    RichText::new(if es {
                                        "Verifica GitHub Releases al abrir, valida SHA-256 y actualiza la app en el lugar."
                                    } else {
                                        "Checks GitHub Releases on startup, verifies SHA-256 checksums, and updates in place."
                                    })
                                    .size(11.0)
                                    .color(theme::TEXT_MUTED),
                                );

                                match &app.update_state {
                                    UpdateState::Checking => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(t.update_checking)
                                                .size(11.0)
                                                .color(theme::ACCENT_CYAN),
                                        );
                                    }
                                    UpdateState::UpToDate => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(format!(
                                                "✓ {} (v{})",
                                                t.update_up_to_date,
                                                updater::CURRENT_VERSION
                                            ))
                                            .size(11.0)
                                            .color(theme::ACCENT_GREEN),
                                        );
                                    }
                                    UpdateState::Downloading { version, progress } => {
                                        ui.add_space(4.0);
                                        let pct = (progress * 100.0).round() as u32;
                                        ui.label(
                                            RichText::new(format!(
                                                "↓ {} v{version} ({pct}%)",
                                                t.update_downloading
                                            ))
                                            .size(11.0)
                                            .color(theme::ACCENT_CYAN),
                                        );
                                    }
                                    UpdateState::ReadyToRestart { version, .. } => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(format!(
                                                "✓ v{version} — {}",
                                                t.update_restart_btn
                                            ))
                                            .size(11.0)
                                            .color(theme::ACCENT_GREEN),
                                        );
                                    }
                                    UpdateState::Error(err) => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(format!("× {err}"))
                                                .size(11.0)
                                                .color(theme::ACCENT_ROSE),
                                        );
                                    }
                                    _ => {}
                                }
                            });
                    }

                    SettingsTab::Providers => {
                        // Top info bar + Show/Hide toggle
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(if es {
                                    "Los tokens se guardan localmente (permisos 0600) y se usan solo para listar, aprobar y fusionar Pull Requests."
                                } else {
                                    "Tokens are saved locally (0600 permissions) and used only to list, review, and merge Pull Requests."
                                })
                                .size(11.5)
                                .color(theme::TEXT_SECONDARY),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let lbl = if app.show_tokens {
                                        t.hide_tokens
                                    } else {
                                        t.show_tokens
                                    };
                                    if ui.small_button(lbl).clicked() {
                                        app.show_tokens = !app.show_tokens;
                                    }
                                },
                            );
                        });

                        ui.add_space(8.0);

                        // --- Card 1: GitHub ---
                        egui::Frame::none()
                            .fill(theme::BG_SURFACE)
                            .rounding(Rounding::same(9.0))
                            .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                            .inner_margin(Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("GitHub")
                                            .size(13.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                    if !app.config.github_token.trim().is_empty() {
                                        ui.label(
                                            RichText::new("✓ Token")
                                                .size(11.0)
                                                .color(theme::ACCENT_GREEN),
                                        );
                                    } else if app.gh_cli_detected {
                                        ui.label(
                                            RichText::new("✓ Auto (gh CLI)")
                                                .size(11.0)
                                                .color(theme::ACCENT_GREEN),
                                        );
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let busy = matches!(
                                                app.test_gh_state,
                                                ProviderTestState::Testing
                                            );
                                            if ui
                                                .add_enabled(
                                                    !busy,
                                                    egui::Button::new(t.test_token_btn),
                                                )
                                                .clicked()
                                            {
                                                test_provider = Some(ProviderKind::GitHub);
                                            }
                                            if ui
                                                .small_button(
                                                    RichText::new(if es {
                                                        "→ Generar token"
                                                    } else {
                                                        "→ Create token"
                                                    })
                                                    .color(theme::ACCENT_CYAN),
                                                )
                                                .clicked()
                                            {
                                                let _ = webbrowser::open(
                                                    "https://github.com/settings/tokens/new?scopes=repo&description=gity-x",
                                                );
                                            }
                                        },
                                    );
                                });
                                ui.label(
                                    RichText::new(if es {
                                        "Necesario para repos privados y para aprobar, comentar o fusionar PRs. Si usas 'gh auth login', se detecta solo."
                                    } else {
                                        "Required for private repos and approving, commenting, or merging PRs. Auto-detected if signed in with 'gh auth login'."
                                    })
                                    .size(11.0)
                                    .color(theme::TEXT_MUTED),
                                );
                                ui.add_space(6.0);
                                let gh_w = (ui.available_width() - 4.0).max(260.0);
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut app.config.github_token)
                                            .password(!app.show_tokens)
                                            .desired_width(gh_w)
                                            .hint_text(if app.gh_cli_detected {
                                                "Optional — already detected via gh CLI"
                                            } else {
                                                "ghp_... or github_pat_..."
                                            }),
                                    )
                                    .changed()
                                {
                                    need_save = true;
                                    app.test_gh_state = ProviderTestState::Idle;
                                }
                                render_test_status(ui, &app.test_gh_state, t.testing_token);
                                ui.add_space(4.0);
                                egui::CollapsingHeader::new(
                                    RichText::new(if es {
                                        "Cómo obtener el token de GitHub (paso a paso)"
                                    } else {
                                        "How to obtain a GitHub token (step by step)"
                                    })
                                    .size(11.5)
                                    .color(theme::ACCENT_CYAN),
                                )
                                .id_salt("guide_github_token")
                                .show(ui, |ui| {
                                    let steps: &[&str] = if es {
                                        &[
                                            "Opción 1 (más rápida): en Terminal ejecuta 'gh auth login' y elige HTTPS + navegador. gity x lo detecta automáticamente.",
                                            "Opción 2 (Classic PAT): pulsa '→ Generar token' arriba (ya marca 'repo'), elige expiración, genera y pega el código 'ghp_...'.",
                                            "Opción 3 (Fine-grained PAT): GitHub → Settings → Developer settings → Fine-grained tokens → selecciona repos → activa 'Pull requests: Read & write' y 'Contents: Read'.",
                                            "Si el repo es de una empresa con SAML SSO: en la lista de tokens pulsa 'Configure SSO' → 'Authorize'.",
                                        ]
                                    } else {
                                        &[
                                            "Option 1 (easiest): run 'gh auth login' in Terminal once. gity x detects your token automatically.",
                                            "Option 2 (Classic PAT): click '→ Create token' above (pre-selects 'repo'), generate, and paste the 'ghp_...' token.",
                                            "Option 3 (Fine-grained PAT): GitHub → Settings → Developer settings → Fine-grained tokens → select repos → grant 'Pull requests: Read & write' + 'Contents: Read'.",
                                            "For SAML SSO organizations: click 'Configure SSO' next to your token on GitHub and authorize the org.",
                                        ]
                                    };
                                    for s in steps {
                                        ui.label(
                                            RichText::new(*s)
                                                .size(11.0)
                                                .color(theme::TEXT_SECONDARY),
                                        );
                                    }
                                });
                            });

                        ui.add_space(8.0);

                        // --- Card 2: GitLab ---
                        egui::Frame::none()
                            .fill(theme::BG_SURFACE)
                            .rounding(Rounding::same(9.0))
                            .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                            .inner_margin(Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("GitLab")
                                            .size(13.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                    if !app.config.gitlab_token.trim().is_empty() {
                                        ui.label(
                                            RichText::new("✓ Token")
                                                .size(11.0)
                                                .color(theme::ACCENT_GREEN),
                                        );
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let busy = matches!(
                                                app.test_gl_state,
                                                ProviderTestState::Testing
                                            );
                                            if ui
                                                .add_enabled(
                                                    !busy,
                                                    egui::Button::new(t.test_token_btn),
                                                )
                                                .clicked()
                                            {
                                                test_provider = Some(ProviderKind::GitLab);
                                            }
                                            if ui
                                                .small_button(
                                                    RichText::new(if es {
                                                        "→ Generar token"
                                                    } else {
                                                        "→ Create token"
                                                    })
                                                    .color(theme::ACCENT_CYAN),
                                                )
                                                .clicked()
                                            {
                                                let _ = webbrowser::open(
                                                    "https://gitlab.com/-/user_settings/personal_access_tokens",
                                                );
                                            }
                                        },
                                    );
                                });
                                ui.label(
                                    RichText::new(if es {
                                        "Compatible con gitlab.com, grupos anidados y servidores self-hosted 'gitlab.*'."
                                    } else {
                                        "Works with gitlab.com, nested subgroups, and self-hosted 'gitlab.*' instances."
                                    })
                                    .size(11.0)
                                    .color(theme::TEXT_MUTED),
                                );
                                ui.add_space(6.0);
                                let gl_w = (ui.available_width() - 4.0).max(260.0);
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut app.config.gitlab_token)
                                            .password(!app.show_tokens)
                                            .desired_width(gl_w)
                                            .hint_text("glpat-..."),
                                    )
                                    .changed()
                                {
                                    need_save = true;
                                    app.test_gl_state = ProviderTestState::Idle;
                                }
                                render_test_status(ui, &app.test_gl_state, t.testing_token);
                                ui.add_space(4.0);
                                egui::CollapsingHeader::new(
                                    RichText::new(if es {
                                        "Cómo obtener el token de GitLab (paso a paso)"
                                    } else {
                                        "How to obtain a GitLab token (step by step)"
                                    })
                                    .size(11.5)
                                    .color(theme::ACCENT_CYAN),
                                )
                                .id_salt("guide_gitlab_token")
                                .show(ui, |ui| {
                                    let steps: &[&str] = if es {
                                        &[
                                            "1. Pulsa '→ Generar token' (o GitLab → Avatar → Edit profile → Access Tokens).",
                                            "2. Pulsa 'Add new token', pon nombre 'gity-x' y fecha de expiración.",
                                            "3. Marca el scope 'api' (necesario para aprobar, comentar, fusionar y cerrar MRs). Usa 'read_api' solo para lectura.",
                                            "4. Copia el token 'glpat-...' y pégalo arriba, luego pulsa 'Probar'.",
                                        ]
                                    } else {
                                        &[
                                            "1. Click '→ Create token' (or GitLab → Avatar → Preferences → Access Tokens).",
                                            "2. Click 'Add new token', name it 'gity-x', and pick an expiration date.",
                                            "3. Check the 'api' scope (required for approve, comment, merge, close). Use 'read_api' for read-only.",
                                            "4. Copy the 'glpat-...' token, paste it above, and click 'Test'.",
                                        ]
                                    };
                                    for s in steps {
                                        ui.label(
                                            RichText::new(*s)
                                                .size(11.0)
                                                .color(theme::TEXT_SECONDARY),
                                        );
                                    }
                                });
                            });

                        ui.add_space(8.0);

                        // --- Card 3: Bitbucket ---
                        egui::Frame::none()
                            .fill(theme::BG_SURFACE)
                            .rounding(Rounding::same(9.0))
                            .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                            .inner_margin(Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Bitbucket Cloud")
                                            .size(13.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                    if !app.config.bitbucket_token.trim().is_empty() {
                                        ui.label(
                                            RichText::new("✓ Token")
                                                .size(11.0)
                                                .color(theme::ACCENT_GREEN),
                                        );
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let busy = matches!(
                                                app.test_bb_state,
                                                ProviderTestState::Testing
                                            );
                                            if ui
                                                .add_enabled(
                                                    !busy,
                                                    egui::Button::new(t.test_token_btn),
                                                )
                                                .clicked()
                                            {
                                                test_provider = Some(ProviderKind::Bitbucket);
                                            }
                                            if ui
                                                .small_button(
                                                    RichText::new(if es {
                                                        "→ Atlassian API Token"
                                                    } else {
                                                        "→ Atlassian API Token"
                                                    })
                                                    .color(theme::ACCENT_CYAN),
                                                )
                                                .clicked()
                                            {
                                                let _ = webbrowser::open(
                                                    "https://id.atlassian.com/manage-profile/security/api-tokens",
                                                );
                                            }
                                        },
                                    );
                                });
                                ui.label(
                                    RichText::new(if es {
                                        "Soporta Atlassian API Token (con tu email arriba) o Repository/Workspace Access Token (deja User vacío)."
                                    } else {
                                        "Supports Atlassian API Token (enter your Atlassian email in User) or Repo/Workspace Access Token (leave User empty)."
                                    })
                                    .size(11.0)
                                    .color(theme::TEXT_MUTED),
                                );
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.add_sized([82.0, 20.0], egui::Label::new(RichText::new("User / Email").size(11.5)));
                                    let w = (ui.available_width() - 4.0).max(220.0);
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(
                                                &mut app.config.bitbucket_user,
                                            )
                                            .desired_width(w)
                                            .hint_text(if es {
                                                "Tu email de Atlassian (vacío si usas Access Token)"
                                            } else {
                                                "Atlassian account email (empty for Access Token)"
                                            }),
                                        )
                                        .changed()
                                    {
                                        need_save = true;
                                        app.test_bb_state = ProviderTestState::Idle;
                                    }
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.add_sized([82.0, 20.0], egui::Label::new(RichText::new("Token").size(11.5)));
                                    let w = (ui.available_width() - 4.0).max(220.0);
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(
                                                &mut app.config.bitbucket_token,
                                            )
                                            .password(!app.show_tokens)
                                            .desired_width(w)
                                            .hint_text("ATATT3x... / ATCTT3x... / App Password"),
                                        )
                                        .changed()
                                    {
                                        need_save = true;
                                        app.test_bb_state = ProviderTestState::Idle;
                                    }
                                });
                                render_test_status(ui, &app.test_bb_state, t.testing_token);
                                ui.add_space(4.0);
                                egui::CollapsingHeader::new(
                                    RichText::new(if es {
                                        "Cómo obtener el token de Bitbucket (paso a paso)"
                                    } else {
                                        "How to obtain a Bitbucket token (step by step)"
                                    })
                                    .size(11.5)
                                    .color(theme::ACCENT_CYAN),
                                )
                                .id_salt("guide_bitbucket_token")
                                .show(ui, |ui| {
                                    let steps: &[&str] = if es {
                                        &[
                                            "Opción A (Atlassian API Token — recomendado para todos tus repos): pulsa '→ Atlassian API Token' → Create API token → pon tu EMAIL de Atlassian en 'User / Email' y pega el token en 'Token'.",
                                            "Opción B (Access Token de repositorio/workspace): en Bitbucket abre Repository settings → Access tokens → Create Repository Access Token con 'Repositories: Read' y 'Pull requests: Read & Write'. Deja 'User / Email' VACÍO y pega el token (ATCTT3x...).",
                                            "Pulsa 'Probar' para verificar la autenticación antes de cerrar.",
                                        ]
                                    } else {
                                        &[
                                            "Option A (Atlassian API Token — works across all your repos): click '→ Atlassian API Token' → Create API token → enter your Atlassian EMAIL in 'User / Email' and paste the token in 'Token'.",
                                            "Option B (Repo/Workspace Access Token): in Bitbucket open Repository settings → Access tokens → create token with 'Repositories: Read' + 'Pull requests: Read & Write'. Leave 'User / Email' EMPTY and paste the token (ATCTT3x...).",
                                            "Click 'Test' to verify authentication immediately.",
                                        ]
                                    };
                                    for s in steps {
                                        ui.label(
                                            RichText::new(*s)
                                                .size(11.0)
                                                .color(theme::TEXT_SECONDARY),
                                        );
                                    }
                                });
                            });
                    }
                });

            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                let cfg_hint = if cfg!(target_os = "windows") {
                    "%APPDATA%\\gity-x\\config.txt"
                } else {
                    "~/.config/gity-x/config.txt"
                };
                ui.label(
                    RichText::new(cfg_hint)
                        .monospace()
                        .size(10.5)
                        .color(theme::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new(t.close).min_size(egui::vec2(76.0, 26.0)))
                        .clicked()
                    {
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
    if let Some(kind) = test_provider {
        app.config.save();
        app.test_provider_token(kind, ctx.clone());
    }
    if open_help {
        app.settings_open = false;
        app.help_open = true;
    }
    if close_now {
        app.settings_open = false;
        if app.view_mode == ViewMode::PullRequests {
            app.fetch_pr_list(Some(ctx.clone()));
        }
    }
}

fn show_shortcuts_modal(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.shortcuts_open {
        return;
    }
    let t = app.t();
    let mut close_now = false;

    let rows = [
        (
            format!("{MOD_KEY} + O / {MOD_KEY} + N"),
            "Open repository in new tab",
        ),
        (format!("{MOD_KEY} + W"), "Close active tab"),
        (format!("{MOD_KEY} + 1..9"), "Switch to tab 1–9"),
        ("Ctrl + Tab".to_string(), "Next repository tab"),
        (format!("{MOD_KEY} + E"), "Switch Git Graph / Agent Mode"),
        (format!("{MOD_KEY} + P"), "Switch to Pull Requests"),
        (format!("{MOD_KEY} + R"), "Refresh repository"),
        (format!("{MOD_KEY} + F"), "Focus search / filter"),
        (format!("{MOD_KEY} + B"), "Toggle left sidebar"),
        (format!("{MOD_KEY} + I"), "Toggle right inspector"),
        (format!("{MOD_KEY} + ,"), "Open Settings"),
        (format!("{MOD_KEY} + /"), "Open Keyboard Shortcuts"),
        ("Esc".to_string(), "Close active modal"),
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
                let cfg_path_label = if cfg!(target_os = "windows") {
                    "Config: %APPDATA%\\gity-x\\config.txt"
                } else {
                    "Config: ~/.config/gity-x/config.txt"
                };
                ui.label(
                    RichText::new(cfg_path_label)
                        .monospace()
                        .size(11.0)
                        .color(theme::TEXT_MUTED),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let checking = matches!(
                        app.update_state,
                        UpdateState::Checking | UpdateState::Downloading { .. }
                    );
                    if ui
                        .add_enabled(!checking, egui::Button::new(t.check_updates_btn))
                        .clicked()
                    {
                        app.check_for_updates(ctx.clone(), true);
                    }
                    if ui.button(t.close).clicked() {
                        close_now = true;
                    }
                });
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
    let mut open_troubleshoot = false;
    let mut open_provider_cfg = false;
    let is_api_error = app.error_modal.raw_error.contains("API error")
        || app.error_modal.raw_error.contains("HTTP 40");

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

            if ui
                .add(
                    egui::Button::new(
                        RichText::new(format!("? {}", t.troubleshoot_btn))
                            .color(theme::ACCENT_CYAN),
                    )
                    .min_size(egui::vec2(0.0, 28.0)),
                )
                .clicked()
            {
                open_troubleshoot = true;
            }

            if is_api_error
                && ui
                    .add(
                        egui::Button::new(
                            RichText::new(t.pr_configure_token).color(theme::ACCENT_AMBER),
                        )
                        .min_size(egui::vec2(0.0, 28.0)),
                    )
                    .clicked()
            {
                open_provider_cfg = true;
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
    if open_troubleshoot {
        let raw_lower = app.error_modal.raw_error.to_lowercase();
        app.help_search = if raw_lower.contains("bitbucket") {
            "Bitbucket".to_string()
        } else if raw_lower.contains("gitlab") {
            "GitLab".to_string()
        } else if raw_lower.contains("github") {
            "GitHub".to_string()
        } else {
            String::new()
        };
        app.error_modal.open = false;
        app.help_open = true;
    }
    if open_provider_cfg {
        app.error_modal.open = false;
        app.settings_tab = SettingsTab::Providers;
        app.settings_open = true;
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
