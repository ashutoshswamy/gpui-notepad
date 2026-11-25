//! Colour themes modelled on popular VS Code themes. Each maps the VS Code workbench
//! keys the notepad actually uses (editor, tabs, buttons, status bar, selection...).
use crate::settings::Settings;
use gpui::{App, Rgba, WindowAppearance, rgb, rgba};

#[derive(Clone, Copy)]
pub struct Theme {
    #[allow(dead_code)]
    pub name: &'static str,
    pub dark: bool,
    /// editor.background
    pub bg: Rgba,
    /// editor.foreground
    pub fg: Rgba,
    /// editorLineNumber.foreground and secondary UI text
    pub muted: Rgba,
    /// editorLineNumber.activeForeground
    pub line_active: Rgba,
    /// editorGroupHeader.tabsBackground, find bar
    pub bar: Rgba,
    /// tab.inactiveBackground
    pub tab_inactive: Rgba,
    /// input/dropdown background
    pub btn: Rgba,
    /// panel.border
    pub border: Rgba,
    /// focusBorder
    pub accent: Rgba,
    /// editor.selectionBackground
    pub selection: Rgba,
    pub status_bg: Rgba,
    pub status_fg: Rgba,
    pub find: Rgba,
    pub find_current: Rgba,
    pub line_hl: Rgba,
    pub warn: Rgba,
    pub hover: Rgba,
}

pub fn alpha(c: Rgba, a: f32) -> Rgba {
    Rgba { a, ..c }
}

struct Def {
    name: &'static str,
    dark: bool,
    // bg, fg, muted, line_active, bar, tab_inactive, btn, border, accent, status_bg, status_fg
    c: [u32; 11],
    selection: u32,
}

const THEMES: &[Def] = &[
    Def { name: "Dark+", dark: true, c: [0x1e1e1e, 0xd4d4d4, 0x858585, 0xc6c6c6, 0x252526, 0x2d2d2d, 0x3c3c3c, 0x454545, 0x007fd4, 0x007acc, 0xffffff], selection: 0x264f78ff },
    Def { name: "Light+", dark: false, c: [0xffffff, 0x1f1f1f, 0x6e7681, 0x1f1f1f, 0xf3f3f3, 0xececec, 0xe8e8e8, 0xd4d4d4, 0x005fb8, 0x007acc, 0xffffff], selection: 0xadd6ffff },
    Def { name: "One Dark Pro", dark: true, c: [0x282c34, 0xabb2bf, 0x495162, 0x737984, 0x21252b, 0x21252b, 0x2c313a, 0x3b4048, 0x528bff, 0x21252b, 0x9da5b4], selection: 0x3e4451ff },
    Def { name: "Dracula", dark: true, c: [0x282a36, 0xf8f8f2, 0x6272a4, 0xf8f8f2, 0x21222c, 0x191a21, 0x343746, 0x44475a, 0xbd93f9, 0x191a21, 0xf8f8f2], selection: 0x44475aff },
    Def { name: "Monokai", dark: true, c: [0x272822, 0xf8f8f2, 0x90908a, 0xc2c2bf, 0x1e1f1c, 0x34352f, 0x414339, 0x49483e, 0xa6e22e, 0x75715e, 0xf8f8f2], selection: 0x878b9180 },
    Def { name: "Solarized Dark", dark: true, c: [0x002b36, 0x839496, 0x586e75, 0x93a1a1, 0x00212b, 0x004052, 0x073642, 0x0a4a5a, 0x2aa198, 0x00212b, 0x93a1a1], selection: 0x274642ff },
    Def { name: "Solarized Light", dark: false, c: [0xfdf6e3, 0x586e75, 0x93a1a1, 0x586e75, 0xeee8d5, 0xddd6c1, 0xe6dfca, 0xd3cbb7, 0x268bd2, 0xeee8d5, 0x586e75], selection: 0xd6cfb7ff },
    Def { name: "GitHub Dark", dark: true, c: [0x0d1117, 0xc9d1d9, 0x6e7681, 0xc9d1d9, 0x010409, 0x010409, 0x21262d, 0x30363d, 0x1f6feb, 0x161b22, 0x8b949e], selection: 0x3392ff44 },
    Def { name: "GitHub Light", dark: false, c: [0xffffff, 0x24292f, 0x8c959f, 0x24292f, 0xf6f8fa, 0xf6f8fa, 0xf3f4f6, 0xd0d7de, 0x0969da, 0xf6f8fa, 0x57606a], selection: 0x0969da33 },
    Def { name: "Nord", dark: true, c: [0x2e3440, 0xd8dee9, 0x4c566a, 0xd8dee9, 0x3b4252, 0x2e3440, 0x434c5e, 0x4c566a, 0x88c0d0, 0x3b4252, 0xd8dee9], selection: 0x434c5eff },
    Def { name: "Tokyo Night", dark: true, c: [0x1a1b26, 0xa9b1d6, 0x3b4261, 0x737aa2, 0x16161e, 0x16161e, 0x292e42, 0x292e42, 0x7aa2f7, 0x16161e, 0x787c99], selection: 0x515c7e66 },
    Def { name: "Catppuccin Mocha", dark: true, c: [0x1e1e2e, 0xcdd6f4, 0x6c7086, 0xcdd6f4, 0x181825, 0x181825, 0x313244, 0x45475a, 0xcba6f7, 0x11111b, 0xcdd6f4], selection: 0x585b7088 },
];

/// Pseudo theme name: follow the OS light/dark appearance.
pub const SYSTEM: &str = "System";

impl Theme {
    /// "System" first, then the real themes.
    pub fn names() -> impl Iterator<Item = &'static str> {
        std::iter::once(SYSTEM).chain(THEMES.iter().map(|d| d.name))
    }

    /// Unknown names fall back to Dark+.
    pub fn by_name(name: &str) -> Self {
        let d = THEMES.iter().find(|d| d.name == name).unwrap_or(&THEMES[0]);
        let [bg, fg, muted, line_active, bar, tab_inactive, btn, border, accent, status_bg, status_fg] = d.c.map(rgb);
        Self {
            name: d.name,
            dark: d.dark,
            bg,
            fg,
            muted,
            line_active,
            bar,
            tab_inactive,
            btn,
            border,
            accent,
            selection: rgba(d.selection),
            status_bg,
            status_fg,
            find: rgba(0xea5c0055),
            find_current: rgba(0xea5c00aa),
            line_hl: if d.dark { rgba(0xffffff0c) } else { rgba(0x0000000a) },
            warn: if d.dark { rgb(0x4d3b0a) } else { rgb(0xfff3cd) },
            hover: if d.dark { rgba(0xffffff14) } else { rgba(0x00000010) },
        }
    }

    pub fn of(cx: &App) -> Self {
        let name = &cx.global::<Settings>().theme;
        if name != SYSTEM {
            return Self::by_name(name);
        }
        let dark = matches!(cx.window_appearance(), WindowAppearance::Dark | WindowAppearance::VibrantDark);
        Self::by_name(if dark { "Dark+" } else { "Light+" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_unique_and_fallback() {
        let mut n: Vec<_> = Theme::names().collect();
        n.sort();
        n.dedup();
        assert_eq!(n.len(), THEMES.len() + 1);
        assert_eq!(Theme::by_name("nope").name, "Dark+");
    }
}
