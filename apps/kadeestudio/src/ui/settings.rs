//! The Settings window: General, Keyboard and About in one popup, with a
//! section list on the left and Cancel / Done at the bottom. Changes show at
//! once; Done keeps them, Cancel (or Esc, or ×) puts back what was there.

use super::icon_button;
use crate::app::{KadeeApp, OS};
use crate::icons::{self, Icon};
use egui::{Align2, Color32, CornerRadius, FontId, Margin, Rect, RichText, Sense, Stroke, Ui, UiBuilder, Vec2, pos2, vec2};
use kadee_commands::{Command, SettingsSection};
use kadee_core::{CRAFT_APPS, LAUNCHER_REPO, ThemePref};
use kadee_ui::{Palette, primary_button};

const HEADER: f32 = 68.0;
const FOOTER: f32 = 72.0;
const NAV: f32 = 232.0;

/// Show the window if it is open.
pub fn window(app: &mut KadeeApp, ui: &Ui) {
    let Some(section) = app.settings_window.as_ref().map(|w| w.section) else { return };
    let ctx = ui.ctx().clone();
    let pal = Palette::of(ui);
    let screen = ctx.content_rect();
    let size = vec2((screen.width() - 80.0).min(980.0), (screen.height() - 80.0).min(740.0));
    let mut close: Option<bool> = None; // Some(true) = Done, Some(false) = Cancel
    let mut goto = None;

    let resp = egui::Modal::new(egui::Id::unique("kadee-settings"))
        .frame(
            egui::Frame::new()
                .fill(pal.surface)
                .stroke(Stroke::new(1.0, pal.border_strong))
                .corner_radius(CornerRadius::same(16))
                .shadow(egui::epaint::Shadow { offset: [0, 12], blur: 40, spread: 0, color: Color32::from_black_alpha(90) }),
        )
        .show(&ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let header = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER));
            let footer = Rect::from_min_max(pos2(rect.min.x, rect.max.y - FOOTER), rect.max);
            let body = Rect::from_min_max(pos2(rect.min.x, header.max.y), pos2(rect.max.x, footer.min.y));
            let nav = Rect::from_min_size(body.min, vec2(NAV, body.height()));
            let content = Rect::from_min_max(pos2(nav.max.x + 1.0, body.min.y), body.max);
            let p = ui.painter();
            p.hline(rect.x_range(), header.max.y, Stroke::new(1.0, pal.border));
            p.hline(rect.x_range(), footer.min.y, Stroke::new(1.0, pal.border));
            p.vline(nav.max.x, body.y_range(), Stroke::new(1.0, pal.border));

            // Header: title and ×
            p.text(pos2(header.min.x + 32.0, header.center().y), Align2::LEFT_CENTER, "Settings", FontId::proportional(21.0), pal.text);
            region(ui, Rect::from_center_size(pos2(header.max.x - 38.0, header.center().y), Vec2::splat(30.0)), |ui| {
                if icon_button(ui, Icon::Close, "Close without saving (Esc)").clicked() {
                    close = Some(false);
                }
            });

            // Sections
            region(ui, nav.shrink2(vec2(16.0, 18.0)), |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                for s in SettingsSection::ALL {
                    if nav_item(ui, s, s == section) {
                        goto = Some(s);
                    }
                }
            });

            // Content
            region(ui, content, |ui| {
                egui::ScrollArea::vertical().id_salt(("settings", section.label())).auto_shrink([false, false]).show(ui, |ui| {
                    egui::Frame::new().inner_margin(Margin { left: 36, right: 36, top: 28, bottom: 28 }).show(ui, |ui| {
                        // fixed width: room for the scroll bar, so cards never run past the edge
                        ui.set_width(content.width() - 72.0 - 16.0);
                        match section {
                            SettingsSection::General => general(app, ui),
                            SettingsSection::Keyboard => super::pages::shortcuts_editor(app, ui),
                            SettingsSection::About => about(app, ui),
                        }
                    });
                });
            });

            // Footer: note, Cancel, Done
            region(ui, footer.shrink2(vec2(32.0, 0.0)), |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    ui.spacing_mut().button_padding = vec2(18.0, 8.0);
                    if primary_button(ui, "Done").clicked() {
                        close = Some(true);
                    }
                    if ui.button(RichText::new("Cancel").size(14.0)).clicked() {
                        close = Some(false);
                    }
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Changes are saved when you click Done.").size(13.0).color(pal.text_faint));
                    });
                });
            });
        });

    // Esc (when not recording a shortcut) is Cancel; a click beside the window does nothing.
    if close.is_none()
        && app.recording.is_none()
        && resp.is_top_modal
        && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
    {
        close = Some(false);
    }
    if let Some(s) = goto {
        app.send(Command::OpenSettings(s));
    }
    if let Some(keep) = close {
        app.close_settings(keep, &ctx);
        if !keep {
            let t = ctx.input(|i| i.time);
            app.notify("Settings not changed", t);
        }
    }
}

