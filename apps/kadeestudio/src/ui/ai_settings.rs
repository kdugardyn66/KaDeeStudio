//! Settings › AI providers and AI models. Everything is edited on a draft
//! (`KadeeApp::ai_ui.draft`) that Done saves to `ai.toml` and Cancel drops.

use super::ai_widgets::{PickEntry, chip, kind_square, model_entries, paint_kind, picker, provider_model_entries};
use super::settings::heading;
use crate::ai_state::{AiScreen, ModelForm, TestState, Tone, add_model, by_region, model_from_form, remove_model, remove_provider, status};
use crate::app::KadeeApp;
use egui::{Align, Align2, Color32, CornerRadius, FontId, Margin, RichText, Sense, Stroke, Ui, pos2, vec2};
use kadee_ai::{AiConfig, AiHub, Category, KeySource, ModelRef, Protocol, ProviderConfig, ProviderKind, Region, Scope};
use kadee_ui::{Palette, mix, primary_button};
use std::time::Duration;

const GOOD: Color32 = Color32::from_rgb(0x1E, 0x8E, 0x3E);

fn tone_colors(t: Tone, pal: &Palette) -> (Color32, Color32) {
    let fg = match t {
        Tone::Good => GOOD,
        Tone::Bad => pal.danger,
        Tone::Muted => pal.text_faint,
    };
    (fg, mix(fg, pal.surface, 0.86))
}

/// Settings › AI providers.
pub fn providers(app: &mut KadeeApp, ui: &mut Ui) {
    let hub = app.ai.clone();
    let st = &mut app.ai_ui;
    let mut draft = st.draft.take().unwrap_or_else(|| hub.config());
    if st.poll_tests() || st.any_running() {
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }
    let mut screen = std::mem::take(&mut st.screen);
    if matches!(screen, AiScreen::AddModel(_)) {
        screen = AiScreen::List;
    }
    ui.set_max_width(700.0);
    match screen.clone() {
        AiScreen::List => list(ui, &hub, &mut draft, &mut screen),
        AiScreen::Edit(id) => edit(ui, &hub, st, &mut draft, &id, &mut screen),
        AiScreen::AddKind => add_kind(ui, &draft, &mut screen),
        AiScreen::AddForm(p) => add_form(ui, &hub, st, &mut draft, *p, &mut screen),
        AiScreen::AddModel(_) => {}
    }
    st.screen = screen;
    st.draft = Some(draft);
}

fn back(ui: &mut Ui, label: &str) -> bool {
    let pal = Palette::of(ui);
    let r = ui.add(egui::Label::new(RichText::new(format!("‹  {label}")).size(13.5).color(pal.accent)).sense(Sense::click()));
    ui.add_space(10.0);
    r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

fn list(ui: &mut Ui, hub: &AiHub, draft: &mut AiConfig, screen: &mut AiScreen) {
    let pal = Palette::of(ui);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width() - 150.0);
            heading(ui, "AI providers", "Connect the services and local servers KaDee AI can use.");
        });
        ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
            if primary_button(ui, "Add provider").clicked() {
                *screen = AiScreen::AddKind;
            }
        });
    });
    let _ = hub;
    for region in Region::ALL {
        let rows: Vec<ProviderConfig> = by_region(&draft.providers).into_iter().filter(|p| p.region() == region).cloned().collect();
        if rows.is_empty() {
            continue;
        }
        ui.add_space(6.0);
        ui.label(RichText::new(region.label().to_uppercase()).size(11.0).color(pal.text_faint));
        ui.add_space(4.0);
        for p in rows {
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 62.0), Sense::click());
            let painter = ui.painter();
            painter.rect(
                r,
                CornerRadius::same(10),
                if resp.hovered() { pal.surface_hover } else { pal.surface },
                Stroke::new(1.0, if resp.hovered() { pal.accent } else { pal.border }),
                egui::StrokeKind::Inside,
            );
            paint_kind(painter, egui::Rect::from_min_size(r.min + vec2(14.0, 14.0), vec2(34.0, 34.0)), p.kind);
            painter.text(pos2(r.min.x + 62.0, r.min.y + 24.0), Align2::LEFT_CENTER, p.display_name(), FontId::proportional(14.5), pal.text);
            let sub = match p.kind {
                ProviderKind::Custom | ProviderKind::OpenAiCompatible => p.base_url(),
                k => k.blurb().to_string(),
            };
            painter.text(
                pos2(r.min.x + 62.0, r.min.y + 43.0),
                Align2::LEFT_CENTER,
                super::truncate(&sub, 62),
                FontId::proportional(12.0),
                pal.text_muted,
            );
            let (text, tone) = status(&p);
            let (fg, bg) = tone_colors(tone, &pal);
            let g = painter.layout_no_wrap(text, FontId::proportional(11.5), fg);
            let cr = egui::Rect::from_min_size(pos2(r.max.x - g.size().x - 30.0, r.center().y - 11.0), vec2(g.size().x + 16.0, 22.0));
            painter.rect_filled(cr, CornerRadius::same(11), bg);
            painter.galley(cr.center() - g.size() / 2.0, g, fg);
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *screen = AiScreen::Edit(p.id.clone());
            }
            ui.add_space(6.0);
        }
    }
}

