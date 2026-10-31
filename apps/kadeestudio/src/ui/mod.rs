//! The shell: tab strip, sidebar, page content, toast and shortcuts.

mod ai_chat;
mod ai_settings;
mod ai_widgets;
pub mod craft_icons;
mod file_tab;
mod home;
pub mod keys;
mod pages;
mod settings;
mod table;

use crate::app::{KadeeApp, file_name};
use crate::icons::{self, Icon};
use egui::{Align2, Color32, CornerRadius, FontId, Margin, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use kadee_commands::{Command, Page};
use kadee_core::{CRAFT_APPS, DocKind, ModuleKind, ThemePref};
use kadee_ui::{Palette, paint_doc_badge, paint_module_badge};

const TOAST_SECS: f64 = 3.5;

/// Keyboard shortcuts → commands (see `keys`).
pub fn shortcuts(app: &mut KadeeApp, ctx: &egui::Context) {
    keys::keyboard(app, ctx);
}

/// Lay out the whole window.
pub fn shell(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    egui::Panel::top("tab_strip")
        .exact_size(46.0)
        .show_separator_line(false)
        .frame(egui::Frame::new().fill(pal.chrome).inner_margin(Margin { left: 10, right: 12, top: 8, bottom: 0 }))
        .show(ui, |ui| tab_strip(app, ui));

    if app.active == 0 {
        egui::Panel::left("sidebar")
            .exact_size(236.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(pal.canvas).inner_margin(Margin { left: 12, right: 8, top: 16, bottom: 12 }))
            .show(ui, |ui| sidebar(app, ui));
    }

    if app.active > 0
        && app.ai_pane
        && let Some(path) = app.tabs.get(app.active - 1).cloned()
    {
        egui::Panel::right("ai_pane")
            .exact_size(400.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(pal.canvas).stroke(Stroke::new(1.0, pal.border)))
            .show(ui, |ui| {
                ai_pane_header(app, ui);
                ai_chat::pane(app, ui, &path);
            });
    }

    egui::CentralPanel::default().frame(egui::Frame::new().fill(pal.canvas)).show(ui, |ui| {
        if app.active == 0 {
            match app.page {
                Page::Home | Page::Recent | Page::Starred | Page::Folder => home::page(app, ui),
                Page::CraftApps => pages::craft_apps(app, ui),
                Page::Ai => ai_chat::page(app, ui),
            }
        } else if let Some(path) = app.tabs.get(app.active - 1).cloned() {
            file_tab::show(app, ui, &path);
        }
    });

    keys::new_document_picker(app, ui);
    settings::window(app, ui);
    keys::sequence_hint(app, ui);
    toast(app, ui);
}

/// A 30×30 icon button with hover background.
pub fn icon_button(ui: &mut Ui, icon: Icon, tip: &str) -> egui::Response {
    let pal = Palette::of(ui);
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), pal.surface_hover);
    }
    icons::paint(ui.painter(), rect.shrink(7.0), icon, if resp.hovered() { pal.text } else { pal.text_muted });
    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn tab_strip(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        // Home tab
        if tab(ui, app.active == 0, TabLabel::Home, false).0.clicked() {
            app.send(Command::ActivateTab(0));
        }
        // File tabs
        let mut close = None;
        let mut activate = None;
        for (i, path) in app.tabs.iter().enumerate() {
            let Some(kind) = DocKind::from_path(path) else { continue };
            let (resp, closed) = tab(ui, app.active == i + 1, TabLabel::File(kind, file_name(path)), true);
            if closed {
                close = Some(i + 1);
            } else if resp.clicked() {
                activate = Some(i + 1);
            } else if resp.middle_clicked() {
                close = Some(i + 1);
            }
            resp.on_hover_text(path.display().to_string());
        }
        if let Some(n) = close {
            app.send(Command::CloseTab(n));
        } else if let Some(n) = activate {
            app.send(Command::ActivateTab(n));
        }

        // New-file menu
        let tip = match app.shortcut_text(&Command::NewDocumentPicker) {
            Some(k) => format!("New document ({k})"),
            None => "New document".to_string(),
        };
        let plus = icon_button(ui, Icon::Plus, &tip);
        egui::Popup::menu(&plus).width(220.0).show(|ui| {
            new_menu(app, ui);
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(2.0);
            let tip = match app.shortcut_text(&Command::ToggleAiPane) {
                Some(k) => format!("KaDee AI ({k})"),
                None => "KaDee AI".to_string(),
            };
            let on = app.active > 0 && app.ai_pane;
            let ai = icon_button(ui, Icon::Sparkles, &tip);
            if on {
                ui.painter().rect_stroke(ai.rect, CornerRadius::same(6), Stroke::new(1.0, pal.accent), StrokeKind::Inside);
            }
            if ai.clicked() {
                app.send(if app.active == 0 { Command::ShowPage(Page::Ai) } else { Command::ToggleAiPane });
            }
            let (icon, tip) = match app.state.settings.theme {
                ThemePref::System => (Icon::Auto, "Theme: System — click for Light"),
                ThemePref::Light => (Icon::Sun, "Theme: Light — click for Dark"),
                ThemePref::Dark => (Icon::Moon, "Theme: Dark — click for System"),
            };
            if icon_button(ui, icon, tip).clicked() {
                app.send(Command::CycleTheme);
            }
            let _ = pal;
        });
    });
}

