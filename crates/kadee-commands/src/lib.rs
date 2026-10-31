//! Commands: every user action in KadeeStudio is one of these.
//!
//! The tab strip, sidebar, cards, context menus and keyboard all produce a
//! [`Command`]; the shell runs it in one place. Each command has a stable id
//! (`file.new.markdown`) so a command palette, the CLI, MCP and AI can drive
//! the same actions later.

use kadee_core::{ModuleKind, ThemePref};
use std::path::PathBuf;

/// Pages of the Home tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Recent,
    Starred,
    Folder,
    CraftApps,
    Settings,
}

/// A user action.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Create a document of this kind in the documents folder and open it.
    NewDocument(ModuleKind),
    /// Open a module that works on existing files (PDF, Convert).
    OpenModule(ModuleKind),
    /// Show the system Open dialog.
    OpenDialog,
    /// Open this file in a tab.
    OpenPath(PathBuf),
    /// Switch to the Home tab and show a page.
    ShowPage(Page),
    /// Activate tab n (0 = Home).
    ActivateTab(usize),
    NextTab,
    PreviousTab,
    /// Close tab n (Home cannot be closed).
    CloseTab(usize),
    CloseActiveTab,
    ReopenClosedTab,
    ToggleStar(PathBuf),
    RemoveFromRecent(PathBuf),
    OpenExternally(PathBuf),
    RevealInFolder(PathBuf),
    CopyPath(PathBuf),
    SetTheme(ThemePref),
    CycleTheme,
    OpenUrl(String),
    Refresh,
}

impl Command {
    /// Stable id for palettes, logs and automation.
    pub fn id(&self) -> String {
        match self {
            Command::NewDocument(k) => format!("file.new.{}", k.id()),
            Command::OpenModule(k) => format!("module.open.{}", k.id()),
            Command::OpenDialog => "file.open".into(),
            Command::OpenPath(_) => "file.open_path".into(),
            Command::ShowPage(p) => format!("home.show.{}", format!("{p:?}").to_lowercase()),
            Command::ActivateTab(_) => "tab.activate".into(),
            Command::NextTab => "tab.next".into(),
            Command::PreviousTab => "tab.previous".into(),
            Command::CloseTab(_) => "tab.close".into(),
            Command::CloseActiveTab => "tab.close_active".into(),
            Command::ReopenClosedTab => "tab.reopen".into(),
            Command::ToggleStar(_) => "library.star".into(),
            Command::RemoveFromRecent(_) => "library.remove_recent".into(),
            Command::OpenExternally(_) => "file.open_external".into(),
            Command::RevealInFolder(_) => "file.reveal".into(),
            Command::CopyPath(_) => "file.copy_path".into(),
            Command::SetTheme(t) => format!("view.theme.{}", t.label().to_lowercase()),
            Command::CycleTheme => "view.theme.cycle".into(),
            Command::OpenUrl(_) => "app.open_url".into(),
            Command::Refresh => "home.refresh".into(),
        }
    }
}

/// A keyboard shortcut, independent of the UI toolkit.
/// `primary` = Cmd on macOS, Ctrl elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    pub primary: bool,
    pub shift: bool,
    pub key: Key,
}

/// Keys used by shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    N,
    O,
    T,
    W,
    D,
    R,
    Comma,
    Tab,
    Num(u8),
}

/// The default key map (shown in Settings › Shortcuts).
pub fn default_keymap() -> Vec<(Shortcut, Command, &'static str)> {
    let p = |key| Shortcut { primary: true, shift: false, key };
    let ps = |key| Shortcut { primary: true, shift: true, key };
    let mut map = vec![
        (p(Key::N), Command::NewDocument(ModuleKind::Markdown), "New Markdown document"),
        (ps(Key::D), Command::NewDocument(ModuleKind::Dashboard), "New dashboard"),
        (ps(Key::R), Command::NewDocument(ModuleKind::Report), "New report"),
        (p(Key::O), Command::OpenDialog, "Open a file"),
        (p(Key::W), Command::CloseActiveTab, "Close tab"),
        (ps(Key::T), Command::ReopenClosedTab, "Reopen closed tab"),
        (p(Key::Comma), Command::ShowPage(Page::Settings), "Settings"),
        (p(Key::Tab), Command::NextTab, "Next tab"),
        (ps(Key::Tab), Command::PreviousTab, "Previous tab"),
        (p(Key::Num(1)), Command::ActivateTab(0), "Home"),
    ];
    for n in 2..=9u8 {
        map.push((p(Key::Num(n)), Command::ActivateTab(usize::from(n - 1)), "Go to tab"));
    }
    map
}

/// Human-readable shortcut, e.g. `Ctrl+Shift+D` or `⌘⇧D`.
pub fn describe(s: Shortcut, mac: bool) -> String {
    let key = match s.key {
        Key::N => "N".to_string(),
        Key::O => "O".into(),
        Key::T => "T".into(),
        Key::W => "W".into(),
        Key::D => "D".into(),
        Key::R => "R".into(),
        Key::Comma => ",".into(),
        Key::Tab => "Tab".into(),
        Key::Num(n) => n.to_string(),
    };
    if mac {
        format!("{}{}{key}", if s.primary { "⌘" } else { "" }, if s.shift { "⇧" } else { "" })
    } else {
        let mut out = String::new();
        if s.primary {
            out.push_str("Ctrl+");
        }
        if s.shift {
            out.push_str("Shift+");
        }
        out + &key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable() {
        assert_eq!(Command::NewDocument(ModuleKind::Markdown).id(), "file.new.markdown");
        assert_eq!(Command::ShowPage(Page::CraftApps).id(), "home.show.craftapps");
        assert_eq!(Command::SetTheme(ThemePref::Dark).id(), "view.theme.dark");
    }

    #[test]
    fn keymap_has_no_duplicate_shortcuts() {
        let map = default_keymap();
        for (i, a) in map.iter().enumerate() {
            for b in &map[i + 1..] {
                assert_ne!(a.0, b.0, "duplicate shortcut for {} and {}", a.2, b.2);
            }
        }
    }

    #[test]
    fn describes_shortcuts() {
        let s = Shortcut { primary: true, shift: true, key: Key::D };
        assert_eq!(describe(s, false), "Ctrl+Shift+D");
        assert_eq!(describe(s, true), "⌘⇧D");
    }
}
