//! A file tab. Until the editors ship, it shows the file and how to open it.

use super::home::content_column;
use crate::app::{KadeeApp, file_name};
use egui::{CornerRadius, RichText, Stroke, Ui, Vec2};
use kadee_commands::Command;
use kadee_core::{DocKind, FileEntry, human_size};
use kadee_ui::{Palette, doc_badge, primary_button};
use std::path::Path;

pub fn show(app: &mut KadeeApp, ui: &mut Ui, path: &Path) {
    let entry = FileEntry::from_path(path);
    let Some(kind) = DocKind::from_path(path) else { return };
    content_column(ui, |ui| {
        let pal = Palette::of(ui);
        ui.add_space(30.0);
        egui::Frame::new()
            .fill(pal.surface)
            .stroke(Stroke::new(1.0, pal.border))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(egui::Margin::same(32))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    doc_badge(ui, &kind, 64.0);
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(file_name(path)).size(24.0).strong().color(pal.text));
                        ui.label(RichText::new(path.display().to_string()).size(12.5).monospace().color(pal.text_faint));
                    });
                });
                ui.add_space(18.0);
                if let Some(e) = &entry {
                    let modified = if e.modified > 0 {
                        chrono::DateTime::from_timestamp(e.modified as i64, 0)
                            .map(|d| d.with_timezone(&chrono::Local).format("%-d %B %Y, %H:%M").to_string())
                            .unwrap_or_default()
                    } else {
                        "—".into()
                    };
                    egui::Grid::new("facts").num_columns(2).spacing([28.0, 8.0]).show(ui, |ui| {
                        fact(ui, "Type", kind.group.label());
                        fact(ui, "Opens in", kind.app_name());
                        fact(ui, "Size", &if e.exists { human_size(e.size) } else { "File not found".into() });
                        fact(ui, "Modified", &modified);
                    });
                }
                ui.add_space(22.0);
                ui.horizontal(|ui| {
                    if primary_button(ui, "Open in default app").clicked() {
                        app.send(Command::OpenExternally(path.to_path_buf()));
                    }
                    if ui.button("Show in folder").clicked() {
                        app.send(Command::RevealInFolder(path.to_path_buf()));
                    }
                    let starred = app.state.library.is_starred(path);
                    if ui.button(if starred { "Unstar" } else { "Star" }).clicked() {
                        app.send(Command::ToggleStar(path.to_path_buf()));
                    }
                });
            });
        ui.add_space(18.0);
        egui::Frame::new().fill(pal.accent_soft).corner_radius(CornerRadius::same(10)).inner_margin(egui::Margin::symmetric(18, 14)).show(
            ui,
            |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                let msg = if cfg!(windows) && kind.group == kadee_core::DocGroup::Markdown {
                    "Kadee Markdown runs on macOS and Linux; Windows support follows.".to_string()
                } else if kind.is_craft() {
                    format!("KadeeStudio hands {} files to {} from Phase {}.", kind.group.label(), kind.app_name(), kind.phase())
                } else {
                    format!("The {} editor arrives in Phase {}.", kind.app_name(), kind.phase())
                };
                ui.label(RichText::new(msg).strong().color(pal.text));
                ui.label(RichText::new("This tab will host it. Until then, open the file in your default app.").color(pal.text_muted));
            },
        );
    });
}

fn fact(ui: &mut Ui, k: &str, v: &str) {
    let pal = Palette::of(ui);
    ui.label(RichText::new(k).color(pal.text_faint));
    ui.label(RichText::new(v).color(pal.text));
    ui.end_row();
}
