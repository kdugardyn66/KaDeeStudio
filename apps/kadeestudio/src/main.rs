//! KadeeStudio — desktop shell (Phase 0: the start screen).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod icons;
mod ui;

use app::KadeeApp;

impl eframe::App for KadeeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui::shortcuts(self, ui.ctx());
        ui::shell(self, ui);
        let ctx = ui.ctx().clone();
        self.run_queue(&ctx);
        self.persist();
    }
}

fn main() -> eframe::Result {
    #[allow(unused_mut)]
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("KadeeStudio")
            .with_app_id("be.kadee.KadeeStudio")
            .with_inner_size([1320.0, 860.0])
            .with_min_inner_size([920.0, 600.0])
            .with_icon(app_icon()),
        ..Default::default()
    };
    #[cfg(feature = "glow")]
    {
        options.renderer = eframe::Renderer::Glow;
    }
    eframe::run_native("KadeeStudio", options, Box::new(|cc| Ok(Box::new(KadeeApp::new(cc)))))
}

/// A 64×64 icon drawn in code: a rounded blue square with a white "K".
fn app_icon() -> egui::IconData {
    const N: usize = 64;
    let mut rgba = vec![0u8; N * N * 4];
    let inside_rounded = |x: f32, y: f32| {
        let r = 14.0;
        let cx = x.clamp(r, N as f32 - r);
        let cy = y.clamp(r, N as f32 - r);
        (x - cx).powi(2) + (y - cy).powi(2) <= r * r
    };
    // "K": a stem and two diagonals, as distance-to-segment strokes.
    let seg = |px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32| {
        let (dx, dy) = (bx - ax, by - ay);
        let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
    };
    for y in 0..N {
        for x in 0..N {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let i = (y * N + x) * 4;
            if !inside_rounded(fx, fy) {
                continue;
            }
            let d = seg(fx, fy, 22.0, 16.0, 22.0, 48.0).min(seg(fx, fy, 23.0, 34.0, 42.0, 16.0)).min(seg(fx, fy, 28.0, 30.0, 43.0, 48.0));
            let on = d < 3.4;
            let (r, g, b) = if on { (255, 255, 255) } else { (0x2F, 0x5D, 0xD6) };
            rgba[i..i + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    egui::IconData { rgba, width: N as u32, height: N as u32 }
}
