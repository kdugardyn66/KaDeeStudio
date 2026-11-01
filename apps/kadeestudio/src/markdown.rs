//! Kadee Markdown: mdcraft's editor inside the Markdown tab.
//!
//! Every Markdown tab runs its own mdcraft core (the renderer and the editor API on
//! 127.0.0.1 with a random token) and shows mdcraft's page in a web view laid over the tab's
//! area: the ribbon (Home, Insert, Design, Slides, Review, View, Table), the CodeMirror
//! editor, the live preview, slides and the slide show, the File view with templates and
//! My templates, export and printing. mdcraft's own AI parts (its MCP server and agent
//! bridge) are not included; the KaDee AI pane next to the tab is the assistant, and it reads
//! the editor's current text ([`Markdown::text`]) and can insert answers ([`Markdown::insert`]).
//!
//! The page asks the app for what a web page can't do through `window.ipc` messages, the same
//! ones mdcraft's own window answers: Open and Save dialogs, printing, full screen, new
//! windows (here: new tabs), and closing with unsaved changes.
//!
//! ```text
//!  ┌──── KadeeStudio window (egui) ─────────────────────────────┐
//!  │ tab strip                                                   │
//!  │ ┌──── Markdown tab: wry web view (child) ────┐ ┌─ KaDee AI ─┐│
//!  │ │ mdcraft page ⇄ mdcraft-core (HTTP, token)  │ │  pane      ││
//!  │ └────────────────────────────────────────────┘ └───────────┘│
//!  └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! On Linux the web view needs X11 (`main` makes winit and GTK use it) and GTK's loop is run
//! next to egui's every frame.

use crate::app::KadeeApp;
use kadee_commands::{Chord, Command};
use mdcraft_core::{App as Core, Mode};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender, channel};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::{DragDropEvent, NewWindowResponse, Rect, WebView, WebViewBuilder};

/// File extensions Kadee Markdown edits (documents and the user's own templates).
pub const EXTENSIONS: [&str; 4] = ["md", "markdown", "mdd", "mds"];

/// True when `path` opens in Kadee Markdown.
pub fn is_markdown(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).map(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str())).unwrap_or(false)
}

/// What the web view's callbacks hand to the app.
enum Event {
    /// A message from the page (`window.ipc.postMessage`).
    Ipc(String),
    /// Files dropped on the editor.
    Dropped(Vec<PathBuf>),
    /// A link to open in the system browser.
    External(String),
    /// Text cut or copied in the page with the Edit menu (macOS), as JSON.
    #[cfg(target_os = "macos")]
    Copied(String),
}

/// One Markdown tab's editor.
pub struct Editor {
    core: Arc<Core>,
    view: Option<WebView>,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    /// The page has booted and answers `requestClose`.
    ready: bool,
    /// The page's title (`• Name.md` while there are unsaved changes).
    title: String,
    /// Shown in the tab instead of the editor when it could not start.
    error: Option<String>,
    visible: bool,
    bounds: Option<[i32; 4]>,
    /// The shortcuts the page hands back to the app, as last sent to it.
    keys_sent: String,
    /// While the window closes: this page was asked about its unsaved changes.
    asked: bool,
}

impl Editor {
    fn start(startup: Value) -> Result<Editor, String> {
        let core = mdcraft_core::start(Mode::Desktop)?;
        if let Ok(mut s) = core.startup.lock() {
            *s = startup;
        }
        let (tx, rx) = channel();
        Ok(Editor {
            core,
            view: None,
            tx,
            rx,
            ready: false,
            title: String::new(),
            error: None,
            visible: false,
            bounds: None,
            keys_sent: String::new(),
            asked: false,
        })
    }

    /// True while the document has changes that are not saved.
    pub fn dirty(&self) -> bool {
        self.title.starts_with('•')
    }

    /// The document's name as the page shows it (`Report.md`).
    pub fn name(&self) -> Option<&str> {
        let t = self.title.trim_start_matches('•').trim();
        let t = t.strip_suffix(" — Kadee Markdown").unwrap_or(t);
        (!t.is_empty() && t != "Kadee Markdown").then_some(t)
    }

    /// The file the page has open and its current text (unsaved edits included).
    fn document(&self) -> Option<(PathBuf, String)> {
        let st = self.core.state.lock().ok()?;
        let path = st.target.clone()?;
        let text = self.core.text_of(&st, &path)?;
        Some((path, text))
    }

