use crate::state::{AppState, TreeNode};
use crate::theme::Palette;
use egui::{Color32, Frame, Margin, RichText};
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
            // Clean Header inside Left Panel (no duplicate Open Folder / New Window buttons)
            egui::TopBottomPanel::top("left_explorer_header")
                .exact_height(32.0)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_SIDEBAR)
                        .inner_margin(Margin::symmetric(10, 0)),
                )
                .show_inside(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.label(
                            RichText::new(state.workspace_name.to_uppercase())
                                .size(11.5)
                                .strong()
                                .color(Palette::TEXT_MUTED),
                        )
                        .on_hover_text(state.workspace_root.display().to_string());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .small_button(
                                    RichText::new("↻").size(12.0).color(Palette::TEXT_MUTED),
                                )
                                .on_hover_text("重新整理檔案樹")
                                .clicked()
                            {
                                state.refresh_tree();
                            }
                        });
                    });
                });

            let active_rel = state
                .active_tab()
                .map(|t| t.rel_path.clone())
                .unwrap_or_default();

            let mut file_to_open: Option<(PathBuf, String)> = None;

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
                    egui::ScrollArea::vertical()
                        .id_salt("explorer_tree_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for node in &mut state.tree {
                                render_tree_node(ui, node, 0, &active_rel, &mut file_to_open);
                            }
                        });
                });

            if let Some((full_path, rel_path)) = file_to_open {
                state.open_file_tab(&full_path, &rel_path);
            }
        });
}

fn render_tree_node(
    ui: &mut egui::Ui,
    node: &mut TreeNode,
    depth: usize,
    active_rel: &str,
    file_to_open: &mut Option<(PathBuf, String)>,
) {
    let indent = (depth as f32) * 14.0;

    if node.is_dir {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let arrow = if node.expanded { "▾" } else { "▸" };
            let label = format!("{arrow}  {}", node.name);
            let resp = ui.selectable_label(
                false,
                RichText::new(label).size(12.5).color(Palette::TEXT_PRIMARY),
            );
            if resp.clicked() {
                node.expanded = !node.expanded;
            }
        });

        if node.expanded {
            for child in &mut node.children {
                render_tree_node(ui, child, depth + 1, active_rel, file_to_open);
            }
        }
    } else {
        let is_selected = node.rel_path == active_rel;
        ui.horizontal(|ui| {
            ui.add_space(indent + 14.0);
            let resp = ui.selectable_label(
                is_selected,
                RichText::new(&node.name).size(12.5).color(if is_selected {
                    Color32::WHITE
                } else {
                    Palette::TEXT_MUTED
                }),
            );
            if resp.clicked() {
                *file_to_open = Some((node.full_path.clone(), node.rel_path.clone()));
            }
        });
    }
}
