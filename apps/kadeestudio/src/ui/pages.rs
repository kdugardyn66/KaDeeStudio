//! The Crafting Apps page and the shortcut editor (Settings › Keyboard).

use super::home::{content_column, page_title};
use crate::app::KadeeApp;
use crate::app::OS;
use crate::ui::keys::{MAX_CHORDS, Recording};
use egui::{Align2, CornerRadius, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use kadee_commands::{Binding, Command, bindable_commands, display_sequence};
use kadee_core::{CRAFT_APPS, CRAFT_PHASE, LAUNCHER_REPO};
use kadee_ui::Palette;

pub fn craft_apps(app: &mut KadeeApp, ui: &mut Ui) {
    content_column(ui, |ui| {
        let pal = Palette::of(ui);
        let n = app.installed.len();
        page_title(
            ui,
            "Crafting Apps",
            &format!(
                "Open-source Rust apps from the ArtCraft team, plus SolidCraft from Tangency Labs. {n} of {} are installed on this computer; \
                 click one to open it. Their files already appear under All; KadeeStudio embeds them in Phase {CRAFT_PHASE}.",
                CRAFT_APPS.len()
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
                    let found = app.installed.iter().find(|i| i.app.id == a.id).cloned();
                    let (r, resp) = ui.allocate_exact_size(vec2(cw, 100.0), Sense::click());
                    let hover = resp.hovered() && found.is_some();
                    let p = ui.painter();
                    p.rect(
                        r,
                        CornerRadius::same(10),
                        if hover { pal.surface_hover } else { pal.surface },
                        Stroke::new(1.0, if hover { pal.accent } else { pal.border }),
                        StrokeKind::Inside,
                    );
                    let icon = Rect::from_min_size(r.min + vec2(16.0, 18.0), Vec2::splat(52.0));
                    app.craft_icons.paint(ui, icon, a, found.is_none());
                    let p = ui.painter();
                    let x = icon.max.x + 14.0;
                    p.text(pos2(x, r.min.y + 28.0), Align2::LEFT_CENTER, a.name, FontId::proportional(15.0), pal.text);
                    p.text(pos2(x, r.min.y + 48.0), Align2::LEFT_CENTER, a.purpose, FontId::proportional(12.5), pal.text_muted);
                    let exts = a.extensions.iter().map(|e| format!(".{e}")).collect::<Vec<_>>().join(" ");
                    let line = super::truncate(&format!("{} · {exts}", a.maker), ((cw - 110.0) / 6.0) as usize);
                    p.text(pos2(x, r.min.y + 72.0), Align2::LEFT_CENTER, line, FontId::proportional(11.0), pal.text_faint);
                    let (chip, fg, bg) = match &found {
                        Some(i) => (
                            match &i.version {
                                Some(v) => format!("Installed · {v}"),
                                None => "Installed".to_string(),
                            },
                            pal.accent,
                            pal.accent_soft,
                        ),
                        None => ("Not installed".to_string(), pal.text_faint, pal.surface_hover),
                    };
                    let g = p.layout_no_wrap(chip, FontId::proportional(11.5), fg);
                    let cr = Rect::from_min_size(pos2(r.max.x - g.size().x - 30.0, r.min.y + 14.0), vec2(g.size().x + 16.0, 22.0));
                    p.rect_filled(cr, CornerRadius::same(11), bg);
                    p.galley(cr.center() - g.size() / 2.0, g, fg);
                    match found {
                        Some(i) => {
                            if resp
                                .on_hover_text(format!("Open {} · {} · {}", a.name, i.source, i.location()))
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                app.send(Command::LaunchCraft(a.id.to_string()));
                            }
                        }
                        None => {
                            resp.on_hover_text(format!("{} is not installed. Get it with the ArtCraft launcher.", a.name));
                        }
                    }
                }
            });
            ui.add_space(gap - ui.spacing().item_spacing.y);
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Install or update them with the").color(pal.text_muted));
            if ui.link("ArtCraft Launcher").clicked() {
                app.send(Command::OpenUrl(LAUNCHER_REPO.into()));
            }
            ui.label(RichText::new("·").color(pal.text_faint));
            if ui.link("Look again").clicked() {
                app.detect_crafts();
            }
        });
    });
}

