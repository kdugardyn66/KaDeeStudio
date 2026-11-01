//! Application state and the single place where commands run.

use crate::ui::keys::Recording;
use kadee_ai::{AiHub, ChatState};
use kadee_commands::{Command, Keymap, Matcher, Os, Page, SettingsSection, display_sequence};
use kadee_core::{
    DocGroup, DocKind, FileEntry, HomeState, InstalledApp, ModuleKind, Settings, Store, StoredShortcut, all_extensions,
    create_new_document, create_office_document, detect_installed, list_documents, now,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The system the app runs on: decides modifier key names and the main shortcut key.
pub const OS: Os = Os::current();

/// Columns the file list can be sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Opened,
    Modified,
    Size,
}

/// The Settings window while it is open, with what to go back to on Cancel.
pub struct SettingsWindow {
    pub section: SettingsSection,
    before: Settings,
    before_keymap: Keymap,
}

/// The whole app.
pub struct KadeeApp {
    pub store: Store,
    pub state: HomeState,
    /// Open file tabs; tab 0 is Home and is not stored here.
    pub tabs: Vec<PathBuf>,
    /// 0 = Home, n = `tabs[n - 1]`.
    pub active: usize,
    pub closed: Vec<PathBuf>,
    pub page: Page,
    /// Selected filter pill; `None` = All (every type, Crafting Apps included).
    pub filter: Option<DocGroup>,
    pub search: String,
    pub sort: SortKey,
    pub sort_desc: bool,
    /// Documents in the documents folder (rescanned on demand).
    pub docs: Vec<FileEntry>,
    pub toast: Option<(String, f64)>,
    pub queue: Vec<Command>,
    /// Keyboard shortcuts in use (defaults or the user's own).
    pub keymap: Keymap,
    pub matcher: Matcher,
    /// A shortcut being recorded on the Keyboard shortcuts page.
    pub recording: Option<Recording>,
    /// The New document chooser is open.
    pub new_picker: bool,
    /// Keys that opened the New document menu (Cmd+N); a letter after them picks the type.
    pub picker_prefix: Vec<kadee_commands::Chord>,
    /// Action picked in "Add a shortcut".
    pub add_choice: usize,
    /// The Windows/Super key is held (egui does not report it on Windows and Linux).
    pub meta_down: bool,
    /// The Settings window, when open.
    pub settings_window: Option<SettingsWindow>,
    /// KaDee AI: providers, models and the request machinery (`ai.toml` next to `home.json`).
    pub ai: AiHub,
    /// KaDee AI conversations: `""` is the KaDee AI page, a file path is that file's pane.
    pub chats: HashMap<String, ChatState>,
    /// The KaDee AI pane beside the open file.
    pub ai_pane: bool,
    /// Edits in Settings › AI providers / AI models, applied on Done.
    pub ai_ui: crate::ai_state::AiSettingsUi,
    /// Crafting Apps found on this computer.
    pub installed: Vec<InstalledApp>,
    /// Crafting App icons, loaded on first use.
    pub craft_icons: crate::ui::craft_icons::CraftIcons,
    /// Kadee Markdown: mdcraft's editor in each Markdown tab (macOS and Linux).
    #[cfg(unix)]
    pub markdown: crate::markdown::Markdown,
    /// macOS menu bar (About KadeeStudio, Settings…).
    #[cfg(target_os = "macos")]
    pub mac_menu: Option<crate::mac_menu::MacMenu>,
    dirty: bool,
    theme_applied: Option<kadee_core::ThemePref>,
}

