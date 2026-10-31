//! The sortable document table on Home, Recent, Starred and the folder view.

use crate::app::{KadeeApp, SortKey};
use crate::icons::{self, Icon};
use egui::{Align2, CornerRadius, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use kadee_commands::{Command, Page};
use kadee_core::{FileEntry, human_size, now, relative_time};
use kadee_ui::{Palette, paint_module_badge};

const ROW_H: f32 = 44.0;

/// x offsets of the columns (from the right edge for the fixed ones).
struct Cols {
    name: f32,
    opened: f32,
    modified: f32,
    size: f32,
    star: f32,
}

fn cols(rect: Rect) -> Cols {
    let r = rect.max.x;
    Cols { name: rect.min.x + 14.0, opened: r - 400.0, modified: r - 270.0, size: r - 140.0, star: r - 26.0 }
}

pub fn file_table(app: &mut KadeeApp, ui: &mut Ui, entries: &[FileEntry]) {
    let pal = Palette::of(ui);
    let w = ui.available_width();
    let frame = egui::Frame::new()
        .fill(pal.surface)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(6, 4));
    frame.show(ui, |ui| {
        ui.set_width(w - 12.0);
        ui.spacing_mut().item_spacing.y = 0.0;
        header(app, ui);
        if entries.is_empty() {
            empty_state(app, ui);
            return;
        }
        let t = now();
        for e in entries {
            row(app, ui, e, t);
        }
    });
    ui.add_space(6.0);
    ui.label(
        RichText::new(format!("{} document{}", entries.len(), if entries.len() == 1 { "" } else { "s" })).size(12.0).color(pal.text_faint),
    );
}

fn header(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
    let c = cols(rect);
    let mut head = |ui: &mut Ui, x: f32, label: &str, key: SortKey| {
        let galley = ui.painter().layout_no_wrap(label.to_string(), FontId::proportional(12.0), pal.text_faint);
        let r = Rect::from_min_size(pos2(x, rect.center().y - 10.0), vec2(galley.size().x + 18.0, 20.0));
        let resp = ui.interact(r, ui.scope_id().with(label), Sense::click());
        let active = app.sort == key;
        let color = if active || resp.hovered() { pal.text } else { pal.text_faint };
        ui.painter().galley(pos2(x, rect.center().y - galley.size().y / 2.0), galley.clone(), color);
        if active {
            let ax = x + galley.size().x + 9.0;
            let ay = rect.center().y;
            let pts = if app.sort_desc {
                vec![pos2(ax - 3.5, ay - 2.0), pos2(ax + 3.5, ay - 2.0), pos2(ax, ay + 2.5)]
            } else {
                vec![pos2(ax - 3.5, ay + 2.0), pos2(ax + 3.5, ay + 2.0), pos2(ax, ay - 2.5)]
            };
            ui.painter().add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            if active {
                app.sort_desc = !app.sort_desc;
            } else {
                app.sort = key;
                app.sort_desc = key != SortKey::Name;
            }
        }
    };
    head(ui, c.name + 40.0, "NAME", SortKey::Name);
    head(ui, c.opened, "OPENED", SortKey::Opened);
    head(ui, c.modified, "MODIFIED", SortKey::Modified);
    head(ui, c.size, "SIZE", SortKey::Size);
    ui.painter().hline(rect.x_range(), rect.max.y, Stroke::new(1.0, pal.border));
}

