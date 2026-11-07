use crate::state::{AppState, FileOpModal, TreeNode};
use crate::theme::Palette;
use egui::{Color32, CornerRadius, Frame, Key, Margin, RichText, Stroke};
use std::path::PathBuf;

pub fn render_left_panel(ctx: &egui::Context, state: &mut AppState) {
    egui::SidePanel::left("vibers_left_explorer")
        .resizable(true)
        .default_width(230.0)
        .width_range(170.0..=450.0)
        .frame(
            Frame::NONE
                .fill(Palette::BG_SIDEBAR)
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            // Clean Header inside Left Panel
            egui::TopBottomPanel::top("left_explorer_header")
                .exact_height(32.0)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_SIDEBAR)
                        .inner_margin(Margin::symmetric(10, 0)),
                )
                .show_inside(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        let header_title = if state.workspace_name.is_empty() {
                            "NO FOLDER OPENED".to_string()
                        } else {
                            state.workspace_name.to_uppercase()
                        };

                        ui.label(
                            RichText::new(header_title)
                                .size(11.5)
                                .strong()
                                .color(Palette::TEXT_MUTED),
                        )
                        .on_hover_text(if state.workspace_root.as_os_str().is_empty() {
                            "尚未開啟任何專案資料夾".to_string()
                        } else {
                            state.workspace_root.display().to_string()
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !state.workspace_name.is_empty() {
                                if ui
                                    .small_button(
                                        RichText::new("↻").size(12.0).color(Palette::TEXT_MUTED),
                                    )
                                    .on_hover_text("重新整理檔案樹")
                                    .clicked()
                                {
                                    state.refresh_tree();
                                }
                            }
                        });
                    });
                });

            let active_rel = state
                .active_tab()
                .map(|t| t.rel_path.clone())
                .unwrap_or_default();

            let modified_files: std::collections::HashSet<String> = state
                .open_tabs
                .iter()
                .filter(|t| t.is_dirty || t.content != t.disk_snapshot)
                .map(|t| t.rel_path.clone())
                .collect();

            let mut file_to_open: Option<(PathBuf, String)> = None;
            let mut requested_op: Option<FileOpModal> = None;

            // Folder Tree ScrollArea
            egui::CentralPanel::default()
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_SIDEBAR)
                        .inner_margin(Margin {
                            left: 8,
                            right: 6,
                            top: 4,
                            bottom: 4,
                        }),
                )
                .show_inside(ui, |ui| {
                    if state.workspace_name.is_empty() {
                        ui.add_space(30.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("尚未開啟資料夾")
                                    .size(12.5)
                                    .color(Palette::TEXT_DIM),
                            );
                            ui.add_space(8.0);
                            if ui
                                .button(
                                    RichText::new("開啟資料夾...")
                                        .size(12.0)
                                        .color(Palette::TEXT_PRIMARY),
                                )
                                .clicked()
                            {
                                state.prompt_open_folder();
                            }
                        });
                        return;
                    }

                    egui::ScrollArea::vertical()
                        .id_salt("explorer_tree_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for node in &mut state.tree {
                                render_tree_node(
                                    ui,
                                    node,
                                    0,
                                    &active_rel,
                                    &modified_files,
                                    &mut file_to_open,
                                    &mut requested_op,
                                );
                            }
                        });
                });

            if let Some(op) = requested_op {
                state.file_op_modal = Some(op);
            }

            if let Some((full_path, rel_path)) = file_to_open {
                state.open_file_tab(&full_path, &rel_path);
            }
        });

    render_file_modals(ctx, state);
}

