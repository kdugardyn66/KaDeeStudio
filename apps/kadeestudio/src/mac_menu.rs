//! The macOS menu bar: KadeeStudio, File, Edit, Window and Help, laid out like
//! ONLYOFFICE's.
//!
//! - **KadeeStudio**: About (opens Settings › About, so the menu bar and the
//!   Settings window show the same information), Check for Updates…, Settings…,
//!   Services, Hide, Hide Others, Show All, Quit.
//! - **File**: New document…, New ▸ (every document type), Open…, Close, and
//!   Save, Save As…, Print… (greyed out until the editors arrive).
//! - **Edit**: Undo, Redo, Cut, Copy, Paste, Select All. macOS adds AutoFill
//!   and Emoji & Symbols itself.
//! - **Window**: Minimize, Zoom, Bring All to Front; macOS adds the tiling items.
//! - **Help**: Acknowledgments, License Agreement, KadeeStudio Help.
//!
//! Shortcuts shown in the menus follow the key map: an item shows the first
//! single-key shortcut its command has (`⌘O`), and none when the shortcut is a
//! sequence (`⌘N M`) or unassigned. While a shortcut is being recorded, the
//! items are switched off so that the menu cannot swallow the keys.
//!
//! Edit items do not use the system's Cut/Copy/Paste actions, which egui's
//! window does not answer; they send the same events egui's own shortcuts do.
//!
//! The menu is installed when the app starts, which is after winit has set up
//! its default menu, so this one replaces it.

use kadee_commands::{Chord, Command, Keymap, SettingsSection};
use kadee_core::{ModuleKind, OfficeKind};
use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};

/// Where "Check for Updates…" and the Help items go (the project's GitHub page).
const REPO: &str = "https://github.com/kdugardyn66/KaDeeStudio";

/// What an Edit menu item asks the focused text field to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditAction {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
}

/// A clicked menu item.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Run(Command),
    Edit(EditAction),
}

pub struct MacMenu {
    _menu: Menu,
    /// Items that follow the key map, with the command they run.
    bound: Vec<(MenuItem, Command)>,
    /// Items with fixed shortcuts (Edit, Check for Updates, Help).
    fixed: Vec<MenuItem>,
    actions: HashMap<MenuId, Action>,
    shown: Vec<Option<Accelerator>>,
    suspended: bool,
    events: Receiver<MenuId>,
}

/// Collects items while the menus are built.
struct Builder {
    bound: Vec<(MenuItem, Command)>,
    fixed: Vec<MenuItem>,
    actions: HashMap<MenuId, Action>,
}

impl Builder {
    /// An item that runs `command` and shows its shortcut from the key map.
    fn command(&mut self, text: &str, command: Command) -> MenuItem {
        let item = MenuItem::new(text, true, None);
        self.actions.insert(item.id().clone(), Action::Run(command.clone()));
        self.bound.push((item.clone(), command));
        item
    }

    /// An item that runs `action` with a shortcut that does not change.
    fn fixed(&mut self, text: &str, action: Action, accelerator: Accelerator) -> MenuItem {
        let item = MenuItem::new(text, true, Some(accelerator));
        self.actions.insert(item.id().clone(), action);
        self.fixed.push(item.clone());
        item
    }
}

fn cmd(key: Code) -> Accelerator {
    Accelerator::new(Some(Modifiers::SUPER), key)
}