fn add_kind(ui: &mut Ui, draft: &AiConfig, screen: &mut AiScreen) {
    let pal = Palette::of(ui);
    if back(ui, "AI providers") {
        *screen = AiScreen::List;
    }
    heading(ui, "Add a provider", "Choose the service, then enter its key or endpoint.");
    for region in Region::ALL {
        ui.add_space(6.0);
        ui.label(RichText::new(region.label().to_uppercase()).size(11.0).color(pal.text_faint));
        ui.add_space(4.0);
        for kind in ProviderKind::in_region(region) {
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
            let p = ui.painter();
            p.rect(
                r,
                CornerRadius::same(9),
                if resp.hovered() { pal.surface_hover } else { pal.surface },
                Stroke::new(1.0, pal.border),
                egui::StrokeKind::Inside,
            );
            paint_kind(p, egui::Rect::from_min_size(r.min + vec2(12.0, 11.0), vec2(30.0, 30.0)), kind);
            p.text(
                pos2(r.min.x + 54.0, r.min.y + 19.0),
                Align2::LEFT_CENTER,
                crate::ai_state::kind_title(kind),
                FontId::proportional(14.0),
                pal.text,
            );
            p.text(
                pos2(r.min.x + 54.0, r.min.y + 36.0),
                Align2::LEFT_CENTER,
                super::truncate(kind.blurb(), 80),
                FontId::proportional(12.0),
                pal.text_muted,
            );
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                let mut p = ProviderConfig::new(kind);
                p.id = draft.free_id(kind.default_id());
                *screen = AiScreen::AddForm(Box::new(p));
            }
            ui.add_space(5.0);
        }
    }
}

fn add_form(
    ui: &mut Ui,
    hub: &AiHub,
    st: &mut crate::ai_state::AiSettingsUi,
    draft: &mut AiConfig,
    mut p: ProviderConfig,
    screen: &mut AiScreen,
) {
    if back(ui, "Choose another provider") {
        *screen = AiScreen::AddKind;
        return;
    }
    heading(ui, &format!("Add {}", p.kind.label()), "");
    form(ui, hub, st, &mut p);
    let mut add = false;
    ui.horizontal(|ui| {
        if primary_button(ui, "Add provider").clicked() {
            add = true;
        }
        if ui.button("Cancel").clicked() {
            *screen = AiScreen::List;
        }
    });
    if add {
        let id = p.id.clone();
        draft.providers.push(p);
        *screen = AiScreen::Edit(id);
    } else if matches!(screen, AiScreen::AddKind) {
    } else if !matches!(screen, AiScreen::List) {
        *screen = AiScreen::AddForm(Box::new(p));
    }
}

fn edit(ui: &mut Ui, hub: &AiHub, st: &mut crate::ai_state::AiSettingsUi, draft: &mut AiConfig, id: &str, screen: &mut AiScreen) {
    let Some(idx) = draft.providers.iter().position(|p| p.id == id) else {
        *screen = AiScreen::List;
        return;
    };
    if back(ui, "AI providers") {
        *screen = AiScreen::List;
        return;
    }
    let mut p = draft.providers[idx].clone();
    heading(ui, &p.display_name(), "");
    // The draft being edited decides what the pickers offer.
    st.draft = Some(draft.clone());
    form(ui, hub, st, &mut p);
    ui.add_space(4.0);
    let pal = Palette::of(ui);
    let mut remove = false;
    if ui.add(egui::Button::new(RichText::new("Remove provider").color(pal.danger))).clicked() {
        remove = true;
    }
    if remove {
        remove_provider(draft, id);
        *screen = AiScreen::List;
    } else {
        draft.providers[idx] = p;
    }
}

