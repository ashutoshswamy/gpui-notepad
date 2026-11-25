//! Text editor view: one entity per tab, also reused (single-line) for find/goto inputs.
mod element;
mod file_state;
mod input;

pub use file_state::FileState;

use crate::actions::*;
use crate::buffer::Buffer;
use element::{EditorElement, Layout};
use gpui::{
    App, ClipboardItem, Context, CursorStyle, EventEmitter, FocusHandle, Focusable, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollWheelEvent, SharedString,
    Window, div, point, prelude::*, px,
};
use std::ops::Range;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// Multi-line document with gutter, wrap, scrolling.
    Document,
    /// Single-line input (find box etc).
    Line,
}

pub enum EditorEvent {
    Changed,
    Submit,
}

pub struct Editor {
    pub buffer: Buffer,
    pub mode: Mode,
    pub file: FileState,
    pub placeholder: SharedString,
    /// Search matches to highlight and which one is current.
    pub highlights: Vec<Range<usize>>,
    pub current_highlight: Option<usize>,
    pub(crate) focus: FocusHandle,
    pub(crate) scroll: Point<Pixels>,
    pub(crate) marked: Option<Range<usize>>,
    pub(crate) layout: Option<Layout>,
    pub(crate) blink: bool,
    blink_hold: bool,
    pub(crate) reveal: bool,
    selecting: bool,
    goal_x: Option<Pixels>,
}

impl EventEmitter<EditorEvent> for Editor {}

