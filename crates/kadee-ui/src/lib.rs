//! KadeeStudio design tokens, light/dark themes and small shared widgets.
//!
//! All colours come from [`Palette`]; UI code never hard-codes a colour,
//! so light and dark stay consistent.

use egui::{Color32, CornerRadius, FontId, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, Visuals, epaint::Shadow};
use kadee_core::{DocGroup, DocKind, ModuleKind, ThemePref};

/// Design tokens for one theme.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    /// Window background behind everything.
    pub canvas: Color32,
    /// Sidebar and tab strip.
    pub chrome: Color32,
    /// Cards and the active tab.
    pub surface: Color32,
    pub surface_hover: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub text_faint: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
    pub star: Color32,
    pub danger: Color32,
}

impl Palette {
    pub const LIGHT: Palette = Palette {
        dark: false,
        canvas: Color32::from_rgb(0xF7, 0xF6, 0xF3),
        chrome: Color32::from_rgb(0xEE, 0xEC, 0xE7),
        surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        surface_hover: Color32::from_rgb(0xF1, 0xF0, 0xEC),
        border: Color32::from_rgb(0xE3, 0xE0, 0xD9),
        border_strong: Color32::from_rgb(0xC9, 0xC5, 0xBC),
        text: Color32::from_rgb(0x1E, 0x1D, 0x1A),
        text_muted: Color32::from_rgb(0x5F, 0x5C, 0x55),
        text_faint: Color32::from_rgb(0x8E, 0x8A, 0x82),
        accent: Color32::from_rgb(0x2F, 0x5D, 0xD6),
        accent_soft: Color32::from_rgb(0xE5, 0xEC, 0xFB),
        on_accent: Color32::WHITE,
        star: Color32::from_rgb(0xD9, 0x9A, 0x06),
        danger: Color32::from_rgb(0xC2, 0x3B, 0x2E),
    };

    pub const DARK: Palette = Palette {
        dark: true,
        canvas: Color32::from_rgb(0x17, 0x18, 0x1B),
        chrome: Color32::from_rgb(0x1F, 0x20, 0x24),
        surface: Color32::from_rgb(0x25, 0x27, 0x2C),
        surface_hover: Color32::from_rgb(0x2E, 0x30, 0x36),
        border: Color32::from_rgb(0x34, 0x36, 0x3C),
        border_strong: Color32::from_rgb(0x4A, 0x4D, 0x55),
        text: Color32::from_rgb(0xEC, 0xEB, 0xE7),
        text_muted: Color32::from_rgb(0xA9, 0xA7, 0xA1),
        text_faint: Color32::from_rgb(0x7C, 0x7A, 0x75),
        accent: Color32::from_rgb(0x6E, 0x93, 0xF2),
        accent_soft: Color32::from_rgb(0x26, 0x31, 0x4D),
        on_accent: Color32::from_rgb(0x10, 0x14, 0x20),
        star: Color32::from_rgb(0xF2, 0xB8, 0x2E),
        danger: Color32::from_rgb(0xF0, 0x6E, 0x62),
    };

    /// The palette for the theme the UI is currently drawn in.
    pub fn of(ui: &Ui) -> Palette {
        if ui.visuals().dark_mode { Palette::DARK } else { Palette::LIGHT }
    }

    /// Colour that identifies a module (badges, card strips).
    pub fn module(&self, kind: ModuleKind) -> Color32 {
        let (l, d) = match kind {
            ModuleKind::Markdown => ((0x2F, 0x5D, 0xD6), (0x6E, 0x93, 0xF2)),
            ModuleKind::Pdf => ((0xC2, 0x3B, 0x2E), (0xF0, 0x6E, 0x62)),
            ModuleKind::Convert => ((0x7A, 0x4C, 0xC9), (0xA9, 0x86, 0xF0)),
            ModuleKind::Dashboard => ((0x0F, 0x86, 0x6C), (0x3E, 0xC4, 0xA2)),
            ModuleKind::Report => ((0xB8, 0x6B, 0x00), (0xF0, 0xA3, 0x3A)),
        };
        let (r, g, b) = if self.dark { d } else { l };
        Color32::from_rgb(r, g, b)
    }

