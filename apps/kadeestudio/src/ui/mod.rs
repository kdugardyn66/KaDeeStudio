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

use crate::app::KadeeApp;
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
    // A slide show (or the editor's full screen) covers the whole window.
    #[cfg(unix)]
    if app.markdown.fullscreen
        && app.active > 0
        && let Some(path) = app.tabs.get(app.active - 1).cloned()
        && app.markdown.has(&path)
    {
        egui::CentralPanel::default().frame(egui::Frame::new().fill(Color32::BLACK)).show(ui, |ui| {
            app.markdown.area = Some((path, ui.max_rect()));
        });
        return;
    }
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

    // KaDee AI on the right: about the open file in a file tab, a general chat on Home.
    if app.ai_pane {
        let file = if app.active > 0 { app.tabs.get(app.active - 1).cloned() } else { None };
        egui::Panel::right("ai_pane")
            .exact_size(400.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(pal.canvas).stroke(Stroke::new(1.0, pal.border)))
            .show(ui, |ui| {
                ai_pane_header(app, ui);
                match &file {
                    Some(path) => ai_chat::pane(app, ui, path),
                    None => ai_chat::home_pane(app, ui),
                }
            });
    }

    egui::CentralPanel::default().frame(egui::Frame::new().fill(pal.canvas)).show(ui, |ui| {
        if app.active == 0 {
            if app.settings_window.is_some() {
                settings::page(app, ui);
                return;
            }
            match app.page {
                Page::Home | Page::Recent | Page::Starred | Page::Folder | Page::Ai => home::page(app, ui),
                Page::New => keys::new_page(app, ui),
                Page::CraftApps => pages::craft_apps(app, ui),
            }
        } else if let Some(path) = app.tabs.get(app.active - 1).cloned() {
            #[cfg(unix)]
            if crate::markdown::is_markdown(&path) {
                markdown_area(app, ui, &path);
                return;
            }
            file_tab::show(app, ui, &path);
        }
    });

    delete_dialog(app, ui);
    folder_dialog(app, ui);
    keys::sequence_hint(app, ui);
    toast(app, ui);
}

/// The Markdown tab: mdcraft's editor is laid over this area (see `crate::markdown`). Under it,
/// a short message shows while the editor starts, or why it could not.
#[cfg(unix)]
fn markdown_area(app: &mut KadeeApp, ui: &mut Ui, path: &std::path::Path) {
    let pal = Palette::of(ui);
    let rect = ui.max_rect();
    app.markdown.area = Some((path.to_path_buf(), rect));
    let msg = match app.markdown.error(path) {
        Some(e) => format!("{e}\n\nOn Linux the editor needs WebKitGTK 4.1 and an X11 or XWayland display."),
        None if !app.markdown.ready(path) => "Opening the Markdown editor…".to_string(),
        None => return,
    };
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, msg, FontId::proportional(14.0), pal.text_muted);
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
            let (resp, closed) = tab(ui, app.active == i + 1, TabLabel::File(kind, app.tab_name(path)), true);
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
            let ai = icon_button(ui, Icon::Sparkles, &tip);
            if app.ai_pane {
                ui.painter().rect_stroke(ai.rect, CornerRadius::same(6), Stroke::new(1.0, pal.accent), StrokeKind::Inside);
            }
            if ai.clicked() {
                app.send(Command::ToggleAiPane);
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

    // A page is selected unless Settings covers the middle pane.
    let on = |p: Page| app.settings_window.is_none() && app.page == p;
    let (home, new, recent, starred, craft, folder) =
        (on(Page::Home) || on(Page::Ai), on(Page::New), on(Page::Recent), on(Page::Starred), on(Page::CraftApps), on(Page::Folder));
    if nav_item(ui, Icon::Home, "Home", None, home) {
        go(app, Page::Home);
    }
    if nav_item(ui, Icon::NewDoc, "New", None, new) {
        go(app, Page::New);
    }
    if nav_item(ui, Icon::Open, "Open…", None, false) {
        app.send(Command::OpenDialog);
    }
    if nav_item(ui, Icon::Clock, "Recent", Some(recent_n), recent) {
        go(app, Page::Recent);
    }
    if nav_item(ui, Icon::Star, "Starred", Some(starred_n), starred) {
        go(app, Page::Starred);
    }
    if nav_item(ui, Icon::Grid, "Crafting Apps", Some(CRAFT_APPS.len()), craft) {
        go(app, Page::CraftApps);
    }

    ui.add_space(14.0);
    // FOLDERS, as in GenOffice: the KadeeStudio folder and the folders you added, as a tree
    ui.horizontal(|ui| {
        ui.label(RichText::new("FOLDERS").size(11.0).color(pal.text_faint));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let target = if app.page == Page::Folder { app.shown_folder() } else { app.state.settings.documents_dir.clone() };
            if small_icon_button(ui, Icon::Plus, &format!("New folder in “{}”", crate::app::file_name(&target))) {
                app.send(Command::NewFolder(target));
            }
            if small_icon_button(ui, Icon::FolderAdd, "Add folder…") {
                app.send(Command::PinFolder);
            }
        });
    });
    ui.add_space(2.0);
    // a long tree scrolls, leaving room for the status card and Settings
    egui::ScrollArea::vertical()
        .id_salt("folder-tree")
        .max_height((ui.available_height() - 120.0).max(40.0))
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let docs_dir = app.state.settings.documents_dir.clone();
            for root in app.folder_roots() {
                let removable = root != docs_dir;
                let label = crate::app::file_name(&root);
                tree_node(app, ui, &root, &label, 0, Some(removable), folder);
            }
        });
    let _ = docs_n;

    // Bottom: collaboration status + settings
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        if nav_item(ui, Icon::Gear, "Settings", None, app.settings_window.is_some() && app.active == 0) {
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

/// A 20×20 icon button for a section header.
fn small_icon_button(ui: &mut Ui, icon: Icon, tip: &str) -> bool {
    let pal = Palette::of(ui);
    let (r, resp) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(5), pal.surface_hover);
    }
    icons::paint(ui.painter(), r.shrink(3.0), icon, if resp.hovered() { pal.text } else { pal.text_faint });
    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// One folder in the sidebar's tree and, when it is open, the folders inside it.
