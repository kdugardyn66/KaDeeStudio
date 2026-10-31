//! Crafting Apps and Settings pages.

use super::home::{content_column, page_title};
use crate::app::KadeeApp;
use egui::{Align2, CornerRadius, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use kadee_commands::{Command, default_keymap, describe};
use kadee_core::{CRAFT_APPS, CRAFT_PHASE, LAUNCHER_REPO, SOLIDCRAFT_REPO, ThemePref};
use kadee_ui::{Palette, pill, primary_button};

pub fn craft_apps(app: &mut KadeeApp, ui: &mut Ui) {
    content_column(ui, |ui| {
        let pal = Palette::of(ui);
        page_title(
            ui,
            "Crafting Apps",
            &format!(
                "Open-source Rust apps from the ArtCraft team, plus SolidCraft from Tangency Labs. \
                 KadeeStudio will install, open and embed them in Phase {CRAFT_PHASE}; their files already appear under All."
            ),
        );
        let gap = 14.0;
        let w = ui.available_width();
        let cols = if w > 900.0 { 3 } else { 2 };
        let cw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
        for row in CRAFT_APPS.chunks(cols) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for a in row {
                    let (r, _) = ui.allocate_exact_size(vec2(cw, 96.0), Sense::hover());
                    let p = ui.painter();
                    p.rect(r, CornerRadius::same(10), pal.surface, Stroke::new(1.0, pal.border), StrokeKind::Inside);
                    let icon = Rect::from_min_size(r.min + vec2(16.0, 16.0), Vec2::splat(40.0));
                    p.rect_filled(icon, CornerRadius::same(10), pal.surface_hover);
                    p.text(icon.center(), Align2::CENTER_CENTER, &a.name[..1], FontId::proportional(18.0), pal.text_muted);
                    let x = icon.max.x + 14.0;
                    p.text(pos2(x, r.min.y + 26.0), Align2::LEFT_CENTER, a.name, FontId::proportional(15.0), pal.text);
                    p.text(pos2(x, r.min.y + 46.0), Align2::LEFT_CENTER, a.purpose, FontId::proportional(12.5), pal.text_muted);
                    let exts = a.extensions.iter().map(|e| format!(".{e}")).collect::<Vec<_>>().join(" ");
                    let line = super::truncate(&format!("{} · {exts}", a.maker), ((cw - 90.0) / 6.0) as usize);
                    p.text(pos2(x, r.min.y + 70.0), Align2::LEFT_CENTER, line, FontId::proportional(11.0), pal.text_faint);
                    let g = p.layout_no_wrap(format!("Phase {CRAFT_PHASE}"), FontId::proportional(11.0), pal.text_muted);
                    let cr = Rect::from_min_size(pos2(r.max.x - g.size().x - 30.0, r.min.y + 16.0), vec2(g.size().x + 14.0, 20.0));
                    p.rect_filled(cr, CornerRadius::same(10), pal.surface_hover);
                    p.galley(cr.center() - g.size() / 2.0, g, pal.text_muted);
                }
            });
            ui.add_space(gap - ui.spacing().item_spacing.y);
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Until then, install them yourself:").color(pal.text_muted));
            if ui.link("ArtCraft craft-launcher").clicked() {
                app.send(Command::OpenUrl(LAUNCHER_REPO.into()));
            }
            ui.label(RichText::new("·").color(pal.text_faint));
            if ui.link("SolidCraft").clicked() {
                app.send(Command::OpenUrl(SOLIDCRAFT_REPO.into()));
            }
        });
    });
}

pub fn settings(app: &mut KadeeApp, ui: &mut Ui) {
    content_column(ui, |ui| {
        let pal = Palette::of(ui);
        page_title(ui, "Settings", "Applied at once and saved on this computer.");

        group(ui, "General", |ui| {
            row(ui, "Your name", "Used in the greeting on the start screen.", |ui| {
                let r =
                    ui.add(egui::TextEdit::singleline(&mut app.state.settings.display_name).hint_text("e.g. Kris").desired_width(260.0));
                if r.changed() {
                    app.mark_dirty();
                }
            });
            row(ui, "Theme", "Follow the system, or always light or dark.", |ui| {
                for t in ThemePref::ALL {
                    if pill(ui, t.label(), app.state.settings.theme == t).clicked() {
                        app.send(Command::SetTheme(t));
                    }
                }
            });
            row(ui, "Documents folder", "New documents are saved here.", |ui| {
                ui.label(
                    RichText::new(app.state.settings.documents_dir.display().to_string()).color(pal.text_muted).monospace().size(12.5),
                );
                if ui.button("Change…").clicked()
                    && let Some(dir) = rfd::FileDialog::new().set_directory(&app.state.settings.documents_dir).pick_folder()
                {
                    app.state.settings.documents_dir = dir;
                    app.mark_dirty();
                    app.rescan();
                }
            });
        });

        group(ui, "Collaboration", |ui| {
            row(ui, "Team server", "Connect to a KadeeStudio server on Docker or Kubernetes.", |ui| {
                ui.add_enabled(false, egui::Button::new("Connect…"));
                ui.label(RichText::new("Phase 3").size(12.0).color(pal.text_faint));
            });
        });

        group(ui, "Keyboard shortcuts", |ui| {
            let mac = cfg!(target_os = "macos");
            let mut seen_goto = false;
            egui::Grid::new("keys").num_columns(2).spacing([40.0, 8.0]).show(ui, |ui| {
                for (s, _, label) in default_keymap() {
                    if label == "Go to tab" {
                        if seen_goto {
                            continue;
                        }
                        seen_goto = true;
                        ui.label(RichText::new(if mac { "⌘2 … ⌘9" } else { "Ctrl+2 … Ctrl+9" }).monospace().color(pal.text));
                        ui.label(RichText::new("Go to tab 2–9").color(pal.text_muted));
                        ui.end_row();
                        continue;
                    }
                    ui.label(RichText::new(describe(s, mac)).monospace().color(pal.text));
                    ui.label(RichText::new(label).color(pal.text_muted));
                    ui.end_row();
                }
            });
        });

        group(ui, "About", |ui| {
            ui.label(RichText::new(format!("KadeeStudio {}", kadee_core::VERSION)).strong().color(pal.text));
            ui.label(
                RichText::new("Native Rust office suite: AI, Markdown, PDF, conversion, dashboards and reports. Phase 0: start screen.")
                    .color(pal.text_muted),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if primary_button(ui, "Open documents folder").clicked() {
                    let dir = app.state.settings.documents_dir.clone();
                    let _ = std::fs::create_dir_all(&dir);
                    if let Err(e) = open::that_detached(&dir) {
                        let t = ui.input(|i| i.time);
                        app.notify(format!("Could not open the folder: {e}"), t);
                    }
                }
                if ui.button("Open settings folder").clicked() {
                    let dir = app.store.dir().to_path_buf();
                    let _ = open::that_detached(dir);
                }
            });
        });
    });
}

fn group(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    let pal = Palette::of(ui);
    ui.label(RichText::new(title).size(16.0).strong().color(pal.text));
    ui.add_space(4.0);
    egui::Frame::new()
        .fill(pal.surface)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(18))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
    ui.add_space(22.0);
}

fn row(ui: &mut Ui, label: &str, help: &str, add: impl FnOnce(&mut Ui)) {
    let pal = Palette::of(ui);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(240.0);
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(RichText::new(label).color(pal.text));
            ui.label(RichText::new(help).size(12.0).color(pal.text_faint));
        });
        ui.horizontal(|ui| add(ui));
    });
    ui.add_space(10.0);
}
