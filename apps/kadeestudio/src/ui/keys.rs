//! Keyboard: running shortcuts (sequences included), recording new ones,
//! the "waiting for the next key" hint and the New document chooser.

use crate::app::{KadeeApp, OS};
use egui::{Align2, Color32, CornerRadius, FontId, Margin, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use kadee_commands::{Chord, Command, Os, Outcome, display_sequence};
use kadee_core::{DocKind, ModuleKind, OfficeKind};
use kadee_ui::{Palette, paint_doc_badge, paint_module_badge};
use std::path::Path;
use std::time::Duration;

/// After the last key of a recording, save it when nothing else is pressed for this long.
pub const RECORD_IDLE: f64 = 1.5;
/// Longest sequence a recording accepts.
pub const MAX_CHORDS: usize = 3;

/// A shortcut being recorded on the Keyboard shortcuts page.
#[derive(Debug, Clone)]
pub struct Recording {
    /// Index into the key map's bindings.
    pub index: usize,
    pub chords: Vec<Chord>,
    /// Time of the last key (or of the start).
    pub last: f64,
    /// A binding added just for this recording: removed again on Esc.
    pub is_new: bool,
}

/// The chord for a key press, with the four modifier keys kept apart.
///
/// On the Mac, Cmd and Control are separate (`mac_cmd`, `ctrl`). On Windows
/// and Linux egui folds the Windows/Super key away, so `meta_down` comes from
/// tracking that key ourselves (see [`key_presses`]).
pub fn chord_from(key: egui::Key, m: egui::Modifiers, meta_down: bool) -> Chord {
    Chord { ctrl: m.ctrl, alt: m.alt, shift: m.shift, meta: if OS == Os::Mac { m.mac_cmd } else { meta_down }, key: key.name().to_string() }
}

/// Modifier keys on their own are not chords: they only change the next key.
fn is_modifier(key: egui::Key) -> bool {
    use egui::Key as K;
    matches!(
        key,
        K::ShiftLeft
            | K::ShiftRight
            | K::ControlLeft
            | K::ControlRight
            | K::AltLeft
            | K::AltRight
            | K::SuperLeft
            | K::SuperRight
            | K::CapsLock
            | K::Fn
    )
}

/// This frame's key presses as chords. Keeps `meta_down` up to date from the
/// Windows/Super key's own down/up events, and drops it when the window loses
/// focus (pressing the Windows key alone opens the Start menu).
fn key_presses(ctx: &egui::Context, meta_down: &mut bool) -> Vec<(egui::Key, Chord)> {
    ctx.input(|i| {
        if !i.focused {
            *meta_down = false;
        }
        let mut out = Vec::new();
        for e in &i.events {
            if let egui::Event::Key { key, pressed, repeat, modifiers, .. } = e {
                if matches!(key, egui::Key::SuperLeft | egui::Key::SuperRight) {
                    *meta_down = *pressed;
                } else if *pressed && !*repeat && !is_modifier(*key) {
                    out.push((*key, chord_from(*key, *modifiers, *meta_down)));
                }
            }
        }
        out
    })
}

/// Take a key press away from the widgets (it was a shortcut).
fn swallow(ctx: &egui::Context, key: egui::Key) {
    ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { key: k, pressed: true, .. } if *k == key)));
}

/// A shortcut pressed while the Markdown editor had the keyboard: the page hands back the
/// chords KadeeStudio has shortcuts for (Cmd+W, Ctrl+Tab, Cmd+J …).
#[cfg(unix)]
pub fn forwarded(app: &mut KadeeApp, chord: Chord, now: f64) {
    if app.recording.is_some() {
        return;
    }
    match app.matcher.feed(&app.keymap, chord, now) {
        Outcome::None => {}
        Outcome::Pending => {
            if app.keymap.command_for(app.matcher.pending()) == Some(&Command::NewDocumentPicker) {
                app.picker_prefix = app.matcher.pending().to_vec();
                app.matcher.cancel();
                app.new_picker = true;
            }
        }
        Outcome::Fire(cmd) => app.send(cmd),
    }
}