/// The fields of one provider, with Test connection.
fn form(ui: &mut Ui, hub: &AiHub, st: &mut crate::ai_state::AiSettingsUi, p: &mut ProviderConfig) {
    let pal = Palette::of(ui);
    ui.horizontal(|ui| {
        kind_square(ui, p.kind, 34.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(RichText::new(p.kind.vendor()).size(14.0).strong().color(pal.text));
            let (text, tone) = status(p);
            let (fg, bg) = tone_colors(tone, &pal);
            chip(ui, &text, fg, bg);
        });
    });
    ui.add_space(8.0);
    ui.label(RichText::new(p.kind.help()).size(13.0).color(pal.text_muted));
    ui.add_space(14.0);

    ui.checkbox(&mut p.enabled, "Use this provider");
    ui.add_space(12.0);

    if p.kind == ProviderKind::Custom {
        label(ui, "Name");
        let mut name = p.name.clone().unwrap_or_default();
        ui.add(
            egui::TextEdit::singleline(&mut name).hint_text("My provider").desired_width(f32::INFINITY).margin(Margin::symmetric(10, 7)),
        );
        p.name = (!name.trim().is_empty()).then_some(name);
        ui.add_space(12.0);
    }

    if p.kind.endpoint_editable() {
        label(ui, "Endpoint");
        let mut url = p.base_url.clone().unwrap_or_default();
        ui.add(
            egui::TextEdit::singleline(&mut url)
                .hint_text(p.kind.endpoint_hint())
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(10, 7)),
        );
        p.base_url = (!url.trim().is_empty()).then_some(url);
        ui.add_space(12.0);
    }

    if p.kind == ProviderKind::Custom {
        label(ui, "API");
        let opts: Vec<PickEntry> = Protocol::CHAT.iter().map(|x| PickEntry::plain(x.label(), x.label(), p.protocol() == *x)).collect();
        if let Some(k) = picker(ui, "protocol", p.protocol().label(), 280.0, &opts) {
            p.protocol = Protocol::CHAT.into_iter().find(|x| x.label() == k);
        }
        ui.add_space(12.0);
        label(ui, "Region");
        let opts: Vec<PickEntry> = Region::ALL.iter().map(|r| PickEntry::plain(r.short(), r.label(), p.region() == *r)).collect();
        if let Some(k) = picker(ui, "region", p.region().label(), 280.0, &opts) {
            p.region = Region::ALL.into_iter().find(|r| r.short() == k);
        }
        ui.add_space(12.0);
    }

    if p.kind.needs_key() || matches!(p.kind, ProviderKind::OpenAiCompatible | ProviderKind::Custom) {
        label(ui, "API key");
        let mut key = p.api_key.clone().unwrap_or_default();
        ui.add(
            egui::TextEdit::singleline(&mut key)
                .password(true)
                .hint_text(if p.kind.needs_key() { "Paste your key" } else { "Only if the server needs one" })
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(10, 7)),
        );
        p.api_key = (!key.trim().is_empty()).then_some(key);
        let note = match p.key_source() {
            KeySource::Stored => "Stored in ai.toml in your settings folder (plain text).".to_string(),
            KeySource::Env(v) => format!("Using ${v} from the environment."),
            KeySource::None => String::new(),
        };
        if !note.is_empty() {
            ui.label(RichText::new(note).size(12.0).color(pal.text_faint));
        }
        ui.add_space(12.0);
    }

    if p.service() == kadee_ai::Service::Chat {
        label(ui, "Default model");
        let entries = provider_model_entries(hub, &st.tests, p);
        let current = match p.default_model.as_deref() {
            Some(m) => hub.catalog().info(p.kind, m).name,
            None => "Automatic".into(),
        };
        if let Some(k) = picker(ui, "provider-default", &current, 340.0, &entries) {
            p.default_model = (!k.is_empty()).then_some(k);
        }
        ui.add_space(12.0);
        if p.kind.lists_own_models() {
            label(ui, "Models");
            let mut text = p.models.join(", ");
            ui.add(
                egui::TextEdit::singleline(&mut text)
                    .hint_text("model-one, model-two  (Test connection lists what the server has)")
                    .desired_width(f32::INFINITY)
                    .margin(Margin::symmetric(10, 7)),
            );
            p.models = text.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            ui.add_space(12.0);
        }
    }

    ui.horizontal(|ui| {
        let running = matches!(st.tests.get(&p.id), Some(TestState::Running(_)));
        if ui.add_enabled(!running, egui::Button::new("Test connection")).clicked() {
            st.start_test(p);
        }
        match st.tests.get(&p.id) {
            Some(TestState::Running(_)) => {
                ui.spinner();
                ui.label(RichText::new("Testing…").color(pal.text_muted));
            }
            Some(TestState::Ok(list)) => {
                ui.label(RichText::new(format!("Connected · {} model{}", list.len(), if list.len() == 1 { "" } else { "s" })).color(GOOD));
            }
            Some(TestState::Err(e)) => {
                ui.add(egui::Label::new(RichText::new(super::truncate(e, 160)).color(pal.danger)).wrap());
            }
            None => {}
        }
    });
    if let Some(n) = st.notes.get(&p.id) {
        ui.label(RichText::new(n).size(12.5).color(pal.text_muted));
    }
    ui.add_space(14.0);
}

