//! Recent and starred documents.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Maximum number of recent entries kept.
pub const MAX_RECENT: usize = 50;

/// A document opened at some point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentEntry {
    pub path: PathBuf,
    /// Unix seconds of the last open.
    pub opened: u64,
}

/// The user's document library.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub recent: Vec<RecentEntry>,
    #[serde(default)]
    pub starred: Vec<PathBuf>,
}

impl Library {
    /// Record that `path` was opened at `now`, moving it to the top.
    pub fn touch(&mut self, path: &Path, now: u64) {
        self.recent.retain(|e| e.path != path);
        self.recent.insert(0, RecentEntry { path: path.to_path_buf(), opened: now });
        self.recent.truncate(MAX_RECENT);
    }

    /// When `path` was last opened, if ever.
    pub fn opened_at(&self, path: &Path) -> Option<u64> {
        self.recent.iter().find(|e| e.path == path).map(|e| e.opened)
    }

    pub fn is_starred(&self, path: &Path) -> bool {
        self.starred.iter().any(|p| p == path)
    }

    /// Star or unstar; returns the new state.
    pub fn toggle_star(&mut self, path: &Path) -> bool {
        if self.is_starred(path) {
            self.starred.retain(|p| p != path);
            false
        } else {
            self.starred.push(path.to_path_buf());
            true
        }
    }

    /// Forget `path` in Recent (the file itself is untouched).
    pub fn remove_recent(&mut self, path: &Path) {
        self.recent.retain(|e| e.path != path);
    }

    /// Follow a rename.
    pub fn rename(&mut self, from: &Path, to: &Path) {
        for e in &mut self.recent {
            if e.path == from {
                e.path = to.to_path_buf();
            }
        }
        for p in &mut self.starred {
            if p == from {
                *p = to.to_path_buf();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_moves_to_top_without_duplicates() {
        let mut lib = Library::default();
        lib.touch(Path::new("/a.md"), 1);
        lib.touch(Path::new("/b.md"), 2);
        lib.touch(Path::new("/a.md"), 3);
        assert_eq!(lib.recent.len(), 2);
        assert_eq!(lib.recent[0].path, PathBuf::from("/a.md"));
        assert_eq!(lib.opened_at(Path::new("/a.md")), Some(3));
    }

    #[test]
    fn recent_is_capped() {
        let mut lib = Library::default();
        for i in 0..(MAX_RECENT + 10) {
            lib.touch(Path::new(&format!("/{i}.md")), i as u64);
        }
        assert_eq!(lib.recent.len(), MAX_RECENT);
    }

    #[test]
    fn star_toggles_and_follows_rename() {
        let mut lib = Library::default();
        assert!(lib.toggle_star(Path::new("/a.md")));
        lib.touch(Path::new("/a.md"), 1);
        lib.rename(Path::new("/a.md"), Path::new("/b.md"));
        assert!(lib.is_starred(Path::new("/b.md")));
        assert_eq!(lib.recent[0].path, PathBuf::from("/b.md"));
        assert!(!lib.toggle_star(Path::new("/b.md")));
        assert!(lib.starred.is_empty());
    }
}