impl MacMenu {
    /// Build the menu bar and make it the app's menu. `None` if muda fails.
    pub fn install(ctx: &egui::Context, keymap: &Keymap) -> Option<Self> {
        let mut b = Builder { bound: Vec::new(), fixed: Vec::new(), actions: HashMap::new() };
        let url = |s: &str| Command::OpenUrl(s.to_string());

        // KadeeStudio
        let about = b.command("About KadeeStudio", Command::OpenSettings(SettingsSection::About));
        let updates = b.fixed("Check for Updates…", Action::Run(url(&format!("{REPO}/releases"))), cmd(Code::KeyU));
        let settings = b.command("Settings…", Command::OpenSettings(SettingsSection::General));
        let app_menu = Submenu::new("KadeeStudio", true);
        app_menu
            .append_items(&[
                &about,
                &updates,
                &PredefinedMenuItem::separator(),
                &settings,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(Some("Hide KadeeStudio")),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(Some("Quit KadeeStudio")),
            ])
            .ok()?;

        // File
        let new_document = b.command("New document…", Command::NewDocumentPicker);
        let new_menu = Submenu::new("New", true);
        let md = b.command("Markdown document", Command::NewDocument(ModuleKind::Markdown));
        let pdf = b.command("PDF document", Command::NewDocument(ModuleKind::Pdf));
        let dash = b.command("Dashboard", Command::NewDocument(ModuleKind::Dashboard));
        let report = b.command("Report", Command::NewDocument(ModuleKind::Report));
        let word = b.command("Word document", Command::NewOffice(OfficeKind::Word));
        let sheet = b.command("Spreadsheet", Command::NewOffice(OfficeKind::Spreadsheet));
        let deck = b.command("Slide deck", Command::NewOffice(OfficeKind::SlideDeck));
        new_menu.append_items(&[&md, &pdf, &dash, &report, &PredefinedMenuItem::separator(), &word, &sheet, &deck]).ok()?;
        let open = b.command("Open…", Command::OpenDialog);
        let close = b.command("Close", Command::CloseActiveTab);
        // Greyed out until the editors exist; the shortcuts show what they will be.
        let save = MenuItem::new("Save", false, Some(cmd(Code::KeyS)));
        let save_as = MenuItem::new("Save As…", false, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyS)));
        let print = MenuItem::new("Print…", false, Some(cmd(Code::KeyP)));
        let file_menu = Submenu::new("File", true);
        file_menu
            .append_items(&[
                &new_document,
                &new_menu,
                &open,
                &PredefinedMenuItem::separator(),
                &close,
                &save,
                &save_as,
                &PredefinedMenuItem::separator(),
                &print,
            ])
            .ok()?;

        // Edit
        let undo = b.fixed("Undo", Action::Edit(EditAction::Undo), cmd(Code::KeyZ));
        let redo = b.fixed("Redo", Action::Edit(EditAction::Redo), cmd(Code::KeyY));
        let cut = b.fixed("Cut", Action::Edit(EditAction::Cut), cmd(Code::KeyX));
        let copy = b.fixed("Copy", Action::Edit(EditAction::Copy), cmd(Code::KeyC));
        let paste = b.fixed("Paste", Action::Edit(EditAction::Paste), cmd(Code::KeyV));
        let select_all = b.fixed("Select All", Action::Edit(EditAction::SelectAll), cmd(Code::KeyA));
        let edit_menu = Submenu::new("Edit", true);
        edit_menu.append_items(&[&undo, &redo, &PredefinedMenuItem::separator(), &cut, &copy, &paste, &select_all]).ok()?;

        // Window
        let window_menu = Submenu::new("Window", true);
        window_menu
            .append_items(&[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::maximize(Some("Zoom")),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::bring_all_to_front(None),
            ])
            .ok()?;

        // Help
        let thanks = b.command("Acknowledgments", Command::OpenSettings(SettingsSection::About));
        let license = b.fixed(
            "License Agreement",
            Action::Run(url(&format!("{REPO}/blob/main/LICENSE-MIT"))),
            Accelerator::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyL),
        );
        let help = b.fixed(
            "KadeeStudio Help",
            Action::Run(url(&format!("{REPO}#readme"))),
            Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Slash),
        );
        let help_menu = Submenu::new("Help", true);
        help_menu.append_items(&[&thanks, &license, &PredefinedMenuItem::separator(), &help]).ok()?;

        let menu = Menu::new();
        menu.append_items(&[&app_menu, &file_menu, &edit_menu, &window_menu, &help_menu]).ok()?;
        attach(&menu, &window_menu, &help_menu);

        // Menu clicks arrive outside egui: pass them on and wake the UI.
        let (tx, events) = channel();
        let ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            let _ = tx.send(e.id);
            ctx.request_repaint();
        }));

        let mut this =
            Self { _menu: menu, bound: b.bound, fixed: b.fixed, actions: b.actions, shown: Vec::new(), suspended: false, events };
        this.shown = vec![None; this.bound.len()];
        this.sync(keymap, false);
        Some(this)
    }

    /// Show the key map's shortcuts on the items, and switch the items off while
    /// a shortcut is being recorded. Cheap enough to call every frame.
    pub fn sync(&mut self, keymap: &Keymap, recording: bool) {
        for (i, (item, command)) in self.bound.iter().enumerate() {
            let wanted = keymap.bindings.iter().filter(|b| &b.command == command).find_map(|b| match b.keys.as_slice() {
                [one] => accelerator(one),
                _ => None,
            });
            if self.shown[i] != wanted {
                let _ = item.set_accelerator(wanted);
                self.shown[i] = wanted;
            }
        }
        if self.suspended != recording {
            self.suspended = recording;
            for item in self.bound.iter().map(|(i, _)| i).chain(&self.fixed) {
                item.set_enabled(!recording);
            }
        }
    }

    /// Items clicked since the last frame.
    pub fn poll(&self) -> Vec<Action> {
        self.events.try_iter().filter_map(|id| self.actions.get(&id).cloned()).collect()
    }
}

/// Make `menu` the app's menu bar; `window` and `help` get the system's extra items.
#[cfg(target_os = "macos")]
fn attach(menu: &Menu, window: &Submenu, help: &Submenu) {
    menu.init_for_nsapp();
    window.set_as_windows_menu_for_nsapp();
    help.set_as_help_menu_for_nsapp();
}

/// Only macOS has an app-wide menu bar.
#[cfg(not(target_os = "macos"))]
fn attach(_menu: &Menu, _window: &Submenu, _help: &Submenu) {}