    fn eval(&self, js: &str) {
        if let Some(v) = &self.view {
            let _ = v.evaluate_script(js);
        }
    }

    fn reply(&self, id: u64, value: Value, error: Option<String>) {
        self.eval(&format!("window.__mdReply({id}, {value}, {})", error.map(|e| json!(e)).unwrap_or(Value::Null)));
    }

    fn set_visible(&mut self, on: bool) {
        if self.visible != on
            && let Some(v) = &self.view
        {
            let _ = v.set_visible(on);
            if !on {
                // Hidden views must not keep the keyboard: give it back to egui.
                let _ = v.focus_parent();
            }
            // GTK may give a shown view its old size back: place it again.
            self.bounds = None;
            self.visible = on;
        }
    }

    fn set_bounds(&mut self, r: egui::Rect, zoom: f32) {
        let b = [(r.min.x * zoom) as i32, (r.min.y * zoom) as i32, (r.width() * zoom) as i32, (r.height() * zoom) as i32];
        if self.bounds != Some(b)
            && let Some(v) = &self.view
        {
            let _ = v.set_bounds(rect(b));
            self.bounds = Some(b);
        }
    }

    /// Build the web view as a child of the KadeeStudio window.
    fn build(&mut self, frame: &eframe::Frame, ctx: &egui::Context, keys: &str, b: [i32; 4]) -> Result<(), String> {
        let origin = format!("http://127.0.0.1:{}", self.core.port.load(Ordering::SeqCst));
        let send = |tx: &Sender<Event>, ctx: &egui::Context, e: Event| {
            let _ = tx.send(e);
            ctx.request_repaint();
        };
        let (ipc_tx, ipc_ctx) = (self.tx.clone(), ctx.clone());
        let (drop_tx, drop_ctx) = (self.tx.clone(), ctx.clone());
        let (nav_tx, nav_ctx) = (self.tx.clone(), ctx.clone());
        let (win_tx, win_ctx) = (self.tx.clone(), ctx.clone());
        let view = WebViewBuilder::new()
            .with_url(self.core.url())
            .with_bounds(rect(b))
            .with_initialization_script(format!("window.__kadeeKeys = {keys};\n{SHORTCUT_BRIDGE}\n{}", edit_bridge()))
            .with_devtools(cfg!(debug_assertions) || std::env::var_os("KADEE_DEVTOOLS").is_some())
            .with_clipboard(true)
            .with_accept_first_mouse(true)
            .with_background_color((246, 246, 246, 255))
            .with_ipc_handler(move |req| send(&ipc_tx, &ipc_ctx, Event::Ipc(req.body().clone())))
            .with_drag_drop_handler(move |e| {
                if let DragDropEvent::Drop { paths, .. } = e {
                    send(&drop_tx, &drop_ctx, Event::Dropped(paths));
                }
                true
            })
            // The view only shows mdcraft itself; other pages open in the system browser.
            .with_navigation_handler(move |url| {
                if url.starts_with(&origin) || url.starts_with("about:") || url.starts_with("blob:") || url.starts_with("data:") {
                    return true;
                }
                send(&nav_tx, &nav_ctx, Event::External(url));
                false
            })
            .with_new_window_req_handler(move |url, _| {
                if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:") {
                    send(&win_tx, &win_ctx, Event::External(url));
                }
                NewWindowResponse::Deny
            })
            .build_as_child(frame)
            .map_err(|e| format!("The Markdown editor could not start: {e}"))?;
        self.view = Some(view);
        self.visible = true;
        self.bounds = Some(b);
        self.keys_sent = keys.to_string();
        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        // a dropped child view does not always disappear by itself (GTK): hide it first
        if let Some(v) = self.view.take() {
            let _ = v.set_visible(false);
            let _ = v.focus_parent();
        }
        self.core.shutdown();
    }
}