fn render_tree_node(
    ui: &mut egui::Ui,
    node: &mut TreeNode,
    depth: usize,
    active_rel: &str,
    modified_files: &std::collections::HashSet<String>,
    file_to_open: &mut Option<(PathBuf, String)>,
    requested_op: &mut Option<FileOpModal>,
) {
    let indent = (depth as f32) * 14.0;

    if node.is_dir {
        let arrow = if node.expanded { "▾" } else { "▸" };
        let label = format!("{arrow}  {}", node.name);

        let row_resp = ui.horizontal(|ui| {
            ui.add_space(indent);
            let resp = ui.selectable_label(
                false,
                RichText::new(label).size(12.5).color(Palette::TEXT_PRIMARY),
            );
            if resp.clicked() {
                node.expanded = !node.expanded;
            }
            resp
        }).inner;

        row_resp.context_menu(|ui| {
            if ui.button("📄  新增檔案 (New File...)").clicked() {
                ui.close_menu();
                *requested_op = Some(FileOpModal::NewFile {
                    parent_dir: node.full_path.clone(),
                    input_name: String::new(),
                });
            }
            if ui.button("📁  新增資料夾 (New Folder...)").clicked() {
                ui.close_menu();
                *requested_op = Some(FileOpModal::NewFolder {
                    parent_dir: node.full_path.clone(),
                    input_name: String::new(),
                });
            }
            ui.separator();
            if ui.button("✏  重新命名 (Rename...)").clicked() {
                ui.close_menu();
                *requested_op = Some(FileOpModal::Rename {
                    target_path: node.full_path.clone(),
                    input_name: node.name.clone(),
                });
            }
            if ui.button("🗑  刪除 (Delete...)").clicked() {
                ui.close_menu();
                *requested_op = Some(FileOpModal::DeleteConfirm {
                    target_path: node.full_path.clone(),
                    is_dir: true,
                });
            }
        });

        if node.expanded {
            for child in &mut node.children {
                render_tree_node(ui, child, depth + 1, active_rel, modified_files, file_to_open, requested_op);
            }
        }
    } else {
        let is_selected = node.rel_path == active_rel;
        let is_modified = modified_files.contains(&node.rel_path);

        let row_resp = ui.horizontal(|ui| {
            ui.add_space(indent + 14.0);
            let resp = ui.selectable_label(
                is_selected,
                RichText::new(&node.name).size(12.5).color(if is_selected {
                    Color32::WHITE
                } else if is_modified {
                    Palette::AMBER
                } else {
                    Palette::TEXT_MUTED
                }),
            );
            if is_modified {
                ui.label(RichText::new("M").size(10.5).color(Palette::AMBER));
            }
            if resp.clicked() {
                *file_to_open = Some((node.full_path.clone(), node.rel_path.clone()));
            }
            resp
        }).inner;

        row_resp.context_menu(|ui| {
            let parent_dir = node.full_path.parent().unwrap_or(&node.full_path).to_path_buf();
            if ui.button("📄  新增檔案 (New File...)").clicked() {
                ui.close_menu();
                *requested_op = Some(FileOpModal::NewFile {
                    parent_dir: parent_dir.clone(),
                    input_name: String::new(),
                });
            }
            if ui.button("📁  新增資料夾 (New Folder...)").clicked() {
                ui.close_menu();
                *requested_op = Some(FileOpModal::NewFolder {
                    parent_dir,
                    input_name: String::new(),
                });
            }
            ui.separator();
            if ui.button("✏  重新命名 (Rename...)").clicked() {
                ui.close_menu();
                *requested_op = Some(FileOpModal::Rename {
                    target_path: node.full_path.clone(),
                    input_name: node.name.clone(),
                });
            }
            if ui.button("🗑  刪除 (Delete...)").clicked() {
                ui.close_menu();
                *requested_op = Some(FileOpModal::DeleteConfirm {
                    target_path: node.full_path.clone(),
                    is_dir: false,
                });
            }
        });
    }
}

