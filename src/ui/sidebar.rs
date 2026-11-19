use eframe::egui::{self, Margin, RichText, Stroke};

use crate::app::GitKrakenApp;
use crate::theme;

pub fn show(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.config.show_left_sidebar {
        return;
    }

    let t = app.t();
    let frame = egui::Frame::none()
        .fill(theme::BG_PANEL)
        .inner_margin(Margin::symmetric(12.0, 10.0))
        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE));

    let mut collapse = false;
    let mut open_settings = false;

    egui::SidePanel::left("left_sidebar")
        .default_width(240.0)
        .min_width(185.0)
        .max_width(360.0)
        .frame(frame)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Workspace")
                        .size(11.5)
                        .color(theme::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("«")
                        .on_hover_text(t.toggle_left_sidebar)
                        .clicked()
                    {
                        collapse = true;
                    }
                });
            });
            ui.add_space(4.0);

            // Antigravity-style bottom-pinned Settings row
            egui::TopBottomPanel::bottom("sidebar_bottom_settings")
                .frame(
                    egui::Frame::none()
                        .fill(theme::BG_PANEL)
                        .inner_margin(Margin {
                            left: 0.0,
                            right: 0.0,
                            top: 6.0,
                            bottom: 2.0,
                        }),
                )
                .show_inside(ui, |ui| {
                    ui.separator();
                    ui.add_space(4.0);
                    if ui
                        .selectable_label(
                            app.settings_open,
                            RichText::new(format!("⚙  {}", t.settings))
                                .size(12.5)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .clicked()
                    {
                        open_settings = true;
                    }
                });

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let local_branches: Vec<_> = app
                        .branches
                        .iter()
                        .filter(|b| !b.is_remote)
                        .cloned()
                        .collect();
                    let remote_branches: Vec<_> = app
                        .branches
                        .iter()
                        .filter(|b| b.is_remote)
                        .cloned()
                        .collect();
                    let tags = app.tags.clone();
                    let stashes = app.stashes.clone();

                    // Local branches
                    egui::CollapsingHeader::new(
                        RichText::new(format!("{} ({})", t.local_branches, local_branches.len()))
                            .strong()
                            .size(11.5)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        for branch in &local_branches {
                            ui.horizontal(|ui| {
                                let label_text = if branch.is_current {
                                    RichText::new(format!("● {}", branch.name))
                                        .color(theme::ACCENT_CYAN)
                                        .strong()
                                } else {
                                    RichText::new(format!("  {}", branch.name))
                                        .color(theme::TEXT_PRIMARY)
                                };

                                let resp = ui.selectable_label(branch.is_current, label_text);
                                if resp.clicked() {
                                    app.jump_to_hash(&branch.target_hash);
                                }
                                if resp.double_clicked() && !branch.is_current {
                                    let name = branch.name.clone();
                                    app.run_git_action(
                                        &format!("Checked out '{name}'."),
                                        |r| r.checkout(&name),
                                    );
                                }

                                resp.context_menu(|ui| {
                                    if !branch.is_current && ui.button(t.checkout).clicked() {
                                        let name = branch.name.clone();
                                        app.run_git_action(
                                            &format!("Checked out '{name}'."),
                                            |r| r.checkout(&name),
                                        );
                                        ui.close_menu();
                                    }
                                    if ui.button(t.create_branch_from).clicked() {
                                        app.open_branch_modal(Some(branch.name.clone()));
                                        ui.close_menu();
                                    }
                                    if ui.button(t.rename_branch).clicked() {
                                        app.open_rename_branch_modal(branch.name.clone());
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    if !branch.is_current {
                                        if ui.button(t.merge_into_current).clicked() {
                                            let name = branch.name.clone();
                                            app.run_git_action(
                                                &format!("Merged '{name}' into current branch."),
                                                |r| r.merge_branch(&name),
                                            );
                                            ui.close_menu();
                                        }
                                        if ui.button(t.merge_no_ff).clicked() {
                                            let name = branch.name.clone();
                                            app.run_git_action(
                                                &format!("Merged '{name}' (--no-ff)."),
                                                |r| r.merge_branch_no_ff(&name),
                                            );
                                            ui.close_menu();
                                        }
                                        if ui.button(t.rebase_onto).clicked() {
                                            let name = branch.name.clone();
                                            app.run_git_action(
                                                &format!("Rebased current branch onto '{name}'."),
                                                |r| r.rebase_onto(&name),
                                            );
                                            ui.close_menu();
                                        }
                                        ui.separator();
                                    }
                                    if ui.button(t.push_branch).clicked() {
                                        let name = branch.name.clone();
                                        app.run_git_action(
                                            &format!("Pushed '{name}' to origin."),
                                            |r| r.push_branch(&name),
                                        );
                                        ui.close_menu();
                                    }
                                    if !branch.is_current {
                                        if ui.button(t.delete_branch).clicked() {
                                            let name = branch.name.clone();
                                            app.run_git_action(
                                                &format!("Deleted branch '{name}'."),
                                                |r| r.delete_branch(&name),
                                            );
                                            ui.close_menu();
                                        }
                                        if ui
                                            .button(
                                                RichText::new(t.force_delete_branch)
                                                    .color(theme::ACCENT_ROSE),
                                            )
                                            .clicked()
                                        {
                                            let name = branch.name.clone();
                                            app.run_git_action(
                                                &format!("Force deleted branch '{name}'."),
                                                |r| r.force_delete_branch(&name),
                                            );
                                            ui.close_menu();
                                        }
                                    }
                                    ui.separator();
                                    if ui.button(t.copy_name).clicked() {
                                        ctx.copy_text(branch.name.clone());
                                        app.set_info(t.msg_name_copied);
                                        ui.close_menu();
                                    }
                                });
                            });
                        }
                    });

                    ui.add_space(8.0);

                    // Remote branches
                    egui::CollapsingHeader::new(
                        RichText::new(format!("{} ({})", t.remote_branches, remote_branches.len()))
                            .strong()
                            .size(11.5)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .default_open(false)
                    .show(ui, |ui| {
                        for branch in &remote_branches {
                            let resp = ui.selectable_label(
                                false,
                                RichText::new(&branch.name)
                                    .color(theme::ACCENT_PURPLE)
                                    .size(12.5),
                            );
                            if resp.clicked() {
                                app.jump_to_hash(&branch.target_hash);
                            }
                            resp.context_menu(|ui| {
                                if ui.button(t.checkout).clicked() {
                                    let name = branch.name.clone();
                                    app.run_git_action(&format!("Checked out '{name}'."), |r| {
                                        r.checkout(&name)
                                    });
                                    ui.close_menu();
                                }
                                if ui.button(t.create_branch_from).clicked() {
                                    app.open_branch_modal(Some(branch.name.clone()));
                                    ui.close_menu();
                                }
                                if ui.button(t.merge_into_current).clicked() {
                                    let name = branch.name.clone();
                                    app.run_git_action(
                                        &format!("Merged '{name}' into current branch."),
                                        |r| r.merge_branch(&name),
                                    );
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui
                                    .button(
                                        RichText::new(t.delete_remote_branch)
                                            .color(theme::ACCENT_ROSE),
                                    )
                                    .clicked()
                                {
                                    let name = branch.name.clone();
                                    app.run_git_action(
                                        &format!("Deleted remote branch '{name}'."),
                                        |r| r.delete_remote_branch(&name),
                                    );
                                    ui.close_menu();
                                }
                                if ui.button(t.copy_name).clicked() {
                                    ctx.copy_text(branch.name.clone());
                                    app.set_info(t.msg_name_copied);
                                    ui.close_menu();
                                }
                            });
                        }
                    });

                    ui.add_space(8.0);

                    // Tags
                    egui::CollapsingHeader::new(
                        RichText::new(format!("{} ({})", t.tags, tags.len()))
                            .strong()
                            .size(11.5)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .default_open(false)
                    .show(ui, |ui| {
                        for tag in &tags {
                            let resp = ui.selectable_label(
                                false,
                                RichText::new(format!("# {}", tag.name))
                                    .color(theme::ACCENT_AMBER)
                                    .size(12.5),
                            );
                            if resp.clicked() {
                                app.jump_to_hash(&tag.target_hash);
                            }
                            resp.context_menu(|ui| {
                                if ui.button(t.checkout).clicked() {
                                    let name = tag.name.clone();
                                    app.run_git_action(&format!("Checked out tag '{name}'."), |r| {
                                        r.checkout(&name)
                                    });
                                    ui.close_menu();
                                }
                                if ui.button(t.create_branch_from).clicked() {
                                    app.open_branch_modal(Some(tag.name.clone()));
                                    ui.close_menu();
                                }
                                if ui.button(t.push_tag).clicked() {
                                    let name = tag.name.clone();
                                    app.run_git_action(&format!("Pushed tag '{name}'."), |r| {
                                        r.push_tag(&name)
                                    });
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui
                                    .button(RichText::new(t.delete_tag).color(theme::ACCENT_ROSE))
                                    .clicked()
                                {
                                    let name = tag.name.clone();
                                    app.run_git_action(&format!("Deleted tag '{name}'."), |r| {
                                        r.delete_tag(&name)
                                    });
                                    ui.close_menu();
                                }
                                if ui.button(t.copy_name).clicked() {
                                    ctx.copy_text(tag.name.clone());
                                    app.set_info(t.msg_name_copied);
                                    ui.close_menu();
                                }
                            });
                        }
                    });

                    ui.add_space(8.0);

                    // Stashes
                    egui::CollapsingHeader::new(
                        RichText::new(format!("{} ({})", t.stashes, stashes.len()))
                            .strong()
                            .size(11.5)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .default_open(!stashes.is_empty())
                    .show(ui, |ui| {
                        if stashes.is_empty() {
                            ui.label(
                                RichText::new(t.no_stashes)
                                    .color(theme::TEXT_MUTED)
                                    .size(12.0)
                                    .italics(),
                            );
                        } else {
                            for stash in &stashes {
                                let resp = ui.selectable_label(
                                    false,
                                    RichText::new(format!("{}: {}", stash.index, stash.message))
                                        .color(theme::TEXT_PRIMARY)
                                        .size(12.0),
                                );
                                resp.context_menu(|ui| {
                                    if ui.button(t.stash_apply).clicked() {
                                        let idx = stash.index.clone();
                                        app.run_git_action(&format!("Applied {idx}."), |r| {
                                            r.stash_apply_ref(&idx)
                                        });
                                        ui.close_menu();
                                    }
                                    if ui.button(t.stash_pop).clicked() {
                                        let idx = stash.index.clone();
                                        app.run_git_action(&format!("Popped {idx}."), |r| {
                                            r.stash_pop_ref(&idx)
                                        });
                                        ui.close_menu();
                                    }
                                    if ui
                                        .button(
                                            RichText::new(t.stash_drop).color(theme::ACCENT_ROSE),
                                        )
                                        .clicked()
                                    {
                                        let idx = stash.index.clone();
                                        app.run_git_action(&format!("Dropped {idx}."), |r| {
                                            r.stash_drop_ref(&idx)
                                        });
                                        ui.close_menu();
                                    }
                                });
                            }
                        }
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.label(
                        RichText::new(t.sidebar_tip)
                            .size(11.0)
                            .color(theme::TEXT_MUTED),
                    );
                });
        });

    if collapse {
        app.toggle_left_sidebar();
    }
    if open_settings {
        app.settings_open = !app.settings_open;
    }
}