/// `root`: `Some(removable)` for a root (the KadeeStudio folder is not removable).
fn tree_node(app: &mut KadeeApp, ui: &mut Ui, dir: &std::path::Path, label: &str, depth: usize, root: Option<bool>, on_folder_page: bool) {
    let pal = Palette::of(ui);
    let available = dir.is_dir();
    let open = available && app.is_expanded(dir);
    let children = if available { app.subfolders(dir) } else { Vec::new() };
    let selected = on_folder_page && app.shown_folder() == dir;

    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
    let indent = 4.0 + depth as f32 * 14.0;
    let chevron = Rect::from_center_size(pos2(rect.min.x + indent + 8.0, rect.center().y), Vec2::splat(16.0));
    let chevron_resp = ui.interact(chevron, resp.id.with("chevron"), Sense::click());
    let more = Rect::from_center_size(pos2(rect.max.x - 14.0, rect.center().y), Vec2::splat(22.0));
    let more_resp = ui.interact(more, resp.id.with("more"), Sense::click());
    let hovered = resp.hovered() || chevron_resp.hovered() || more_resp.hovered();
    let menu_open = egui::Popup::is_id_open(ui.ctx(), more_resp.id.with("menu"));
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, CornerRadius::same(7), pal.accent_soft);
    } else if hovered || menu_open {
        p.rect_filled(rect, CornerRadius::same(7), pal.surface_hover);
    }
    if available && (!children.is_empty() || root.is_some()) {
        let icon = if open { Icon::Chevron } else { Icon::ChevronRight };
        icons::paint(p, chevron.shrink(3.0), icon, if chevron_resp.hovered() { pal.text } else { pal.text_faint });
    }
    let fg = if selected { pal.accent } else { pal.text_muted };
    icons::paint(p, Rect::from_center_size(pos2(chevron.max.x + 10.0, rect.center().y), Vec2::splat(15.0)), Icon::Folder, fg);
    let name_x = chevron.max.x + 24.0;
    let max_chars = (((more.min.x - name_x) / 7.2).max(4.0)) as usize;
    p.text(
        pos2(name_x, rect.center().y),
        Align2::LEFT_CENTER,
        truncate(label, max_chars),
        FontId::proportional(13.5),
        if available { pal.text } else { pal.text_faint },
    );
    if !available && !(hovered || menu_open) {
        p.text(pos2(rect.max.x - 8.0, rect.center().y), Align2::RIGHT_CENTER, "Not available", FontId::proportional(11.0), pal.text_faint);
    }
    if hovered || menu_open {
        if more_resp.hovered() || menu_open {
            p.rect_filled(more, CornerRadius::same(5), pal.border);
        }
        icons::paint(p, more.shrink(5.0), Icon::MoreDots, pal.text_muted);
    }

    // the same menu from "…" and from a right-click
    let menu = |app: &mut KadeeApp, ui: &mut Ui| {
        ui.set_min_width(190.0);
        let mut item_cmd: Option<Command> = None;
        let mut item = |ui: &mut Ui, label: &str, cmd: Command| {
            if ui.button(label).clicked() {
                app.send(cmd);
                ui.close();
            }
        };
        if available {
            item(ui, "New subfolder", Command::NewFolder(dir.to_path_buf()));
        }
        if root.is_none() {
            item(ui, "Rename…", Command::RenameFolder(dir.to_path_buf()));
        }
        if available {
            item(ui, "Show in folder", Command::OpenExternally(dir.to_path_buf()));
        }
        if root.is_none() {
            ui.separator();
            let danger = Palette::of(ui).danger;
            if ui.button(RichText::new("Delete folder…").color(danger)).clicked() {
                item_cmd = Some(Command::DeleteFolder(dir.to_path_buf()));
                ui.close();
            }
        }
        if root == Some(true) {
            ui.separator();
            item(ui, "Remove from list", Command::UnpinFolder(dir.to_path_buf()));
        }
        if let Some(c) = item_cmd {
            app.send(c);
        }
    };
    egui::Popup::menu(&more_resp).id(more_resp.id.with("menu")).show(|ui| menu(app, ui));
    let resp = if root.is_some() { resp.on_hover_text(dir.display().to_string()) } else { resp };
    resp.context_menu(|ui| menu(app, ui));
    if chevron_resp.clicked() && available {
        app.send(Command::ToggleFolder(dir.to_path_buf()));
    } else if resp.clicked() && available && !more_resp.clicked() {
        app.send(Command::ShowFolder(dir.to_path_buf()));
    } else if resp.double_clicked() && available {
        app.send(Command::ToggleFolder(dir.to_path_buf()));
    }

    if open {
        for child in children {
            let name = crate::app::file_name(&child);
            tree_node(app, ui, &child, &name, depth + 1, None, on_folder_page);
        }
    }
}