/// The Edit menu's actions on the page (macOS, where the menu's shortcuts reach the app rather
/// than the web view): on the focused text field, else on the editor. Returns the text cut or
/// copied.
#[cfg(target_os = "macos")]
const EDIT_BRIDGE: &str = r#"window.__kadeeEdit = function (what, text) {
  const a = document.activeElement;
  const field = a && (a.tagName === 'INPUT' || a.tagName === 'TEXTAREA') ? a : null;
  if (field) {
    const s = field.value.slice(field.selectionStart || 0, field.selectionEnd || 0);
    const put = (t) => { field.setRangeText(t, field.selectionStart || 0, field.selectionEnd || 0, 'end'); field.dispatchEvent(new Event('input', { bubbles: true })); };
    switch (what) {
      case 'selectAll': field.select(); return '';
      case 'undo': case 'redo': document.execCommand(what); return '';
      case 'copy': return s;
      case 'cut': put(''); return s;
      case 'paste': put(text || ''); return '';
    }
    return '';
  }
  const E = window.MD && MD.editor;
  if (!E || !MD.doc || !MD.doc.open) return '';
  switch (what) {
    case 'undo': E.undo(); break;
    case 'redo': E.redo(); break;
    case 'selectAll': E.selectAll(); break;
    case 'copy': return E.selectedText();
    case 'cut': { const s = E.selectedText(); if (s) E.insert(''); return s; }
    case 'paste': E.insert(text || ''); break;
  }
  return '';
};"#;

/// Ask the page to close: it asks about unsaved changes and sends `quit`, then always
/// `kadee-close-done` (after `quit`, or alone when the user chose Cancel).
const CLOSE_JS: &str = "window.__mdNative && Promise.resolve(window.__mdNative.requestClose()).then(() => \
    window.ipc.postMessage(JSON.stringify({cmd: 'kadee-close-done', id: 0})))";

fn edit_bridge() -> &'static str {
    #[cfg(target_os = "macos")]
    return EDIT_BRIDGE;
    #[cfg(not(target_os = "macos"))]
    ""
}

fn rect(b: [i32; 4]) -> Rect {
    Rect { position: LogicalPosition::new(b[0], b[1]).into(), size: LogicalSize::new(b[2].max(1) as u32, b[3].max(1) as u32).into() }
}

/// Hands the app's shortcuts (Cmd+W, Ctrl+Tab, Cmd+J …) back to it while the editor has the
/// keyboard; every other key stays with mdcraft. `window.__kadeeKeys` holds the chords.
const SHORTCUT_BRIDGE: &str = r#"(function () {
  const names = { Comma: 'Comma', Period: 'Period', Minus: 'Minus', Equal: 'Equals', Slash: 'Slash', Backslash: 'Backslash',
    Semicolon: 'Semicolon', Quote: 'Quote', Backquote: 'Backtick', BracketLeft: 'OpenBracket', BracketRight: 'CloseBracket',
    Tab: 'Tab', Enter: 'Enter', Escape: 'Escape', Space: 'Space', Backspace: 'Backspace', Delete: 'Delete' };
  function keyName(e) {
    const c = e.code || '';
    if (c.startsWith('Key')) return c.slice(3);
    if (c.startsWith('Digit')) return c.slice(5);
    if (/^F\d+$/.test(c)) return c;
    return names[c] || null;
  }
  window.addEventListener('keydown', function (e) {
    if (!(e.ctrlKey || e.altKey || e.metaKey) || !window.ipc) return;
    const key = keyName(e);
    if (!key) return;
    const hit = (window.__kadeeKeys || []).find((k) => k.key === key && k.ctrl === e.ctrlKey && k.alt === e.altKey && k.shift === e.shiftKey && k.meta === e.metaKey);
    if (!hit) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    window.ipc.postMessage(JSON.stringify({ cmd: 'kadee-key', id: 0, ctrl: e.ctrlKey, alt: e.altKey, shift: e.shiftKey, meta: e.metaKey, key }));
  }, true);
})();"#;

/// The first chord of every KadeeStudio shortcut that uses a modifier key, as JSON for the page.
fn shortcut_json(app: &KadeeApp) -> String {
    let mut seen: Vec<&Chord> = Vec::new();
    for b in &app.keymap.bindings {
        if let Some(c) = b.keys.first()
            && c.has_modifier()
            && !seen.contains(&c)
        {
            seen.push(c);
        }
    }
    let list: Vec<Value> =
        seen.iter().map(|c| json!({"ctrl": c.ctrl, "alt": c.alt, "shift": c.shift, "meta": c.meta, "key": c.key})).collect();
    Value::Array(list).to_string()
}

