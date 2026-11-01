//! Line icons painted with egui shapes (no icon font needed).

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

/// Icons used by the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Home,
    Clock,
    Star,
    StarFilled,
    Folder,
    Grid,
    Gear,
    Plus,
    Close,
    Sun,
    Moon,
    Auto,
    Search,
    Refresh,
    Open,
    Keyboard,
    Info,
    Sparkles,
    Send,
    Stop,
    Chevron,
    /// A document with a plus: New.
    NewDoc,
    /// Pointing right (a closed folder in the tree).
    ChevronRight,
    /// Three dots: more actions.
    MoreDots,
    /// A folder with an arrow in: Add folder.
    FolderAdd,
}

/// Paint `icon` centred in `rect`.
pub fn paint(p: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let s = rect.width().min(rect.height());
    let c = rect.center();
    let u = s / 16.0; // design grid: 16 units
    let at = |x: f32, y: f32| pos2(c.x + (x - 8.0) * u, c.y + (y - 8.0) * u);
    let st = Stroke::new((1.4 * u).max(1.2), color);
    let line = |pts: &[Pos2]| p.add(Shape::line(pts.to_vec(), st));
    match icon {
        Icon::Home => {
            line(&[at(2.0, 7.5), at(8.0, 2.5), at(14.0, 7.5)]);
            line(&[at(3.8, 6.2), at(3.8, 13.5), at(12.2, 13.5), at(12.2, 6.2)]);
            line(&[at(6.6, 13.5), at(6.6, 9.5), at(9.4, 9.5), at(9.4, 13.5)]);
        }
        Icon::Clock => {
            p.circle_stroke(c, 6.0 * u, st);
            line(&[at(8.0, 4.6), at(8.0, 8.0), at(10.6, 9.6)]);
        }
        Icon::Star | Icon::StarFilled => {
            let pts: Vec<Pos2> = (0..10)
                .map(|i| {
                    let a = std::f32::consts::PI * (i as f32) / 5.0 - std::f32::consts::FRAC_PI_2;
                    let r = if i % 2 == 0 { 6.6 } else { 2.9 } * u;
                    pos2(c.x + r * a.cos(), c.y + 0.6 * u + r * a.sin())
                })
                .collect();
            if icon == Icon::StarFilled {
                p.add(Shape::convex_polygon(pts.clone(), color, Stroke::NONE));
                // convex_polygon fills the hull; overdraw the outline for a crisp star.
                p.add(Shape::closed_line(pts, st));
            } else {
                p.add(Shape::closed_line(pts, st));
            }
        }
        Icon::Folder => {
            line(&[at(2.0, 12.8), at(2.0, 3.6), at(6.2, 3.6), at(7.6, 5.2), at(14.0, 5.2), at(14.0, 12.8), at(2.0, 12.8)]);
            line(&[at(2.0, 7.0), at(14.0, 7.0)]);
        }
        Icon::Grid => {
            for (x, y) in [(2.5, 2.5), (9.0, 2.5), (2.5, 9.0), (9.0, 9.0)] {
                p.rect_stroke(
                    Rect::from_min_size(at(x, y), vec2(4.5 * u, 4.5 * u)),
                    egui::CornerRadius::same((1.2 * u) as u8),
                    st,
                    egui::StrokeKind::Middle,
                );
            }
        }
        Icon::Gear => {
            p.circle_stroke(c, 2.2 * u, st);
            p.circle_stroke(c, 5.0 * u, st);
            for i in 0..8 {
                let a = std::f32::consts::PI * (i as f32) / 4.0;
                let (sn, cs) = a.sin_cos();
                p.line_segment([pos2(c.x + 5.0 * u * cs, c.y + 5.0 * u * sn), pos2(c.x + 7.0 * u * cs, c.y + 7.0 * u * sn)], st);
            }
        }
        Icon::Keyboard => {
            let r = Rect::from_min_max(at(1.5, 4.0), at(14.5, 12.0));
            p.rect_stroke(r, egui::CornerRadius::same((1.6 * u) as u8), st, egui::StrokeKind::Middle);
            let dot = (0.75 * u).max(0.9);
            for (x, y) in [(4.2, 6.6), (6.8, 6.6), (9.4, 6.6), (12.0, 6.6)] {
                p.circle_filled(at(x, y), dot, color);
            }
            line(&[at(5.0, 9.6), at(11.0, 9.6)]);
        }
        Icon::Info => {
            p.circle_stroke(c, 6.2 * u, st);
            line(&[at(8.0, 7.4), at(8.0, 11.4)]);
            p.circle_filled(at(8.0, 5.0), (0.85 * u).max(1.0), color);
        }
        Icon::Plus => {
            line(&[at(8.0, 3.0), at(8.0, 13.0)]);
            line(&[at(3.0, 8.0), at(13.0, 8.0)]);
        }
        Icon::Close => {
            line(&[at(4.5, 4.5), at(11.5, 11.5)]);
            line(&[at(11.5, 4.5), at(4.5, 11.5)]);
        }
        Icon::ChevronRight => {
            line(&[at(6.0, 4.0), at(10.0, 8.0), at(6.0, 12.0)]);
        }
        Icon::MoreDots => {
            for x in [3.5, 8.0, 12.5] {
                p.circle_filled(at(x, 8.0), 1.25 * u, color);
            }
        }
        Icon::FolderAdd => {
            line(&[at(2.0, 6.5), at(2.0, 3.6), at(6.2, 3.6), at(7.6, 5.2), at(14.0, 5.2), at(14.0, 12.8), at(2.0, 12.8), at(2.0, 11.6)]);
            line(&[at(1.2, 9.2), at(8.2, 9.2)]);
            line(&[at(6.0, 7.0), at(8.2, 9.2), at(6.0, 11.4)]);
        }
        Icon::NewDoc => {
            line(&[at(9.5, 13.8), at(3.2, 13.8), at(3.2, 2.2), at(9.2, 2.2), at(12.8, 5.8), at(12.8, 8.6)]);
            line(&[at(9.2, 2.2), at(9.2, 5.8), at(12.8, 5.8)]);
            line(&[at(12.5, 10.2), at(12.5, 14.8)]);
            line(&[at(10.2, 12.5), at(14.8, 12.5)]);
        }
        Icon::Sun => {
            p.circle_stroke(c, 3.0 * u, st);
            for i in 0..8 {
                let a = std::f32::consts::PI * (i as f32) / 4.0;
                let (sn, cs) = a.sin_cos();
                p.line_segment([pos2(c.x + 5.0 * u * cs, c.y + 5.0 * u * sn), pos2(c.x + 6.6 * u * cs, c.y + 6.6 * u * sn)], st);
            }
        }
        Icon::Moon => {
            // Crescent = outer circle minus a shifted inner circle.
            let (r1, r2) = (6.0 * u, 4.4 * u);
            let o2 = vec2(3.6 * u, -2.6 * u);
            let d = o2.length();
            let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
            let h = (r1 * r1 - a * a).max(0.0).sqrt();
            let dir = o2 / d;
            let base = c + dir * a;
            let perp = vec2(-dir.y, dir.x);
            let (i1, i2) = (base + perp * h, base - perp * h);
            let ang = |center: Pos2, q: Pos2| (q.y - center.y).atan2(q.x - center.x);
            let tau = std::f32::consts::TAU;
            // Outer arc: the sweep from i1 to i2 whose midpoint lies outside the inner circle.
            let c2 = c + o2;
            let pick = |center: Pos2, from: Pos2, to: Pos2, keep: &dyn Fn(Pos2) -> bool, r: f32| {
                let s = ang(center, from);
                let mut e = ang(center, to);
                if e < s {
                    e += tau;
                }
                let mid = (s + e) / 2.0;
                if keep(center + vec2(mid.cos(), mid.sin()) * r) { (s, e) } else { (s, e - tau) }
            };
            let (s1, e1) = pick(c, i1, i2, &|q: Pos2| (q - c2).length() > r2, r1);
            let (s2, e2) = pick(c2, i2, i1, &|q: Pos2| (q - c).length() < r1, r2);
            let mut pts: Vec<Pos2> = (0..=32).map(|i| s1 + (e1 - s1) * i as f32 / 32.0).map(|t| c + vec2(t.cos(), t.sin()) * r1).collect();
            pts.extend((1..32).map(|i| s2 + (e2 - s2) * i as f32 / 32.0).map(|t| c2 + vec2(t.cos(), t.sin()) * r2));
            p.add(Shape::closed_line(pts, st));
        }
        Icon::Auto => {
            p.circle_stroke(c, 6.0 * u, st);
            let half: Vec<Pos2> = (0..=24)
                .map(|i| {
                    let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * (i as f32) / 24.0;
                    pos2(c.x + 6.0 * u * a.cos(), c.y + 6.0 * u * a.sin())
                })
                .collect();
            p.add(Shape::convex_polygon(half, color, Stroke::NONE));
        }
        Icon::Search => {
            p.circle_stroke(at(7.0, 7.0), 4.4 * u, st);
            line(&[at(10.3, 10.3), at(13.8, 13.8)]);
        }
        Icon::Refresh => {
            let pts: Vec<Pos2> = (0..=28)
                .map(|i| {
                    let a = 0.3 + 5.2 * (i as f32) / 28.0;
                    pos2(c.x + 5.4 * u * a.cos(), c.y + 5.4 * u * a.sin())
                })
                .collect();
            line(&pts);
            line(&[at(13.6, 3.2), at(13.4, 6.4), at(10.2, 6.2)]);
        }
        Icon::Sparkles => {
            // A large four-point star and two small ones.
            let star = |cx: f32, cy: f32, r: f32| {
                let pts = [
                    at(cx, cy - r),
                    at(cx + r * 0.28, cy - r * 0.28),
                    at(cx + r, cy),
                    at(cx + r * 0.28, cy + r * 0.28),
                    at(cx, cy + r),
                    at(cx - r * 0.28, cy + r * 0.28),
                    at(cx - r, cy),
                    at(cx - r * 0.28, cy - r * 0.28),
                ];
                p.add(Shape::closed_line(pts.to_vec(), st));
            };
            star(6.6, 9.0, 5.0);
            star(12.4, 4.0, 2.3);
        }
        Icon::Send => {
            p.add(Shape::closed_line(vec![at(2.0, 8.0), at(14.0, 2.5), at(10.0, 13.5), at(7.6, 9.0)], st));
            line(&[at(7.6, 9.0), at(14.0, 2.5)]);
        }
        Icon::Stop => {
            p.rect_filled(Rect::from_min_max(at(4.0, 4.0), at(12.0, 12.0)), egui::CornerRadius::same((1.4 * u) as u8), color);
        }
        Icon::Chevron => {
            line(&[at(4.0, 6.0), at(8.0, 10.0), at(12.0, 6.0)]);
        }
        Icon::Open => {
            line(&[at(2.0, 12.8), at(2.0, 3.6), at(6.2, 3.6), at(7.6, 5.2), at(12.5, 5.2), at(12.5, 7.0)]);
            line(&[at(2.0, 12.8), at(4.6, 7.4), at(14.6, 7.4), at(12.0, 12.8), at(2.0, 12.8)]);
        }
    }
}
