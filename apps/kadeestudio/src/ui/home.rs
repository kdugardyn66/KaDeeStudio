//! The Home page (also used for Recent, Starred and the folder view).

use super::{icon_button, table};
use crate::app::KadeeApp;
use crate::icons::{self, Icon};
use chrono::{Datelike, Local, Timelike};
use egui::{Align2, CornerRadius, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use kadee_commands::{Command, Page};
use kadee_core::{CRAFT_APPS, DocGroup, ModuleKind};
use kadee_ui::{Palette, paint_module_badge, pill};

pub const CONTENT_MAX: f32 = 1120.0;

/// Centre a column of at most `CONTENT_MAX` and give it side padding.
pub fn content_column(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        let avail = ui.available_width();
        let w = (avail - 64.0).min(CONTENT_MAX);
        let pad = ((avail - w) / 2.0).max(24.0);
        ui.horizontal(|ui| {
            ui.add_space(pad);
            ui.vertical(|ui| {
                ui.set_width(w);
                ui.add_space(28.0);
                add(ui);
                ui.add_space(40.0);
            });
        });
    });
}

pub fn page(app: &mut KadeeApp, ui: &mut Ui) {
    content_column(ui, |ui| match app.page {
        Page::Home => {
            greeting(app, ui);
            ui.add_space(22.0);
            quick_start(app, ui);
            ui.add_space(26.0);
            craft_strip(app, ui);
            ui.add_space(26.0);
            documents(app, ui, "Documents");
        }
        Page::Recent => {
            page_title(ui, "Recent", "Documents you opened, newest first");
            documents(app, ui, "");
        }
        Page::Starred => {
            page_title(ui, "Starred", "Documents you marked with a star");
            documents(app, ui, "");
        }
        // a folder from the sidebar's tree: quick start (new documents go into it), its
        // sub-folders, then its documents — as in GenOffice
        _ => {
            let dir = app.shown_folder();
            folder_heading(app, ui, &dir);
            ui.add_space(14.0);
            quick_start(app, ui);
            ui.add_space(22.0);
            documents(app, ui, "");
        }
    });
}

/// The folder's name, its path as clickable steps from the root it belongs to, and its actions.
fn folder_heading(app: &mut KadeeApp, ui: &mut Ui, dir: &std::path::Path) {
    let pal = Palette::of(ui);
    let roots = app.folder_roots();
    let docs_dir = app.state.settings.documents_dir.clone();
    let root = roots.iter().filter(|r| dir.starts_with(r)).max_by_key(|r| r.components().count()).cloned();
    let name = crate::app::file_name(dir);
    ui.horizontal(|ui| {
        ui.label(RichText::new(name).size(26.0).strong().color(pal.text));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if root.as_deref() == Some(dir) && *dir != docs_dir && ui.button("Remove from list").clicked() {
                app.send(Command::UnpinFolder(dir.to_path_buf()));
            }
            if ui.button("Show in folder").clicked() {
                app.send(Command::OpenExternally(dir.to_path_buf()));
            }
            if dir.is_dir() && ui.button("New folder").clicked() {
                app.send(Command::NewFolder(dir.to_path_buf()));
            }
        });
    });
    // KadeeStudio › Clients › Contracts
    match root {
        Some(root) if root.as_path() != dir => {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let mut acc = root.clone();
                let first = crate::app::file_name(&root);
                let mut steps = vec![(first, root.clone())];
                if let Ok(rest) = dir.strip_prefix(&root) {
                    for part in rest.components() {
                        acc = acc.join(part);
                        steps.push((part.as_os_str().to_string_lossy().into_owned(), acc.clone()));
                    }
                }
                let n = steps.len();
                for (i, (label, path)) in steps.into_iter().enumerate() {
                    if i + 1 == n {
                        ui.label(RichText::new(label).size(13.5).color(pal.text_muted));
                    } else {
                        if ui.link(RichText::new(label).size(13.5)).clicked() {
                            app.send(Command::ShowFolder(path));
                        }
                        ui.label(RichText::new("›").size(13.5).color(pal.text_faint));
                    }
                }
            });
        }
        _ => {
            ui.label(RichText::new(dir.display().to_string()).size(13.5).color(pal.text_muted));
        }
    }
}

pub fn page_title(ui: &mut Ui, title: &str, sub: &str) {
    let pal = Palette::of(ui);
    ui.label(RichText::new(title).size(26.0).strong().color(pal.text));
    ui.label(RichText::new(sub).size(14.0).color(pal.text_muted));
    ui.add_space(18.0);
}