impl Focusable for Editor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Editor {
    pub fn new(mode: Mode, cx: &mut Context<Self>) -> Self {
        // Cursor blink timer; ends when the entity is dropped.
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(530)).await;
                if this.update(cx, |e, cx| {
                    // Stay solid for one full period after any edit/move, like native text fields.
                    let next = std::mem::take(&mut e.blink_hold) || !e.blink;
                    if next != e.blink {
                        e.blink = next;
                        cx.notify();
                    }
                })
                .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        Self {
            buffer: Buffer::default(),
            mode,
            file: FileState::new(),
            placeholder: "".into(),
            highlights: Vec::new(),
            current_highlight: None,
            focus: cx.focus_handle(),
            scroll: point(px(0.), px(0.)),
            marked: None,
            layout: None,
            blink: true,
            blink_hold: false,
            reveal: false,
            selecting: false,
            goal_x: None,
        }
    }

    pub fn dirty(&self) -> bool {
        self.buffer.version != self.file.saved_version
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    /// Replace the whole content (no undo history), cursor to the start.
    pub fn load(&mut self, text: &str, cx: &mut Context<Self>) {
        self.buffer.reset(text);
        self.scroll = point(px(0.), px(0.));
        self.reveal = true;
        cx.notify();
    }

    /// Set content of a Line input and select it all.
    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.buffer.reset(text);
        self.buffer.select_all();
        cx.notify();
    }

    pub fn replace_range(&mut self, r: Range<usize>, text: &str, cx: &mut Context<Self>) {
        self.buffer.replace(r, text);
        self.changed(cx);
    }

    pub fn select_range(&mut self, r: Range<usize>, cx: &mut Context<Self>) {
        self.buffer.set_selection(r.start, r.end);
        self.touched(cx);
    }

    // ---- state change helpers ----
    fn touched(&mut self, cx: &mut Context<Self>) {
        self.reveal = true;
        self.blink = true;
        self.blink_hold = true;
        self.goal_x = None;
        cx.notify();
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        self.marked = None;
        self.touched(cx);
        cx.emit(EditorEvent::Changed);
    }

    /// Insert text at the selection; newlines are dropped in Line mode.
    pub fn insert(&mut self, text: &str, cx: &mut Context<Self>) {
        if self.mode == Mode::Line {
            self.buffer.insert(&text.replace(['\n', '\r'], " "));
        } else {
            self.buffer.insert(text);
        }
        self.changed(cx);
    }

    fn mv(&mut self, to: usize, extend: bool, cx: &mut Context<Self>) {
        self.buffer.move_to(to, extend);
        self.marked = None;
        self.touched(cx);
    }

    fn horiz(&mut self, left: bool, extend: bool, cx: &mut Context<Self>) {
        let sel = self.buffer.selection();
        let to = match (self.buffer.has_selection() && !extend, left) {
            (true, true) => sel.start,
            (true, false) => sel.end,
            (false, true) => self.buffer.prev_boundary(self.buffer.cursor()),
            (false, false) => self.buffer.next_boundary(self.buffer.cursor()),
        };
        self.mv(to, extend, cx);
    }

    fn words(&mut self, left: bool, extend: bool, cx: &mut Context<Self>) {
        let c = self.buffer.cursor();
        let to = if left { self.buffer.word_left(c) } else { self.buffer.word_right(c) };
        self.mv(to, extend, cx);
    }

    /// Move the cursor `dy` pixels up/down, keeping the horizontal goal column.
    fn vertical(&mut self, dy: Pixels, extend: bool, cx: &mut Context<Self>) {
        if self.mode == Mode::Line {
            return;
        }
        let Some(l) = &self.layout else { return };
        let Some(p) = l.point_for_index(self.buffer.cursor()) else { return };
        let gx = self.goal_x.unwrap_or(p.x);
        let y = p.y + dy + l.lh / 2.;
        let to = if y < px(0.) {
            0
        } else if y > l.content_h {
            self.buffer.len()
        } else {
            l.index_at(point(gx, y))
        };
        self.buffer.move_to(to, extend);
        self.touched(cx);
        self.goal_x = Some(gx);
    }

    fn page(&self) -> Pixels {
        self.layout.as_ref().map_or(px(200.), |l| l.bounds.size.height - l.lh)
    }

    fn copy_selection(&self, cx: &mut Context<Self>) -> bool {
        if !self.buffer.has_selection() {
            return false;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(self.buffer.selected_text().to_string()));
        true
    }

    fn undo_redo(&mut self, undo: bool, cx: &mut Context<Self>) {
        let done = if undo { self.buffer.undo() } else { self.buffer.redo() };
        if done {
            self.changed(cx);
        }
    }

    // ---- mouse ----
    fn mouse_down(&mut self, e: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus);
        let Some(l) = &self.layout else { return };
        let idx = l.index_at(e.position - l.origin);
        match e.click_count {
            1 => {
                self.buffer.move_to(idx, e.modifiers.shift);
                self.selecting = true;
            }
            2 => {
                let r = self.buffer.word_range(idx);
                self.buffer.set_selection(r.start, r.end);
            }
            _ => {
                let s = self.buffer.line_start(idx);
                let end = (self.buffer.line_end(idx) + 1).min(self.buffer.len());
                self.buffer.set_selection(s, end);
            }
        }
        self.marked = None;
        self.touched(cx);
    }

    fn mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }

    fn mouse_move(&mut self, e: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selecting {
            return;
        }
        if e.pressed_button != Some(MouseButton::Left) {
            self.selecting = false;
            return;
        }
        if let Some(l) = &self.layout {
            let idx = l.index_at(e.position - l.origin);
            self.buffer.move_to(idx, true);
            self.touched(cx);
        }
    }

    fn scroll_wheel(&mut self, e: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let lh = self.layout.as_ref().map_or(px(20.), |l| l.lh);
        let d = e.delta.pixel_delta(lh);
        self.scroll.y -= d.y;
        self.scroll.x -= d.x;
        cx.notify();
    }
}