fn section_icon(s: SettingsSection) -> Icon {
    match s {
        SettingsSection::General => Icon::Gear,
        SettingsSection::Keyboard => Icon::Keyboard,
        SettingsSection::About => Icon::Info,
    }
}

fn nav_item(ui: &mut Ui, s: SettingsSection, selected: bool) -> bool {
    let pal = Palette::of(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, CornerRadius::same(9), pal.accent_soft);
    } else if resp.hovered() {
        p.rect_filled(rect, CornerRadius::same(9), pal.surface_hover);
    }
    let fg = if selected { pal.accent } else { pal.text_muted };
    icons::paint(p, Rect::from_center_size(pos2(rect.min.x + 24.0, rect.center().y), Vec2::splat(19.0)), section_icon(s), fg);
    p.text(pos2(rect.min.x + 48.0, rect.center().y), Align2::LEFT_CENTER, s.label(), FontId::proportional(15.5), pal.text);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// Draw into `rect` without letting the content resize the window: clipped, and the
/// parent's layout does not move.
fn region(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) {
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    add(&mut child);
}

/// A row of buttons centred in the available width; returns the one clicked.
fn button_row(ui: &mut Ui, labels: &[&str]) -> Option<usize> {
    let gap = 8.0;
    let font = egui::TextStyle::Button.resolve(ui.style());
    let pad = ui.spacing().button_padding.x * 2.0;
    let widths: Vec<f32> =
        labels.iter().map(|l| ui.painter().layout_no_wrap((*l).to_string(), font.clone(), Color32::WHITE).size().x + pad).collect();
    let total = widths.iter().sum::<f32>() + gap * (labels.len() as f32 - 1.0);
    let mut clicked = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
        for (i, l) in labels.iter().enumerate() {
            if ui.button(*l).clicked() {
                clicked = Some(i);
            }
        }
    });
    clicked
}

/// A long path cut in the middle: `/tmp/…/home.json`.
fn middle(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(1);
    let head: String = s.chars().take(keep / 2).collect();
    let tail: String = s.chars().skip(n - (keep - keep / 2)).collect();
    format!("{head}…{tail}")
}

/// Section title and a line under it.
pub fn heading(ui: &mut Ui, title: &str, sub: &str) {
    let pal = Palette::of(ui);
    ui.label(RichText::new(title).size(22.0).strong().color(pal.text));
    if !sub.is_empty() {
        ui.add_space(2.0);
        ui.label(RichText::new(sub).size(13.5).color(pal.text_muted));
    }
    ui.add_space(18.0);
}

/// A labelled field, label above the control.
fn field(ui: &mut Ui, label: &str, hint: &str, add: impl FnOnce(&mut Ui)) {
    let pal = Palette::of(ui);
    ui.label(RichText::new(label).size(14.0).strong().color(pal.text));
    ui.add_space(4.0);
    add(ui);
    if !hint.is_empty() {
        ui.add_space(2.0);
        ui.label(RichText::new(hint).size(12.5).color(pal.text_faint));
    }
    ui.add_space(18.0);
}

/// A segmented control: one choice out of a few.
fn segmented<T: Copy + PartialEq>(ui: &mut Ui, value: T, options: &[(T, &str)]) -> Option<T> {
    let pal = Palette::of(ui);
    let mut picked = None;
    egui::Frame::new().fill(pal.canvas).corner_radius(CornerRadius::same(9)).inner_margin(Margin::same(3)).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for (v, label) in options {
                let on = *v == value;
                let text = RichText::new(*label).size(13.5).color(if on { pal.text } else { pal.text_muted });
                let b = egui::Button::new(text)
                    .fill(if on { pal.surface_hover } else { Color32::TRANSPARENT })
                    .stroke(if on { Stroke::new(1.0, pal.border) } else { Stroke::NONE })
                    .corner_radius(CornerRadius::same(7))
                    .min_size(vec2(76.0, 28.0));
                if ui.add(b).clicked() && !on {
                    picked = Some(*v);
                }
            }
        });
    });
    picked
}

