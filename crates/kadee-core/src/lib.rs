//! KadeeStudio core.
//!
//! Everything the start screen needs that is not UI: the module catalogue,
//! the ArtCraft app catalogue, the recent/starred library, user settings,
//! `home.json` persistence and file helpers. This crate must never depend on
//! egui so the CLI and server can reuse it later.

pub mod craft;
pub mod doctype;
pub mod files;
pub mod installed;
pub mod library;
pub mod modules;
pub mod office;
pub mod settings;
pub mod store;

pub use craft::{CRAFT_APPS, CRAFT_PHASE, CraftApp, LAUNCHER_REPO, SOLIDCRAFT_REPO};
pub use doctype::{DocGroup, DocKind, Opener, all_extensions, open_dialog_types};
pub use files::{
    FileEntry, create_folder, create_new_document, human_size, list_documents, list_subfolders, now, relative_time, rename_folder,
};
pub use installed::{InstalledApp, Launch, detect_installed};
pub use library::{Library, RecentEntry};
pub use modules::ModuleKind;
pub use office::{OfficeKind, create_office_document};
pub use settings::{OpenIn, Settings, StoredShortcut, ThemePref};
pub use store::{HomeState, Store};

/// The source code on GitHub.
pub const SOURCE_REPO: &str = "https://github.com/kdugardyn66/KaDeeStudio";
/// Product name shown in the UI.
pub const APP_NAME: &str = "KadeeStudio";
/// Version from Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
