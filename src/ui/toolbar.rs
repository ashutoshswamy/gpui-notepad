use super::button::button;
use super::dropdown::dropdown;
use crate::notepad::{Menu, Notepad};
use crate::settings::{SIZES, Settings};
use crate::theme::Theme;
use gpui::{Context, Window, div, prelude::*, px};

fn divider(th: &Theme) -> impl IntoElement {
    div().w(px(1.)).h(px(16.)).mx_1().bg(th.border)
}

fn toggle(m: Menu) -> impl Fn(&mut Notepad, &mut Window, &mut Context<Notepad>) + 'static {
    move |np, _, cx| {
        np.menu = if np.menu == Some(m) { None } else { Some(m) };
        cx.notify();
    }
}

pub fn toolbar(np: &Notepad, th: &Theme, cx: &mut Context<Notepad>) -> impl IntoElement {
    let st = cx.global::<Settings>();
    let (font, size, wrap, nums, theme_name) = (st.font_family.clone(), st.font_size, st.wrap, st.line_numbers, st.theme.clone());
    let recent: Vec<_> = st.recent.clone();
    let recent_names: Vec<String> = recent
        .iter()
        .map(|p| p.file_name().map_or(p.display().to_string(), |n| n.to_string_lossy().into_owned()))
        .collect();
    let themes: Vec<&'static str> = Theme::names().collect();
    let cur_theme = themes.iter().position(|n| *n == theme_name);
    let fonts = np.fonts.clone();
    let cur_font = fonts.iter().position(|f| *f == font);
    let cur_size = SIZES.iter().position(|s| *s == size);
    let open = |m| np.menu == Some(m);

    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap_1()
        .px_2()
        .py(px(5.))
        .bg(th.bg)
        .border_b_1()
        .border_color(th.border)
        .text_color(th.fg)
        .child(button("new", "New", th, false, cx, |np: &mut Notepad, w, cx| np.new_tab(w, cx)))
        .child(button("open", "Open", th, false, cx, |np: &mut Notepad, w, cx| np.open_dialog(w, cx)))
        .child(button("save", "Save", th, false, cx, |np: &mut Notepad, w, cx| np.save(w, cx)))
        .child(dropdown(
            "recent",
            "Recent".into(),
            open(Menu::Recent),
            if recent_names.is_empty() { vec!["No recent files".into()] } else { recent_names },
            None,
            false,
            th,
            cx,
            toggle(Menu::Recent),
            move |np: &mut Notepad, i, w, cx| {
                np.menu = None;
                if let Some(p) = recent.get(i) {
                    np.open_path(p.clone(), w, cx);
                }
            },
        ))
        .child(divider(th))
        .child(dropdown(
            "font",
            font,
            open(Menu::Font),
            fonts.to_vec(),
            cur_font,
            false,
            th,
            cx,
            toggle(Menu::Font),
            move |np: &mut Notepad, i, _, cx| {
                np.menu = None;
                np.update_settings(cx, |s| s.font_family = fonts[i].clone());
            },
        ))
        .child(dropdown(
            "size",
            format!("{size} pt"),
            open(Menu::Size),
            SIZES.iter().map(|s| format!("{s} pt")).collect(),
            cur_size,
            false,
            th,
            cx,
            toggle(Menu::Size),
            |np: &mut Notepad, i, _, cx| {
                np.menu = None;
                np.update_settings(cx, |s| s.font_size = SIZES[i]);
            },
        ))
        .child(divider(th))
        .child(button("wrap", "Wrap", th, wrap, cx, |np: &mut Notepad, _, cx| np.update_settings(cx, |s| s.wrap = !s.wrap)))
        .child(button("nums", "Line numbers", th, nums, cx, |np: &mut Notepad, _, cx| np.update_settings(cx, |s| s.line_numbers = !s.line_numbers)))
        .child(button("date", "Date/Time", th, false, cx, |np: &mut Notepad, _, cx| np.insert_datetime(cx)))
        .child(div().flex_1())
        .child(dropdown(
            "theme",
            theme_name,
            open(Menu::Theme),
            themes.iter().map(|n| n.to_string()).collect(),
            cur_theme,
            true,
            th,
            cx,
            toggle(Menu::Theme),
            move |np: &mut Notepad, i, _, cx| {
                np.menu = None;
                np.update_settings(cx, |s| s.theme = themes[i].into());
            },
        ))
}
