//! Small pieces shared by the KaDee AI chat and the AI settings: a drop-down with grouped
//! rows (the model pickers), status chips and the provider badge.

use crate::ai_state::{TestState, by_region, flag_emoji, kind_badge, offered, provider_fallback};
use crate::icons::{self, Icon};
use egui::{Align2, Color32, CornerRadius, FontId, Rect, RichText, Sense, Stroke, Ui, Vec2, pos2, vec2};
use kadee_ai::{AiConfig, AiHub, ModelRef, ProviderConfig, ProviderKind};
use kadee_ui::Palette;
use std::collections::HashMap;

/// One row of a picker.
pub enum PickEntry {
    Header { country: String, text: String },
    Item { key: String, name: String, tag: Option<&'static str>, price: String, selected: bool },
}

impl PickEntry {
    pub fn plain(key: &str, name: impl Into<String>, selected: bool) -> PickEntry {
        PickEntry::Item { key: key.into(), name: name.into(), tag: None, price: String::new(), selected }
    }
}

/// A button showing `label` that opens the list; returns the key of the row picked.
pub fn picker(ui: &mut Ui, id: &str, label: &str, width: f32, entries: &[PickEntry]) -> Option<String> {
    let pal = Palette::of(ui);
    let font = FontId::proportional(13.5);
    let text = super::truncate(label, ((width - 40.0) / 7.0).max(8.0) as usize);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    let p = ui.painter();
    p.rect(
        rect,
        CornerRadius::same(8),
        if resp.hovered() { pal.surface_hover } else { pal.surface },
        Stroke::new(1.0, pal.border),
        egui::StrokeKind::Inside,
    );
    p.text(pos2(rect.min.x + 12.0, rect.center().y), Align2::LEFT_CENTER, text, font, pal.text);
    icons::paint(p, Rect::from_center_size(pos2(rect.max.x - 16.0, rect.center().y), Vec2::splat(14.0)), Icon::Chevron, pal.text_muted);
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let mut picked = None;
    egui::Popup::menu(&resp).id(egui::Id::unique("kadee-picker").with(id)).width(width.max(300.0)).show(|ui| {
        ui.set_min_width(width.max(300.0) - 16.0);
        egui::ScrollArea::vertical().max_height(380.0).auto_shrink([false, true]).show(ui, |ui| {
            for e in entries {
                match e {
                    PickEntry::Header { country, text } => {
                        ui.add_space(4.0);
                        let flag = flag_emoji(country);
                        let t = if flag.is_empty() { text.clone() } else { format!("{flag}  {text}") };
                        ui.label(RichText::new(t).size(11.5).strong().color(pal.text_faint));
                    }
                    PickEntry::Item { key, name, tag, price, selected } => {
                        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
                        if *selected {
                            ui.painter().rect_filled(r, CornerRadius::same(6), pal.accent_soft);
                        } else if resp.hovered() {
                            ui.painter().rect_filled(r, CornerRadius::same(6), pal.surface_hover);
                        }
                        let p = ui.painter();
                        let g = p.layout_no_wrap(name.clone(), FontId::proportional(13.5), pal.text);
                        let nx = r.min.x + 10.0;
                        p.galley(pos2(nx, r.center().y - g.size().y / 2.0), g.clone(), pal.text);
                        if let Some(t) = tag {
                            let tg = p.layout_no_wrap((*t).to_string(), FontId::proportional(10.5), pal.accent);
                            let tr = Rect::from_min_size(pos2(nx + g.size().x + 8.0, r.center().y - 9.0), vec2(tg.size().x + 12.0, 18.0));
                            p.rect_filled(tr, CornerRadius::same(9), pal.accent_soft);
                            p.galley(tr.center() - tg.size() / 2.0, tg, pal.accent);
                        }
                        if !price.is_empty() {
                            p.text(
                                pos2(r.max.x - 10.0, r.center().y),
                                Align2::RIGHT_CENTER,
                                price,
                                FontId::proportional(12.0),
                                pal.text_faint,
                            );
                        }
                        if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            picked = Some(key.clone());
                            ui.close();
                        }
                    }
                }
            }
        });
    });
    picked
}

/// The rows for choosing a model: every usable chat provider's models, grouped by provider.
/// `none_label` adds a first row with the empty key (e.g. "Default for this module").
pub fn model_entries(
    hub: &AiHub,
    config: &AiConfig,
    tests: &HashMap<String, TestState>,
    value: Option<&ModelRef>,
    none_label: Option<&str>,
) -> Vec<PickEntry> {
    let mut out = Vec::new();
    if let Some(l) = none_label {
        out.push(PickEntry::plain("", l, value.is_none()));
    }
    for p in by_region(&config.providers).into_iter().filter(|p| p.usable_for_chat()) {
        let models = offered(hub, tests, p);
        if models.is_empty() {
            continue;
        }
        let default = p.default_model.clone().or_else(|| provider_fallback(hub, tests, p));
        out.push(PickEntry::Header { country: p.country(), text: p.display_name() });
        for m in models {
            let r = ModelRef::new(p.id.clone(), m.clone());
            let info = hub.model_info(&r);
            out.push(PickEntry::Item {
                key: r.to_string(),
                name: info.name.clone(),
                tag: (default.as_deref() == Some(m.as_str())).then_some("default"),
                price: info.price_tier().to_string(),
                selected: value == Some(&r),
            });
        }
    }
    out
}

/// The rows of a provider's "Default model" picker; key `""` is Automatic.
pub fn provider_model_entries(hub: &AiHub, tests: &HashMap<String, TestState>, p: &ProviderConfig) -> Vec<PickEntry> {
    let fallback = provider_fallback(hub, tests, p);
    let info_of = |m: &str| hub.catalog().info(p.kind, m);
    let auto = fallback.as_deref().map(|m| format!("Automatic ({})", info_of(m).name)).unwrap_or_else(|| "Automatic".into());
    let mut out = vec![PickEntry::plain("", auto, p.default_model.is_none())];
    for m in offered(hub, tests, p) {
        let info = info_of(&m);
        out.push(PickEntry::Item {
            key: m.clone(),
            name: info.name.clone(),
            tag: info.recommended.then_some("recommended"),
            price: info.price_tier().to_string(),
            selected: p.default_model.as_deref() == Some(m.as_str()),
        });
    }
    out
}

/// A rounded chip with text.
pub fn chip(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) {
    let g = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(11.5), fg);
    let (r, _) = ui.allocate_exact_size(vec2(g.size().x + 16.0, 22.0), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::same(11), bg);
    ui.painter().galley(r.center() - g.size() / 2.0, g, fg);
}

/// Paint the provider's coloured square with its initials.
pub fn paint_kind(p: &egui::Painter, r: Rect, kind: ProviderKind) {
    let (hex, initials) = kind_badge(kind);
    let c = Color32::from_hex(hex).unwrap_or(Color32::GRAY);
    p.rect_filled(r, CornerRadius::same((r.width() * 0.24) as u8), c);
    p.text(r.center(), Align2::CENTER_CENTER, initials, FontId::proportional(r.width() * 0.42), Color32::WHITE);
}

/// The provider's badge as a widget.
pub fn kind_square(ui: &mut Ui, kind: ProviderKind, size: f32) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    paint_kind(ui.painter(), r, kind);
}
