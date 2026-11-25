use crate::notepad::Notepad;
use crate::theme::{Theme, alpha};
use gpui::{ClickEvent, Context, MouseButton, SharedString, Window, div, prelude::*, px, transparent_black};

/// Payload (and drag preview) when a tab is dragged to reorder.
#[derive(Clone)]
pub struct DraggedTab {
    from: usize,
    name: String,
}

impl Render for DraggedTab {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = Theme::of(cx);
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(th.accent)
            .bg(th.bg)
            .text_color(th.fg)
            .text_size(px(12.5))
            .shadow_lg()
            .child(self.name.clone())
    }
}

/// VS Code style tab strip: the active tab shares the editor background and joins the toolbar below.
/// Dirty tabs show a dot that turns into × on hover; tabs can be dragged to reorder, middle-click closes.
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
            let group = SharedString::from(format!("tab{i}"));
            let close = div()
                .id(("close", i))
                .flex()
                .items_center()
                .justify_center()
                .flex_none()
                .size(px(20.))
                .rounded_sm()
                .text_color(th.muted)
                .hover(move |d| d.bg(th.hover).text_color(th.fg))
                .when(dirty, |d| {
                    d.child(div().text_color(th.accent).child("●").group_hover(group.clone(), |s| s.hidden()))
                        .child(div().hidden().group_hover(group.clone(), |s| s.flex()).child("×"))
                })
                .when(!dirty, |d| {
                    d.child(div().when(!active, |d| d.invisible().group_hover(group.clone(), |s| s.visible())).child("×"))
                })
                .on_click(cx.listener(move |np, _: &ClickEvent, w, cx| {
                    cx.stop_propagation();
                    np.close_tab(i, w, cx)
                }));
            div()
                .id(("tab", i))
                .group(group.clone())
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .h(px(30.))
                .max_w(px(220.))
                .pl_3()
                .pr_1()
                .rounded_t_md()
                .border_t_2()
                .border_color(if active { th.accent } else { transparent_black().into() })
                .cursor_pointer()
                .bg(if active { th.bg } else { th.tab_inactive })
                .text_color(if active { th.fg } else { th.muted })
                .hover(move |d| if active { d } else { d.text_color(th.fg) })
                .drag_over::<DraggedTab>(move |s, _, _, _| s.bg(alpha(th.accent, 0.18)))
                .child(div().min_w_0().truncate().child(name.clone()))
                .child(close)
                .on_click(cx.listener(move |np, _: &ClickEvent, w, cx| np.select_tab(i, w, cx)))
                .on_mouse_up(MouseButton::Middle, cx.listener(move |np, _, w, cx| np.close_tab(i, w, cx)))
                .on_drag(DraggedTab { from: i, name }, |d, _, _, cx| cx.new(|_| d.clone()))
                .on_drop(cx.listener(move |np, d: &DraggedTab, w, cx| np.move_tab(d.from, i, w, cx)))
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
