//! KaDee AI chat: the full page (sidebar › KaDee AI) and the compact pane beside an open file.
//! Both draw a [`kadee_ai::ChatState`] kept in `KadeeApp::chats`, so a conversation survives
//! switching tabs. Answers stream in; Stop cancels; the footer shows tokens and cost.

use super::ai_widgets::{PickEntry, model_entries, picker};
use crate::ai_state::model_summary;
use crate::app::KadeeApp;
use crate::icons::{self, Icon};
use egui::{Align, CornerRadius, FontId, Key, Margin, Modifiers, RichText, Sense, Stroke, Ui, Vec2, pos2, vec2};
use kadee_ai::{AppKind, ModelRef, Speaker, starred_context};
use kadee_commands::{Command, SettingsSection};
use kadee_core::{DocGroup, DocKind};
use kadee_ui::{Palette, primary_button};
use std::path::{Path, PathBuf};

const SUGGESTIONS: [(AppKind, &str); 4] = [
    (AppKind::Report, "Draft a one-page project report"),
    (AppKind::Sheet, "Build a monthly household budget"),
    (AppKind::Slides, "Outline a 10-slide product pitch"),
    (AppKind::Markdown, "Write release notes from my changelog"),
];

/// The module a file belongs to for KaDee AI.
pub fn app_kind_of(path: &Path) -> Option<AppKind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "docx" | "doc" | "odt" | "rtf" => return Some(AppKind::Word),
        "xlsx" | "xls" | "ods" | "csv" => return Some(AppKind::Sheet),
        "pptx" | "ppt" | "odp" => return Some(AppKind::Slides),
        _ => {}
    }
    match DocKind::from_path(path)?.group {
        DocGroup::Markdown => Some(AppKind::Markdown),
        DocGroup::Pdf => Some(AppKind::Pdf),
        DocGroup::Dashboard => Some(AppKind::Dashboard),
        DocGroup::Report => Some(AppKind::Report),
        _ => None,
    }
}

/// The pane on Home: the general KaDee AI chat (starred files can be its context).
pub fn home_pane(app: &mut KadeeApp, ui: &mut Ui) {
    chat(app, ui, "", None, true);
}

/// The pane beside a file tab.
pub fn pane(app: &mut KadeeApp, ui: &mut Ui, path: &Path) {
    chat(app, ui, &path.display().to_string(), Some(path.to_path_buf()), true);
}