/// The pane's title bar: KaDee AI, and a button that hides it.
fn ai_pane_header(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
        icons::paint(ui.painter(), r, Icon::Sparkles, pal.accent);
        ui.label(RichText::new("KaDee AI").size(14.0).strong().color(pal.text));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            if icon_button(ui, Icon::Close, "Hide KaDee AI").clicked() {
                app.send(Command::ToggleAiPane);
            }
        });
    });
    ui.separator();
}

fn new_menu(app: &mut KadeeApp, ui: &mut Ui) {
    ui.set_min_width(210.0);
    for kind in [ModuleKind::Markdown, ModuleKind::Pdf, ModuleKind::Dashboard, ModuleKind::Report] {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
            paint_module_badge(ui, r, kind);
            let label = if kind == ModuleKind::Pdf { "New PDF".to_string() } else { format!("New {}", kind.short().to_lowercase()) };
            if ui.button(label).clicked() {
                app.send(Command::NewDocument(kind));
                ui.close();
            }
        });
    }
    for kind in kadee_core::OfficeKind::ALL {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
            if let Some(dk) = kadee_core::DocKind::from_path(std::path::Path::new(&format!("x.{}", kind.extension()))) {
                kadee_ui::paint_doc_badge(ui, r, &dk);
            }
            let label = match kind {
                kadee_core::OfficeKind::Word => "New Word document",
                kadee_core::OfficeKind::Spreadsheet => "New spreadsheet",
                kadee_core::OfficeKind::SlideDeck => "New slide deck",
            };
            if ui.button(label).clicked() {
                app.send(Command::NewOffice(kind));
                ui.close();
            }
        });
    }
    ui.separator();
    if ui.button("Open file…").clicked() {
        app.send(Command::OpenDialog);
        ui.close();
    }
    if !app.closed.is_empty() && ui.button("Reopen closed tab").clicked() {
        app.send(Command::ReopenClosedTab);
        ui.close();
    }
}

enum TabLabel {
    Home,
    File(DocKind, String),
}

/// One tab; returns (tab response, close clicked).
fn tab(ui: &mut Ui, active: bool, label: TabLabel, closable: bool) -> (egui::Response, bool) {
    let pal = Palette::of(ui);
    let text = match &label {
        TabLabel::Home => "Home".to_string(),
        TabLabel::File(_, name) => truncate(name, 24),
    };
    let galley = ui.painter().layout_no_wrap(text, FontId::proportional(13.5), pal.text);
    let w = (galley.size().x + 44.0 + if closable { 22.0 } else { 0.0 }).clamp(96.0, 240.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 38.0), Sense::click());
    let hovered = resp.hovered();
    let p = ui.painter();
    let shape_rect = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.max.y + 1.0));
    if active {
        p.rect(shape_rect, CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 }, pal.canvas, Stroke::NONE, StrokeKind::Inside);
    } else if hovered {
        p.rect_filled(shape_rect.shrink2(vec2(0.0, 3.0)), CornerRadius::same(7), pal.surface_hover);
    }
    let icon_rect = Rect::from_center_size(pos2(rect.min.x + 20.0, rect.center().y), Vec2::splat(18.0));
    match &label {
        TabLabel::Home => icons::paint(p, icon_rect.shrink(1.0), Icon::Home, if active { pal.accent } else { pal.text_muted }),
        TabLabel::File(kind, _) => paint_doc_badge(ui, icon_rect, kind),
    }
    let color = if active { pal.text } else { pal.text_muted };
    ui.painter().galley(pos2(rect.min.x + 36.0, rect.center().y - galley.size().y / 2.0), galley, color);

    let mut closed = false;
    if closable {
        let cr = Rect::from_center_size(pos2(rect.max.x - 16.0, rect.center().y), Vec2::splat(18.0));
        let cresp = ui.interact(cr, resp.id.with("close"), Sense::click());
        if cresp.hovered() {
            ui.painter().rect_filled(cr, CornerRadius::same(4), pal.border);
        }
        if hovered || active || cresp.hovered() {
            icons::paint(ui.painter(), cr.shrink(3.0), Icon::Close, pal.text_muted);
        }
        closed = cresp.clicked();
    }
    (resp.on_hover_cursor(egui::CursorIcon::PointingHand), closed)
}