/// All Markdown editors and what the shell decided about them this frame.
#[derive(Default)]
pub struct Markdown {
    editors: HashMap<PathBuf, Editor>,
    /// The active Markdown tab and the area its editor covers (egui points), set by the shell.
    pub area: Option<(PathBuf, egui::Rect)>,
    /// A slide show or the page's full screen is on: the editor covers the whole window.
    pub fullscreen: bool,
    /// What tabs opened from the editor show first (`{"template": "report"}` …).
    pending: HashMap<PathBuf, Value>,
    /// Text to insert into an editor at the cursor (from KaDee AI).
    inserts: Vec<(PathBuf, String)>,
    /// The window is closing: each Markdown tab with unsaved changes asks first.
    pub quitting: bool,
    host_dirs: Option<(PathBuf, PathBuf)>,
    #[cfg(target_os = "linux")]
    gtk: Option<bool>,
}

impl Markdown {
    /// The page's title for a tab (`• Report.md` while unsaved), once it has booted.
    pub fn tab_name(&self, path: &Path) -> Option<String> {
        let e = self.editors.get(path)?;
        let name = e.name()?;
        Some(if e.dirty() { format!("• {name}") } else { name.to_string() })
    }

    /// True once the page of `path` has booted.
    pub fn ready(&self, path: &Path) -> bool {
        self.editors.get(path).is_some_and(|e| e.ready)
    }

    /// Why the editor of `path` could not start, if it couldn't.
    pub fn error(&self, path: &Path) -> Option<&str> {
        self.editors.get(path).and_then(|e| e.error.as_deref())
    }

    /// The current text of the Markdown tab `path`, unsaved edits included.
    pub fn text(&self, path: &Path) -> Option<(String, String)> {
        let (file, text) = self.editors.get(path)?.document()?;
        let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        Some((name, text))
    }

    /// True when `path` has a running editor.
    pub fn has(&self, path: &Path) -> bool {
        self.editors.contains_key(path)
    }

    /// The Edit menu (macOS): undo, redo, cut, copy, paste or select all in the editor shown in
    /// the active tab. False when no editor is shown (the menu then acts on egui).
    #[cfg(target_os = "macos")]
    pub fn edit(&mut self, path: &Path, action: crate::mac_menu::EditAction) -> bool {
        use crate::mac_menu::EditAction as A;
        let Some(e) = self.editors.get(path).filter(|e| e.visible && e.view.is_some()) else { return false };
        let (what, text) = match action {
            A::Undo => ("undo", String::new()),
            A::Redo => ("redo", String::new()),
            A::Cut => ("cut", String::new()),
            A::Copy => ("copy", String::new()),
            A::SelectAll => ("selectAll", String::new()),
            A::Paste => ("paste", arboard::Clipboard::new().and_then(|mut c| c.get_text()).unwrap_or_default()),
        };
        let js = format!("window.__kadeeEdit && window.__kadeeEdit({}, {})", json!(what), json!(text));
        if let Some(v) = &e.view {
            if matches!(action, A::Cut | A::Copy) {
                let tx = e.tx.clone();
                let _ = v.evaluate_script_with_callback(&js, move |result| {
                    let _ = tx.send(Event::Copied(result));
                });
            } else {
                let _ = v.evaluate_script(&js);
            }
        }
        true
    }

    /// Run the editor command `id` (`file.save`, `file.saveAs`, `file.print`) in the tab of
    /// `path`. False when that tab has no editor that has booted.
    pub fn run_command(&self, path: &Path, id: &str) -> bool {
        match self.editors.get(path) {
            Some(e) if e.ready && e.view.is_some() => {
                e.eval(&format!("window.MD && MD.run && MD.run({})", json!(id)));
                true
            }
            _ => false,
        }
    }

    /// Insert `text` at the cursor of the Markdown tab `path` (an answer from KaDee AI).
    pub fn insert(&mut self, path: &Path, text: String) {
        self.inserts.push((path.to_path_buf(), text));
    }

    /// Ask the page of `path` to close: it asks about unsaved changes and answers `quit`.
    /// Returns false when there is nothing to ask (no editor, or it hasn't booted).
    pub fn request_close(&mut self, path: &Path) -> bool {
        match self.editors.get(path) {
            Some(e) if e.ready && e.view.is_some() => {
                e.eval(CLOSE_JS);
                true
            }
            _ => false,
        }
    }

