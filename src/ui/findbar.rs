use super::button::button;
use crate::editor::Editor;
use crate::notepad::{Bar, Notepad};
use crate::theme::Theme;
use gpui::{Context, Entity, Window, div, prelude::*, px};

fn input_box(e: Entity<Editor>, th: &Theme, window: &Window, cx: &gpui::App) -> impl IntoElement {
    let focused = e.read(cx).focus.is_focused(window);
    div()
        .w(px(220.))
        .h(px(26.))
        .px_1()
        .rounded_md()
        .border_1()
        .border_color(if focused { th.accent } else { th.border })
        .bg(th.btn)
        .child(e)
}

/// Find/replace or go-to-line bar, depending on `np.bar`.
pub fn bar(np: &Notepad, th: &Theme, window: &Window, cx: &mut Context<Notepad>) -> Option<impl IntoElement> {
    let row = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap_1()
        .px_2()
        .py(px(5.))
        .bg(th.bar)
        .border_b_1()
        .border_color(th.border)
        .text_color(th.fg)
        .text_size(px(12.5));
    match np.bar {
        Bar::None => None,
        Bar::Goto => Some(
            row.child(div().text_color(th.muted).pr_1().child("Go to line"))
                .child(input_box(np.goto_input.clone(), th, window, cx))
                .child(button("go", "Go", th, false, cx, |np: &mut Notepad, w, cx| np.goto_line(w, cx)))
                .child(div().flex_1())
                .child(button("gox", "Close", th, false, cx, |np: &mut Notepad, w, cx| np.close_bar(w, cx))),
        ),
        Bar::Find { replace } => {
            let empty_query = np.find_input.read(cx).text().is_empty();
            let count = if np.matches.is_empty() {
                if empty_query { String::new() } else { "No results".into() }
            } else {
                format!("{} of {}", np.cur_match.map_or(0, |i| i + 1), np.matches.len())
            };
            let mut r = row
                .child(input_box(np.find_input.clone(), th, window, cx))
                .child(button("prev", "Previous", th, false, cx, |np: &mut Notepad, w, cx| np.find_step(false, w, cx)))
                .child(button("next", "Next", th, false, cx, |np: &mut Notepad, w, cx| np.find_step(true, w, cx)))
                .child(button("case", "Match case", th, np.case_sensitive, cx, |np: &mut Notepad, _, cx| {
                    np.case_sensitive = !np.case_sensitive;
                    np.refresh_matches(cx);
                }))
                .child(div().text_color(th.muted).min_w(px(64.)).px_1().child(count));
            if replace {
                r = r
                    .child(input_box(np.replace_input.clone(), th, window, cx))
                    .child(button("rep", "Replace", th, false, cx, |np: &mut Notepad, w, cx| np.replace_one(w, cx)))
                    .child(button("repall", "Replace all", th, false, cx, |np: &mut Notepad, _, cx| np.replace_all(cx)));
            } else {
                r = r.child(button("showrep", "Replace…", th, false, cx, |np: &mut Notepad, w, cx| np.open_find(true, w, cx)));
            }
            Some(r.child(div().flex_1()).child(button("findx", "Close", th, false, cx, |np: &mut Notepad, w, cx| np.close_bar(w, cx))))
        }
    }
}
