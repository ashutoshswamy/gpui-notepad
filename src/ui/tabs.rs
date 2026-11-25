use crate::notepad::Notepad;
use crate::theme::Theme;
use gpui::{ClickEvent, Context, div, prelude::*, px};

/// VS Code style tab strip: the active tab shares the editor background and joins the toolbar below.
pub fn tab_strip(np: &Notepad, th: &Theme, cx: &mut Context<Notepad>) -> impl IntoElement {
    let th = *th;
    let tabs: Vec<(String, bool)> = np.tabs.iter().map(|t| {
        let e = t.read(cx);
        (e.file.name(), e.dirty())
    }).collect();
    div()
        .flex()
        .flex_row()
        .items_end()
        .gap(px(2.))
        .pl_2()
        .pt(px(6.))
        .bg(th.bar)
        .text_size(px(12.5))
        .children(tabs.into_iter().enumerate().map(|(i, (name, dirty))| {
            let active = i == np.active;
            div()
                .id(("tab", i))
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .h(px(30.))
                .pl_3()
                .pr_1()
                .rounded_t_md()
                .cursor_pointer()
                .bg(if active { th.bg } else { th.tab_inactive })
                .text_color(if active { th.fg } else { th.muted })
                .hover(move |d| if active { d } else { d.text_color(th.fg) })
                .child(name)
                .child(
                    div()
                        .id(("close", i))
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(20.))
                        .rounded_sm()
                        .text_color(if dirty { th.accent } else { th.muted })
                        .hover(move |d| d.bg(th.hover).text_color(th.fg))
                        .child(if dirty { "●" } else { "×" })
                        .on_click(cx.listener(move |np, _: &ClickEvent, w, cx| {
                            cx.stop_propagation();
                            np.close_tab(i, w, cx)
                        })),
                )
                .on_click(cx.listener(move |np, _: &ClickEvent, w, cx| np.select_tab(i, w, cx)))
        }))
        .child(
            div()
                .id("newtab")
                .flex()
                .items_center()
                .justify_center()
                .size(px(26.))
                .mb(px(2.))
                .ml_1()
                .rounded_md()
                .cursor_pointer()
                .text_color(th.muted)
                .hover(move |d| d.bg(th.hover).text_color(th.fg))
                .child("+")
                .on_click(cx.listener(|np, _: &ClickEvent, w, cx| np.new_tab(w, cx))),
        )
}