    /// Stop the editor of a closed tab.
    pub fn forget(&mut self, path: &Path) {
        self.editors.remove(path);
        self.pending.remove(path);
        if self.editors.is_empty() {
            self.fullscreen = false;
        }
    }

    /// Open a tab for a document the editor will create (a template, a copy): `path` is where
    /// it would be saved, `startup` what the page shows first.
    fn open_new(app: &mut KadeeApp, md: &mut Markdown, dir: &Path, stem: &str, startup: Value) {
        let path = free_path(dir, stem, |p| app.tabs.iter().any(|t| t == p) || md.editors.contains_key(p));
        md.pending.insert(path.clone(), startup);
        app.open_tab(path);
    }
}

/// `dir/stem.md`, or `dir/stem 2.md` … when that file exists or is taken.
fn free_path(dir: &Path, stem: &str, taken: impl Fn(&Path) -> bool) -> PathBuf {
    let mut n = 1;
    loop {
        let name = if n == 1 { format!("{stem}.md") } else { format!("{stem} {n}.md") };
        let p = dir.join(name);
        if !p.exists() && !taken(&p) {
            return p;
        }
        n += 1;
    }
}

/// Run after the shell has laid out the frame: start editors, place and show the active one,
/// hide the others, and answer what the pages asked for.
pub fn frame(app: &mut KadeeApp, ctx: &egui::Context, frame: &eframe::Frame) {
    let mut md = std::mem::take(&mut app.markdown);
    let now = ctx.input(|i| i.time);

    // mdcraft creates documents in KadeeStudio's folder and keeps "My templates" in Templates.
    let dirs = (app.state.settings.documents_dir.clone(), app.store.templates_dir());
    if md.host_dirs.as_ref() != Some(&dirs) {
        mdcraft_core::util::set_host_dirs(Some(dirs.0.clone()), Some(dirs.1.clone()));
        md.host_dirs = Some(dirs);
    }

    // Editors for open Markdown tabs; editors of closed tabs stop.
    for path in app.tabs.clone() {
        if is_markdown(&path) && !md.editors.contains_key(&path) {
            let startup = md
                .pending
                .remove(&path)
                .unwrap_or_else(|| if path.is_file() { json!({"open": path.to_string_lossy()}) } else { json!({"template": "blank"}) });
            match Editor::start(startup) {
                Ok(e) => {
                    md.editors.insert(path, e);
                }
                Err(e) => {
                    app.notify(format!("The Markdown editor could not start: {e}"), now);
                }
            }
        }
    }
    md.editors.retain(|p, _| app.tabs.contains(p));

    #[cfg(target_os = "linux")]
    if !md.editors.is_empty() {
        if md.gtk.is_none() {
            md.gtk = Some(gtk::init().is_ok());
        }
        if md.gtk == Some(true) {
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }
        // GTK only runs when egui draws a frame: keep it ticking while editors are open.
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }

    // Place the active editor; hide it under egui's own windows and menus, which a native
    // view would cover.
    let covered = app.settings_window.is_some() || app.new_picker || egui::Popup::is_any_open(ctx);
    let zoom = ctx.zoom_factor();
    let keys = shortcut_json(app);
    let area = md.area.take();
    for (path, e) in md.editors.iter_mut() {
        let here = area.as_ref().filter(|(p, _)| p == path).map(|(_, r)| *r);
        match here {
            Some(r) if !covered && e.error.is_none() => {
                if e.view.is_none() {
                    let b = [(r.min.x * zoom) as i32, (r.min.y * zoom) as i32, (r.width() * zoom) as i32, (r.height() * zoom) as i32];
                    #[cfg(target_os = "linux")]
                    let gtk_ok = md.gtk == Some(true);
                    #[cfg(not(target_os = "linux"))]
                    let gtk_ok = true;
                    let built = if gtk_ok { e.build(frame, ctx, &keys, b) } else { Err("GTK could not start".into()) };
                    if let Err(err) = built {
                        e.error = Some(err);
                    }
                } else {
                    e.set_visible(true);
                    e.set_bounds(r, zoom);
                }
                if e.keys_sent != keys {
                    e.eval(&format!("window.__kadeeKeys = {keys};"));
                    e.keys_sent = keys.clone();
                }
            }
            _ => e.set_visible(false),
        }
    }

    // Text from KaDee AI goes in at the cursor, as one undoable edit.
    for (path, text) in std::mem::take(&mut md.inserts) {
        if let Some(e) = md.editors.get(&path) {
            e.eval(&format!(
                "window.MD && MD.doc && MD.doc.open && MD.editor.insert({}) , window.MD && MD.toast && MD.toast('Inserted from KaDee AI')",
                json!(text)
            ));
        }
    }

    // Messages from the pages.
    let paths: Vec<PathBuf> = md.editors.keys().cloned().collect();
    for path in paths {
        while let Some(ev) = md.editors.get(&path).and_then(|e| e.rx.try_recv().ok()) {
            match ev {
                Event::Ipc(body) => handle(app, &mut md, &path, &body, ctx, now),
                Event::External(url) => open_external(&url),
                #[cfg(target_os = "macos")]
                Event::Copied(result) => {
                    let text: String = serde_json::from_str(&result).unwrap_or_default();
                    if !text.is_empty()
                        && let Ok(mut c) = arboard::Clipboard::new()
                    {
                        let _ = c.set_text(text);
                    }
                }
                Event::Dropped(files) => {
                    let list: Vec<String> = files.iter().map(|p| p.to_string_lossy().into_owned()).collect();
                    if let Some(e) = md.editors.get(&path) {
                        e.eval(&format!("window.__mdNative && window.__mdNative.dropped({})", json!(list)));
                    }
                }
            }
        }
    }

    // Save As / a new document from a template moves the page to another file: the tab follows.
    let moves: Vec<(PathBuf, PathBuf)> = md
        .editors
        .iter()
        .filter(|(_, e)| e.ready)
        .filter_map(|(p, e)| {
            let target = e.core.state.lock().ok()?.target.clone()?;
            (target != *p && !app.tabs.contains(&target)).then(|| (p.clone(), target))
        })
        .collect();
    for (from, to) in moves {
        if let Some(e) = md.editors.remove(&from) {
            md.editors.insert(to.clone(), e);
        }
        if let Some(t) = app.tabs.iter_mut().find(|t| **t == from) {
            *t = to.clone();
        }
        if to.is_file() {
            app.state.library.touch(&to, kadee_core::now());
            app.mark_dirty();
            app.rescan();
        }
    }

    // Closing the window: Markdown tabs with unsaved changes ask first, one after the other.
    // A page that is cancelled (Cancel in its dialog) stays open and stops the closing; closing
    // the window again asks again.
    if ctx.input(|i| i.viewport().close_requested()) && md.editors.values().any(|e| e.dirty() && e.ready) {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        md.quitting = true;
        for e in md.editors.values_mut() {
            e.asked = false;
        }
    }
    if md.quitting {
        let next = app.tabs.iter().position(|p| md.editors.get(p).is_some_and(|e| e.dirty() && e.ready));
        match next {
            Some(i) => {
                let path = app.tabs[i].clone();
                if let Some(e) = md.editors.get_mut(&path)
                    && !e.asked
                {
                    app.active = i + 1;
                    e.asked = true;
                    e.eval(CLOSE_JS);
                }
            }
            None => {
                md.quitting = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    app.markdown = md;
}

/// Answer one message from the page of tab `path`.
fn handle(app: &mut KadeeApp, md: &mut Markdown, path: &Path, body: &str, ctx: &egui::Context, now: f64) {
    let msg: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let cmd = msg["cmd"].as_str().unwrap_or("");
    let id = msg["id"].as_u64().unwrap_or(0);
    match cmd {
        "ready" => {
            if let Some(e) = md.editors.get_mut(path) {
                e.ready = true;
            }
        }
        "set-title" => {
            if let Some(e) = md.editors.get_mut(path) {
                e.title = msg["title"].as_str().unwrap_or("").to_string();
            }
        }
        "quit" => {
            app.remove_tab(path);
            md.forget(path);
        }
        // The page is still here after being asked to close: Cancel was chosen.
        "kadee-close-done" => md.quitting = false,
        // "Where" in the keep-new-document question (MD.confirmClose)
        "folder-dialog" => {
            let mut d = rfd::FileDialog::new().set_title(msg["title"].as_str().unwrap_or("Where"));
            if let Some(dir) = msg["dir"].as_str().map(PathBuf::from).filter(|d| d.is_dir()) {
                d = d.set_directory(dir);
            }
            let picked = d.pick_folder();
            if let Some(e) = md.editors.get(path) {
                e.reply(id, picked.map(|p| json!(p.to_string_lossy())).unwrap_or(Value::Null), None);
            }
        }
        "open-dialog" => {
            let mut d = rfd::FileDialog::new().set_title(msg["title"].as_str().unwrap_or("Open"));
            d = add_filters(d, &msg["filters"]);
            if let Some(dir) = msg["dir"].as_str().map(PathBuf::from).filter(|d| d.is_dir()) {
                d = d.set_directory(dir);
            }
            let picked = d.pick_file();
            if let Some(e) = md.editors.get(path) {
                e.reply(id, picked.map(|p| json!(p.to_string_lossy())).unwrap_or(Value::Null), None);
            }
        }
        "save-dialog" => {
            let answer = if msg["types"].is_array() {
                save_with_types(&msg).map(|(p, kind)| json!({"path": p.to_string_lossy(), "type": kind}))
            } else {
                let mut d = rfd::FileDialog::new().set_title("Save As");
                d = add_filters(d, &msg["filters"]);
                if let Some(dir) = msg["dir"].as_str().map(PathBuf::from).filter(|d| d.is_dir()) {
                    d = d.set_directory(dir);
                }
                if let Some(name) = msg["name"].as_str() {
                    d = d.set_file_name(name);
                }
                d.save_file().map(|p| json!(p.to_string_lossy()))
            };
            if let Some(e) = md.editors.get(path) {
                e.reply(id, answer.unwrap_or(Value::Null), None);
            }
        }
        "print" => {
            if let Some(e) = md.editors.get(path) {
                match e.view.as_ref().map(|v| v.print()) {
                    Some(Ok(())) => e.reply(id, json!(true), None),
                    Some(Err(err)) => e.reply(id, Value::Null, Some(format!("printing is not available: {err}"))),
                    None => {}
                }
            }
        }
        "fullscreen" => {
            let on = msg["on"].as_bool().unwrap_or(false);
            md.fullscreen = on;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(on));
        }
        // KadeeStudio's theme decides the window's look; the page keeps its own preference.
        "theme" | "zoom" => {}
        "open-external" => {
            if let Some(u) = msg["url"].as_str() {
                open_external(u);
            }
        }
        "reveal" => {
            if let Some(p) = msg["path"].as_str() {
                app.send(Command::RevealInFolder(PathBuf::from(p)));
            }
        }
        "new-window" => {
            let docs = app.state.settings.documents_dir.clone();
            if let Some(t) = msg["template"].as_str() {
                let stem = mdcraft_templates::get(t).map(|t| t.name.to_string()).unwrap_or_else(|| "Document".into());
                Markdown::open_new(app, md, &docs, &stem, json!({"template": t}));
            } else if let Some(p) = msg["userTemplate"].as_str() {
                let stem = Path::new(p).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "Document".into());
                Markdown::open_new(app, md, &docs, &stem, json!({"userTemplate": p}));
            } else if let Some(k) = msg["newTemplate"].as_str() {
                let dir = app.store.templates_dir();
                Markdown::open_new(app, md, &dir, "New template", json!({"newTemplate": k}));
            } else if let Some(p) = msg["path"].as_str() {
                let p = PathBuf::from(p);
                if msg["copy"].as_bool().unwrap_or(false) {
                    let dir = p.parent().map(Path::to_path_buf).unwrap_or(docs);
                    let stem = format!("{} (version on disk)", p.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default());
                    Markdown::open_new(app, md, &dir, &stem, json!({"copyOf": p.to_string_lossy()}));
                } else if is_markdown(&p) {
                    app.open_tab(p);
                } else {
                    app.send(Command::OpenPath(p));
                }
            } else {
                Markdown::open_new(app, md, &docs, "Document", Value::Null);
            }
        }
        "kadee-key" => {
            let chord = Chord {
                ctrl: msg["ctrl"].as_bool().unwrap_or(false),
                alt: msg["alt"].as_bool().unwrap_or(false),
                shift: msg["shift"].as_bool().unwrap_or(false),
                meta: msg["meta"].as_bool().unwrap_or(false),
                key: msg["key"].as_str().unwrap_or("").to_string(),
            };
            crate::ui::keys::forwarded(app, chord, now);
        }
        other => {
            if let Some(e) = md.editors.get(path) {
                e.reply(id, Value::Null, Some(format!("unknown command {other}")));
            }
        }
    }
}

fn open_external(url: &str) {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:") {
        let _ = open::that_detached(url);
    }
}

