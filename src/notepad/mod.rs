//! Root view: tabs, toolbar, find/goto bars, status bar. Behaviour is split by concern.
mod docs;
mod save;
mod updates;
mod safety;
mod search;

use crate::actions::*;
use crate::editor::{Editor, Mode};
use crate::settings::Settings;
use crate::theme::Theme;
use crate::ui::{banner, findbar, statusbar, tabs, toolbar};
use gpui::{
    App, Context, Entity, ExternalPaths, FocusHandle, Focusable, Window, div, prelude::*,
};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Clone, Copy, PartialEq)]
pub enum Menu {
    Font,
    Size,
    Recent,
    Theme,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Bar {
    None,
    Find { replace: bool },
    Goto,
}

pub struct Notepad {
    pub tabs: Vec<Entity<Editor>>,
    pub active: usize,
    pub menu: Option<Menu>,
    pub bar: Bar,
    pub find_input: Entity<Editor>,
    pub replace_input: Entity<Editor>,
    pub goto_input: Entity<Editor>,
    pub case_sensitive: bool,
    pub matches: Vec<Range<usize>>,
    pub cur_match: Option<usize>,
    pub status: String,
    pub update: Option<crate::update::Update>,
    /// tab id -> buffer version last written to the recovery file
    autosaved: HashMap<u64, u64>,
    /// Set once the user confirmed discarding unsaved work, so closing proceeds.
    force_close: bool,
    focus: FocusHandle,
}

impl Focusable for Notepad {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Notepad {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let line = |ph: &'static str, window: &mut Window, cx: &mut Context<Self>| {
            let e = cx.new(|cx| {
                let mut e = Editor::new(Mode::Line, cx);
                e.placeholder = ph.into();
                e
            });
            cx.subscribe_in(&e, window, Self::on_input_event).detach();
            e
        };
        let mut np = Self {
            tabs: Vec::new(),
            active: 0,
            menu: None,
            bar: Bar::None,
            find_input: line("Find", window, cx),
            replace_input: line("Replace with", window, cx),
            goto_input: line("Line number", window, cx),
            case_sensitive: false,
            matches: Vec::new(),
            cur_match: None,
            status: String::new(),
            update: None,
            autosaved: HashMap::new(),
            force_close: false,
            focus: cx.focus_handle(),
        };
        np.restore_recovered(window, cx);
        if np.tabs.is_empty() {
            np.new_tab(window, cx);
        }
        np.start_background_tasks(window, cx);
        if cx.global::<Settings>().check_updates {
            np.check_updates(false, cx);
        }
        np
    }

    pub fn active_editor(&self) -> &Entity<Editor> {
        &self.tabs[self.active]
    }

    pub fn update_settings(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Settings)) {
        cx.update_global(|s: &mut Settings, _| {
            f(s);
            s.save();
        });
        cx.notify();
    }

    pub fn insert_datetime(&mut self, cx: &mut Context<Self>) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        self.active_editor().clone().update(cx, |e, cx| e.insert(&now, cx));
    }
}

impl Render for Notepad {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = Theme::of(cx);
        let ed = self.active_editor().read(cx);
        window.set_window_title(&format!("{}{} - Notepad", ed.file.name(), if ed.dirty() { "*" } else { "" }));

        let mut root = div()
            .key_context("Notepad")
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .flex_col()
            .bg(th.bg)
            .text_color(th.fg)
            .on_drop(cx.listener(|this, p: &ExternalPaths, w, cx| {
                for path in p.paths() {
                    this.open_path(path.clone(), w, cx);
                }
            }));
        macro_rules! act {
            ($($a:ty => |$t:ident, $w:tt, $c:ident| $body:expr),* $(,)?) => {
                $( root = root.on_action(cx.listener(|$t, _: &$a, $w, $c| { $body })); )*
            };
        }
        act! {
            NewTab => |t, w, c| t.new_tab(w, c),
            OpenFile => |t, w, c| t.open_dialog(w, c),
            Save => |t, w, c| t.save(w, c),
            SaveAs => |t, w, c| t.save_as(w, c),
            CloseTab => |t, w, c| { let i = t.active; t.close_tab(i, w, c) },
            NextTab => |t, w, c| { let i = (t.active + 1) % t.tabs.len(); t.select_tab(i, w, c) },
            PrevTab => |t, w, c| { let i = (t.active + t.tabs.len() - 1) % t.tabs.len(); t.select_tab(i, w, c) },
            Find => |t, w, c| t.open_find(false, w, c),
            Replace => |t, w, c| t.open_find(true, w, c),
            FindNext => |t, w, c| t.find_step(true, w, c),
            FindPrev => |t, w, c| t.find_step(false, w, c),
            GoToLine => |t, w, c| t.open_goto(w, c),
            ToggleWrap => |t, _, c| t.update_settings(c, |s| s.wrap = !s.wrap),
            ToggleLineNumbers => |t, _, c| t.update_settings(c, |s| s.line_numbers = !s.line_numbers),
            ToggleTheme => |t, _, c| t.update_settings(c, |s| {
                s.theme = if Theme::by_name(&s.theme).dark { "Light+" } else { "Dark+" }.into()
            }),
            ZoomIn => |t, _, c| t.update_settings(c, |s| s.zoom(2.)),
            ZoomOut => |t, _, c| t.update_settings(c, |s| s.zoom(-2.)),
            InsertDateTime => |t, _, c| t.insert_datetime(c),
            Escape => |t, w, c| { t.menu = None; t.close_bar(w, c) },
            Quit => |t, w, c| t.quit(w, c),
            CheckForUpdates => |t, _, c| t.check_updates(true, c),
            ToggleUpdateCheck => |t, _, c| t.update_settings(c, |s| s.check_updates = !s.check_updates),
        }

        root.child(tabs::tab_strip(self, &th, cx))
            .child(toolbar::toolbar(self, &th, cx))
            .children(findbar::bar(self, &th, window, cx))
            .children(banner::update_banner(self, &th, cx))
            .children(banner::disk_banner(self, &th, cx))
            .child(div().flex_1().min_h_0().child(self.active_editor().clone()))
            .child(statusbar::status_bar(self, &th, cx))
    }
}
