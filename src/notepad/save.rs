//! Saving: Save / Save As, shared by tab close and quit flows.
use super::Notepad;
use crate::editor::Editor;
use crate::files;
use crate::settings::Settings;
use gpui::{prelude::*, Context, Entity, Window};
use std::path::{Path, PathBuf};

impl Notepad {
    pub fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tab = self.active_editor().clone();
        self.save_tab(tab, false, window, cx);
    }

    pub fn save_as(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tab = self.active_editor().clone();
        self.prompt_save_path(tab, false, window, cx);
    }

    /// Save to the tab's path, or ask for one. Optionally close the tab afterwards.
    pub(super) fn save_tab(&mut self, tab: Entity<Editor>, close: bool, window: &mut Window, cx: &mut Context<Self>) {
        match tab.read(cx).file.path.clone() {
            Some(p) => self.write_tab(tab, p, close, window, cx),
            None => self.prompt_save_path(tab, close, window, cx),
        }
    }

    fn prompt_save_path(&mut self, tab: Entity<Editor>, close: bool, window: &mut Window, cx: &mut Context<Self>) {
        let dir = tab
            .read(cx)
            .file
            .path
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_default();
        let name = if tab.read(cx).file.path.is_some() { tab.read(cx).file.name() } else { "note.txt".into() };
        let rx = cx.prompt_for_new_path(&dir, Some(&name));
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(p))) = rx.await {
                this.update_in(cx, |this, w, cx| this.write_tab(tab, p, close, w, cx)).ok();
            }
        })
        .detach();
    }

    /// Write a tab to `path` and mark it clean. Shared by Save and autosave.
    pub(super) fn write_file(tab: &Entity<Editor>, path: &Path, cx: &mut Context<Self>) -> Result<(), String> {
        let (text, crlf, version, id) = {
            let e = tab.read(cx);
            (e.text().to_string(), e.file.crlf, e.buffer.version, e.file.id)
        };
        let mtime = files::write(path, &text, crlf)?;
        tab.update(cx, |e, _| {
            e.file.path = Some(path.to_path_buf());
            e.file.saved_version = version;
            e.file.mtime = mtime;
            e.file.changed_on_disk = false;
        });
        files::remove_recovery(id);
        Ok(())
    }

    fn write_tab(&mut self, tab: Entity<Editor>, path: PathBuf, close: bool, window: &mut Window, cx: &mut Context<Self>) {
        match Self::write_file(&tab, &path, cx) {
            Ok(()) => {
                cx.update_global(|s: &mut Settings, _| {
                    s.push_recent(path);
                    s.save();
                });
                self.status = "Saved".into();
                if close {
                    self.remove_tab(&tab, window, cx);
                }
            }
            Err(e) => self.status = format!("Save failed: {e}"),
        }
        cx.notify();
    }
}