fn add_filters(mut d: rfd::FileDialog, filters: &Value) -> rfd::FileDialog {
    for f in filters.as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
        let name = f.get(0).and_then(|n| n.as_str()).unwrap_or("Files");
        let exts: Vec<String> = f
            .get(1)
            .and_then(|e| e.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default();
        if !exts.is_empty() && !exts.iter().any(|e| e == "*") {
            d = d.add_filter(name, &exts);
        }
    }
    d
}

/// Save As with a File type choice: `msg` = {title, name, dir, types: [{label, ext, dir}], type}.
/// The types are listed as filters and the extension typed in the name decides the type.
/// Returns the path and the index of the type it is saved as.
fn save_with_types(msg: &Value) -> Option<(PathBuf, usize)> {
    let types: Vec<(String, String, Option<PathBuf>)> = msg["types"]
        .as_array()?
        .iter()
        .filter_map(|t| {
            Some((
                t["label"].as_str()?.to_string(),
                t["ext"].as_str()?.trim_start_matches('.').to_string(),
                t["dir"].as_str().map(PathBuf::from),
            ))
        })
        .collect();
    if types.is_empty() {
        return None;
    }
    let selected = (msg["type"].as_u64().unwrap_or(0) as usize).min(types.len() - 1);
    let name = msg["name"].as_str().unwrap_or("Document.md");
    let (_, ext, type_dir) = &types[selected];
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    let mut d = rfd::FileDialog::new().set_title(msg["title"].as_str().unwrap_or("Save As")).set_file_name(format!("{stem}.{ext}"));
    if let Some(dir) = msg["dir"].as_str().map(PathBuf::from).or_else(|| type_dir.clone()).filter(|d| d.is_dir()) {
        d = d.set_directory(dir);
    }
    for i in std::iter::once(selected).chain((0..types.len()).filter(|i| *i != selected)) {
        d = d.add_filter(&types[i].0, &[&types[i].1]);
    }
    let mut p = d.save_file()?;
    let found = p.extension().and_then(|e| e.to_str()).and_then(|e| types.iter().position(|(_, x, _)| x.eq_ignore_ascii_case(e)));
    let kind = match found {
        Some(i) => i,
        None => {
            p.set_extension(ext);
            selected
        }
    };
    Some((p, kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_extensions() {
        assert!(is_markdown(Path::new("/a/Notes.md")));
        assert!(is_markdown(Path::new("/a/deck.MDS")));
        assert!(is_markdown(Path::new("/a/mine.mdd")));
        assert!(!is_markdown(Path::new("/a/doc.pdf")));
        assert!(!is_markdown(Path::new("/a/README")));
    }

    #[test]
    fn free_path_skips_existing_and_taken() {
        let dir = std::env::temp_dir().join(format!("kadee-md-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Report.md"), "x").unwrap();
        let taken = dir.join("Report 2.md");
        let p = free_path(&dir, "Report", |p| p == taken);
        assert_eq!(p, dir.join("Report 3.md"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn kadee_ai_reads_the_unsaved_text() {
        let mut md = Markdown::default();
        let tab = PathBuf::from("/tmp/kadee-tab/Notes.md");
        let e = Editor::start(Value::Null).unwrap();
        // what the page posts while the user types (POST /api/doc)
        e.core.set_document(tab.clone(), "# Notes\n\nNot saved yet".into());
        md.editors.insert(tab.clone(), e);
        let (name, text) = md.text(&tab).unwrap();
        assert_eq!(name, "Notes.md");
        assert!(text.contains("Not saved yet"));
        assert!(md.has(&tab));
        md.forget(&tab);
        assert!(md.text(&tab).is_none());
    }

    #[test]
    fn editor_names_from_titles() {
        let mut e = Editor::start(Value::Null).unwrap();
        e.title = "• Report.md — Kadee Markdown".into();
        assert!(e.dirty());
        assert_eq!(e.name(), Some("Report.md"));
        e.title = "Kadee Markdown".into();
        assert_eq!(e.name(), None);
        assert!(!e.dirty());
    }
}
