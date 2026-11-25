//! Platform text input (typing, IME composition, accents/emoji) for `Editor`.
use super::{Editor, Mode};
use gpui::{Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window, point, size};
use std::ops::Range;

impl Editor {
    fn from_utf16(&self, o: usize) -> usize {
        let (mut u8o, mut u16o) = (0, 0);
        for ch in self.buffer.text().chars() {
            if u16o >= o {
                break;
            }
            u16o += ch.len_utf16();
            u8o += ch.len_utf8();
        }
        u8o
    }

    fn to_utf16(&self, o: usize) -> usize {
        self.buffer.text()[..o].chars().map(char::len_utf16).sum()
    }

    fn range_from_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.from_utf16(r.start)..self.from_utf16(r.end)
    }

    fn range_to_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.to_utf16(r.start)..self.to_utf16(r.end)
    }

    fn target_range(&self, r: Option<Range<usize>>) -> Range<usize> {
        r.map(|r| self.range_from_utf16(&r))
            .or(self.marked.clone())
            .unwrap_or_else(|| self.buffer.selection())
    }
}

impl EntityInputHandler for Editor {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.range_from_utf16(&range);
        actual.replace(self.range_to_utf16(&r));
        Some(self.buffer.text()[r].to_string())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let sel = self.buffer.selection();
        Some(UTF16Selection {
            range: self.range_to_utf16(&sel),
            reversed: self.buffer.cursor() == sel.start && sel.start != sel.end,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|r| self.range_to_utf16(r))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked = None;
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = self.target_range(range);
        self.buffer.set_selection(r.start, r.end);
        self.insert(text, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        new_sel: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = self.target_range(range);
        self.buffer.set_selection(r.start, r.end);
        let text = if self.mode == Mode::Line { text.replace('\n', " ") } else { text.to_string() };
        self.buffer.insert(&text);
        self.changed(cx);
        self.marked = (!text.is_empty()).then(|| r.start..r.start + text.len());
        if let Some(s) = new_sel {
            let s = self.range_from_utf16(&s);
            self.buffer.set_selection(r.start + s.start, r.start + s.end);
        }
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let l = self.layout.as_ref()?;
        let r = self.range_from_utf16(&range);
        let a = l.point_for_index(r.start)? + l.origin;
        let b = l.point_for_index(r.end)? + l.origin;
        Some(Bounds::new(point(a.x, a.y), size((b.x - a.x).max(Pixels::ZERO), l.lh)))
    }

    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let l = self.layout.as_ref()?;
        Some(self.to_utf16(l.index_at(p - l.origin)))
    }
}