fn label(ui: &mut Ui, text: &str) {
    let pal = Palette::of(ui);
    ui.label(RichText::new(text).size(13.5).strong().color(pal.text));
    ui.add_space(3.0);
}

/// Settings › AI models.
pub fn models(app: &mut KadeeApp, ui: &mut Ui) {
    let hub = app.ai.clone();
    let st = &mut app.ai_ui;
    let mut draft = st.draft.take().unwrap_or_else(|| hub.config());
    st.poll_tests();
    let mut screen = std::mem::take(&mut st.screen);
    if !matches!(screen, AiScreen::AddModel(_)) {
        screen = AiScreen::List;
    }
    ui.set_max_width(700.0);
    let pal = Palette::of(ui);
    match screen.clone() {
        AiScreen::AddModel(form) => add_model_form(ui, &mut draft, *form, &mut screen),
        _ => {
            heading(ui, "AI models", "Which model KaDee AI uses, in general and for each part of KadeeStudio.");
            label(ui, "Default model");
            let value = draft.default_model.clone();
            let entries = model_entries(&hub, &draft, &st.tests, value.as_ref(), Some("Automatic (first ready provider)"));
            let current = value
                .as_ref()
                .map(|m| crate::ai_state::model_summary(&hub, &draft, m))
                .unwrap_or_else(|| "Automatic (first ready provider)".into());
            if let Some(k) = picker(ui, "default-model", &current, 420.0, &entries) {
                draft.default_model = ModelRef::parse(&k);
            }
            ui.add_space(4.0);
            ui.label(RichText::new("Used wherever a part of KadeeStudio has no model of its own.").size(12.5).color(pal.text_faint));
            ui.add_space(20.0);
            ui.label(RichText::new("PER PART OF KADEESTUDIO").size(11.0).color(pal.text_faint));
            ui.add_space(6.0);
            for scope in Scope::ALL {
                let value = draft.models.get(scope.id()).cloned();
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(vec2(170.0, 34.0), egui::Layout::left_to_right(Align::Center), |ui| {
                        ui.label(RichText::new(scope.label()).size(14.0).color(pal.text));
                    });
                    let entries = model_entries(&hub, &draft, &st.tests, value.as_ref(), Some("Use the default"));
                    let current =
                        value.as_ref().map(|m| crate::ai_state::model_summary(&hub, &draft, m)).unwrap_or_else(|| "Use the default".into());
                    if let Some(k) = picker(ui, scope.id(), &current, 420.0, &entries) {
                        match ModelRef::parse(&k) {
                            Some(m) => draft.models.insert(scope.id().to_string(), m),
                            None => draft.models.remove(scope.id()),
                        };
                    }
                });
                ui.add_space(4.0);
            }
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("MODELS YOU ADDED").size(11.0).color(pal.text_faint));
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Add a model").clicked() {
                        let provider = draft
                            .providers
                            .iter()
                            .find(|p| p.service() == kadee_ai::Service::Chat)
                            .map(|p| p.id.clone())
                            .unwrap_or_default();
                        screen = AiScreen::AddModel(Box::new(ModelForm { provider, tools: true, ..ModelForm::default() }));
                    }
                });
            });
            ui.add_space(6.0);
            let added: Vec<(String, String, String)> =
                draft.providers.iter().flat_map(|p| p.models.iter().map(move |m| (p.id.clone(), p.display_name(), m.clone()))).collect();
            if added.is_empty() {
                ui.label(
                    RichText::new("None yet. Add a model the catalog doesn't list, with its prices and limits.")
                        .size(13.0)
                        .color(pal.text_muted),
                );
            }
            for (pid, pname, m) in added {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&m).size(14.0).color(pal.text));
                    ui.label(RichText::new(pname).size(12.5).color(pal.text_faint));
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("Remove").clicked() {
                            remove_model(&mut draft, &pid, &m);
                        }
                    });
                });
            }
        }
    }
    st.screen = screen;
    st.draft = Some(draft);
}