fn chat(app: &mut KadeeApp, ui: &mut Ui, key: &str, file: Option<PathBuf>, compact: bool) {
    let pal = Palette::of(ui);
    // A file's pane is for that file's module.
    if let Some(f) = &file {
        let kind = app_kind_of(f);
        let c = app.chat(key);
        if c.messages.is_empty() && !c.running() {
            c.target = kind;
        }
    }
    let hub = app.ai.clone();
    let config = hub.config();
    let (empty, running, status, picked, target) = {
        let c = app.chat(key);
        (c.messages.is_empty(), c.running(), c.status.clone(), c.model.clone(), c.target)
    };
    let scope = target.map(kadee_ai::Scope::App).unwrap_or(kadee_ai::Scope::Home);
    let model = picked.clone().or_else(|| hub.model_for(scope));
    let model_label = match &model {
        Some(m) => model_summary(&hub, &config, m),
        None => "No model ready".to_string(),
    };
    let side: i8 = if compact { 12 } else { 28 };
    let mut set_draft: Option<String> = None;
    let mut open_providers = false;
    let mut do_send = false;
    let mut do_stop = false;
    let mut do_clear = false;

    egui::Panel::bottom(egui::Id::unique("kadee-ai-composer").with(key))
        .show_separator_line(false)
        .frame(egui::Frame::new().fill(pal.canvas).inner_margin(Margin { left: side, right: side, top: 4, bottom: 16 }))
        .show(ui, |ui| {
            centered(ui, compact, |ui| {
                egui::Frame::new()
                    .fill(pal.surface)
                    .stroke(Stroke::new(1.0, pal.border_strong))
                    .corner_radius(CornerRadius::same(14))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        let id = egui::Id::unique("kadee-ai-input").with(key);
                        let focused = ui.ctx().memory(|m| m.has_focus(id));
                        // Enter sends, Shift+Enter adds a line: take Enter before the editor sees it.
                        if focused && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                            do_send = true;
                        }
                        let c = app.chat(key);
                        ui.add(
                            egui::TextEdit::multiline(&mut c.draft)
                                .id(id)
                                .hint_text(if compact {
                                    "Ask anything…"
                                } else {
                                    "Ask anything…  (Enter to send, Shift+Enter for a new line)"
                                })
                                .desired_rows(if compact { 2 } else { 3 })
                                .desired_width(f32::INFINITY)
                                .frame(egui::Frame::NONE)
                                .margin(Margin::symmetric(6, 4)),
                        );
                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
                            if !compact {
                                let mut labels = vec![PickEntry::plain("", "General chat", target.is_none())];
                                labels.extend(AppKind::ALL.into_iter().map(|k| PickEntry::plain(k.id(), k.name(), target == Some(k))));
                                let l = target.map(|k| format!("For {}", k.name())).unwrap_or_else(|| "General chat".into());
                                if let Some(k) = picker(ui, "for", &l, 190.0, &labels) {
                                    app.chat(key).target = AppKind::ALL.into_iter().find(|a| a.id() == k);
                                }
                            }
                            let entries =
                                model_entries(&hub, &config, &Default::default(), picked.as_ref(), Some("Default for this module"));
                            // the Home pane also has the Starred files button: a narrower model picker
                            let w = match (compact, key.is_empty()) {
                                (true, true) => 170.0,
                                (true, false) => 220.0,
                                _ => 330.0,
                            };
                            if let Some(k) = picker(ui, "model", &model_label, w, &entries) {
                                app.chat(key).model = ModelRef::parse(&k);
                            }
                            if key.is_empty() {
                                let n = app.state.library.starred.len();
                                let on = app.chat(key).use_starred;
                                let b = egui::Button::new(RichText::new(format!("Starred files ({n})")).size(13.0).color(if on {
                                    pal.accent
                                } else {
                                    pal.text_muted
                                }))
                                .fill(if on { pal.accent_soft } else { pal.surface })
                                .stroke(Stroke::new(1.0, if on { pal.accent } else { pal.border }))
                                .corner_radius(CornerRadius::same(8))
                                .min_size(vec2(0.0, 34.0));
                                if ui.add(b).on_hover_text("Answer from your starred files").clicked() {
                                    app.chat(key).use_starred = !on;
                                }
                            }
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                if running {
                                    if round_button(ui, Icon::Stop, "Stop", true) {
                                        do_stop = true;
                                    }
                                } else {
                                    let has_text = !app.chat(key).draft.trim().is_empty();
                                    ui.add_enabled_ui(has_text, |ui| {
                                        if round_button(ui, Icon::Send, "Send", false) {
                                            do_send = true;
                                        }
                                    });
                                }
                                if !empty {
                                    let b = egui::Button::new(RichText::new("New chat").size(13.0).color(pal.text_muted))
                                        .fill(egui::Color32::TRANSPARENT)
                                        .min_size(vec2(0.0, 34.0));
                                    if ui.add(b).clicked() {
                                        do_clear = true;
                                    }
                                }
                            });
                        });
                    });
            });
        });

    egui::CentralPanel::default().frame(egui::Frame::new().fill(pal.canvas)).show(ui, |ui| {
        egui::ScrollArea::vertical().id_salt(("kadee-ai-scroll", key)).auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
            ui.add_space(18.0);
            centered(ui, compact, |ui| {
                if empty {
                    welcome(ui, compact, model.is_none(), &mut set_draft, &mut open_providers, &pal, file.as_deref());
                } else {
                    let n = app.chat(key).messages.len();
                    for i in 0..n {
                        bubble(app, ui, key, i, running && i + 1 == n);
                    }
                    if let Some(s) = status {
                        ui.label(RichText::new(s).size(12.5).color(pal.text_faint));
                    }
                }
            });
            ui.add_space(12.0);
        });
    });

    if let Some(text) = set_draft {
        let c = app.chat(key);
        c.draft = text;
    }
    if open_providers {
        app.send(Command::OpenSettings(SettingsSection::AiProviders));
    }
    if do_clear {
        app.chat(key).clear();
    }
    if do_stop {
        app.chat(key).stop();
    }
    if do_send {
        let context = context_for(app, key, file.as_deref());
        let hub = app.ai.clone();
        app.chat(key).send(&hub, context);
        ui.ctx().request_repaint();
    }
}

