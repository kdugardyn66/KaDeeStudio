//! KadeeStudio — desktop shell (Phase 0: the start screen).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai_state;
mod app;
mod icons;
#[cfg(target_os = "macos")]
mod mac_menu;
mod ui;

use app::KadeeApp;

impl eframe::App for KadeeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "macos")]
        {
            let actions = match self.mac_menu.as_mut() {
                Some(menu) => {
                    menu.sync(&self.keymap, self.recording.is_some());
                    menu.poll()
                }
                None => Vec::new(),
            };
            for action in actions {
                match action {
                    mac_menu::Action::Run(cmd) => self.send(cmd),
                    mac_menu::Action::Edit(edit) => mac_menu::edit(ui.ctx(), edit),
                }
            }
        }
        if self.pump_ai() {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(40));
        }
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

/// The window/taskbar icon: the KadeeStudio logo (module grid with an AI spark).
fn app_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/app-icon/kadeestudio-tile-512.png")).unwrap_or_default()
}