    /// The module colour mixed into the surface, for soft badge backgrounds.
    pub fn module_soft(&self, kind: ModuleKind) -> Color32 {
        mix(self.surface, self.module(kind), if self.dark { 0.22 } else { 0.12 })
    }

    /// Colour of a start-screen document group.
    pub fn group(&self, group: DocGroup) -> Color32 {
        let (l, d) = match group {
            DocGroup::Markdown => return self.module(ModuleKind::Markdown),
            DocGroup::Pdf => return self.module(ModuleKind::Pdf),
            DocGroup::Dashboard => return self.module(ModuleKind::Dashboard),
            DocGroup::Report => return self.module(ModuleKind::Report),
            DocGroup::Office => ((0x2B, 0x6C, 0x8F), (0x5F, 0xB0, 0xD8)),
            DocGroup::Graphics => ((0xB0, 0x3A, 0x7A), (0xE8, 0x7C, 0xB6)),
            DocGroup::Media => ((0x7A, 0x4C, 0xC9), (0xA9, 0x86, 0xF0)),
            DocGroup::Cad => ((0x4F, 0x6B, 0x2E), (0x9B, 0xC2, 0x6A)),
        };
        let (r, g, b) = if self.dark { d } else { l };
        Color32::from_rgb(r, g, b)
    }

    /// egui visuals built from this palette.
    pub fn visuals(&self) -> Visuals {
        let mut v = if self.dark { Visuals::dark() } else { Visuals::light() };
        let r = CornerRadius::same(6);
        v.panel_fill = self.canvas;
        v.window_fill = self.surface;
        v.window_stroke = Stroke::new(1.0, self.border);
        v.window_corner_radius = CornerRadius::same(10);
        v.window_shadow =
            Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(if self.dark { 120 } else { 40 }) };
        v.popup_shadow = Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(if self.dark { 110 } else { 32 }) };
        v.menu_corner_radius = CornerRadius::same(8);
        v.faint_bg_color = self.surface_hover;
        v.extreme_bg_color = self.surface;
        v.hyperlink_color = self.accent;
        v.selection.bg_fill = self.accent_soft;
        v.selection.stroke = Stroke::new(1.0, self.accent);
        v.override_text_color = None;

        let w = &mut v.widgets;
        w.noninteractive.bg_fill = self.surface;
        w.noninteractive.weak_bg_fill = self.surface;
        w.noninteractive.bg_stroke = Stroke::new(1.0, self.border);
        w.noninteractive.fg_stroke = Stroke::new(1.0, self.text);
        w.noninteractive.corner_radius = r;

        w.inactive.bg_fill = self.surface;
        w.inactive.weak_bg_fill = self.surface;
        w.inactive.bg_stroke = Stroke::new(1.0, self.border);
        w.inactive.fg_stroke = Stroke::new(1.0, self.text);
        w.inactive.corner_radius = r;

        w.hovered.bg_fill = self.surface_hover;
        w.hovered.weak_bg_fill = self.surface_hover;
        w.hovered.bg_stroke = Stroke::new(1.0, self.border_strong);
        w.hovered.fg_stroke = Stroke::new(1.0, self.text);
        w.hovered.corner_radius = r;
        w.hovered.expansion = 0.0;

        w.active.bg_fill = self.accent_soft;
        w.active.weak_bg_fill = self.accent_soft;
        w.active.bg_stroke = Stroke::new(1.0, self.accent);
        w.active.fg_stroke = Stroke::new(1.0, self.text);
        w.active.corner_radius = r;
        w.active.expansion = 0.0;

        w.open = w.hovered;
        v
    }
}

/// Linear mix of two colours; `t = 0` gives `a`.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let f = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