/// Run `add` in a column centred in the available width: at most 780 wide on the page, the
/// whole width in the pane. The composer and the messages use it, so they line up.
fn centered(ui: &mut Ui, compact: bool, add: impl FnOnce(&mut Ui)) {
    let r = ui.available_rect_before_wrap();
    let w = if compact { r.width() } else { r.width().min(780.0) };
    let x = r.center().x - w / 2.0;
    let col = egui::Rect::from_min_max(pos2(x, r.min.y), pos2(x + w, r.max.y));
    let used = ui.scope_builder(egui::UiBuilder::new().max_rect(col), |ui| {
        ui.set_width(w);
        add(ui);
    });
    ui.allocate_rect(used.response.rect, Sense::hover());
}

/// What the model gets besides the question: the starred files (page) or the open file (pane).
fn context_for(app: &mut KadeeApp, key: &str, file: Option<&Path>) -> Option<String> {
    if let Some(f) = file {
        // A Markdown tab: the editor's current text, unsaved edits included.
        #[cfg(unix)]
        if let Some((name, text)) = app.markdown.text(f) {
            return Some(kadee_ai::text_context(&name, &text, kadee_ai::CONTEXT_BUDGET));
        }
        return kadee_ai::file_context(f, kadee_ai::CONTEXT_BUDGET);
    }
    if app.chat(key).use_starred {
        let paths: Vec<PathBuf> = app.state.library.starred.clone();
        return starred_context(&paths, kadee_ai::CONTEXT_BUDGET);
    }
    None
}

fn round_button(ui: &mut Ui, icon: Icon, tip: &str, dark: bool) -> bool {
    let pal = Palette::of(ui);
    let (r, resp) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::click());
    let enabled = ui.is_enabled();
    let bg = if dark {
        pal.text
    } else if enabled {
        pal.accent
    } else {
        pal.border
    };
    let bg = if resp.hovered() && enabled { kadee_ui::mix(bg, pal.canvas, 0.15) } else { bg };
    ui.painter().rect_filled(r, CornerRadius::same(9), bg);
    icons::paint(ui.painter(), r.shrink(9.0), icon, if dark { pal.canvas } else { pal.on_accent });
    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