fn sidebar(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    // Brand
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
        kadee_ui::paint_logo(ui, r);
        ui.add_space(2.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(RichText::new("KadeeStudio").size(15.5).strong().color(pal.text));
            ui.label(RichText::new(format!("Office suite · v{}", kadee_core::VERSION)).size(11.5).color(pal.text_faint));
        });
    });
    ui.add_space(18.0);

    let lib = &app.state.library;
    let recent_n = lib.recent.len();
    let starred_n = lib.starred.len();
    let docs_n = app.docs.len();
    let go = |app: &mut KadeeApp, page| app.send(Command::ShowPage(page));

    if nav_item(ui, Icon::Home, "Home", None, app.page == Page::Home) {
        go(app, Page::Home);
    }
    if nav_item(ui, Icon::Clock, "Recent", Some(recent_n), app.page == Page::Recent) {
        go(app, Page::Recent);
    }
    if nav_item(ui, Icon::Star, "Starred", Some(starred_n), app.page == Page::Starred) {
        go(app, Page::Starred);
    }
    if nav_item(ui, Icon::Sparkles, "KaDee AI", None, app.page == Page::Ai) {
        go(app, Page::Ai);
    }
    if nav_item(ui, Icon::Grid, "Crafting Apps", Some(CRAFT_APPS.len()), app.page == Page::CraftApps) {
        go(app, Page::CraftApps);
    }

    ui.add_space(14.0);
    ui.label(RichText::new("FOLDERS").size(11.0).color(pal.text_faint));
    ui.add_space(2.0);
    if nav_item(ui, Icon::Folder, "KadeeStudio", Some(docs_n), app.page == Page::Folder) {
        go(app, Page::Folder);
    }

    // Bottom: collaboration status + settings
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        if nav_item(ui, Icon::Gear, "Settings", None, app.settings_window.is_some()) {
            app.send(Command::OpenSettings(kadee_commands::SettingsSection::General));
        }
        ui.add_space(4.0);
        let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::hover());
        let p = ui.painter();
        p.rect(r, CornerRadius::same(8), pal.surface, Stroke::new(1.0, pal.border), StrokeKind::Inside);
        p.circle_filled(pos2(r.min.x + 16.0, r.min.y + 18.0), 4.0, pal.text_faint);
        p.text(pos2(r.min.x + 28.0, r.min.y + 18.0), Align2::LEFT_CENTER, "Working locally", FontId::proportional(13.0), pal.text);
        p.text(
            pos2(r.min.x + 28.0, r.min.y + 36.0),
            Align2::LEFT_CENTER,
            "Team server arrives in Phase 3",
            FontId::proportional(11.5),
            pal.text_faint,
        );
    });
}

/// A sidebar row; returns true when clicked.
fn nav_item(ui: &mut Ui, icon: Icon, label: &str, count: Option<usize>, selected: bool) -> bool {
    let pal = Palette::of(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, CornerRadius::same(7), pal.accent_soft);
    } else if resp.hovered() {
        p.rect_filled(rect, CornerRadius::same(7), pal.surface_hover);
    }
    let fg = if selected { pal.accent } else { pal.text_muted };
    icons::paint(p, Rect::from_center_size(pos2(rect.min.x + 18.0, rect.center().y), Vec2::splat(16.0)), icon, fg);
    p.text(pos2(rect.min.x + 36.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(14.0), pal.text);
    if let Some(n) = count {
        p.text(pos2(rect.max.x - 10.0, rect.center().y), Align2::RIGHT_CENTER, n.to_string(), FontId::proportional(12.0), pal.text_faint);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

fn toast(app: &mut KadeeApp, ui: &mut Ui) {
    let Some((msg, t0)) = app.toast.clone() else { return };
    let now = ui.input(|i| i.time);
    if now - t0 > TOAST_SECS {
        app.toast = None;
        return;
    }
    ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
    let pal = Palette::of(ui);
    egui::Area::new(egui::Id::unique("kadee-toast"))
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, if app.matcher.pending().is_empty() { -24.0 } else { -84.0 }))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(pal.text)
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(16, 10))
                .shadow(egui::epaint::Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(60) })
                .show(ui, |ui| {
                    ui.add(egui::Label::new(RichText::new(msg).color(pal.surface).size(13.5)).wrap_mode(egui::TextWrapMode::Extend));
                });
        });
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{t}…")
    }
}