fn greeting(app: &KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    let now = Local::now();
    let part = match now.hour() {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    };
    let name = app.state.settings.display_name.trim();
    let title = if name.is_empty() { part.to_string() } else { format!("{part}, {name}") };
    ui.label(RichText::new(title).size(28.0).strong().color(pal.text));
    let weekday = now.format("%A").to_string();
    let month = now.format("%B").to_string();
    ui.label(
        RichText::new(format!("{weekday} {} {month} · Start something new or pick up where you left off.", now.day()))
            .size(14.0)
            .color(pal.text_muted),
    );
}

fn quick_start(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    ui.label(RichText::new("Start something new").size(16.0).strong().color(pal.text));
    ui.add_space(4.0);
    let gap = 14.0;
    let n = ModuleKind::TILES.len() + 1;
    let w = ui.available_width();
    // 6 across when they fit, otherwise 3 (never a lone card on the last row).
    let cols = if w >= 6.0 * 146.0 + 5.0 * gap {
        n
    } else if w >= 3.0 * 146.0 + 2.0 * gap {
        3
    } else {
        2
    };
    let card_w = (w - gap * (cols as f32 - 1.0)) / cols as f32;
    let items: Vec<Option<ModuleKind>> = ModuleKind::TILES.iter().copied().map(Some).chain(std::iter::once(None)).collect();
    for row in items.chunks(cols) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for item in row {
                if let Some(cmd) = card(ui, *item, card_w) {
                    app.send(cmd);
                }
            }
        });
        ui.add_space(gap - ui.spacing().item_spacing.y);
    }
    let _ = pal;
}

/// A quick-start card; `None` = "Open file".
fn card(ui: &mut Ui, kind: Option<ModuleKind>, w: f32) -> Option<Command> {
    let pal = Palette::of(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 132.0), Sense::click());
    let hovered = resp.hovered();
    let p = ui.painter();
    p.rect(
        rect,
        CornerRadius::same(10),
        pal.surface,
        Stroke::new(1.0, if hovered { pal.border_strong } else { pal.border }),
        StrokeKind::Inside,
    );
    let inner = rect.shrink(16.0);
    let badge = Rect::from_min_size(inner.min, Vec2::splat(38.0));
    let (title, tagline, foot) = match kind {
        Some(k) => {
            paint_module_badge(ui, badge, k);
            let foot = match k {
                ModuleKind::Markdown | ModuleKind::Dashboard | ModuleKind::Report => "New document".to_string(),
                ModuleKind::Pdf => "Open a PDF".to_string(),
                ModuleKind::Convert => format!("Coming in Phase {}", k.phase()),
            };
            (k.short().to_string(), k.tagline(), foot)
        }
        None => {
            p.rect_filled(badge, CornerRadius::same(9), pal.surface_hover);
            icons::paint(p, badge.shrink(10.0), Icon::Open, pal.text_muted);
            ("Open file".to_string(), "From this computer", "Ctrl+O".to_string())
        }
    };
    let p = ui.painter();
    p.text(pos2(inner.min.x, badge.max.y + 14.0), Align2::LEFT_TOP, &title, FontId::proportional(15.0), pal.text);
    let tag = p.layout(tagline.to_string(), FontId::proportional(12.5), pal.text_muted, inner.width());
    p.galley(pos2(inner.min.x, badge.max.y + 34.0), tag, pal.text_muted);
    if hovered {
        p.text(pos2(inner.max.x, inner.min.y + 4.0), Align2::RIGHT_TOP, foot, FontId::proportional(11.5), pal.accent);
    }
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if !resp.clicked() {
        return None;
    }
    Some(match kind {
        Some(k @ (ModuleKind::Markdown | ModuleKind::Dashboard | ModuleKind::Report)) => Command::NewDocument(k),
        Some(k) => Command::OpenModule(k),
        None => Command::OpenDialog,
    })
}