/// Install both palettes and select the user's theme.
pub fn apply_theme(ctx: &egui::Context, pref: ThemePref) {
    ctx.set_visuals_of(egui::Theme::Light, Palette::LIGHT.visuals());
    ctx.set_visuals_of(egui::Theme::Dark, Palette::DARK.visuals());
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        ctx.style_mut_of(theme, |s| {
            s.spacing.item_spacing = Vec2::new(8.0, 8.0);
            s.spacing.button_padding = Vec2::new(10.0, 5.0);
            s.spacing.interact_size.y = 28.0;
            s.text_styles.insert(egui::TextStyle::Body, FontId::proportional(14.0));
            s.text_styles.insert(egui::TextStyle::Button, FontId::proportional(14.0));
            s.text_styles.insert(egui::TextStyle::Small, FontId::proportional(12.0));
            s.text_styles.insert(egui::TextStyle::Heading, FontId::proportional(26.0));
        });
    }
    ctx.set_theme(match pref {
        ThemePref::System => egui::ThemePreference::System,
        ThemePref::Light => egui::ThemePreference::Light,
        ThemePref::Dark => egui::ThemePreference::Dark,
    });
}

/// A rounded square with the module's badge letters.
pub fn module_badge(ui: &mut Ui, kind: ModuleKind, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    paint_module_badge(ui, rect, kind);
    resp
}

/// Paint a module badge into `rect`.
pub fn paint_module_badge(ui: &Ui, rect: Rect, kind: ModuleKind) {
    let pal = Palette::of(ui);
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same((rect.height() * 0.24) as u8), pal.module_soft(kind));
    let font = FontId::proportional((rect.height() * 0.34).max(9.0));
    p.text(rect.center(), egui::Align2::CENTER_CENTER, kind.badge(), font, pal.module(kind));
}

/// Paint the badge of any document type into `rect`, coloured by its group.
pub fn paint_doc_badge(ui: &Ui, rect: Rect, kind: &DocKind) {
    let pal = Palette::of(ui);
    let color = pal.group(kind.group);
    let soft = mix(pal.surface, color, if pal.dark { 0.22 } else { 0.12 });
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same((rect.height() * 0.24) as u8), soft);
    let scale = if kind.badge().len() > 2 { 0.31 } else { 0.34 };
    let font = FontId::proportional((rect.height() * scale).max(8.5));
    p.text(rect.center(), egui::Align2::CENTER_CENTER, kind.badge(), font, color);
}

/// A document badge widget.
pub fn doc_badge(ui: &mut Ui, kind: &DocKind, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    paint_doc_badge(ui, rect, kind);
    resp
}

/// A rounded "pill" toggle used for type filters.
pub fn pill(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let pal = Palette::of(ui);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), FontId::proportional(13.0), Color32::PLACEHOLDER);
    let size = Vec2::new(galley.size().x + 24.0, 28.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let (fill, stroke, text) = if selected {
        (pal.text, pal.text, pal.surface)
    } else if resp.hovered() {
        (pal.surface_hover, pal.border_strong, pal.text)
    } else {
        (pal.surface, pal.border, pal.text_muted)
    };
    let p = ui.painter();
    p.rect(rect, CornerRadius::same(14), fill, Stroke::new(1.0, stroke), StrokeKind::Inside);
    p.galley(rect.center() - galley.size() / 2.0, galley, text);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Section title with an optional right-aligned muted note.
pub fn section_title(ui: &mut Ui, title: &str, note: Option<&str>) {
    let pal = Palette::of(ui);
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).size(16.0).strong().color(pal.text));
        if let Some(n) = note {
            ui.label(RichText::new(n).size(13.0).color(pal.text_faint));
        }
    });
}

/// A primary (accent) button.
pub fn primary_button(ui: &mut Ui, label: &str) -> Response {
    let pal = Palette::of(ui);
    ui.add(
        egui::Button::new(RichText::new(label).color(pal.on_accent).strong())
            .fill(pal.accent)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(6)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_endpoints() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(200, 100, 50);
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
        assert_eq!(mix(a, b, 0.5), Color32::from_rgb(100, 50, 25));
    }

    #[test]
    fn text_contrasts_with_surface() {
        for p in [Palette::LIGHT, Palette::DARK] {
            let lum = |c: Color32| 0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b());
            assert!((lum(p.text) - lum(p.surface)).abs() > 150.0);
        }
    }
}