/// A sidebar row; returns true when clicked.
fn nav_item(ui: &mut Ui, icon: Icon, label: &str, count: Option<usize>, selected: bool) -> bool {
    nav_row(ui, icon, label, count, selected, true).clicked()
}

/// A sidebar row's response (for a right-click menu); `present` false greys it (a missing folder).
fn nav_row(ui: &mut Ui, icon: Icon, label: &str, count: Option<usize>, selected: bool, present: bool) -> egui::Response {
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
    let label = truncate(label, 22);
    p.text(
        pos2(rect.min.x + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.0),
        if present { pal.text } else { pal.text_faint },
    );
    if let Some(n) = count {
        p.text(pos2(rect.max.x - 10.0, rect.center().y), Align2::RIGHT_CENTER, n.to_string(), FontId::proportional(12.0), pal.text_faint);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// New folder, Rename folder and Delete folder.
fn folder_dialog(app: &mut KadeeApp, ui: &mut Ui) {
    use crate::app::FolderDialog;
    let Some(mut dialog) = app.folder_dialog.clone() else { return };
    let mut done: Option<bool> = None; // Some(true) = do it, Some(false) = cancel
    let resp = egui::Modal::new(egui::Id::unique("kadee-folder-dialog")).show(ui.ctx(), |ui| {
        let pal = Palette::of(ui);
        ui.set_width(440.0);
        let (title, hint, action) = match &dialog {
            FolderDialog::New { parent, .. } => ("New folder".to_string(), format!("In “{}”", crate::app::file_name(parent)), "Create"),
            FolderDialog::Rename { dir, .. } => ("Rename folder".to_string(), dir.display().to_string(), "Rename"),
            FolderDialog::Delete { dir } => (
                format!("Delete “{}”?", crate::app::file_name(dir)),
                "The folder and everything inside it are moved to the Trash.".to_string(),
                "Move to Trash",
            ),
        };
        ui.label(RichText::new(title).size(18.0).strong().color(pal.text));
        ui.add_space(2.0);
        ui.label(RichText::new(hint).size(12.5).color(pal.text_muted));
        ui.add_space(12.0);
        if let FolderDialog::New { name, .. } | FolderDialog::Rename { name, .. } = &mut dialog {
            let mut out = egui::TextEdit::singleline(name).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 7)).show(ui);
            // the first frame: focus the name with all of it selected, ready to type over
            let fresh = egui::Id::unique("kadee-folder-dialog-fresh");
            if !ui.data(|d| d.get_temp::<bool>(fresh).unwrap_or(false)) {
                ui.data_mut(|d| d.insert_temp(fresh, true));
                out.response.request_focus();
                let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(name.chars().count()));
                out.state.cursor.set_char_range(Some(all));
                out.state.store(ui.ctx(), out.response.id);
            }
            if out.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                done = Some(true);
            }
            ui.add_space(12.0);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let danger = matches!(dialog, FolderDialog::Delete { .. });
            let b = egui::Button::new(RichText::new(action).color(Color32::WHITE)).fill(if danger { pal.danger } else { pal.accent });
            if ui.add(b).clicked() {
                done = Some(true);
            }
            if ui.button("Cancel").clicked() {
                done = Some(false);
            }
        });
    });
    if resp.should_close() && done.is_none() {
        done = Some(false);
    }
    let t = ui.input(|i| i.time);
    if done.is_some() {
        ui.data_mut(|d| d.remove::<bool>(egui::Id::unique("kadee-folder-dialog-fresh")));
    }
    match done {
        None => app.folder_dialog = Some(dialog),
        Some(false) => app.folder_dialog = None,
        Some(true) => {
            let ok = match &dialog {
                FolderDialog::New { parent, name } => app.create_folder(parent, name, t),
                FolderDialog::Rename { dir, name } => app.rename_folder(dir, name, t),
                FolderDialog::Delete { dir } => {
                    app.trash_folder(dir, t);
                    true
                }
            };
            // a name that did not work keeps the dialog open to try another
            app.folder_dialog = if ok { None } else { Some(dialog) };
        }
    }
}