/// Run every frame before the UI: turns key presses into commands.
pub fn keyboard(app: &mut KadeeApp, ctx: &egui::Context) {
    let now = ctx.input(|i| i.time);
    if app.recording.is_some() {
        record(app, ctx, now);
        return;
    }
    if let Some(cmd) = app.matcher.tick(&app.keymap, now) {
        app.send(cmd);
    }
    for (key, chord) in key_presses(ctx, &mut app.meta_down) {
        // The New document menu is open: a letter (M, P, D, R, W, G, S …) picks the type.
        if app.new_picker {
            if key == egui::Key::Escape {
                app.new_picker = false;
                swallow(ctx, key);
                continue;
            }
            let mut seq = app.picker_prefix.clone();
            seq.push(chord.clone());
            if let Some(cmd) = app.keymap.command_for(&seq).cloned() {
                swallow(ctx, key);
                app.new_picker = false;
                app.send(cmd);
                continue;
            }
        }
        if key == egui::Key::Escape && !app.matcher.pending().is_empty() {
            app.matcher.cancel();
            swallow(ctx, key);
            continue;
        }
        // Plain keys belong to the text field being typed in, unless a sequence is under way.
        if ctx.egui_wants_keyboard_input() && app.matcher.pending().is_empty() && !chord.has_modifier() {
            continue;
        }
        match app.matcher.feed(&app.keymap, chord, now) {
            Outcome::None => {}
            Outcome::Pending => {
                swallow(ctx, key);
                // Cmd+N opens its menu at once instead of waiting for the next key.
                if app.keymap.command_for(app.matcher.pending()) == Some(&Command::NewDocumentPicker) {
                    app.picker_prefix = app.matcher.pending().to_vec();
                    app.matcher.cancel();
                    app.new_picker = true;
                }
            }
            Outcome::Fire(cmd) => {
                swallow(ctx, key);
                app.send(cmd);
            }
        }
    }
    if !app.matcher.pending().is_empty() {
        ctx.request_repaint_after(Duration::from_secs_f64(app.matcher.time_left(now) + 0.02));
    }
}

/// While recording, every key press goes into the recording and nowhere else.
fn record(app: &mut KadeeApp, ctx: &egui::Context, now: f64) {
    let presses = key_presses(ctx, &mut app.meta_down);
    ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { .. } | egui::Event::Text(_))));
    let Some(rec) = app.recording.as_mut() else { return };
    let mut finish: Option<bool> = None; // Some(true) = save, Some(false) = cancel
    for (key, chord) in presses {
        let plain = !chord.has_modifier() && !chord.shift;
        match key {
            egui::Key::Escape if plain => {
                finish = Some(false);
                break;
            }
            egui::Key::Enter if plain && !rec.chords.is_empty() => {
                finish = Some(true);
                break;
            }
            egui::Key::Backspace if plain => {
                rec.chords.clear();
                rec.last = now;
            }
            _ => {
                rec.chords.push(chord);
                rec.last = now;
                if rec.chords.len() >= MAX_CHORDS {
                    finish = Some(true);
                    break;
                }
            }
        }
    }
    if finish.is_none() && !rec.chords.is_empty() && now - rec.last > RECORD_IDLE {
        finish = Some(true);
    }
    match finish {
        Some(true) => {
            let Some(r) = app.recording.take() else { return };
            if let Some(b) = app.keymap.bindings.get_mut(r.index) {
                b.keys = r.chords;
            }
            app.save_keymap();
            let msg = match app.keymap.conflict(r.index) {
                Some(j) => format!("Saved — also used by “{}”", app.keymap.bindings[j].command.label()),
                None => "Shortcut saved".to_string(),
            };
            app.notify(msg, now);
        }
        Some(false) => {
            if let Some(r) = app.recording.take()
                && r.is_new
                && r.index < app.keymap.bindings.len()
            {
                app.keymap.bindings.remove(r.index);
            }
        }
        None => ctx.request_repaint_after(Duration::from_millis(100)),
    }
}

/// The bar shown while a sequence waits for its next key.
pub fn sequence_hint(app: &KadeeApp, ui: &Ui) {
    let pending = app.matcher.pending();
    if pending.is_empty() {
        return;
    }
    let pal = Palette::of(ui);
    let nexts: Vec<String> =
        app.matcher.continuations(&app.keymap).into_iter().map(|(c, cmd)| format!("{}  {}", c.display(OS), cmd.label())).collect();
    egui::Area::new(egui::Id::unique("kadee-sequence-hint")).anchor(Align2::CENTER_BOTTOM, vec2(0.0, -24.0)).interactable(false).show(
        ui.ctx(),
        |ui| {
            egui::Frame::new()
                .fill(pal.surface)
                .stroke(Stroke::new(1.0, pal.border_strong))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(16, 10))
                .shadow(egui::epaint::Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(40) })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{} …", display_sequence(pending, OS))).strong().color(pal.accent));
                        ui.add_space(8.0);
                        ui.add(
                            egui::Label::new(RichText::new(nexts.join("   ·   ")).color(pal.text)).wrap_mode(egui::TextWrapMode::Extend),
                        );
                        ui.add_space(8.0);
                        ui.label(RichText::new("Esc cancels").size(12.0).color(pal.text_faint));
                    });
                });
        },
    );
}

/// One choice in the New document menu.
#[derive(Clone, Copy)]
enum Choice {
    Module(ModuleKind),
    Office(OfficeKind),
}