fn render_file_modals(ctx: &egui::Context, state: &mut AppState) {
    let Some(mut modal) = state.file_op_modal.take() else {
        return;
    };

    let mut close_modal = false;
    let mut tree_changed = false;
    let mut tab_to_open = None;

    let screen_rect = ctx.screen_rect();
    let modal_width = 380.0;
    let pos = egui::pos2(
        screen_rect.center().x - modal_width / 2.0,
        screen_rect.center().y - 80.0,
    );

    egui::Window::new("file_op_modal")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(pos)
        .fixed_size([modal_width, 140.0])
        .frame(
            Frame::NONE
                .fill(Palette::BG_SIDEBAR)
                .stroke(Stroke::new(1.0, Palette::BORDER))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(14, 12)),
        )
        .show(ctx, |ui| {
            ui.set_width(modal_width - 28.0);

            match &mut modal {
                FileOpModal::NewFile { parent_dir, input_name } => {
                    ui.label(RichText::new("📄 新增檔案").strong().size(13.5).color(Palette::TEXT_PRIMARY));
                    ui.add_space(4.0);
                    let resp = ui.add(
                        egui::TextEdit::singleline(input_name)
                            .hint_text("檔案名稱 (例如: utils.rs)")
                            .desired_width(ui.available_width()),
                    );
                    resp.request_focus();

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let confirm = ui.button(RichText::new("建立").color(Color32::WHITE)).clicked()
                            || (resp.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)));
                        let cancel = ui.button("取消").clicked()
                            || ctx.input(|i| i.key_pressed(Key::Escape));

                        if confirm {
                            let trimmed = input_name.trim();
                            if !trimmed.is_empty() {
                                let full = parent_dir.join(trimmed);
                                if !full.exists() {
                                    let _ = std::fs::write(&full, "");
                                    let rel = full.strip_prefix(&state.workspace_root)
                                        .unwrap_or(&full)
                                        .to_string_lossy()
                                        .replace('\\', "/");
                                    tab_to_open = Some((full, rel));
                                    tree_changed = true;
                                }
                            }
                            close_modal = true;
                        }
                        if cancel {
                            close_modal = true;
                        }
                    });
                }
                FileOpModal::NewFolder { parent_dir, input_name } => {
                    ui.label(RichText::new("📁 新增資料夾").strong().size(13.5).color(Palette::TEXT_PRIMARY));
                    ui.add_space(4.0);
                    let resp = ui.add(
                        egui::TextEdit::singleline(input_name)
                            .hint_text("資料夾名稱")
                            .desired_width(ui.available_width()),
                    );
                    resp.request_focus();

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let confirm = ui.button(RichText::new("建立").color(Color32::WHITE)).clicked()
                            || (resp.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)));
                        let cancel = ui.button("取消").clicked()
                            || ctx.input(|i| i.key_pressed(Key::Escape));

                        if confirm {
                            let trimmed = input_name.trim();
                            if !trimmed.is_empty() {
                                let full = parent_dir.join(trimmed);
                                let _ = std::fs::create_dir_all(&full);
                                tree_changed = true;
                            }
                            close_modal = true;
                        }
                        if cancel {
                            close_modal = true;
                        }
                    });
                }
                FileOpModal::Rename { target_path, input_name } => {
                    ui.label(RichText::new("✏ 重新命名").strong().size(13.5).color(Palette::TEXT_PRIMARY));
                    ui.add_space(4.0);
                    let resp = ui.add(
                        egui::TextEdit::singleline(input_name)
                            .desired_width(ui.available_width()),
                    );
                    resp.request_focus();

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let confirm = ui.button(RichText::new("確認").color(Color32::WHITE)).clicked()
                            || (resp.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)));
                        let cancel = ui.button("取消").clicked()
                            || ctx.input(|i| i.key_pressed(Key::Escape));

                        if confirm {
                            let trimmed = input_name.trim();
                            if !trimmed.is_empty() {
                                if let Some(parent) = target_path.parent() {
                                    let new_full = parent.join(trimmed);
                                    if std::fs::rename(&*target_path, &new_full).is_ok() {
                                        for tab in &mut state.open_tabs {
                                            if tab.full_path == *target_path {
                                                tab.full_path = new_full.clone();
                                                tab.name = trimmed.to_string();
                                                tab.rel_path = new_full.strip_prefix(&state.workspace_root)
                                                    .unwrap_or(&new_full)
                                                    .to_string_lossy()
                                                    .replace('\\', "/");
                                            }
                                        }
                                        tree_changed = true;
                                    }
                                }
                            }
                            close_modal = true;
                        }
                        if cancel {
                            close_modal = true;
                        }
                    });
                }
                FileOpModal::DeleteConfirm { target_path, is_dir } => {
                    let item_type = if *is_dir { "資料夾" } else { "檔案" };
                    let name = target_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    ui.label(
                        RichText::new(format!("確定要刪除此{item_type}嗎？"))
                            .strong()
                            .size(13.5)
                            .color(Palette::TEXT_PRIMARY),
                    );
                    ui.add_space(2.0);
                    ui.label(RichText::new(name).size(12.0).color(Palette::AMBER));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("刪除").color(Color32::from_rgb(240, 100, 100))).clicked() {
                            let target_p = target_path.clone();
                            if *is_dir {
                                let _ = std::fs::remove_dir_all(&target_p);
                            } else {
                                let _ = std::fs::remove_file(&target_p);
                            }

                            state.open_tabs.retain(|t| t.full_path != target_p);
                            if state.active_tab_idx >= state.open_tabs.len() && !state.open_tabs.is_empty() {
                                state.active_tab_idx = state.open_tabs.len() - 1;
                            }

                            tree_changed = true;
                            close_modal = true;
                        }
                        if ui.button("取消").clicked() || ctx.input(|i| i.key_pressed(Key::Escape)) {
                            close_modal = true;
                        }
                    });
                }
                FileOpModal::CloseDirtyConfirm { tab_idx, tab_name } => {
                    let idx = *tab_idx;
                    ui.label(
                        RichText::new(format!("檔案 \"{tab_name}\" 尚未儲存"))
                            .strong()
                            .size(13.5)
                            .color(Palette::AMBER),
                    );
                    ui.add_space(2.0);
                    ui.label(RichText::new("關閉前是否要儲存變更？").size(12.0).color(Palette::TEXT_MUTED));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("儲存並關閉").color(Palette::GREEN)).clicked() {
                            if let Some(tab) = state.open_tabs.get_mut(idx) {
                                let _ = std::fs::write(&tab.full_path, &tab.content);
                                tab.is_dirty = false;
                            }
                            state.close_tab(idx);
                            close_modal = true;
                        }
                        if ui.button(RichText::new("捨棄變更").color(Color32::from_rgb(240, 100, 100))).clicked() {
                            state.close_tab(idx);
                            close_modal = true;
                        }
                        if ui.button("取消").clicked() || ctx.input(|i| i.key_pressed(Key::Escape)) {
                            close_modal = true;
                        }
                    });
                }
            }
        });

    if let Some((path, rel)) = tab_to_open {
        state.open_file_tab(&path, &rel);
    }
    if tree_changed {
        state.refresh_tree();
    }
    if !close_modal {
        state.file_op_modal = Some(modal);
    }
}
