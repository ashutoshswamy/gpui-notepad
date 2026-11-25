use crate::editor::Editor;
use crate::notepad::Notepad;
use crate::theme::{Theme, alpha};
use gpui::{Context, div, prelude::*, px};

pub fn status_bar(np: &Notepad, th: &Theme, cx: &mut Context<Notepad>) -> impl IntoElement {
    let e: &Editor = np.active_editor().read(cx);
    let (ln, col) = e.buffer.line_col(e.buffer.cursor());
    let sel = e.buffer.selection();
    let selected = (!sel.is_empty()).then(|| format!("{} selected", e.buffer.selected_text().chars().count()));
    let path = e.file.path.as_ref().map_or("Untitled".to_string(), |p| p.display().to_string());
    let mut right: Vec<String> = vec![format!("Ln {ln}, Col {col}")];
    right.extend(selected);
    right.push(format!("{} chars", e.buffer.text().chars().count()));
    right.push("UTF-8".into());
    right.push(if e.file.crlf { "CRLF" } else { "LF" }.into());
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(px(24.))
        .px_3()
        .bg(th.status_bg)
        .text_color(th.status_fg)
        .text_size(px(12.))
        .child(
            div()
                .flex()
                .gap_3()
                .child(path)
                .children((!np.status.is_empty()).then(|| div().text_color(alpha(th.status_fg, 0.8)).child(np.status.clone()))),
        )
        .child(div().flex().gap_4().children(right))
}