/// The four modifier keys of this system, as chips: Cmd Ctrl Option Shift on a Mac,
/// Win Ctrl Alt Shift on Windows, Super Ctrl Alt Shift on Linux.
fn modifier_legend(ui: &mut Ui) {
    let pal = Palette::of(ui);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(format!("Modifier keys on {}:", OS.name())).color(pal.text_muted));
        for name in OS.modifier_names() {
            egui::Frame::new()
                .fill(pal.surface_hover)
                .stroke(Stroke::new(1.0, pal.border))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(8, 2))
                .show(ui, |ui| {
                    ui.label(RichText::new(name).monospace().color(pal.text));
                });
        }
        let note = match OS {
            kadee_commands::Os::Mac => "Cmd and Ctrl are different keys; Cmd is the main shortcut key.",
            kadee_commands::Os::Windows => "Ctrl is the main shortcut key. Windows keeps most Win+letter combinations for itself.",
            kadee_commands::Os::Linux => "Ctrl is the main shortcut key. Super is the key with the Windows logo on most PC keyboards.",
        };
        ui.label(RichText::new(note).size(12.5).color(pal.text_faint));
    });
}

/// A shortcut as text, or a muted "Not assigned".
fn shortcut_text(ui: &mut Ui, keys: Option<&str>) {
    let pal = Palette::of(ui);
    match keys {
        Some(k) => ui.label(RichText::new(k).monospace().color(pal.text)),
        None => ui.label(RichText::new("Not assigned").italics().color(pal.text_faint)),
    };
}

/// What a click on the Keyboard shortcuts page asks for (applied after drawing).
enum Edit {
    Record(usize),
    Clear(usize),
    Remove(usize),
    Add,
    Reset,
}