fn welcome(
    ui: &mut Ui,
    compact: bool,
    no_model: bool,
    set_draft: &mut Option<String>,
    open_providers: &mut bool,
    pal: &Palette,
    file: Option<&Path>,
) {
    ui.add_space(if compact { 12.0 } else { 56.0 });
    ui.vertical_centered(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::splat(52.0), Sense::hover());
        ui.painter().rect_filled(r, CornerRadius::same(15), pal.accent_soft);
        icons::paint(ui.painter(), r.shrink(13.0), Icon::Sparkles, pal.accent);
        ui.add_space(10.0);
        ui.label(
            RichText::new(if compact { "How can I help?" } else { "KaDee AI" })
                .size(if compact { 18.0 } else { 26.0 })
                .strong()
                .color(pal.text),
        );
        ui.add_space(2.0);
        let sub = match (compact, file) {
            (true, Some(f)) => format!("Ask about {}, or describe what to write.", crate::app::file_name(f)),
            (true, None) => "Ask a question or describe what to write.".to_string(),
            _ => "Ask a question, or describe a report, spreadsheet or deck and KaDee AI drafts it.".to_string(),
        };
        ui.label(RichText::new(sub).size(14.0).color(pal.text_muted));
    });
    if no_model {
        ui.add_space(22.0);
        egui::Frame::new()
            .fill(pal.surface)
            .stroke(Stroke::new(1.0, pal.border))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("No AI provider is ready yet").strong().size(15.0).color(pal.text));
                ui.add_space(2.0);
                ui.label(
                    RichText::new(
                        "Add an API key for Anthropic, OpenAI, Gemini or another provider — or connect a local Ollama or LM Studio server.",
                    )
                    .size(13.0)
                    .color(pal.text_muted),
                );
                ui.add_space(10.0);
                if primary_button(ui, "Set up AI providers").clicked() {
                    *open_providers = true;
                }
            });
    }
    if !compact {
        ui.add_space(22.0);
        let gap = 10.0;
        let w = (ui.available_width() - gap) / 2.0;
        for pair in SUGGESTIONS.chunks(2) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (kind, text) in pair {
                    let (r, resp) = ui.allocate_exact_size(vec2(w, 52.0), Sense::click());
                    let p = ui.painter();
                    p.rect(
                        r,
                        CornerRadius::same(10),
                        if resp.hovered() { pal.surface_hover } else { pal.surface },
                        Stroke::new(1.0, pal.border),
                        egui::StrokeKind::Inside,
                    );
                    p.text(pos2(r.min.x + 16.0, r.center().y), egui::Align2::LEFT_CENTER, *text, FontId::proportional(14.0), pal.text);
                    if resp.on_hover_text(format!("For {}", kind.name())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        *set_draft = Some((*text).to_string());
                    }
                }
            });
            ui.add_space(gap - 4.0);
        }
    }
}

fn bubble(app: &mut KadeeApp, ui: &mut Ui, key: &str, i: usize, streaming: bool) {
    let pal = Palette::of(ui);
    let m = app.chat(key).messages[i].clone();
    match m.speaker {
        Speaker::User => {
            ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                egui::Frame::new().fill(pal.accent_soft).corner_radius(CornerRadius::same(14)).inner_margin(Margin::symmetric(14, 9)).show(
                    ui,
                    |ui| {
                        ui.set_max_width((ui.available_width() * 0.85).max(120.0));
                        ui.add(egui::Label::new(RichText::new(&m.text).size(14.0).color(pal.text)).selectable(true));
                    },
                );
            });
            ui.add_space(14.0);
        }
        Speaker::Assistant => {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                let (r, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                icons::paint(ui.painter(), r, Icon::Sparkles, pal.accent);
                let who = m.model.as_ref().map(|r| r.model.clone()).unwrap_or_else(|| "KaDee AI".into());
                ui.label(RichText::new(who).size(12.0).color(pal.text_faint));
            });
            ui.add_space(2.0);
            if streaming && m.text.is_empty() {
                ui.label(RichText::new("Thinking…").color(pal.text_faint));
            } else if m.error {
                egui::Frame::new()
                    .fill(kadee_ui::mix(pal.danger, pal.surface, 0.86))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(RichText::new(&m.text).color(pal.danger)).selectable(true));
                    });
            } else {
                ui.add(egui::Label::new(RichText::new(&m.text).size(14.0).color(pal.text)).selectable(true));
            }
            if !streaming {
                ui.horizontal(|ui| {
                    if let Some(f) = &m.footer {
                        ui.label(RichText::new(f).size(11.5).color(pal.text_faint));
                    }
                    if !m.error && !m.text.is_empty() && ui.small_button("Copy").clicked() {
                        ui.ctx().copy_text(m.text.clone());
                        let t = ui.input(|i| i.time);
                        app.notify("Answer copied", t);
                    }
                    // The pane of a Markdown tab can put the answer into the document.
                    #[cfg(unix)]
                    if !m.error && !m.text.is_empty() {
                        let path = std::path::PathBuf::from(key);
                        if app.markdown.has(&path)
                            && ui.small_button("Insert into document").on_hover_text("Insert at the cursor in the editor").clicked()
                        {
                            app.markdown.insert(&path, m.text.clone());
                        }
                    }
                });
            }
            ui.add_space(16.0);
        }
    }
}
