//! Custom element: shapes text, paints gutter/selection/matches/cursor, caches layout for hit-testing.
use super::{Editor, Mode};
use crate::settings::Settings;
use crate::theme::{Theme, alpha};
use gpui::{
    App, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity, GlobalElementId,
    InspectorElementId, LayoutId, PaintQuad, Pixels, Point, Rgba, SharedString, Style,
    TextAlign, TextRun, UnderlineStyle, Window, WrappedLine, fill, point, px, relative, size,
};
use std::ops::Range;

const PAD_X: f32 = 10.;
const PAD_Y: f32 = 6.;

#[derive(Clone)]
pub struct LineLayout {
    pub start: usize,
    /// Top of this logical line in content coordinates.
    pub y: Pixels,
    pub line: WrappedLine,
}

/// Cached geometry from the last paint. "Content" coords: origin at text top-left, ignoring scroll.
#[derive(Clone)]
pub struct Layout {
    pub bounds: Bounds<Pixels>,
    /// Screen position of content (0,0), scroll applied.
    pub origin: Point<Pixels>,
    pub lines: Vec<LineLayout>,
    pub lh: Pixels,
    pub content_h: Pixels,
}

impl Layout {
    pub fn point_for_index(&self, idx: usize) -> Option<Point<Pixels>> {
        let l = self.lines.iter().rev().find(|l| l.start <= idx)?;
        let p = l.line.position_for_index((idx - l.start).min(l.line.len()), self.lh)?;
        Some(point(p.x, l.y + p.y))
    }

    pub fn index_at(&self, p: Point<Pixels>) -> usize {
        let Some(l) = self.lines.iter().rev().find(|l| l.y <= p.y).or(self.lines.first()) else {
            return 0;
        };
        let local = point(p.x, (p.y - l.y).max(Pixels::ZERO));
        let i = l.line.closest_index_for_position(local, self.lh).unwrap_or_else(|i| i);
        l.start + i.min(l.line.len())
    }

    /// Rectangles (content coords) covering a byte range.
    fn rects(&self, r: &Range<usize>, right_edge: Pixels) -> Vec<Bounds<Pixels>> {
        let mut out = Vec::new();
        for l in &self.lines {
            let (ls, le) = (l.start, l.start + l.line.len());
            if r.end < ls || r.start > le {
                continue;
            }
            let nl = r.end > le;
            let (s, e) = (r.start.max(ls) - ls, r.end.min(le) - ls);
            if s == e && !nl {
                continue;
            }
            let (Some(a), Some(b)) =
                (l.line.position_for_index(s, self.lh), l.line.position_for_index(e, self.lh))
            else {
                continue;
            };
            let extra = if nl { px(6.) } else { px(0.) };
            if a.y == b.y {
                out.push(Bounds::new(point(a.x, l.y + a.y), size(b.x - a.x + extra, self.lh)));
            } else {
                out.push(Bounds::new(point(a.x, l.y + a.y), size(right_edge - a.x, self.lh)));
                let mut y = a.y + self.lh;
                while y < b.y {
                    out.push(Bounds::new(point(px(0.), l.y + y), size(right_edge, self.lh)));
                    y += self.lh;
                }
                out.push(Bounds::new(point(px(0.), l.y + b.y), size(b.x + extra, self.lh)));
            }
        }
        out
    }
}

pub struct EditorElement {
    pub editor: Entity<Editor>,
}

pub struct Prepaint {
    layout: Layout,
    gutter: Pixels,
    numbers: Vec<(Pixels, gpui::ShapedLine)>,
    quads: Vec<PaintQuad>,
    cursor: Option<PaintQuad>,
    scrollbar: Option<PaintQuad>,
    bg: Rgba,
}

impl gpui::IntoElement for EditorElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = Prepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Prepaint {
        let ed = self.editor.read(cx);
        let st = cx.global::<Settings>();
        let th = Theme::of(cx);
        let doc = ed.mode == Mode::Document;
        let font_size = px(if doc { st.font_size } else { 14. });
        let lh = px((f32::from(font_size) * 1.45).round());
        let mut font = window.text_style().font();
        font.family = st.font_family.clone().into();