impl KadeeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let store = Store::platform();
        let (state, first) = store.load_or_init();
        let ai = AiHub::load(store.dir().to_path_buf());
        let mut app = Self {
            store,
            state,
            tabs: Vec::new(),
            active: 0,
            closed: Vec::new(),
            page: Page::Home,
            filter: None,
            search: String::new(),
            sort: SortKey::Modified,
            sort_desc: true,
            docs: Vec::new(),
            toast: None,
            queue: Vec::new(),
            keymap: Keymap::defaults(OS),
            matcher: Matcher::default(),
            recording: None,
            new_picker: false,
            picker_prefix: Vec::new(),
            add_choice: 0,
            meta_down: false,
            settings_window: None,
            ai,
            chats: HashMap::new(),
            ai_pane: false,
            ai_ui: Default::default(),
            installed: detect_installed(),
            craft_icons: Default::default(),
            #[cfg(unix)]
            markdown: Default::default(),
            #[cfg(target_os = "macos")]
            mac_menu: None,
            dirty: false,
            theme_applied: None,
        };
        if let Some(saved) = &app.state.settings.shortcuts {
            app.keymap = Keymap::from_stored(saved.iter().map(|s| (s.command.as_str(), s.keys.as_str())), OS);
        }
        app.rescan();
        app.apply_theme(&cc.egui_ctx);
        #[cfg(target_os = "macos")]
        {
            app.mac_menu = crate::mac_menu::MacMenu::install(&cc.egui_ctx, &app.keymap);
        }
        // `kadeestudio file.md other.pdf` opens those files in tabs;
        // `--page=craft` (recent, starred, craft, folder) opens a page; `--page=settings`
        // (settings, keyboard, about) opens the Settings window.
        for arg in std::env::args_os().skip(1) {
            if let Some(name) = arg.to_str().and_then(|a| a.strip_prefix("--page=")) {
                match name {
                    "settings" | "general" => app.open_settings(SettingsSection::General),
                    "keyboard" | "shortcuts" => app.open_settings(SettingsSection::Keyboard),
                    "about" => app.open_settings(SettingsSection::About),
                    "ai-providers" | "providers" => app.open_settings(SettingsSection::AiProviders),
                    "ai-models" | "models" => app.open_settings(SettingsSection::AiModels),
                    _ => {
                        app.page = match name {
                            "recent" => Page::Recent,
                            "starred" => Page::Starred,
                            "craft" => Page::CraftApps,
                            "ai" => Page::Ai,
                            "folder" => Page::Folder,
                            _ => Page::Home,
                        }
                    }
                }
                continue;
            }
            let path = PathBuf::from(arg);
            if path.is_file() && DocKind::from_path(&path).is_some() {
                let path = path.canonicalize().unwrap_or(path);
                app.open_tab(path);
            }
        }
        if first {
            app.notify("Welcome! Your documents live in ~/Documents/KadeeStudio", 0.0);
        }
        app
    }

    pub fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.theme_applied != Some(self.state.settings.theme) {
            kadee_ui::apply_theme(ctx, self.state.settings.theme);
            self.theme_applied = Some(self.state.settings.theme);
        }
    }

    pub fn rescan(&mut self) {
        self.docs = list_documents(&self.state.settings.documents_dir);
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn notify(&mut self, msg: impl Into<String>, time: f64) {
        self.toast = Some((msg.into(), time));
    }

    pub fn send(&mut self, cmd: Command) {
        self.queue.push(cmd);
    }

    /// Keep the current key map in the settings (saved with `home.json`).
    pub fn save_keymap(&mut self) {
        self.state.settings.shortcuts =
            Some(self.keymap.to_stored(OS).into_iter().map(|(command, keys)| StoredShortcut { command, keys }).collect());
        self.mark_dirty();
    }

    /// Back to the shipped shortcuts.
    pub fn reset_keymap(&mut self) {
        self.keymap = Keymap::defaults(OS);
        self.recording = None;
        self.state.settings.shortcuts = None;
        self.mark_dirty();
    }

    /// The shortcut of a command as people see it (`Cmd+N`), if it has one.
    pub fn shortcut_text(&self, cmd: &Command) -> Option<String> {
        self.keymap.keys_for(cmd).map(|k| display_sequence(k, OS))
    }

    /// Open the Settings window at `section`; remembers the settings for Cancel.
    pub fn open_settings(&mut self, section: SettingsSection) {
        match &mut self.settings_window {
            Some(w) => w.section = section,
            None => {
                self.settings_window =
                    Some(SettingsWindow { section, before: self.state.settings.clone(), before_keymap: self.keymap.clone() })
            }
        }
    }

    /// Close the Settings window: Done keeps the changes, Cancel puts back what was there.
    pub fn close_settings(&mut self, keep: bool, ctx: &egui::Context) {
        let Some(w) = self.settings_window.take() else { return };
        self.recording = None;
        let ai_draft = self.ai_ui.draft.take();
        self.ai_ui = Default::default();
        if keep && let Some(draft) = ai_draft {
            let time = ctx.input(|i| i.time);
            if let Err(e) = self.ai.set_config(draft) {
                self.notify(format!("Could not save the AI settings: {e}"), time);
            }
        }
        if !keep {
            let folder_changed = w.before.documents_dir != self.state.settings.documents_dir;
            self.state.settings = w.before;
            self.keymap = w.before_keymap;
            self.apply_theme(ctx);
            if folder_changed {
                self.rescan();
            }
        }
        self.mark_dirty();
    }

    /// Look for installed Crafting Apps again.
    pub fn detect_crafts(&mut self) {
        self.installed = detect_installed();
    }

    /// Save `home.json` if anything changed (settings wait for Done).
    pub fn persist(&mut self) {
        if self.dirty && self.settings_window.is_none() {
            if let Err(e) = self.store.save(&self.state) {
                eprintln!("KadeeStudio: could not save settings: {e}");
            }
            self.dirty = false;
        }
    }

    /// Documents for the current page, filtered, searched and sorted.
    pub fn visible_entries(&self) -> Vec<FileEntry> {
        let lib = &self.state.library;
        let mut entries: Vec<FileEntry> = match self.page {
            Page::Recent => lib.recent.iter().filter_map(|r| FileEntry::from_path(&r.path)).collect(),
            Page::Starred => lib.starred.iter().filter_map(|p| FileEntry::from_path(p)).collect(),
            Page::Folder => self.docs.clone(),
            _ => {
                // Home: the documents folder plus anything opened from elsewhere.
                let mut v = self.docs.clone();
                for r in &lib.recent {
                    if !v.iter().any(|e| e.path == r.path)
                        && let Some(e) = FileEntry::from_path(&r.path)
                    {
                        v.push(e);
                    }
                }
                v
            }
        };
        if let Some(group) = self.filter {
            entries.retain(|e| e.kind.group == group);
        }
        let q = self.search.trim().to_lowercase();
        if !q.is_empty() {
            entries.retain(|e| e.name.to_lowercase().contains(&q));
        }
        if self.page != Page::Recent || self.sort != SortKey::Opened {
            let opened = |e: &FileEntry| lib.opened_at(&e.path).unwrap_or(0);
            entries.sort_by(|a, b| {
                let o = match self.sort {
                    SortKey::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortKey::Opened => opened(a).cmp(&opened(b)),
                    SortKey::Modified => a.modified.cmp(&b.modified),
                    SortKey::Size => a.size.cmp(&b.size),
                };
                if self.sort_desc { o.reverse() } else { o }
            });
        }
        entries
    }

    pub fn open_tab(&mut self, path: PathBuf) {
        // A document the Markdown editor is about to create has no file yet: it joins Recent
        // when it is saved.
        if path.exists() {
            self.state.library.touch(&path, now());
            self.mark_dirty();
        }
        if let Some(i) = self.tabs.iter().position(|p| *p == path) {
            self.active = i + 1;
        } else {
            self.tabs.push(path);
            self.active = self.tabs.len();
        }
    }

    fn close_tab(&mut self, n: usize) {
        if n == 0 || n > self.tabs.len() {
            return;
        }
        // A Markdown tab asks about unsaved changes first; its page closes it (`quit`).
        #[cfg(unix)]
        if self.markdown.request_close(&self.tabs[n - 1]) {
            self.active = n;
            return;
        }
        let path = self.tabs.remove(n - 1);
        #[cfg(unix)]
        self.markdown.forget(&path);
        if path.exists() {
            self.closed.push(path);
        }
        if self.active >= n {
            self.active = self.active.saturating_sub(1).min(self.tabs.len());
        }
    }

    /// Close the tab of `path` without asking (its editor already did).
    pub fn remove_tab(&mut self, path: &Path) {
        if let Some(i) = self.tabs.iter().position(|p| p == path) {
            let n = i + 1;
            let path = self.tabs.remove(i);
            if path.exists() {
                self.closed.push(path);
            }
            if self.active >= n {
                self.active = self.active.saturating_sub(1).min(self.tabs.len());
            }
        }
    }

    /// The name shown on a file's tab: the Markdown editor's (with • while unsaved), else the file's.
    pub fn tab_name(&self, path: &Path) -> String {
        #[cfg(unix)]
        if let Some(n) = self.markdown.tab_name(path) {
            return n;
        }
        file_name(path)
    }

    /// Run every queued command.
    pub fn run_queue(&mut self, ctx: &egui::Context) {
        let time = ctx.input(|i| i.time);
        while !self.queue.is_empty() {
            let cmd = self.queue.remove(0);
            self.run(cmd, ctx, time);
        }
    }

    fn run(&mut self, cmd: Command, ctx: &egui::Context, time: f64) {
        match cmd {
            Command::NewDocumentPicker => {
                self.picker_prefix = self.keymap.keys_for(&Command::NewDocumentPicker).map(<[_]>::to_vec).unwrap_or_default();
                self.new_picker = true;
            }
            Command::NewOffice(kind) => match create_office_document(kind, &self.state.settings.documents_dir) {
                Ok(path) => {
                    let name = file_name(&path);
                    self.open_tab(path);
                    self.rescan();
                    self.notify(format!("Created {name}"), time);
                }
                Err(e) => self.notify(format!("Could not create the file: {e}"), time),
            },
            Command::NewDocument(kind) => match create_new_document(kind, &self.state.settings.documents_dir) {
                Ok(path) => {
                    let name = file_name(&path);
                    self.open_tab(path);
                    self.rescan();
                    self.notify(format!("Created {name}"), time);
                }
                Err(e) => self.notify(format!("Could not create the file: {e}"), time),
            },
            Command::OpenModule(ModuleKind::Pdf) => {
                if let Some(path) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    self.open_tab(path);
                }
            }
            Command::OpenModule(kind) => {
                self.notify(format!("{} arrives in Phase {} of the roadmap", kind.name(), kind.phase()), time);
            }
            Command::OpenDialog => {
                let dialog = rfd::FileDialog::new()
                    .set_directory(&self.state.settings.documents_dir)
                    .add_filter("All supported documents", &all_extensions());
                if let Some(path) = dialog.pick_file() {
                    self.send(Command::OpenPath(path));
                }
            }
            Command::OpenPath(path) => {
                if DocKind::from_path(&path).is_some() {
                    self.open_tab(path);
                } else {
                    self.notify("KadeeStudio cannot open this file type yet", time);
                }
            }
            Command::OpenSettings(section) => self.open_settings(section),
            Command::ToggleAiPane => self.ai_pane = !self.ai_pane,
            Command::LaunchCraft(id) => match self.installed.iter().find(|i| i.app.id == id) {
                Some(found) => match found.launch(None) {
                    Ok(()) => self.notify(format!("Opening {}…", found.app.name), time),
                    Err(e) => self.notify(format!("Could not open {}: {e}", found.app.name), time),
                },
                None => self.notify("That Crafting App is not installed on this computer", time),
            },
            Command::ShowPage(page) => {
                self.page = page;
                self.active = 0;
                if page == Page::Folder || page == Page::Home {
                    self.rescan();
                }
            }
            Command::ActivateTab(n) => {
                if n <= self.tabs.len() {
                    self.active = n;
                }
            }
            Command::NextTab => self.active = (self.active + 1) % (self.tabs.len() + 1),
            Command::PreviousTab => self.active = (self.active + self.tabs.len()) % (self.tabs.len() + 1),
            Command::CloseTab(n) => self.close_tab(n),
            Command::CloseActiveTab => self.close_tab(self.active),
            Command::ReopenClosedTab => {
                if let Some(p) = self.closed.pop() {
                    self.open_tab(p);
                }
            }
            Command::ToggleStar(path) => {
                let on = self.state.library.toggle_star(&path);
                self.mark_dirty();
                self.notify(if on { "Added to Starred" } else { "Removed from Starred" }, time);
            }
            Command::RemoveFromRecent(path) => {
                self.state.library.remove_recent(&path);
                self.mark_dirty();
            }
            Command::OpenExternally(path) => {
                if let Err(e) = open::that_detached(&path) {
                    self.notify(format!("Could not open: {e}"), time);
                }
            }
            Command::RevealInFolder(path) => {
                let dir = path.parent().map(Path::to_path_buf).unwrap_or(path);
                if let Err(e) = open::that_detached(dir) {
                    self.notify(format!("Could not open the folder: {e}"), time);
                }
            }
            Command::CopyPath(path) => {
                ctx.copy_text(path.display().to_string());
                self.notify("Path copied", time);
            }
            Command::SetTheme(t) => {
                self.state.settings.theme = t;
                self.mark_dirty();
                self.apply_theme(ctx);
            }
            Command::CycleTheme => {
                let t = self.state.settings.theme.cycle();
                self.send(Command::SetTheme(t));
                self.notify(format!("Theme: {}", t.label()), time);
            }
            Command::OpenUrl(url) => ctx.open_url(egui::OpenUrl::new_tab(url)),
            Command::Refresh => {
                self.rescan();
                self.detect_crafts();
                self.notify("Refreshed", time);
            }
        }
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string())
}

impl KadeeApp {
    /// The chat for `key` (`""` = the KaDee AI page, else a file path).
    pub fn chat(&mut self, key: &str) -> &mut ChatState {
        self.chats.entry(key.to_string()).or_insert_with(|| ChatState::new(None, key.is_empty()))
    }

    /// Take in news from running AI requests; true while any is still running.
    pub fn pump_ai(&mut self) -> bool {
        let mut running = false;
        for c in self.chats.values_mut() {
            c.pump();
            running |= c.running();
        }
        running
    }
}
