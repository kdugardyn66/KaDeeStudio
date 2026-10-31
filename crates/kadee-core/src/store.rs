//! Persistence of the start-screen state in `home.json`.
//!
//! Location: the platform config directory for KadeeStudio
//! (`~/Library/Application Support/be.Kadee.KadeeStudio` on macOS,
//! `~/.config/kadeestudio` on Linux, `%APPDATA%\Kadee\KadeeStudio\config` on Windows).
//! `KADEE_CONFIG_DIR` overrides it.

use crate::library::Library;
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "home.json";
const WELCOME_NAME: &str = "Welcome to KadeeStudio.md";
const WELCOME: &str = include_str!("../assets/welcome.md");

/// Everything the start screen persists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeState {
    pub version: u32,
    pub settings: Settings,
    #[serde(default)]
    pub library: Library,
}

impl HomeState {
    pub fn new(documents_dir: PathBuf) -> Self {
        Self { version: 1, settings: Settings::with_documents_dir(documents_dir), library: Library::default() }
    }
}

/// Reads and writes `home.json` in one directory.
#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    /// The store in the platform config dir (or `KADEE_CONFIG_DIR`).
    pub fn platform() -> Self {
        if let Some(dir) = std::env::var_os("KADEE_CONFIG_DIR") {
            return Self::at(PathBuf::from(dir));
        }
        let dir = directories::ProjectDirs::from("be", "Kadee", "KadeeStudio")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".kadeestudio"));
        Self::at(dir)
    }

    pub fn at(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn file(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    /// Default documents folder: `~/Documents/KadeeStudio`.
    pub fn default_documents_dir() -> PathBuf {
        directories::UserDirs::new()
            .and_then(|u| u.document_dir().map(Path::to_path_buf).or_else(|| Some(u.home_dir().join("Documents"))))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("KadeeStudio")
    }

    /// Load the state; on first launch create it, make the documents
    /// folder and drop the welcome document into it.
    /// Returns the state and whether this was a first launch.
    pub fn load_or_init(&self) -> (HomeState, bool) {
        match fs::read_to_string(self.file()).ok().and_then(|s| serde_json::from_str::<HomeState>(&s).ok()) {
            Some(state) => (state, false),
            None => {
                let state = HomeState::new(Self::default_documents_dir());
                let _ = seed_documents(&state.settings.documents_dir);
                let _ = self.save(&state);
                (state, true)
            }
        }
    }

    /// Write atomically (temp file + rename).
    pub fn save(&self, state: &HomeState) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let tmp = self.dir.join(format!("{FILE_NAME}.tmp"));
        let json = serde_json::to_string_pretty(state).map_err(io::Error::other)?;
        fs::write(&tmp, json)?;
        fs::rename(tmp, self.file())
    }
}

/// Create the documents folder with a welcome document if it is new.
pub fn seed_documents(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let welcome = dir.join(WELCOME_NAME);
    if !welcome.exists() {
        fs::write(welcome, WELCOME)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::at(tmp.path().join("cfg"));
        let mut state = HomeState::new(tmp.path().join("docs"));
        state.settings.display_name = "Kris".into();
        state.library.touch(Path::new("/x.md"), 42);
        store.save(&state).unwrap();
        let (loaded, first) = store.load_or_init();
        assert!(!first);
        assert_eq!(loaded, state);
    }

    #[test]
    fn corrupt_file_falls_back_to_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::at(tmp.path().to_path_buf());
        fs::write(store.file(), "{ not json").unwrap();
        let (state, first) = store.load_or_init();
        assert!(first);
        assert_eq!(state.version, 1);
    }

    #[test]
    fn seeding_writes_welcome_once() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("KadeeStudio");
        seed_documents(&dir).unwrap();
        let w = dir.join(WELCOME_NAME);
        fs::write(&w, "edited").unwrap();
        seed_documents(&dir).unwrap();
        assert_eq!(fs::read_to_string(w).unwrap(), "edited");
    }
}