fn general(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    heading(ui, "General", "");
    ui.set_max_width(640.0);

    field(ui, "Theme", "", |ui| {
        let opts = ThemePref::ALL.map(|t| (t, t.label()));
        if let Some(t) = segmented(ui, app.state.settings.theme, &opts) {
            app.send(Command::SetTheme(t));
        }
    });

    field(ui, "Name in the greeting", "Shown on the start screen: “Good evening, Kris”.", |ui| {
        let r = ui.add(
            egui::TextEdit::singleline(&mut app.state.settings.display_name)
                .hint_text("Your name")
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(10, 7)),
        );
        if r.changed() {
            app.mark_dirty();
        }
    });

    field(ui, "Save new documents in", "Also the KadeeStudio folder in the sidebar.", |ui| {
        ui.horizontal(|ui| {
            let w = ui.available_width() - 110.0;
            egui::Frame::new()
                .fill(pal.canvas)
                .stroke(Stroke::new(1.0, pal.border))
                .corner_radius(CornerRadius::same(7))
                .inner_margin(Margin::symmetric(10, 7))
                .show(ui, |ui| {
                    ui.set_width(w - 22.0);
                    let path = tilde(&app.state.settings.documents_dir);
                    ui.add(egui::Label::new(RichText::new(path).color(pal.text_muted)).truncate())
                        .on_hover_text(app.state.settings.documents_dir.display().to_string());
                });
            if ui.button("Change…").clicked()
                && let Some(dir) = rfd::FileDialog::new().set_directory(&app.state.settings.documents_dir).pick_folder()
            {
                app.state.settings.documents_dir = dir;
                app.mark_dirty();
                app.rescan();
            }
        });
    });

    field(ui, "Team server", "Real-time collaboration through your own server on Docker or Kubernetes — Phase 3.", |ui| {
        ui.add_enabled(false, egui::Button::new("Connect…"));
    });

    ui.add_space(6.0);
    ui.label(RichText::new(format!("Settings file: {}", middle(&tilde(&app.store.file()), 70))).size(12.5).color(pal.text_faint));
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label(RichText::new(format!("KadeeStudio {} ·", kadee_core::VERSION)).size(12.5).color(pal.text_faint));
        if ui.link(RichText::new("About").size(12.5)).clicked() {
            app.send(Command::OpenSettings(SettingsSection::About));
        }
    });
}

/// Version, build and system in one line, for "Copy version info".
fn version_info(app: &KadeeApp) -> String {
    format!(
        "KadeeStudio {} (build {}, {})\n{} {} · {} renderer\n{} of {} Crafting Apps installed{}",
        kadee_core::VERSION,
        env!("KADEE_COMMIT"),
        env!("KADEE_BUILD_DATE"),
        OS.name(),
        std::env::consts::ARCH,
        renderer(),
        app.installed.len(),
        CRAFT_APPS.len(),
        app.installed.iter().map(|i| format!("\n  {} {}", i.app.name, i.version.as_deref().unwrap_or(""))).collect::<String>()
    )
}

fn renderer() -> &'static str {
    if cfg!(feature = "glow") { "OpenGL (glow)" } else { "wgpu" }
}

