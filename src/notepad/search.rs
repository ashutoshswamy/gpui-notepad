//! Find / replace / go-to-line bars.
use super::{Bar, Notepad};
use crate::editor::{Editor, EditorEvent};
use crate::find;
use gpui::{Context, Entity, Focusable, Window};

impl Notepad {
    /// Events from the tab editors and the three single-line inputs.
    pub(super) fn on_input_event(
        &mut self,
        src: &Entity<Editor>,
        ev: &EditorEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if src == &self.find_input {
            match ev {
                EditorEvent::Changed => {
                    self.refresh_matches(cx);
                    self.jump_to_match(true, false, cx);
                }
                EditorEvent::Submit => self.find_step(true, window, cx),
            }
        } else if src == &self.replace_input {
            if matches!(ev, EditorEvent::Submit) {
                self.replace_one(window, cx);
            }
        } else if src == &self.goto_input {
            if matches!(ev, EditorEvent::Submit) {
                self.goto_line(window, cx);
            }
        } else if matches!(ev, EditorEvent::Changed) && matches!(self.bar, Bar::Find { .. }) {
            self.refresh_matches(cx);
        }
    }

    pub fn open_find(&mut self, replace: bool, window: &mut Window, cx: &mut Context<Self>) {
        let seed = {
            let e = self.active_editor().read(cx);
            let s = e.buffer.selected_text();
            (!s.is_empty() && !s.contains('\n')).then(|| s.to_string())
        };
        self.bar = Bar::Find { replace };
        if let Some(s) = seed {
            self.find_input.update(cx, |e, cx| e.set_text(&s, cx));
        }
        self.refresh_matches(cx);
        window.focus(&self.find_input.focus_handle(cx));
        cx.notify();
    }

    pub fn open_goto(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.bar = Bar::Goto;
        self.goto_input.update(cx, |e, cx| e.set_text("", cx));
        self.refresh_matches(cx);
        window.focus(&self.goto_input.focus_handle(cx));
        cx.notify();
    }

    pub fn close_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.bar == Bar::None {
            return;
        }
        self.bar = Bar::None;
        self.refresh_matches(cx);
        window.focus(&self.active_editor().focus_handle(cx));
        cx.notify();
    }

    /// Recompute matches for the active tab and push highlights to the editors.
    pub fn refresh_matches(&mut self, cx: &mut Context<Self>) {
        self.matches = if matches!(self.bar, Bar::Find { .. }) {
            let q = self.find_input.read(cx).text().to_string();
            find::find_all(self.active_editor().read(cx).text(), &q, self.case_sensitive)
        } else {
            Vec::new()
        };
        let sel = self.active_editor().read(cx).buffer.selection();
        self.cur_match = self.matches.iter().position(|m| *m == sel);
        for (i, t) in self.tabs.clone().into_iter().enumerate() {
            let (m, c) = if i == self.active { (self.matches.clone(), self.cur_match) } else { (Vec::new(), None) };
            t.update(cx, |e, cx| {
                e.highlights = m;
                e.current_highlight = c;
                cx.notify();
            });
        }
        cx.notify();
    }

    /// Select the next/previous match relative to the current selection.
    fn jump_to_match(&mut self, forward: bool, after_sel: bool, cx: &mut Context<Self>) {
        let sel = self.active_editor().read(cx).buffer.selection();
        let from = if forward && after_sel { sel.end } else { sel.start };
        let idx = if forward { find::next_index(&self.matches, from) } else { find::prev_index(&self.matches, from) };
        let Some(i) = idx else { return };
        let r = self.matches[i].clone();
        self.cur_match = Some(i);
        self.active_editor().clone().update(cx, |e, cx| {
            e.current_highlight = Some(i);
            e.select_range(r, cx);
        });
    }

    pub fn find_step(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.bar == Bar::None {
            return self.open_find(false, window, cx);
        }
        self.refresh_matches(cx);
        self.jump_to_match(forward, true, cx);
        cx.notify();
    }

    pub fn replace_one(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let with = self.replace_input.read(cx).text().to_string();
        let editor = self.active_editor().clone();
        let on_match = self.cur_match.is_some_and(|i| self.matches.get(i) == Some(&editor.read(cx).buffer.selection()));
        if on_match {
            editor.update(cx, |e, cx| e.insert(&with, cx));
            self.refresh_matches(cx);
        }
        self.find_step(true, window, cx);
    }

    pub fn replace_all(&mut self, cx: &mut Context<Self>) {
        self.refresh_matches(cx);
        if self.matches.is_empty() {
            return;
        }
        let with = self.replace_input.read(cx).text().to_string();
        let editor = self.active_editor().clone();
        let text = editor.read(cx).text().to_string();
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for m in &self.matches {
            out.push_str(&text[last..m.start]);
            out.push_str(&with);
            last = m.end;
        }
        out.push_str(&text[last..]);
        self.status = format!("Replaced {}", self.matches.len());
        editor.update(cx, |e, cx| e.replace_range(0..text.len(), &out, cx));
        self.refresh_matches(cx);
    }

    pub fn goto_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let n = self.goto_input.read(cx).text().trim().parse::<usize>().ok();
        if let Some(n) = n {
            self.active_editor().clone().update(cx, |e, cx| {
                let off = e.buffer.offset_of_line(n);
                e.select_range(off..off, cx);
            });
        }
        self.close_bar(window, cx);
    }
}
