//! Pure text buffer: content, selection, undo/redo. No UI types, unit-testable.
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
struct Edit {
    start: usize,
    removed: String,
    inserted: String,
    before: (usize, usize),
    after: (usize, usize),
}

#[derive(Default)]
pub struct Buffer {
    text: String,
    anchor: usize,
    head: usize,
    undo: Vec<Edit>,
    redo: Vec<Edit>,
    coalesce: bool,
    /// Bumped on every content change; used for dirty tracking and caches.
    pub version: u64,
}

fn class(c: char) -> u8 {
    if c.is_alphanumeric() || c == '_' {
        1
    } else if c.is_whitespace() {
        0
    } else {
        2
    }
}

impl Buffer {
    #[cfg(test)]
    pub fn new(text: &str) -> Self {
        Self { text: text.to_string(), ..Default::default() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Replace everything and drop history (used when loading a file).
    pub fn reset(&mut self, text: &str) {
        self.text = text.to_string();
        self.anchor = 0;
        self.head = 0;
        self.undo.clear();
        self.redo.clear();
        self.coalesce = false;
        self.version += 1;
    }

    // ---- selection ----
    pub fn cursor(&self) -> usize {
        self.head
    }

    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    pub fn has_selection(&self) -> bool {
        self.anchor != self.head
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }

    pub fn set_selection(&mut self, anchor: usize, head: usize) {
        self.anchor = anchor.min(self.len());
        self.head = head.min(self.len());
        self.coalesce = false;
    }

    pub fn move_to(&mut self, offset: usize, extend: bool) {
        let o = offset.min(self.len());
        self.head = o;
        if !extend {
            self.anchor = o;
        }
        self.coalesce = false;
    }

    pub fn select_all(&mut self) {
        self.set_selection(0, self.len());
    }

    // ---- editing ----
    pub fn replace(&mut self, r: Range<usize>, new: &str) {
        let before = (self.anchor, self.head);
        let removed = self.text[r.clone()].to_string();
        self.text.replace_range(r.clone(), new);
        let pos = r.start + new.len();
        self.anchor = pos;
        self.head = pos;
        self.version += 1;
        self.redo.clear();
        let simple = removed.is_empty() && !new.is_empty() && !new.contains('\n');
        if self.coalesce && simple {
            if let Some(last) = self.undo.last_mut() {
                if last.start + last.inserted.len() == r.start {
                    last.inserted.push_str(new);
                    last.after = (pos, pos);
                    return;
                }
            }
        }
        self.undo.push(Edit {
            start: r.start,
            removed,
            inserted: new.to_string(),
            before,
            after: (pos, pos),
        });
        self.coalesce = simple;
    }

    /// Replace the selection (or insert at the cursor).
    pub fn insert(&mut self, s: &str) {
        self.replace(self.selection(), s);
    }

    pub fn delete_back(&mut self) {
        if !self.has_selection() {
            let p = self.prev_boundary(self.head);
            self.anchor = p;
        }
        self.insert("");
    }

    pub fn delete_forward(&mut self) {
        if !self.has_selection() {
            let n = self.next_boundary(self.head);
            self.anchor = n;
        }
        self.insert("");
    }

    pub fn undo(&mut self) -> bool {
        let Some(e) = self.undo.pop() else { return false };
        self.text.replace_range(e.start..e.start + e.inserted.len(), &e.removed);
        (self.anchor, self.head) = e.before;
        self.redo.push(e);
        self.version += 1;
        self.coalesce = false;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(e) = self.redo.pop() else { return false };
        self.text.replace_range(e.start..e.start + e.removed.len(), &e.inserted);
        (self.anchor, self.head) = e.after;
        self.undo.push(e);
        self.version += 1;
        self.coalesce = false;
        true
    }

    // ---- navigation ----
    pub fn prev_boundary(&self, o: usize) -> usize {
        self.text[..o].graphemes(true).next_back().map_or(0, |g| o - g.len())
    }

    pub fn next_boundary(&self, o: usize) -> usize {
        self.text[o..].graphemes(true).next().map_or(self.len(), |g| o + g.len())
    }

    pub fn word_left(&self, o: usize) -> usize {
        let mut i = o;
        let mut it = self.text[..o].chars().rev().peekable();
        while let Some(&c) = it.peek() {
            if class(c) != 0 {
                break;
            }
            i -= c.len_utf8();
            it.next();
        }
        if let Some(&first) = it.peek() {
            let k = class(first);
            while let Some(&c) = it.peek() {
                if class(c) != k {
                    break;
                }
                i -= c.len_utf8();
                it.next();
            }
        }
        i
    }

    pub fn word_right(&self, o: usize) -> usize {
        let mut i = o;
        let mut it = self.text[o..].chars().peekable();
        while let Some(&c) = it.peek() {
            if class(c) != 0 {
                break;
            }
            i += c.len_utf8();
            it.next();
        }
        if let Some(&first) = it.peek() {
            let k = class(first);
            while let Some(&c) = it.peek() {
                if class(c) != k {
                    break;
                }
                i += c.len_utf8();
                it.next();
            }
        }
        i
    }

    /// Word under `o` (double click).
    pub fn word_range(&self, o: usize) -> Range<usize> {
        let k = self.text[o..].chars().next().or_else(|| self.text[..o].chars().next_back()).map_or(0, class);
        let mut s = o;
        for c in self.text[..o].chars().rev() {
            if class(c) != k {
                break;
            }
            s -= c.len_utf8();
        }
        let mut e = o;
        for c in self.text[o..].chars() {
            if class(c) != k {
                break;
            }
            e += c.len_utf8();
        }
        s..e
    }

    pub fn line_start(&self, o: usize) -> usize {
        self.text[..o].rfind('\n').map_or(0, |i| i + 1)
    }

    pub fn line_end(&self, o: usize) -> usize {
        self.text[o..].find('\n').map_or(self.len(), |i| o + i)
    }

    /// 1-based (line, column in chars).
    pub fn line_col(&self, o: usize) -> (usize, usize) {
        let line = self.text[..o].matches('\n').count() + 1;
        (line, self.text[self.line_start(o)..o].chars().count() + 1)
    }

    pub fn line_count(&self) -> usize {
        self.text.matches('\n').count() + 1
    }

    /// Start offset of 1-based `line`, clamped to the last line.
    pub fn offset_of_line(&self, line: usize) -> usize {
        let mut off = 0;
        for _ in 1..line.max(1) {
            match self.text[off..].find('\n') {
                Some(i) => off += i + 1,
                None => break,
            }
        }
        off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_coalesces_and_undoes() {
        let mut b = Buffer::new("");
        for c in "hello".chars() {
            b.insert(&c.to_string());
        }
        assert_eq!(b.text(), "hello");
        assert!(b.undo());
        assert_eq!(b.text(), "");
        assert!(b.redo());
        assert_eq!(b.text(), "hello");
        b.move_to(0, false);
        b.move_to(5, true);
        b.insert("x");
        assert_eq!(b.text(), "x");
        b.undo();
        assert_eq!(b.text(), "hello");
        assert_eq!(b.selection(), 0..5);
    }

    #[test]
    fn words_and_lines() {
        let b = Buffer::new("foo bar_1  baz\nsecond");
        assert_eq!(b.word_right(0), 3);
        assert_eq!(b.word_right(3), 9);
        assert_eq!(b.word_left(9), 4);
        assert_eq!(b.word_range(5), 4..9);
        assert_eq!(b.line_col(16), (2, 2));
        assert_eq!(b.offset_of_line(2), 15);
        assert_eq!(b.offset_of_line(99), 15);
    }
}
