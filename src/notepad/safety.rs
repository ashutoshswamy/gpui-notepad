//! Data-loss protection: autosave/crash recovery, disk-change watch, close confirmation.
use super::Notepad;
use crate::editor::Editor;
use crate::files;
use crate::settings::Settings;
use gpui::{prelude::*, Context, PromptLevel, Window};
use std::time::Duration;

const AUTOSAVE_EVERY_TICKS: u32 = 5;

impl Notepad {
    /// Re-open unsaved notes left behind by a crash or forced quit.
    pub(super) fn restore_recovered(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let recovered = files::take_recovered();
        if recovered.is_empty() {
            return;
        }
        self.status = format!("Recovered {} unsaved note(s)", recovered.len());
        for r in recovered {
            let e = self.make_editor(window, cx);
            e.update(cx, |e, cx| {
                e.load(&r.text, cx);
                e.file.saved_version = e.buffer.version.wrapping_sub(1); // mark dirty
                e.file.mtime = r.path.as_deref().and_then(files::mtime);
                e.file.path = r.path;
            });
            self.tabs.push(e);
        }
    }

    pub(super) fn start_background_tasks(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Autosave when the user switches away; re-render when the OS goes light/dark.
        cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() {
                this.autosave_files(cx);
            }
        })
        .detach();
        cx.observe_window_appearance(window, |_, _, cx| cx.notify()).detach();
        cx.spawn(async move |this, cx| {
            let mut tick = 0u32;
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                tick += 1;
                let alive = this.update(cx, |this, cx| {
                    this.check_disk(cx);
                    if tick % AUTOSAVE_EVERY_TICKS == 0 {
                        this.autosave(cx);
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    /// Reopen the files that were open last run, with their cursors and active tab.
    pub(super) fn restore_session(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (session, active) = {
            let s = cx.global::<Settings>();
            (s.session.clone(), s.session_active)
        };
        let mut active_tab = None;
        for (i, (path, cursor)) in session.into_iter().enumerate() {
            if !path.exists() || !self.open_path_opts(path, false, window, cx) {
                continue;
            }
            let tab = self.active_editor().clone();
            tab.update(cx, |e, cx| {
                let c = cursor.min(e.text().len());
                if e.text().is_char_boundary(c) {
                    e.select_range(c..c, cx);
                }
            });
            if i == active {
                active_tab = Some(tab);
            }
        }
        if let Some(i) = active_tab.and_then(|t| self.tabs.iter().position(|x| *x == t)) {
            self.select_tab(i, window, cx);
        }
    }

    /// Remember open files (untitled notes are covered by crash recovery). Writes only on change.
    pub(super) fn save_session(&mut self, cx: &mut Context<Self>) {
        let mut session = Vec::new();
        let mut active = 0;
        for (i, t) in self.tabs.iter().enumerate() {
            let e = t.read(cx);
            if let Some(p) = &e.file.path {
                if i == self.active {
                    active = session.len();
                }
                session.push((p.clone(), e.buffer.cursor()));
            }
        }
        let s = cx.global::<Settings>();
        if s.session != session || s.session_active != active {
            cx.update_global(|s: &mut Settings, _| {
                s.session = session;
                s.session_active = active;
                s.save();
            });
        }
    }

    /// Autosave setting: write dirty tabs that already have a file. Skips files changed on disk
    /// so we never clobber someone else's edit.
    pub(super) fn autosave_files(&mut self, cx: &mut Context<Self>) {
        if !cx.global::<Settings>().autosave {
            return;
        }
        let mut saved = false;
        for t in self.tabs.clone() {
            let path = {
                let e = t.read(cx);
                if !e.dirty() || e.file.changed_on_disk { continue }
                let Some(p) = e.file.path.clone() else { continue };
                p
            };
            match Self::write_file(&t, &path, cx) {
                Ok(()) => saved = true,
                Err(err) => self.status = format!("Autosave failed: {err}"),
            }
        }
        if saved {
            cx.notify();
        }
    }

    fn autosave(&mut self, cx: &mut Context<Self>) {
        self.autosave_files(cx);
        self.save_session(cx);
        for t in &self.tabs {
            let e = t.read(cx);
            if e.dirty() && self.autosaved.get(&e.file.id) != Some(&e.buffer.version) {
                files::save_recovery(e.file.id, e.file.path.as_deref(), e.text());
                self.autosaved.insert(e.file.id, e.buffer.version);
            }
        }
    }

    fn check_disk(&mut self, cx: &mut Context<Self>) {
        let mut flagged = false;
        for t in self.tabs.clone() {
            t.update(cx, |e, _| {
                if let Some(p) = &e.file.path {
                    if !e.file.changed_on_disk && files::mtime(p) != e.file.mtime {
                        e.file.changed_on_disk = true;
                        flagged = true;
                    }
                }
            });
        }
        if flagged {
            cx.notify();
        }
    }

    pub fn reload_active(&mut self, cx: &mut Context<Self>) {
        let editor = self.active_editor().clone();
        let Some(path) = editor.read(cx).file.path.clone() else { return };
        match files::read(&path) {
            Ok(l) => editor.update(cx, |e, cx| {
                e.load(&l.text, cx);
                e.file.saved_version = e.buffer.version;
                e.file.mtime = l.mtime;
                e.file.crlf = l.crlf;
                e.file.changed_on_disk = false;
            }),
            Err(err) => self.status = format!("Reload failed: {err}"),
        }
        self.refresh_matches(cx);
        cx.notify();
    }

    pub fn dismiss_disk_change(&mut self, cx: &mut Context<Self>) {
        self.active_editor().clone().update(cx, |e, _| {
            e.file.mtime = e.file.path.as_deref().and_then(files::mtime);
            e.file.changed_on_disk = false;
        });
        cx.notify();
    }

    pub fn quit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.request_close(window, cx, true) {
            cx.quit();
        }
    }

    /// True when it is fine to close now. Otherwise asks the user and, on "Discard", closes later.
    pub fn request_close(&mut self, window: &mut Window, cx: &mut Context<Self>, quit: bool) -> bool {
        self.autosave_files(cx);
        self.save_session(cx);
        let dirty: Vec<&Editor> = self.tabs.iter().map(|t| t.read(cx)).filter(|e| e.dirty()).collect();
        if self.force_close || dirty.is_empty() {
            return true;
        }
        let msg = format!("{} note(s) have unsaved changes.", dirty.len());
        let rx = window.prompt(
            PromptLevel::Warning,
            &msg,
            Some("Discard them and close, or cancel and save first."),
            &["Cancel", "Discard"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(1) = rx.await {
                this.update_in(cx, |this, w, cx| {
                    this.force_close = true;
                    for t in &this.tabs {
                        files::remove_recovery(t.read(cx).file.id);
                    }
                    if quit {
                        cx.quit();
                    } else {
                        w.remove_window();
                    }
                })
                .ok();
            }
        })
        .detach();
        false
    }
}
