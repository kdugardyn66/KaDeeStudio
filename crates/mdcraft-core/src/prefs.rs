//! mdcraft's preferences (`prefs.json`) and recent documents (`recent.json`), both in
//! the per-user config directory.
//!
//! `prefs.json` = `{"ui": {…}, "render": {…}}`. The UI owns the `ui` object (author
//! name, editor font size, view options…); `render` holds the renderer [`Settings`]
//! (same keys as MarkDown Live's settings: `math`, `mermaidTheme`, `breaks`…).

use crate::config::Settings;
use crate::util;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const RECENT_MAX: usize = 30;

fn prefs_path() -> PathBuf {
    util::config_dir().join("prefs.json")
}

fn recent_path() -> PathBuf {
    util::config_dir().join("recent.json")
}

fn read_json(p: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(p).ok()?;
    serde_json::from_str(&util::jsonc_to_json(&text)).ok()
}

/// Writes through a temporary file and a rename, so a crash never leaves half a file.
pub fn write_atomic(p: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
    let tmp = p.with_file_name(format!(".{name}.mdcraft-{}.tmp", std::process::id()));
    std::fs::write(&tmp, data)?;
    match std::fs::rename(&tmp, p) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Some file systems (network shares) refuse renames over a file: write in place.
            let _ = std::fs::remove_file(&tmp);
            std::fs::write(p, data).map_err(|_| e)
        }
    }
}

/// All preferences, with empty objects for missing parts.
pub fn load() -> Value {
    let mut v = read_json(&prefs_path()).filter(|v| v.is_object()).unwrap_or_else(|| json!({}));
    for k in ["ui", "render"] {
        if !v.get(k).map(|x| x.is_object()).unwrap_or(false) {
            v[k] = json!({});
        }
    }
    v
}

/// Renderer settings: `render` from prefs.json, then `MDCRAFT_SETTINGS` (JSON) on top.
pub fn render_settings() -> Settings {
    let mut v = load()["render"].clone();
    if let Some(env) = std::env::var("MDCRAFT_SETTINGS").ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()) {
        if let (Some(base), Some(over)) = (v.as_object_mut(), env.as_object()) {
            for (k, val) in over {
                base.insert(k.clone(), val.clone());
            }
        }
    }
    Settings::from_json(&v)
}

/// Merges `patch` (`{"ui": {…}, "render": {…}}`, either part optional) into the saved
/// preferences and returns the result. `null` values remove keys.
pub fn update(patch: &Value) -> Result<Value, String> {
    let mut v = load();
    for part in ["ui", "render"] {
        if let Some(obj) = patch.get(part).and_then(|p| p.as_object()) {
            let Some(target) = v.get_mut(part).and_then(|t| t.as_object_mut()) else { continue };
            for (k, val) in obj {
                if val.is_null() {
                    target.remove(k);
                } else {
                    target.insert(k.clone(), val.clone());
                }
            }
        }
    }
    let text = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    write_atomic(&prefs_path(), text.as_bytes()).map_err(|e| format!("cannot save preferences: {e}"))?;
    Ok(v)
}

/// The author name for new documents: preferences, else the account name.
pub fn author() -> String {
    let saved = load()["ui"]["author"].as_str().map(|s| s.trim().to_string()).unwrap_or_default();
    if !saved.is_empty() {
        return saved;
    }
    std::env::var("USER").or_else(|_| std::env::var("USERNAME")).map(|u| capitalise(&u)).unwrap_or_else(|_| "Your Name".into())
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Recent documents, newest first; files that no longer exist are marked `missing`.
pub fn recent() -> Vec<Value> {
    let list = read_json(&recent_path()).and_then(|v| v.as_array().cloned()).unwrap_or_default();
    list.into_iter()
        .filter_map(|e| {
            let path = e.get("path")?.as_str()?.to_string();
            let p = PathBuf::from(&path);
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone());
            let folder = p.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default();
            Some(json!({
                "path": path,
                "name": name,
                "folder": folder,
                "opened": e.get("opened").and_then(|o| o.as_u64()).unwrap_or(0),
                "pinned": e.get("pinned").and_then(|o| o.as_bool()).unwrap_or(false),
                "missing": !p.is_file(),
            }))
        })
        .collect()
}

fn save_recent(list: &[Value]) {
    let slim: Vec<Value> = list.iter().map(|e| json!({"path": e["path"], "opened": e["opened"], "pinned": e["pinned"]})).collect();
    if let Ok(text) = serde_json::to_string_pretty(&slim) {
        let _ = write_atomic(&recent_path(), text.as_bytes());
    }
}

/// Puts `path` at the top of the recent list.
pub fn add_recent(path: &Path) {
    let p = path.to_string_lossy().to_string();
    let mut list = recent();
    let pinned = list.iter().find(|e| e["path"] == p.as_str()).map(|e| e["pinned"].as_bool().unwrap_or(false)).unwrap_or(false);
    list.retain(|e| e["path"] != p.as_str());
    list.insert(0, json!({"path": p, "opened": now_secs(), "pinned": pinned}));
    list.truncate(RECENT_MAX);
    save_recent(&list);
}

/// Removes one entry (`path`) or, with `None`, every unpinned entry.
pub fn remove_recent(path: Option<&str>) {
    let mut list = recent();
    match path {
        Some(p) => list.retain(|e| e["path"] != p),
        None => list.retain(|e| e["pinned"].as_bool().unwrap_or(false)),
    }
    save_recent(&list);
}

pub fn pin_recent(path: &str, pinned: bool) {
    let mut list = recent();
    for e in list.iter_mut() {
        if e["path"] == path {
            e["pinned"] = json!(pinned);
        }
    }
    save_recent(&list);
}

/// Today's date as (year, month, day) in UTC, without a date crate.
pub fn today() -> (i64, u32, u32) {
    civil_from_days((now_secs() / 86_400) as i64)
}

/// Days since 1970-01-01 → (year, month, day). Howard Hinnant's algorithm.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_740), (2026, 10, 14));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    }
    #[test]
    fn prefs_roundtrip() {
        let dir = std::env::temp_dir().join(format!("mdcraft-prefs-test-{}", std::process::id()));
        std::env::set_var("MDCRAFT_CONFIG_DIR", &dir);
        let v = update(&json!({"ui": {"author": "Ada"}, "render": {"math": "mathjax"}})).unwrap();
        assert_eq!(v["ui"]["author"], "Ada");
        assert_eq!(author(), "Ada");
        assert_eq!(render_settings().math, "mathjax");
        let v = update(&json!({"ui": {"author": null}})).unwrap();
        assert!(v["ui"].get("author").is_none());
        let f = dir.join("doc.md");
        std::fs::write(&f, "x").unwrap();
        add_recent(&f);
        assert_eq!(recent()[0]["name"], "doc.md");
        remove_recent(Some(&f.to_string_lossy()));
        assert!(recent().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
