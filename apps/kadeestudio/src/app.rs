//! Application state and the single place where commands run.

use kadee_commands::{Command, Page};
use kadee_core::{FileEntry, HomeState, ModuleKind, Store, create_new_document, list_documents, now};
use std::path::{Path, PathBuf};

/// Columns the file list can be sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Opened,
    Modified,
    Size,
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
    pub filter: Option<ModuleKind>,
    pub search: String,
    pub sort: SortKey,
    pub sort_desc: bool,
    /// Documents in the documents folder (rescanned on demand).
    pub docs: Vec<FileEntry>,
    pub toast: Option<(String, f64)>,
    pub queue: Vec<Command>,
    dirty: bool,
    theme_applied: Option<kadee_core::ThemePref>,
}

impl KadeeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let store = Store::platform();
        let (state, first) = store.load_or_init();
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
            dirty: false,
            theme_applied: None,
        };
        app.rescan();
        app.apply_theme(&cc.egui_ctx);
        // `kadeestudio file.md other.pdf` opens those files in tabs.
        for arg in std::env::args_os().skip(1) {
            let path = PathBuf::from(arg);
            if path.is_file() && ModuleKind::from_path(&path).is_some() {
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

    /// Save `home.json` if anything changed.
    pub fn persist(&mut self) {
        if self.dirty {
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
        if let Some(kind) = self.filter {
            entries.retain(|e| e.kind == kind);
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
        self.state.library.touch(&path, now());
        self.mark_dirty();
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
        let path = self.tabs.remove(n - 1);
        self.closed.push(path);
        if self.active >= n {
            self.active = self.active.saturating_sub(1).min(self.tabs.len());
        }
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
                    .add_filter("KadeeStudio documents", &["md", "markdown", "pdf", "kdash", "kreport"]);
                if let Some(path) = dialog.pick_file() {
                    self.send(Command::OpenPath(path));
                }
            }
            Command::OpenPath(path) => {
                if ModuleKind::from_path(&path).is_some() {
                    self.open_tab(path);
                } else {
                    self.notify("KadeeStudio cannot open this file type yet", time);
                }
            }
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
                self.notify("Refreshed", time);
            }
        }
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string())
}