        let text = ed.buffer.text();
        let show_nums = doc && st.line_numbers;
        let digits = ed.buffer.line_count().to_string().len().max(2);
        let sys = window.text_system().clone();
        let num_run = |len, c: Rgba| TextRun {
            len,
            font: font.clone(),
            color: c.into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let gutter = if show_nums {
            let w = sys.shape_line("0".repeat(digits).into(), font_size, &[num_run(digits, th.muted)], None).width;
            w + px(20.)
        } else {
            px(0.)
        };
        let view_w = (bounds.size.width - gutter - px(PAD_X * 2.)).max(px(50.));
        let wrap = (doc && st.wrap).then_some(view_w);

        // Text runs (marked/IME range gets an underline).
        let run = TextRun {
            len: text.len(),
            font: font.clone(),
            color: th.fg.into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let mut runs = vec![run.clone()];
        if let Some(m) = ed.marked.clone().filter(|m| m.end <= text.len() && !m.is_empty()) {
            let ul = UnderlineStyle { color: Some(run.color), thickness: px(1.), wavy: false };
            runs = vec![
                TextRun { len: m.start, ..run.clone() },
                TextRun { len: m.end - m.start, underline: Some(ul), ..run.clone() },
                TextRun { len: text.len() - m.end, ..run.clone() },
            ];
        }
        let (shown, runs): (SharedString, _) = if text.is_empty() && !ed.placeholder.is_empty() {
            let r = TextRun { len: ed.placeholder.len(), color: th.muted.into(), ..run };
            (ed.placeholder.clone(), vec![r])
        } else {
            (text.to_string().into(), runs)
        };
        let placeholder_shown = text.is_empty() && !ed.placeholder.is_empty();

        let shaped = sys.shape_text(shown, font_size, &runs, wrap, None).unwrap_or_default();
        let mut lines = Vec::with_capacity(shaped.len());
        let (mut start, mut y) = (0usize, px(0.));
        for line in shaped {
            let h = line.size(lh).height;
            let len = line.len();
            lines.push(LineLayout { start, y, line });
            start += len + 1;
            y += h;
        }
        let content_h = y;

        let mut layout = Layout {
            bounds,
            origin: point(px(0.), px(0.)),
            lines,
            lh,
            content_h,
        };
        if placeholder_shown {
            layout.lines.iter_mut().for_each(|l| l.start = 0);
        }

        // Scroll: clamp, then reveal the cursor if requested.
        let view_h = bounds.size.height - px(PAD_Y * 2.);
        let cursor_pt = layout.point_for_index(ed.buffer.cursor()).unwrap_or(point(px(0.), px(0.)));
        let mut scroll = ed.scroll;
        if ed.reveal {
            if cursor_pt.y < scroll.y {
                scroll.y = cursor_pt.y;
            } else if cursor_pt.y + lh > scroll.y + view_h {
                scroll.y = cursor_pt.y + lh - view_h;
            }
            if cursor_pt.x < scroll.x {
                scroll.x = (cursor_pt.x - px(20.)).max(px(0.));
            } else if cursor_pt.x + px(4.) > scroll.x + view_w {
                scroll.x = cursor_pt.x + px(24.) - view_w;
            }
        }
        let max_y = (content_h - view_h).max(px(0.));
        scroll.y = scroll.y.clamp(px(0.), max_y);
        let max_w = layout.lines.iter().map(|l| l.line.size(lh).width).fold(px(0.), |a, b| a.max(b));
        let max_x = (max_w - view_w + px(24.)).max(px(0.));
        scroll.x = if wrap.is_some() { px(0.) } else { scroll.x.clamp(px(0.), max_x) };
        layout.origin = point(
            bounds.left() + gutter + px(PAD_X) - scroll.x,
            bounds.top() + px(PAD_Y) - scroll.y,
        );

        // Selection, search matches, cursor.
        let right = wrap.unwrap_or(px(100_000.));
        let to_quad = |r: Bounds<Pixels>, c: Rgba| fill(Bounds::new(r.origin + layout.origin, r.size), c);
        let mut quads = Vec::new();
        let focused = ed.focus.is_focused(window);
        if doc && focused && ed.buffer.selection().is_empty() {
            let row = Bounds::new(point(bounds.left() + gutter, cursor_pt.y + layout.origin.y), size(bounds.size.width - gutter, lh));
            quads.push(fill(row, th.line_hl));
        }
        for (i, m) in ed.highlights.iter().enumerate() {
            let c = if Some(i) == ed.current_highlight { th.find_current } else { th.find };
            quads.extend(layout.rects(m, right).into_iter().map(|r| to_quad(r, c)));
        }
        let sel = ed.buffer.selection();
        if sel.start != sel.end {
            quads.extend(layout.rects(&sel, right).into_iter().map(|r| to_quad(r, th.selection)));
        }
        let cursor = (focused && ed.blink && sel.start == sel.end).then(|| {
            fill(Bounds::new(cursor_pt + layout.origin, size(px(2.), lh)), th.fg)
        });

        // Line numbers (screen y of each logical line).
        let cur_line = layout.lines.iter().rposition(|l| l.start <= ed.buffer.cursor()).unwrap_or(0);
        let numbers = if show_nums {
            layout
                .lines
                .iter()
                .enumerate()
                .filter(|(_, l)| {
                    let top = l.y + layout.origin.y;
                    top + l.line.size(lh).height >= bounds.top() && top <= bounds.bottom()
                })
                .map(|(i, l)| {
                    let s: SharedString = (i + 1).to_string().into();
                    let n = s.len();
                    let c = if i == cur_line { th.line_active } else { th.muted };
                    (l.y + layout.origin.y, sys.shape_line(s, font_size, &[num_run(n, c)], None))
                })
                .collect()
        } else {
            Vec::new()
        };

        let scrollbar = (doc && content_h > view_h).then(|| {
            let track = bounds.size.height - px(8.);
            let thumb = (track * (view_h / content_h)).max(px(28.));
            let t = if max_y > px(0.) { scroll.y / max_y } else { 0. };
            let y = bounds.top() + px(4.) + (track - thumb) * t;
            fill(Bounds::new(point(bounds.right() - px(9.), y), size(px(5.), thumb)), alpha(th.fg, 0.22))
        });
        let (reveal, bounds_copy) = (false, layout.clone());
        self.editor.update(cx, |e, _| {
            e.scroll = scroll;
            e.reveal = reveal;
            e.layout = Some(bounds_copy);
        });
        Prepaint { layout, gutter, numbers, quads, cursor, scrollbar, bg: th.bg }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        pre: &mut Prepaint,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.editor.read(cx).focus.clone();
        window.handle_input(&focus, ElementInputHandler::new(bounds, self.editor.clone()), cx);
        let lh = pre.layout.lh;
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for q in pre.quads.drain(..) {
                window.paint_quad(q);
            }
            for l in &pre.layout.lines {
                let o = pre.layout.origin + point(px(0.), l.y);
                if o.y + l.line.size(lh).height < bounds.top() || o.y > bounds.bottom() {
                    continue;
                }
                let _ = l.line.paint(o, lh, TextAlign::Left, None, window, cx);
            }
            if let Some(c) = pre.cursor.take() {
                window.paint_quad(c);
            }
            if let Some(s) = pre.scrollbar.take() {
                window.paint_quad(s);
            }
            if pre.gutter > px(0.) {
                let g = Bounds::new(bounds.origin, size(pre.gutter, bounds.size.height));
                window.paint_quad(fill(g, pre.bg));
                for (y, n) in &pre.numbers {
                    let x = bounds.left() + pre.gutter - px(10.) - n.width;
                    let _ = n.paint(point(x, *y), lh, window, cx);
                }
            }
        });
    }
}