fn add_model_form(ui: &mut Ui, draft: &mut AiConfig, mut f: ModelForm, screen: &mut AiScreen) {
    let pal = Palette::of(ui);
    if back(ui, "AI models") {
        *screen = AiScreen::List;
        return;
    }
    heading(ui, "Add a model", "For a model the catalog doesn't list yet, or to correct its prices.");
    label(ui, "Provider");
    let entries: Vec<PickEntry> = draft
        .providers
        .iter()
        .filter(|p| p.service() == kadee_ai::Service::Chat)
        .map(|p| PickEntry::plain(&p.id, p.display_name(), p.id == f.provider))
        .collect();
    let current = draft.provider(&f.provider).map(|p| p.display_name()).unwrap_or_else(|| "Choose a provider".into());
    if let Some(k) = picker(ui, "add-model-provider", &current, 340.0, &entries) {
        f.provider = k;
    }
    ui.add_space(12.0);
    let line = |ui: &mut Ui, name: &str, hint: &str, value: &mut String| {
        label(ui, name);
        ui.add(egui::TextEdit::singleline(value).hint_text(hint).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 7)));
        ui.add_space(10.0);
    };
    line(ui, "Model id", "the id the provider uses, e.g. llama3.2:3b", &mut f.id);
    line(ui, "Name (optional)", "shown in the pickers", &mut f.name);
    line(ui, "Context window (tokens, optional)", "128000", &mut f.context);
    line(ui, "Max output (tokens, optional)", "8192", &mut f.max_output);
    line(ui, "Input price per 1M tokens (USD, optional)", "3", &mut f.input);
    line(ui, "Output price per 1M tokens (USD, optional)", "15", &mut f.output);
    ui.horizontal(|ui| {
        ui.checkbox(&mut f.vision, "Sees images");
        ui.checkbox(&mut f.pdf, "Reads PDFs");
        ui.checkbox(&mut f.tools, "Uses tools");
    });
    ui.add_space(10.0);
    label(ui, &format!("Good at (at most {})", Category::MAX_PER_MODEL));
    ui.horizontal_wrapped(|ui| {
        for c in Category::ALL {
            let on = f.categories.contains(&c);
            if ui.selectable_label(on, c.label()).clicked() {
                if on {
                    f.categories.retain(|x| *x != c);
                } else {
                    f.categories.push(c);
                }
            }
        }
    });
    ui.add_space(12.0);
    if let Some(e) = &f.error {
        ui.label(RichText::new(e).color(pal.danger));
        ui.add_space(6.0);
    }
    let mut done = false;
    ui.horizontal(|ui| {
        if primary_button(ui, "Add model").clicked() {
            let kind = draft.provider(&f.provider).map(|p| p.kind);
            match model_from_form(&f, kind) {
                Ok(info) => {
                    add_model(draft, &f.provider, info);
                    done = true;
                }
                Err(e) => f.error = Some(e),
            }
        }
        if ui.button("Cancel").clicked() {
            done = true;
        }
    });
    *screen = if done { AiScreen::List } else { AiScreen::AddModel(Box::new(f)) };
}