/// Settings › Keyboard: record, clear, remove, add and reset shortcuts.
pub fn shortcuts_editor(app: &mut KadeeApp, ui: &mut Ui) {
    let now = ui.input(|i| i.time);
    let mut edit: Option<Edit> = None;
    {
        let pal = Palette::of(ui);
        super::settings::heading(
            ui,
            "Keyboard shortcuts",
            &format!(
                "Click Record and press the keys. For a sequence, press them one after the other, such as {}+N then M. \
                 Enter saves at once, Esc cancels, Backspace starts over; after {:.1} s without a key the shortcut is saved.",
                OS.primary_name(),
                crate::ui::keys::RECORD_IDLE
            ),
        );
        modifier_legend(ui);
        ui.add_space(14.0);

        let recording_index = app.recording.as_ref().map(|r| r.index);
        egui::Frame::new()
            .fill(pal.surface)
            .stroke(Stroke::new(1.0, pal.border))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(egui::Margin::symmetric(18, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::Grid::new("shortcut-editor").num_columns(3).spacing([20.0, 10.0]).striped(false).show(ui, |ui| {
                    ui.label(RichText::new("ACTION").size(12.0).color(pal.text_faint));
                    ui.label(RichText::new("SHORTCUT").size(12.0).color(pal.text_faint));
                    ui.label("");
                    ui.end_row();
                    for (i, b) in app.keymap.bindings.iter().enumerate() {
                        ui.label(RichText::new(b.command.label()).color(pal.text));
                        ui.vertical(|ui| {
                            ui.set_min_width(150.0);
                            if recording_index == Some(i) {
                                let rec = app.recording.as_ref().map(|r| r.chords.clone()).unwrap_or_default();
                                let text = if rec.is_empty() {
                                    "Press keys…".to_string()
                                } else {
                                    format!("{} …", display_sequence(&rec, OS))
                                };
                                egui::Frame::new()
                                    .fill(pal.accent_soft)
                                    .stroke(Stroke::new(1.0, pal.accent))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(egui::Margin::symmetric(10, 4))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(text).monospace().strong().color(pal.accent));
                                    });
                            } else if b.keys.is_empty() {
                                shortcut_text(ui, None);
                            } else {
                                egui::Frame::new()
                                    .fill(pal.surface_hover)
                                    .stroke(Stroke::new(1.0, pal.border))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(egui::Margin::symmetric(10, 4))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(display_sequence(&b.keys, OS)).monospace().color(pal.text));
                                    });
                                if let Some(j) = app.keymap.conflict(i) {
                                    ui.label(
                                        RichText::new(format!("Also used by “{}”", app.keymap.bindings[j].command.label()))
                                            .size(12.0)
                                            .color(pal.danger),
                                    );
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            let busy = recording_index.is_some();
                            let record_label = if b.keys.is_empty() { "Record" } else { "Change" };
                            if ui.add_enabled(!busy, egui::Button::new(record_label)).clicked() {
                                edit = Some(Edit::Record(i));
                            }
                            if !b.keys.is_empty() && ui.add_enabled(!busy, egui::Button::new("Clear")).clicked() {
                                edit = Some(Edit::Clear(i));
                            }
                            if ui
                                .add_enabled(!busy, egui::Button::new("Remove"))
                                .on_hover_text("Remove this shortcut from the list")
                                .clicked()
                            {
                                edit = Some(Edit::Remove(i));
                            }
                        });
                        ui.end_row();
                    }
                });
            });

        ui.add_space(18.0);
        ui.label(RichText::new("Add a shortcut").size(16.0).strong().color(pal.text));
        ui.label(
            RichText::new(format!(
                "Pick any action, then record its keys (up to {MAX_CHORDS} in a sequence). An action can have several shortcuts."
            ))
            .color(pal.text_muted),
        );
        ui.add_space(4.0);
        let actions = bindable_commands();
        ui.horizontal(|ui| {
            let current = actions.get(app.add_choice).map(|(_, l)| l.clone()).unwrap_or_default();
            egui::ComboBox::from_id_salt("add-shortcut-action").width(300.0).selected_text(current).show_ui(ui, |ui| {
                for (k, (_, label)) in actions.iter().enumerate() {
                    ui.selectable_value(&mut app.add_choice, k, label);
                }
            });
            if ui.add_enabled(app.recording.is_none(), egui::Button::new("Add and record")).clicked() {
                edit = Some(Edit::Add);
            }
        });

        ui.add_space(24.0);
        ui.horizontal(|ui| {
            if ui.button("Reset to defaults").clicked() {
                edit = Some(Edit::Reset);
            }
            ui.label(RichText::new("Your shortcuts are saved with your settings on this computer.").size(12.0).color(pal.text_faint));
        });
    }

    match edit {
        Some(Edit::Record(i)) => app.recording = Some(Recording { index: i, chords: Vec::new(), last: now, is_new: false }),
        Some(Edit::Clear(i)) => {
            app.keymap.bindings[i].keys.clear();
            app.save_keymap();
        }
        Some(Edit::Remove(i)) => {
            app.keymap.bindings.remove(i);
            app.save_keymap();
        }
        Some(Edit::Add) => {
            if let Some((command, _)) = bindable_commands().into_iter().nth(app.add_choice) {
                app.keymap.bindings.push(Binding { command, keys: Vec::new() });
                let index = app.keymap.bindings.len() - 1;
                app.recording = Some(Recording { index, chords: Vec::new(), last: now, is_new: true });
            }
        }
        Some(Edit::Reset) => {
            app.reset_keymap();
            app.notify("Shortcuts reset to the defaults", now);
        }
        None => {}
    }
}
