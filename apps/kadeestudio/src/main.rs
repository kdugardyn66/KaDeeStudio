//! KadeeStudio — desktop shell (Phase 0: the start screen).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai_state;
mod app;
mod icons;
#[cfg(target_os = "macos")]
mod mac_menu;
#[cfg(target_os = "macos")]
mod mac_open;
#[cfg(unix)]
mod markdown;
mod ui;

use app::KadeeApp;

impl eframe::App for KadeeApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        #[cfg(target_os = "macos")]
        {
            // read what the menu shows before borrowing the menu itself
            let has_editor = self.has_editor();
            let recent: Vec<std::path::PathBuf> =
                self.state.library.recent.iter().take(mac_menu::RECENT_IN_MENU).map(|e| e.path.clone()).collect();
            let actions = match self.mac_menu.as_mut() {
                Some(menu) => {
                    menu.sync(&self.keymap, self.recording.is_some());
                    menu.sync_documents(has_editor, &recent);
                    menu.poll()
                }
                None => Vec::new(),
            };
            for action in actions {
                match action {
                    mac_menu::Action::Run(cmd) => self.send(cmd),
                    mac_menu::Action::Edit(edit) => {
                        // In a Markdown tab the menu acts on the editor, unless an egui field
                        // (the KaDee AI pane) has the keyboard.
                        let on_editor = !ui.ctx().egui_wants_keyboard_input()
                            && self.active > 0
                            && self.tabs.get(self.active - 1).cloned().is_some_and(|p| self.markdown.edit(&p, edit));
                        if !on_editor {
                            mac_menu::edit(ui.ctx(), edit);
                        }
                    }
                }
            }
        }
        if self.pump_ai() {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(40));
        }
        ui::shortcuts(self, ui.ctx());
        ui::shell(self, ui);
        let ctx = ui.ctx().clone();
        #[cfg(unix)]
        markdown::frame(self, &ctx, frame);
        #[cfg(not(unix))]
        let _ = frame;
        self.run_queue(&ctx);
        self.persist();
    }
}

fn main() -> eframe::Result {
    prepare_markdown();
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

/// Kadee Markdown runs mdcraft: its settings, recent files and recovery copies live in
/// `Markdown` next to `home.json`. On Linux its web view needs X11, so winit and GTK use X11
/// (XWayland on a Wayland desktop), and WebKitGTK draws without DMA-BUF, which shows a blank
/// view with some GPU drivers.
fn prepare_markdown() {
    #[cfg(unix)]
    {
        let dir = kadee_core::Store::platform().dir().join("Markdown");
        // SAFETY: called first thing in `main`, before any other thread exists.
        unsafe {
            if std::env::var_os("MDCRAFT_CONFIG_DIR").is_none() {
                std::env::set_var("MDCRAFT_CONFIG_DIR", &dir);
            }
            #[cfg(target_os = "linux")]
            {
                if std::env::var_os("KADEE_KEEP_WAYLAND").is_none() {
                    std::env::remove_var("WAYLAND_DISPLAY");
                    std::env::set_var("GDK_BACKEND", "x11");
                }
                if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
                    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
                }
            }
        }
    }
}

/// The window/taskbar icon: the KadeeStudio logo (module grid with an AI spark).
fn app_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/app-icon/kadeestudio-tile-512.png")).unwrap_or_default()
}
