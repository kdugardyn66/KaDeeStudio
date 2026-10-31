//! Document listing, creation and display helpers.

use crate::modules::ModuleKind;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A document shown in the start-screen file list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub kind: ModuleKind,
    pub size: u64,
    /// Unix seconds; 0 when unknown.
    pub modified: u64,
    /// False when the file no longer exists (kept in Recent).
    pub exists: bool,
}

impl FileEntry {
    /// Read metadata for `path`; `None` if no module owns the file type.
    pub fn from_path(path: &Path) -> Option<FileEntry> {
        let kind = ModuleKind::from_path(path)?;
        let name = path.file_name()?.to_string_lossy().into_owned();
        let meta = fs::metadata(path).ok();
        let modified = meta.as_ref().and_then(|m| m.modified().ok()).map(unix).unwrap_or(0);
        Some(FileEntry {
            path: path.to_path_buf(),
            name,
            kind,
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            modified,
            exists: meta.is_some(),
        })
    }
}

/// Unix seconds now.
pub fn now() -> u64 {
    unix(SystemTime::now())
}

fn unix(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// KadeeStudio documents directly inside `dir` (not recursive).
pub fn list_documents(dir: &Path) -> Vec<FileEntry> {
    let Ok(rd) = fs::read_dir(dir) else { return Vec::new() };
    let mut out: Vec<FileEntry> = rd
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')))
        .filter_map(|p| FileEntry::from_path(&p))
        .collect();
    out.sort_by_key(|e| std::cmp::Reverse(e.modified));
    out
}

/// Create `Untitled <kind>.<ext>` (numbered if taken) in `dir` and return its path.
pub fn create_new_document(kind: ModuleKind, dir: &Path) -> io::Result<PathBuf> {
    let ext = kind
        .new_extension()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, format!("{} does not create documents", kind.name())))?;
    fs::create_dir_all(dir)?;
    let base = format!("Untitled {}", kind.short().to_lowercase());
    let path = unique_path(dir, &base, ext);
    let title = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or(base);
    fs::write(&path, kind.starter_content(&title))?;
    Ok(path)
}

/// `dir/base.ext`, or `dir/base 2.ext`, `dir/base 3.ext`, … if taken.
pub fn unique_path(dir: &Path, base: &str, ext: &str) -> PathBuf {
    let first = dir.join(format!("{base}.{ext}"));
    if !first.exists() {
        return first;
    }
    (2..).map(|n| dir.join(format!("{base} {n}.{ext}"))).find(|p| !p.exists()).expect("an unused name exists")
}

/// `1.2 KB`, `3.4 MB`.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.1} {}", UNITS[i])
}

/// `just now`, `5 min ago`, `3 h ago`, `yesterday`, `4 days ago`, or a day count.
pub fn relative_time(then: u64, now: u64) -> String {
    if then == 0 {
        return "—".into();
    }
    let d = now.saturating_sub(then);
    match d {
        0..=59 => "just now".into(),
        60..=3_599 => format!("{} min ago", d / 60),
        3_600..=86_399 => format!("{} h ago", d / 3_600),
        86_400..=172_799 => "yesterday".into(),
        _ if d < 30 * 86_400 => format!("{} days ago", d / 86_400),
        _ if d < 365 * 86_400 => format!("{} months ago", d / (30 * 86_400)),
        _ => format!("{} years ago", d / (365 * 86_400)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_numbered_documents() {
        let tmp = tempfile::tempdir().unwrap();
        let a = create_new_document(ModuleKind::Markdown, tmp.path()).unwrap();
        let b = create_new_document(ModuleKind::Markdown, tmp.path()).unwrap();
        assert_eq!(a.file_name().unwrap(), "Untitled markdown.md");
        assert_eq!(b.file_name().unwrap(), "Untitled markdown 2.md");
        assert_eq!(fs::read_to_string(&b).unwrap(), "# Untitled markdown 2\n\n");
        assert!(create_new_document(ModuleKind::Pdf, tmp.path()).is_err());
    }

    #[test]
    fn lists_only_known_documents() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("a.md"), "x").unwrap();
        fs::write(tmp.path().join("b.txt"), "x").unwrap();
        fs::write(tmp.path().join(".hidden.md"), "x").unwrap();
        fs::write(tmp.path().join("c.kdash"), "{}").unwrap();
        let names: Vec<_> = list_documents(tmp.path()).into_iter().map(|e| e.name).collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a.md".to_string()));
        assert!(names.contains(&"c.kdash".to_string()));
    }

    #[test]
    fn formats_sizes_and_times() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1536), "1.5 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(relative_time(100, 130), "just now");
        assert_eq!(relative_time(0, 130), "—");
        assert_eq!(relative_time(1000, 1000 + 7200), "2 h ago");
        assert_eq!(relative_time(1000, 1000 + 90_000), "yesterday");
    }
}