impl Render for Editor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut d = div()
            .key_context("Editor")
            .track_focus(&self.focus)
            .size_full()
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_mouse_move(cx.listener(Self::mouse_move))
            .on_scroll_wheel(cx.listener(Self::scroll_wheel));
        macro_rules! act {
            ($($a:ty => |$t:ident, $c:ident| $body:expr),* $(,)?) => {
                $( d = d.on_action(cx.listener(|$t, _: &$a, _, $c| { $body })); )*
            };
        }
        act! {
            Backspace => |t, c| { t.marked = None; t.buffer.delete_back(); t.changed(c) },
            Delete => |t, c| { t.buffer.delete_forward(); t.changed(c) },
            DeleteWordBack => |t, c| {
                if !t.buffer.has_selection() {
                    let h = t.buffer.cursor();
                    t.buffer.set_selection(t.buffer.word_left(h), h);
                }
                t.buffer.insert(""); t.changed(c)
            },
            DeleteWordFwd => |t, c| {
                if !t.buffer.has_selection() {
                    let h = t.buffer.cursor();
                    t.buffer.set_selection(h, t.buffer.word_right(h));
                }
                t.buffer.insert(""); t.changed(c)
            },
            Left => |t, c| t.horiz(true, false, c),
            Right => |t, c| t.horiz(false, false, c),
            SelectLeft => |t, c| t.horiz(true, true, c),
            SelectRight => |t, c| t.horiz(false, true, c),
            Up => |t, c| { let h = t.layout.as_ref().map_or(px(20.), |l| l.lh); t.vertical(-h, false, c) },
            Down => |t, c| { let h = t.layout.as_ref().map_or(px(20.), |l| l.lh); t.vertical(h, false, c) },
            SelectUp => |t, c| { let h = t.layout.as_ref().map_or(px(20.), |l| l.lh); t.vertical(-h, true, c) },
            SelectDown => |t, c| { let h = t.layout.as_ref().map_or(px(20.), |l| l.lh); t.vertical(h, true, c) },
            PageUp => |t, c| { let p = t.page(); t.vertical(-p, false, c) },
            PageDown => |t, c| { let p = t.page(); t.vertical(p, false, c) },
            WordLeft => |t, c| t.words(true, false, c),
            WordRight => |t, c| t.words(false, false, c),
            SelectWordLeft => |t, c| t.words(true, true, c),
            SelectWordRight => |t, c| t.words(false, true, c),
            LineStart => |t, c| { let to = t.buffer.line_start(t.buffer.cursor()); t.mv(to, false, c) },
            LineEnd => |t, c| { let to = t.buffer.line_end(t.buffer.cursor()); t.mv(to, false, c) },
            SelectLineStart => |t, c| { let to = t.buffer.line_start(t.buffer.cursor()); t.mv(to, true, c) },
            SelectLineEnd => |t, c| { let to = t.buffer.line_end(t.buffer.cursor()); t.mv(to, true, c) },
            DocStart => |t, c| t.mv(0, false, c),
            DocEnd => |t, c| { let n = t.buffer.len(); t.mv(n, false, c) },
            SelectDocStart => |t, c| t.mv(0, true, c),
            SelectDocEnd => |t, c| { let n = t.buffer.len(); t.mv(n, true, c) },
            SelectAll => |t, c| { t.buffer.select_all(); t.touched(c) },
            Copy => |t, c| { t.copy_selection(c); },
            Cut => |t, c| if t.copy_selection(c) { t.buffer.insert(""); t.changed(c) },
            Paste => |t, c| {
                if let Some(s) = c.read_from_clipboard().and_then(|i| i.text()) { t.insert(&s, c) }
            },
            Undo => |t, c| t.undo_redo(true, c),
            Redo => |t, c| t.undo_redo(false, c),
            Enter => |t, c| {
                if t.mode == Mode::Line { c.emit(EditorEvent::Submit) } else { t.insert("\n", c) }
            },
            Tab => |t, c| if t.mode == Mode::Document { t.insert("    ", c) },
        }
        d = d.on_action(cx.listener(|_, _: &ShowCharacterPalette, w, _| w.show_character_palette()));
        d.child(EditorElement { editor: cx.entity() })
    }
}