/// The About section; also what the macOS menu's "About KadeeStudio" opens.
pub fn about(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    let now = ui.input(|i| i.time);
    let w = ui.available_width() - 2.0; // border

    // Logo, name, version
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::splat(84.0), Sense::hover());
        kadee_ui::paint_logo(ui, r);
        ui.add_space(14.0);
        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.label(RichText::new("KadeeStudio").size(26.0).strong().color(pal.text));
            ui.label(RichText::new(format!("Version {}", kadee_core::VERSION)).size(15.0).color(pal.text));
            ui.label(
                RichText::new(format!("Build {} · {}", env!("KADEE_COMMIT"), env!("KADEE_BUILD_DATE"))).size(12.5).color(pal.text_faint),
            );
        });
    });
    ui.add_space(14.0);
    ui.label(
        RichText::new(
            "A native Rust office suite: AI, Markdown, PDF, conversion, dashboards and reports, with real-time \
             collaboration through your own server on Docker or Kubernetes. It also brings together the Crafting Apps. \
             This release is Phase 0: the start screen.",
        )
        .size(14.0)
        .color(pal.text_muted),
    );
    ui.add_space(14.0);
    let docs = button_row(ui, &["Open documents folder", "Open templates folder", "Open settings folder"]);
    ui.add_space(6.0);
    let more = button_row(ui, &["Source code", "Copy version info"]);
    match docs {
        Some(0) => {
            let dir = app.state.settings.documents_dir.clone();
            let _ = std::fs::create_dir_all(&dir);
            if let Err(e) = open::that_detached(&dir) {
                app.notify(format!("Could not open the folder: {e}"), now);
            }
        }
        Some(1) => {
            let dir = app.store.templates_dir();
            match std::fs::create_dir_all(&dir)
                .map_err(|e| e.to_string())
                .and_then(|()| open::that_detached(&dir).map_err(|e| e.to_string()))
            {
                Ok(()) => {}
                Err(e) => app.notify(format!("Could not open the templates folder: {e}"), now),
            }
        }
        Some(_) => {
            let _ = open::that_detached(app.store.dir());
        }
        None => {}
    }
    match more {
        Some(0) => app.send(Command::OpenUrl(kadee_core::SOURCE_REPO.into())),
        Some(_) => {
            ui.ctx().copy_text(version_info(app));
            app.notify("Version info copied", now);
        }
        None => {}
    }
    ui.add_space(22.0);

    card(ui, w, "This copy", |ui| {
        egui::Grid::new("about-details").num_columns(2).spacing([28.0, 8.0]).show(ui, |ui| {
            let row = |ui: &mut Ui, k: &str, v: String| {
                ui.label(RichText::new(k).color(pal.text_faint));
                ui.label(RichText::new(v).color(pal.text));
                ui.end_row();
            };
            row(ui, "Version", format!("{} (Phase 0)", kadee_core::VERSION));
            row(ui, "Build", format!("{} · {}", env!("KADEE_COMMIT"), env!("KADEE_BUILD_DATE")));
            row(ui, "System", format!("{} · {}", OS.name(), std::env::consts::ARCH));
            row(ui, "Renderer", renderer().to_string());
            row(ui, "Documents", middle(&tilde(&app.state.settings.documents_dir), 64));
            row(ui, "Templates", middle(&tilde(&app.store.templates_dir()), 64));
            row(ui, "Settings", middle(&tilde(&app.store.file()), 64));
        });
    });

    card(ui, w, "Crafting Apps", |ui| {
        let n = app.installed.len();
        ui.label(RichText::new(format!("{n} of {} installed on this computer", CRAFT_APPS.len())).color(if n > 0 {
            pal.text
        } else {
            pal.text_muted
        }));
        ui.add_space(8.0);
        let installed = app.installed.clone();
        let cols = 3usize;
        let gap = 12.0;
        let cw = ((w - 40.0) - gap * (cols as f32 - 1.0)) / cols as f32;
        for row in installed.chunks(cols) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for i in row {
                    let (r, resp) = ui.allocate_exact_size(vec2(cw, 34.0), Sense::hover());
                    let icon = Rect::from_min_size(r.min + vec2(0.0, 3.0), Vec2::splat(28.0));
                    app.craft_icons.paint(ui, icon, i.app, false);
                    let p = ui.painter();
                    p.text(
                        pos2(icon.max.x + 10.0, r.center().y - 7.0),
                        Align2::LEFT_CENTER,
                        i.app.name,
                        FontId::proportional(14.0),
                        pal.text,
                    );
                    p.text(
                        pos2(icon.max.x + 10.0, r.center().y + 9.0),
                        Align2::LEFT_CENTER,
                        i.version.as_deref().unwrap_or(i.source),
                        FontId::proportional(11.5),
                        pal.text_faint,
                    );
                    resp.on_hover_text(format!("{} · {}", i.source, i.location()));
                }
            });
            ui.add_space(4.0);
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("ArtCraft apps by the ArtCraft team, SolidCraft by Tangency Labs ·").size(12.5).color(pal.text_faint));
            if ui.link(RichText::new("ArtCraft launcher").size(12.5)).clicked() {
                app.send(Command::OpenUrl(LAUNCHER_REPO.into()));
            }
        });
    });

    card(ui, w, "Credits and license", |ui| {
        let line = |ui: &mut Ui, t: &str| {
            ui.add(egui::Label::new(RichText::new(t).color(pal.text_muted)).halign(egui::Align::Center));
        };
        ui.vertical_centered(|ui| {
        line(ui, "Made by Kris Dugardyn. Built with Rust, egui and eframe.");
        line(
            ui,
            "The Crafting App names and icons belong to the ArtCraft team, Tangency Labs and each app's contributors; the icons are used under MIT OR Apache-2.0.",
        );
        line(ui, "KadeeStudio is licensed under MIT OR Apache-2.0.");
        ui.add_space(4.0);
        ui.label(RichText::new("© 2026 Kris Dugardyn").color(pal.text_faint));
        });
    });
}

fn card(ui: &mut Ui, width: f32, title: &str, add: impl FnOnce(&mut Ui)) {
    let pal = Palette::of(ui);
    ui.label(RichText::new(title.to_uppercase()).size(12.0).strong().color(pal.text_faint));
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(pal.canvas)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(20, 16))
        .show(ui, |ui| {
            // fixed: contents wrap inside the card and cannot make it wider
            ui.set_width(width - 40.0);
            add(ui);
        });
    ui.add_space(20.0);
}

/// `~/Documents/KadeeStudio` instead of the full home path.
pub fn tilde(p: &std::path::Path) -> String {
    if let Some(home) = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf())
        && let Ok(rest) = p.strip_prefix(&home)
    {
        return format!("~/{}", rest.display());
    }
    p.display().to_string()
}