/// A chord as a menu shortcut. `None` for keys the menu cannot show, and for keys
/// without a ⌘, ⌃ or ⌥: a plain letter in a menu would take it away from typing.
pub fn accelerator(chord: &Chord) -> Option<Accelerator> {
    if !(chord.meta || chord.ctrl || chord.alt) {
        return None;
    }
    let mut mods = Modifiers::empty();
    if chord.meta {
        mods |= Modifiers::SUPER;
    }
    if chord.ctrl {
        mods |= Modifiers::CONTROL;
    }
    if chord.alt {
        mods |= Modifiers::ALT;
    }
    if chord.shift {
        mods |= Modifiers::SHIFT;
    }
    Some(Accelerator::new(Some(mods), code(&chord.key)?))
}

/// The menu's key code for an egui key name (`A`, `Num1`, `Comma`, `F5` …).
fn code(name: &str) -> Option<Code> {
    const LETTERS: [Code; 26] = [
        Code::KeyA,
        Code::KeyB,
        Code::KeyC,
        Code::KeyD,
        Code::KeyE,
        Code::KeyF,
        Code::KeyG,
        Code::KeyH,
        Code::KeyI,
        Code::KeyJ,
        Code::KeyK,
        Code::KeyL,
        Code::KeyM,
        Code::KeyN,
        Code::KeyO,
        Code::KeyP,
        Code::KeyQ,
        Code::KeyR,
        Code::KeyS,
        Code::KeyT,
        Code::KeyU,
        Code::KeyV,
        Code::KeyW,
        Code::KeyX,
        Code::KeyY,
        Code::KeyZ,
    ];
    const DIGITS: [Code; 10] = [
        Code::Digit0,
        Code::Digit1,
        Code::Digit2,
        Code::Digit3,
        Code::Digit4,
        Code::Digit5,
        Code::Digit6,
        Code::Digit7,
        Code::Digit8,
        Code::Digit9,
    ];
    const FUNCTION: [Code; 12] =
        [Code::F1, Code::F2, Code::F3, Code::F4, Code::F5, Code::F6, Code::F7, Code::F8, Code::F9, Code::F10, Code::F11, Code::F12];
    let bytes = name.as_bytes();
    if let [c @ b'A'..=b'Z'] = bytes {
        return Some(LETTERS[usize::from(c - b'A')]);
    }
    if let Some(d) = name.strip_prefix("Num").and_then(|d| d.parse::<usize>().ok()).filter(|d| *d < 10) {
        return Some(DIGITS[d]);
    }
    if let Some(n) = name.strip_prefix('F').and_then(|n| n.parse::<usize>().ok()).filter(|n| (1..=12).contains(n)) {
        return Some(FUNCTION[n - 1]);
    }
    Some(match name {
        "Comma" => Code::Comma,
        "Period" => Code::Period,
        "Slash" => Code::Slash,
        "Minus" => Code::Minus,
        "Equals" => Code::Equal,
        "Tab" => Code::Tab,
        "Enter" => Code::Enter,
        "Space" => Code::Space,
        _ => return None,
    })
}

/// Do what an Edit menu item asks, on whatever text field has the focus, by
/// sending egui the events its own shortcuts produce.
pub fn edit(ctx: &egui::Context, action: EditAction) {
    use egui::{Event, Key, Modifiers};
    let cmd = Modifiers { mac_cmd: true, command: true, ..Modifiers::default() };
    let key = |key: Key, modifiers: Modifiers| Event::Key { key, physical_key: Some(key), pressed: true, repeat: false, modifiers };
    let event = match action {
        EditAction::Undo => key(Key::Z, cmd),
        EditAction::Redo => key(Key::Z, Modifiers { shift: true, ..cmd }),
        EditAction::SelectAll => key(Key::A, cmd),
        EditAction::Cut => Event::Cut,
        EditAction::Copy => Event::Copy,
        EditAction::Paste => match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
            Ok(text) if !text.is_empty() => Event::Paste(text),
            _ => return,
        },
    };
    ctx.input_mut(|i| i.events.push(event));
    ctx.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(key: &str) -> Chord {
        Chord { ctrl: false, alt: false, shift: false, meta: true, key: key.into() }
    }

    #[test]
    fn keys_become_menu_shortcuts() {
        assert_eq!(accelerator(&chord("O")), Some(cmd(Code::KeyO)));
        assert_eq!(accelerator(&chord("Num3")), Some(cmd(Code::Digit3)));
        assert_eq!(accelerator(&chord("Comma")), Some(cmd(Code::Comma)));
        assert_eq!(accelerator(&chord("F5")), Some(cmd(Code::F5)));
        let both = Chord { shift: true, ..chord("D") };
        assert_eq!(accelerator(&both), Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyD)));
        // unknown keys, and keys without ⌘ / ⌃ / ⌥, stay out of the menu
        assert_eq!(accelerator(&chord("PageUp")), None);
        assert_eq!(accelerator(&Chord { meta: false, shift: true, ..chord("D") }), None);
    }
}
