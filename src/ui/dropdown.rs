use super::button::base;
use crate::theme::Theme;
use gpui::{ClickEvent, SharedString, Context, FontWeight, Window, deferred, div, prelude::*, px};
use std::rc::Rc;

/// Select-style button (bordered) that toggles a floating list. `current` marks the selected item.
pub fn dropdown<V: 'static>(
    id: &'static str,
    label: String,
    open: bool,
    items: Vec<String>,
    current: Option<usize>,
    right_align: bool,
    th: &Theme,
    cx: &mut Context<V>,
    on_toggle: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
    on_pick: impl Fn(&mut V, usize, &mut Window, &mut Context<V>) + 'static,
) -> impl IntoElement {
    let on_pick = Rc::new(on_pick);
    let th = *th;
    let mut panel = div()
        .id(SharedString::from(format!("{id}-panel")))
        .absolute()
        .top(px(30.))
        .occlude()
        .flex()
        .flex_col()
        .min_w(px(168.))
        .max_h(px(360.))
        .overflow_y_scroll()
        .p_1()
        .rounded_md()
        .border_1()
        .border_color(th.border)
        .bg(th.bar)
        .shadow_lg()
        .text_color(th.fg)
        .text_size(px(12.5))
        .children(items.into_iter().enumerate().map(|(i, text)| {
            let pick = on_pick.clone();
            let is_cur = current == Some(i);
            div()
                .id((id, i))
                .px_2()
                .h(px(26.))
                .flex()
                .items_center()
                .rounded_sm()
                .cursor_pointer()
                .text_color(if is_cur { th.accent } else { th.fg })
                .font_weight(if is_cur { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                .hover(move |d| d.bg(th.hover))
                .child(text)
                .on_click(cx.listener(move |v, _: &ClickEvent, w, cx| pick(v, i, w, cx)))
        }));
    panel = if right_align { panel.right_0() } else { panel.left_0() };
    div()
        .relative()
        .child(
            base(id, label, &th, false)
                .border_1()
                .border_color(if open { th.accent } else { th.border })
                .on_click(cx.listener(move |v, _: &ClickEvent, w, cx| on_toggle(v, w, cx))),
        )
        .when(open, |d| d.child(deferred(panel).with_priority(1)))
}
