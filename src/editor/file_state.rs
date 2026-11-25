use std::path::PathBuf;
use std::time::SystemTime;

/// File-related metadata for a document tab.
pub struct FileState {
    /// Stable id, names the crash-recovery file.
    pub id: u64,
    pub path: Option<PathBuf>,
    /// `Buffer::version` at the last save/load; differs => unsaved changes.
    pub saved_version: u64,
    /// Disk mtime when we last read/wrote the file.
    pub mtime: Option<SystemTime>,
    pub crlf: bool,
    /// Set when the file changed on disk behind our back.
    pub changed_on_disk: bool,
}

impl FileState {
    pub fn new() -> Self {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        Self { id, path: None, saved_version: 0, mtime: None, crlf: false, changed_on_disk: false }
    }

    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or("Untitled".into(), |n| n.to_string_lossy().into_owned())
    }
}
