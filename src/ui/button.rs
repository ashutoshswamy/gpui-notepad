use crate::theme::{Theme, alpha};
use gpui::{ClickEvent, Context, Div, ElementId, SharedString, Stateful, Window, div, prelude::*, px};

/// Shared look: flat, 6px radius, quiet hover, soft accent tint when `active`.
pub fn base(id: impl Into<ElementId>, label: impl Into<SharedString>, th: &Theme, active: bool) -> Stateful<Div> {
    let th = *th;
    div()
        .id(id)
        .flex()
        .items_center()
        .h(px(26.))
        .px(px(9.))
        .rounded_md()
        .text_size(px(12.5))
        .whitespace_nowrap()
        .cursor_pointer()
        .bg(if active { alpha(th.accent, 0.22) } else { alpha(th.fg, 0.0) })
        .text_color(if active { th.accent } else { th.fg })
        .hover(move |d| d.bg(if active { alpha(th.accent, 0.32) } else { th.hover }))
        .active(|d| d.opacity(0.75))
        .child(label.into())
}

pub fn button<V: 'static>(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    th: &Theme,
    active: bool,
    cx: &mut Context<V>,
    f: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
) -> Stateful<Div> {
    base(id, label, th, active).on_click(cx.listener(move |v, _: &ClickEvent, w, cx| f(v, w, cx)))
}