impl Choice {
    fn command(self) -> Command {
        match self {
            Choice::Module(k) => Command::NewDocument(k),
            Choice::Office(k) => Command::NewOffice(k),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Choice::Module(ModuleKind::Pdf) => "PDF",
            Choice::Module(k) => k.short(),
            Choice::Office(k) => k.short(),
        }
    }
}

const KADEE: [Choice; 4] = [
    Choice::Module(ModuleKind::Markdown),
    Choice::Module(ModuleKind::Pdf),
    Choice::Module(ModuleKind::Dashboard),
    Choice::Module(ModuleKind::Report),
];
const OFFICE: [Choice; 3] =
    [Choice::Office(OfficeKind::Word), Choice::Office(OfficeKind::Spreadsheet), Choice::Office(OfficeKind::SlideDeck)];

/// The New document menu (Cmd+N by default). Each tile shows the letter that
/// picks it; the letters are ordinary shortcuts (`Cmd+N M`), so they can be
/// changed on the Keyboard shortcuts page.
pub fn new_document_picker(app: &mut KadeeApp, ui: &Ui) {
    if !app.new_picker {
        return;
    }
    let ctx = ui.ctx().clone();
    let letter = |c: Choice| -> Option<String> {
        app.keymap.next_keys(&app.picker_prefix).into_iter().find(|(_, cmd)| **cmd == c.command()).map(|(k, _)| k.display(OS))
    };
    let letters: Vec<(Choice, Option<String>)> = KADEE.iter().chain(OFFICE.iter()).map(|&c| (c, letter(c))).collect();
    let mut chosen = None;
    let resp = egui::Modal::new(egui::Id::unique("kadee-new-document")).show(&ctx, |ui| {
        let pal = Palette::of(ui);
        ui.set_width(520.0);
        ui.label(RichText::new("New document").size(20.0).strong().color(pal.text));
        let hint = if letters.iter().any(|(_, l)| l.is_some()) {
            "Press a letter or click. It is saved in your KadeeStudio folder and opens in a new tab."
        } else {
            "It is saved in your KadeeStudio folder and opens in a new tab."
        };
        ui.label(RichText::new(hint).color(pal.text_muted));
        for (title, row) in [("KadeeStudio", &letters[..4]), ("Office — opens in WordCraft, GridCraft and DeckCraft", &letters[4..])] {
            ui.add_space(14.0);
            ui.label(RichText::new(title).size(12.0).strong().color(pal.text_faint));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                for (choice, key) in row {
                    if tile(ui, *choice, key.as_deref()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        chosen = Some(*choice);
                    }
                }
            });
        }
        ui.add_space(14.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Cancel").clicked() {
                ui.close();
            }
            ui.label(RichText::new("Esc closes").size(12.0).color(pal.text_faint));
        });
    });
    if let Some(choice) = chosen {
        app.new_picker = false;
        app.send(choice.command());
    } else if resp.should_close() {
        app.new_picker = false;
    }
}

fn tile(ui: &mut Ui, choice: Choice, key: Option<&str>) -> egui::Response {
    let pal = Palette::of(ui);
    let (rect, r) = ui.allocate_exact_size(vec2(121.0, 104.0), Sense::click());
    ui.painter().rect(
        rect,
        CornerRadius::same(10),
        if r.hovered() { pal.surface_hover } else { pal.surface },
        Stroke::new(1.0, if r.hovered() { pal.accent } else { pal.border }),
        StrokeKind::Inside,
    );
    let badge = egui::Rect::from_center_size(pos2(rect.center().x, rect.min.y + 38.0), Vec2::splat(38.0));
    match choice {
        Choice::Module(k) => paint_module_badge(ui, badge, k),
        Choice::Office(k) => {
            if let Some(kind) = DocKind::from_path(Path::new(&format!("x.{}", k.extension()))) {
                paint_doc_badge(ui, badge, &kind);
            }
        }
    }
    let p = ui.painter();
    p.text(pos2(rect.center().x, rect.min.y + 80.0), Align2::CENTER_CENTER, choice.label(), FontId::proportional(14.0), pal.text);
    if let Some(key) = key {
        // the letter as a key cap in the top-right corner
        let font = FontId::monospace(12.0);
        let w = p.layout_no_wrap(key.to_string(), font.clone(), pal.text).size().x.max(10.0) + 10.0;
        let cap = egui::Rect::from_min_size(pos2(rect.max.x - w - 8.0, rect.min.y + 8.0), vec2(w, 20.0));
        p.rect(cap, CornerRadius::same(5), pal.canvas, Stroke::new(1.0, pal.border_strong), StrokeKind::Inside);
        p.text(cap.center(), Align2::CENTER_CENTER, key, font, pal.text_muted);
    }
    r
}
