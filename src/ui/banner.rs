use super::button::button;
use crate::notepad::Notepad;
use crate::theme::Theme;
use gpui::{Context, div, prelude::*, px};

/// Shown when the active file changed on disk behind our back.
pub fn disk_banner(np: &Notepad, th: &Theme, cx: &mut Context<Notepad>) -> Option<impl IntoElement> {
    let e = np.active_editor().read(cx);
    if !e.file.changed_on_disk {
        return None;
    }
    let msg = format!("“{}” changed on disk.", e.file.name());
    Some(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .py(px(5.))
            .bg(th.warn)
            .border_b_1()
            .border_color(th.border)
            .text_color(th.fg)
            .text_size(px(12.5))
            .child(div().flex_1().child(msg))
            .child(button("reload", "Reload", th, false, cx, |np: &mut Notepad, _, cx| np.reload_active(cx)))
            .child(button("keep", "Keep mine", th, false, cx, |np: &mut Notepad, _, cx| np.dismiss_disk_change(cx))),
    )
}
