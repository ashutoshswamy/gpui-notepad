//! Disk I/O: reading/writing notes, config dir, crash-recovery files.
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn config_dir() -> PathBuf {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).unwrap_or_default();
    PathBuf::from(home).join(".config").join("gpui-notepad")
}

fn recovery_dir() -> PathBuf {
    config_dir().join("recovery")
}

/// Loaded file: text normalised to `\n`, plus whether it used CRLF.
pub struct Loaded {
    pub text: String,
    pub crlf: bool,
    pub mtime: Option<SystemTime>,
}

pub fn read(path: &Path) -> Result<Loaded, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let raw = String::from_utf8(bytes).map_err(|_| "not valid UTF-8".to_string())?;
    let crlf = raw.contains("\r\n");
    let text = if crlf { raw.replace("\r\n", "\n") } else { raw };
    Ok(Loaded { text, crlf, mtime: mtime(path) })
}

pub fn write(path: &Path, text: &str, crlf: bool) -> Result<Option<SystemTime>, String> {
    let out = if crlf { text.replace('\n', "\r\n") } else { text.to_string() };
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(mtime(path))
}

pub fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

// ---- crash recovery: one `<id>.rec` per unsaved tab; first line is the original path (or empty) ----

pub struct Recovered {
    pub path: Option<PathBuf>,
    pub text: String,
}

pub fn save_recovery(id: u64, path: Option<&Path>, text: &str) {
    let _ = std::fs::create_dir_all(recovery_dir());
    let head = path.map(|p| p.display().to_string()).unwrap_or_default();
    let _ = std::fs::write(recovery_dir().join(format!("{id}.rec")), format!("{head}\n{text}"));
}

pub fn remove_recovery(id: u64) {
    let _ = std::fs::remove_file(recovery_dir().join(format!("{id}.rec")));
}

/// Load and delete leftover recovery files (they get re-created by autosave if still dirty).
pub fn take_recovered() -> Vec<Recovered> {
    let Ok(rd) = std::fs::read_dir(recovery_dir()) else { return Vec::new() };
    let mut out = Vec::new();
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "rec") {
            if let Ok(raw) = std::fs::read_to_string(&p) {
                let (head, text) = raw.split_once('\n').unwrap_or(("", ""));
                out.push(Recovered {
                    path: (!head.is_empty()).then(|| PathBuf::from(head)),
                    text: text.to_string(),
                });
            }
            let _ = std::fs::remove_file(p);
        }
    }
    out
}
