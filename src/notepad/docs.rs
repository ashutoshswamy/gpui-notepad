//! Tab lifecycle: create, select, open (file/drag-drop/recent), close.
use super::Notepad;
use crate::editor::{Editor, Mode};
use crate::files;
use crate::settings::Settings;
use gpui::{prelude::*, Context, Entity, Focusable, PathPromptOptions, PromptLevel, Window};
use std::path::PathBuf;

impl Notepad {
    pub(super) fn make_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<Editor> {
        let e = cx.new(|cx| Editor::new(Mode::Document, cx)); // empty state overlay stands in for a placeholder
        cx.subscribe_in(&e, window, Self::on_input_event).detach();
        e
    }

    pub fn new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let e = self.make_editor(window, cx);
        self.tabs.push(e);
        let i = self.tabs.len() - 1;
        self.select_tab(i, window, cx);
    }

    pub fn select_tab(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.active = i.min(self.tabs.len() - 1);
        self.menu = None;
        self.status.clear();
        self.refresh_matches(cx);
        window.focus(&self.tabs[self.active].focus_handle(cx));
        cx.notify();
    }

    pub fn open_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = rx.await {
                this.update_in(cx, |this, w, cx| {
                    for p in paths {
                        this.open_path(p, w, cx);
                    }
                })
                .ok();
            }
        })
        .detach();
    }

    pub fn open_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.open_path_opts(path, true, window, cx);
    }

    /// Open (or focus) a file. `remember` adds it to Recent. False when it could not be read.
    pub(super) fn open_path_opts(&mut self, path: PathBuf, remember: bool, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if let Some(i) = self.tabs.iter().position(|t| t.read(cx).file.path.as_ref() == Some(&path)) {
            self.select_tab(i, window, cx);
            return true;
        }
        let loaded = match files::read(&path) {
            Ok(l) => l,
            Err(e) => {
                self.status = format!("Open failed: {e}");
                cx.notify();
                return false;
            }
        };
        // Reuse a lone empty untitled tab instead of stacking tabs.
        let reuse = self.tabs.len() == 1 && {
            let e = self.tabs[0].read(cx);
            e.file.path.is_none() && !e.dirty() && e.text().is_empty()
        };
        if !reuse {
            let e = self.make_editor(window, cx);
            self.tabs.push(e);
            self.active = self.tabs.len() - 1;
        } else {
            self.active = 0;
        }
        self.tabs[self.active].update(cx, |e, cx| {
            e.load(&loaded.text, cx);
            e.file.saved_version = e.buffer.version;
            e.file.path = Some(path.clone());
            e.file.mtime = loaded.mtime;
            e.file.crlf = loaded.crlf;
            e.file.changed_on_disk = false;
        });
        if remember {
            cx.update_global(|s: &mut Settings, _| {
                s.push_recent(path);
                s.save();
            });
        }
        self.select_tab(self.active, window, cx);
        true
    }

    /// Drag-reorder: move tab `from` to index `to`, keeping the active tab active.
    pub fn move_tab(&mut self, from: usize, to: usize, window: &mut Window, cx: &mut Context<Self>) {
        if from == to || from >= self.tabs.len() || to >= self.tabs.len() {
            return;
        }
        let active = self.tabs[self.active].clone();
        let t = self.tabs.remove(from);
        self.tabs.insert(to, t);
        let i = self.tabs.iter().position(|t| *t == active).unwrap_or(0);
        self.select_tab(i, window, cx);
    }

    pub fn close_tab(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get(i).cloned() else { return };
        if !tab.read(cx).dirty() {
            return self.remove_tab(&tab, window, cx);
        }
        let name = tab.read(cx).file.name();
        let rx = window.prompt(
            PromptLevel::Warning,
            &format!("Save changes to “{name}”?"),
            Some("Your changes will be lost if you don't save them."),
            &["Save", "Don't Save", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| match rx.await {
            Ok(0) => {
                this.update_in(cx, |this, w, cx| this.save_tab(tab, true, w, cx)).ok();
            }
            Ok(1) => {
                this.update_in(cx, |this, w, cx| this.remove_tab(&tab, w, cx)).ok();
            }
            _ => {}
        })
        .detach();
    }

    /// Drop a tab (by handle, since indexes shift) and its recovery file.
    pub(super) fn remove_tab(&mut self, tab: &Entity<Editor>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(i) = self.tabs.iter().position(|t| t == tab) else { return };
        files::remove_recovery(tab.read(cx).file.id);
        self.tabs.remove(i);
        if self.tabs.is_empty() {
            return self.new_tab(window, cx);
        }
        let next = if self.active > i || self.active >= self.tabs.len() { self.active.saturating_sub(1) } else { self.active };
        self.select_tab(next, window, cx);
    }
}