/// "Move to Trash?" for the Recent list's bin.
fn delete_dialog(app: &mut KadeeApp, ui: &mut Ui) {
    let Some(path) = app.delete_confirm.clone() else { return };
    let mut answer: Option<bool> = None;
    let resp = egui::Modal::new(egui::Id::unique("kadee-delete")).show(ui.ctx(), |ui| {
        let pal = Palette::of(ui);
        ui.set_width(440.0);
        ui.label(RichText::new(format!("Delete “{}”?", crate::app::file_name(&path))).size(18.0).strong().color(pal.text));
        ui.add_space(4.0);
        ui.label(RichText::new("The file is moved to the Trash and removed from Recent and Starred.").color(pal.text_muted));
        ui.label(RichText::new(path.display().to_string()).size(12.0).monospace().color(pal.text_faint));
        ui.add_space(14.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let del = egui::Button::new(RichText::new("Move to Trash").color(Color32::WHITE)).fill(pal.danger);
            if ui.add(del).clicked() {
                answer = Some(true);
            }
            if ui.button("Cancel").clicked() {
                answer = Some(false);
            }
        });
    });
    if resp.should_close() && answer.is_none() {
        answer = Some(false);
    }
    if let Some(yes) = answer {
        app.delete_confirm = None;
        if yes {
            let t = ui.input(|i| i.time);
            crate::app::trash_file(app, &path, t);
        }
    }
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
        .anchor(
            // over a Markdown tab the editor would cover the bottom: show it in the tab strip
            if markdown_active(app) { Align2::CENTER_TOP } else { Align2::CENTER_BOTTOM },
            vec2(
                0.0,
                if markdown_active(app) {
                    4.0
                } else if app.matcher.pending().is_empty() {
                    -24.0
                } else {
                    -84.0
                },
            ),
        )
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

/// True when the active tab shows the Markdown editor.
fn markdown_active(app: &KadeeApp) -> bool {
    #[cfg(unix)]
    {
        app.active > 0 && app.tabs.get(app.active - 1).is_some_and(|p| app.markdown.has(p))
    }
    #[cfg(not(unix))]
    {
        let _ = app;
        false
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{t}…")
    }
}