fn row(app: &mut KadeeApp, ui: &mut Ui, e: &FileEntry, t: u64) {
    let pal = Palette::of(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::click());
    let c = cols(rect);
    let starred = app.state.library.is_starred(&e.path);
    let star_rect = Rect::from_center_size(pos2(c.star, rect.center().y), Vec2::splat(26.0));
    let star_resp = ui.interact(star_rect, resp.id.with("star"), Sense::click());
    let hovered = resp.hovered() || star_resp.hovered();
    let p = ui.painter();
    if hovered {
        p.rect_filled(rect.shrink2(vec2(0.0, 2.0)), CornerRadius::same(7), pal.surface_hover);
    }
    let badge = Rect::from_center_size(pos2(c.name + 14.0, rect.center().y), Vec2::splat(28.0));
    paint_module_badge(ui, badge, e.kind);
    let p = ui.painter();
    let name_color = if e.exists { pal.text } else { pal.text_faint };
    let max_name_w = c.opened - c.name - 60.0;
    let name = p.layout(e.name.clone(), FontId::proportional(14.0), name_color, max_name_w);
    let name = if name.rows.len() > 1 {
        p.layout_no_wrap(super::truncate(&e.name, (max_name_w / 7.5) as usize), FontId::proportional(14.0), name_color)
    } else {
        name
    };
    let folder = e.path.parent().and_then(|d| d.file_name()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let sub = if e.exists { format!("{} · {folder}", e.kind.short()) } else { "File not found".into() };
    p.galley(pos2(c.name + 40.0, rect.center().y - 17.0), name, name_color);
    p.text(
        pos2(c.name + 40.0, rect.center().y + 9.0),
        Align2::LEFT_CENTER,
        sub,
        FontId::proportional(11.5),
        if e.exists { pal.text_faint } else { pal.danger },
    );
    let opened = app.state.library.opened_at(&e.path).map(|o| relative_time(o, t)).unwrap_or_else(|| "—".into());
    let small = FontId::proportional(13.0);
    p.text(pos2(c.opened, rect.center().y), Align2::LEFT_CENTER, opened, small.clone(), pal.text_muted);
    p.text(pos2(c.modified, rect.center().y), Align2::LEFT_CENTER, relative_time(e.modified, t), small.clone(), pal.text_muted);
    p.text(
        pos2(c.size, rect.center().y),
        Align2::LEFT_CENTER,
        if e.exists { human_size(e.size) } else { "—".into() },
        small,
        pal.text_muted,
    );
    if starred || hovered {
        let icon = if starred { Icon::StarFilled } else { Icon::Star };
        let col = if starred {
            pal.star
        } else if star_resp.hovered() {
            pal.text
        } else {
            pal.text_faint
        };
        icons::paint(p, star_rect.shrink(5.0), icon, col);
    }
    p.hline(rect.x_range().shrink(8.0), rect.max.y, Stroke::new(1.0, pal.border.gamma_multiply(0.6)));

    if star_resp.on_hover_text(if starred { "Unstar" } else { "Star" }).clicked() {
        app.send(Command::ToggleStar(e.path.clone()));
        return;
    }
    let path = e.path.clone();
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let in_recent = app.state.library.opened_at(&path).is_some();
    resp.context_menu(|ui| {
        ui.set_min_width(200.0);
        let mut item = |ui: &mut Ui, label: &str, cmd: Command| {
            if ui.button(label).clicked() {
                app.send(cmd);
                ui.close();
            }
        };
        item(ui, "Open", Command::OpenPath(path.clone()));
        item(ui, "Open in default app", Command::OpenExternally(path.clone()));
        item(ui, "Show in folder", Command::RevealInFolder(path.clone()));
        item(ui, "Copy path", Command::CopyPath(path.clone()));
        ui.separator();
        item(ui, if starred { "Unstar" } else { "Star" }, Command::ToggleStar(path.clone()));
        if in_recent {
            item(ui, "Remove from Recent", Command::RemoveFromRecent(path.clone()));
        }
    });
    if resp.clicked() || resp.double_clicked() {
        app.send(Command::OpenPath(e.path.clone()));
    }
}

fn empty_state(app: &mut KadeeApp, ui: &mut Ui) {
    let pal = Palette::of(ui);
    ui.add_space(28.0);
    ui.vertical_centered(|ui| {
        let (msg, hint) = match app.page {
            Page::Starred => ("No starred documents", "Hover a document and click the star to keep it here."),
            Page::Recent => ("Nothing opened yet", "Documents you open appear here."),
            _ if !app.search.is_empty() || app.filter.is_some() => ("No documents match", "Clear the search or pick another type."),
            _ => ("No documents yet", "Create one from the cards above, or open a file."),
        };
        ui.label(RichText::new(msg).size(15.0).color(pal.text));
        ui.label(RichText::new(hint).size(13.0).color(pal.text_muted));
    });
    ui.add_space(28.0);
    let _ = StrokeKind::Inside;
}