fn craft_strip(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    let n = app.installed.len();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Crafting Apps").size(16.0).strong().color(pal.text));
        let sub = if n == 0 {
            "None installed yet · get them with the ArtCraft launcher".to_string()
        } else {
            format!("{n} of {} installed on this computer", CRAFT_APPS.len())
        };
        ui.label(RichText::new(sub).size(13.0).color(pal.text_faint));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.link("See all").clicked() {
                app.send(Command::ShowPage(Page::CraftApps));
            }
        });
    });
    ui.add_space(2.0);
    let installed = app.installed.clone();
    if installed.is_empty() {
        ui.label(RichText::new("Installed Crafting Apps appear here with their icons; click one to open it.").color(pal.text_muted));
        return;
    }
    // The same tiles at every width: as many per row as fit (at least TILE_MIN wide), so the
    // rows rearrange when the window narrows or the KaDee AI pane opens.
    const TILE_MIN: f32 = 150.0;
    let gap = 10.0;
    let avail = ui.available_width();
    let cols = (((avail + gap) / (TILE_MIN + gap)).floor() as usize).max(1);
    let cw = (avail - gap * (cols as f32 - 1.0)) / cols as f32;
    for row in installed.chunks(cols) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for found in row {
                let (r, resp) = ui.allocate_exact_size(vec2(cw, 58.0), Sense::click());
                ui.painter().rect(
                    r,
                    CornerRadius::same(8),
                    if resp.hovered() { pal.surface_hover } else { pal.surface },
                    Stroke::new(1.0, if resp.hovered() { pal.accent } else { pal.border }),
                    StrokeKind::Inside,
                );
                let icon = Rect::from_center_size(pos2(r.min.x + 29.0, r.center().y), Vec2::splat(38.0));
                app.craft_icons.paint(ui, icon, found.app, false);
                let p = ui.painter();
                p.text(
                    pos2(r.min.x + 56.0, r.min.y + 21.0),
                    Align2::LEFT_CENTER,
                    super::truncate(found.app.name, ((cw - 62.0) / 7.0) as usize),
                    FontId::proportional(13.5),
                    pal.text,
                );
                p.text(
                    pos2(r.min.x + 56.0, r.min.y + 39.0),
                    Align2::LEFT_CENTER,
                    super::truncate(found.app.purpose, ((cw - 64.0) / 6.0) as usize),
                    FontId::proportional(11.5),
                    pal.text_faint,
                );
                if resp
                    .on_hover_text(format!("Open {} · {} · {}", found.app.name, found.app.purpose, found.location()))
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    app.send(Command::LaunchCraft(found.app.id.to_string()));
                }
            }
        });
        ui.add_space(gap - ui.spacing().item_spacing.y);
    }
}

fn documents(app: &mut KadeeApp, ui: &mut Ui, title: &str) {
    let pal = Palette::of(ui);
    // Narrow (the KaDee AI pane is open): the search box gets a row of its own.
    let one_row = ui.available_width() >= 1000.0;
    let search_row = |app: &mut KadeeApp, ui: &mut Ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icon_button(ui, Icon::Refresh, "Refresh").clicked() {
                app.send(Command::Refresh);
            }
            ui.scope(|ui| search_box(ui, &mut app.search));
        });
    };
    ui.horizontal_wrapped(|ui| {
        if !title.is_empty() {
            ui.label(RichText::new(title).size(16.0).strong().color(pal.text));
            ui.add_space(8.0);
        }
        if pill(ui, "All", app.filter.is_none()).clicked() {
            app.filter = None;
        }
        for group in DocGroup::ALL {
            if pill(ui, group.label(), app.filter == Some(group)).clicked() {
                app.filter = if app.filter == Some(group) { None } else { Some(group) };
            }
        }
        if one_row {
            search_row(app, ui);
        }
    });
    if !one_row {
        ui.add_space(6.0);
        ui.horizontal(|ui| search_row(app, ui));
    }
    ui.add_space(8.0);
    let entries = app.visible_entries();
    // a folder page lists its sub-folders first (the search applies to their names too)
    let folders: Vec<std::path::PathBuf> = if app.page == Page::Folder {
        let q = app.search.trim().to_lowercase();
        app.folder_subdirs.iter().filter(|d| q.is_empty() || crate::app::file_name(d).to_lowercase().contains(&q)).cloned().collect()
    } else {
        Vec::new()
    };
    table::file_table(app, ui, &folders, &entries);
}

fn search_box(ui: &mut Ui, text: &mut String) {
    let pal = Palette::of(ui);
    ui.set_max_width(250.0);
    egui::Frame::new()
        .fill(pal.surface)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(7))
        .inner_margin(egui::Margin { left: 8, right: 8, top: 2, bottom: 2 })
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                icons::paint(ui.painter(), r, Icon::Search, pal.text_faint);
                ui.add(egui::TextEdit::singleline(text).hint_text("Search documents").frame(egui::Frame::NONE).desired_width(190.0));
            });
        });
}
