//! All actions, key bindings and the native menu bar.
use gpui::{KeyBinding, Menu, MenuItem, OsAction, actions};

actions!(
    editor,
    [
        Backspace, Delete, DeleteWordBack, DeleteWordFwd, Left, Right, Up, Down, SelectLeft,
        SelectRight, SelectUp, SelectDown, WordLeft, WordRight, SelectWordLeft, SelectWordRight,
        LineStart, LineEnd, SelectLineStart, SelectLineEnd, DocStart, DocEnd, SelectDocStart,
        SelectDocEnd, PageUp, PageDown, SelectAll, Copy, Cut, Paste, Undo, Redo, Enter, Tab,
        ShowCharacterPalette,
    ]
);

actions!(
    notepad,
    [
        NewTab, OpenFile, Save, SaveAs, CloseTab, NextTab, PrevTab, Find, Replace, FindNext,
        FindPrev, GoToLine, ToggleWrap, ToggleLineNumbers, InsertDateTime, ZoomIn, ZoomOut,
        ToggleTheme, Quit, Escape, CheckForUpdates, ToggleUpdateCheck, ToggleAutosave,
    ]
);

const M: &str = if cfg!(target_os = "macos") { "cmd" } else { "ctrl" };

pub fn bindings() -> Vec<KeyBinding> {
    let e = Some("Editor");
    let n = Some("Notepad");
    let k = |keys: &str, f: &dyn Fn(&str) -> KeyBinding| f(&keys.replace("M-", &format!("{M}-")));
    let mut b: Vec<KeyBinding> = Vec::new();
    macro_rules! ed { ($($keys:expr => $a:expr),* $(,)?) => { $( b.push(k($keys, &|s| KeyBinding::new(s, $a, e))); )* } }
    macro_rules! np { ($($keys:expr => $a:expr),* $(,)?) => { $( b.push(k($keys, &|s| KeyBinding::new(s, $a, n))); )* } }
    ed! {
        "backspace" => Backspace, "delete" => Delete, "alt-backspace" => DeleteWordBack,
        "alt-delete" => DeleteWordFwd, "left" => Left, "right" => Right, "up" => Up, "down" => Down,
        "shift-left" => SelectLeft, "shift-right" => SelectRight, "shift-up" => SelectUp,
        "shift-down" => SelectDown, "alt-left" => WordLeft, "alt-right" => WordRight,
        "alt-shift-left" => SelectWordLeft, "alt-shift-right" => SelectWordRight,
        "home" => LineStart, "end" => LineEnd, "M-left" => LineStart, "M-right" => LineEnd,
        "shift-home" => SelectLineStart, "shift-end" => SelectLineEnd,
        "M-shift-left" => SelectLineStart, "M-shift-right" => SelectLineEnd,
        "M-up" => DocStart, "M-down" => DocEnd, "M-shift-up" => SelectDocStart,
        "M-shift-down" => SelectDocEnd, "pageup" => PageUp, "pagedown" => PageDown,
        "M-a" => SelectAll, "M-c" => Copy, "M-x" => Cut, "M-v" => Paste, "M-z" => Undo,
        "M-shift-z" => Redo, "M-y" => Redo, "enter" => Enter, "tab" => Tab,
        "ctrl-cmd-space" => ShowCharacterPalette,
    }
    np! {
        "M-n" => NewTab, "M-o" => OpenFile, "M-s" => Save, "M-shift-s" => SaveAs,
        "M-w" => CloseTab, "ctrl-tab" => NextTab, "ctrl-shift-tab" => PrevTab, "M-f" => Find,
        "M-alt-f" => Replace, "M-g" => FindNext, "M-shift-g" => FindPrev, "M-l" => GoToLine,
        "M-alt-z" => ToggleWrap, "M-alt-l" => ToggleLineNumbers, "f5" => InsertDateTime,
        "M-=" => ZoomIn, "M-+" => ZoomIn, "M--" => ZoomOut, "M-q" => Quit, "escape" => Escape,
    }
    b
}

fn item(name: &'static str, a: impl gpui::Action) -> MenuItem {
    MenuItem::action(name, a)
}

pub fn menus() -> Vec<Menu> {
    vec![
        Menu {
            name: "Notepad".into(),
            items: vec![
                item("Check for Updates…", CheckForUpdates),
                item("Check for Updates Automatically", ToggleUpdateCheck),
                MenuItem::separator(),
                item("Quit", Quit),
            ],
        },
        Menu {
            name: "File".into(),
            items: vec![
                item("New Tab", NewTab),
                item("Open…", OpenFile),
                MenuItem::separator(),
                item("Save", Save),
                item("Save As…", SaveAs),
                item("Autosave", ToggleAutosave),
                MenuItem::separator(),
                item("Close Tab", CloseTab),
            ],
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::os_action("Undo", Undo, OsAction::Undo),
                MenuItem::os_action("Redo", Redo, OsAction::Redo),
                MenuItem::separator(),
                MenuItem::os_action("Cut", Cut, OsAction::Cut),
                MenuItem::os_action("Copy", Copy, OsAction::Copy),
                MenuItem::os_action("Paste", Paste, OsAction::Paste),
                MenuItem::os_action("Select All", SelectAll, OsAction::SelectAll),
                MenuItem::separator(),
                item("Find…", Find),
                item("Replace…", Replace),
                item("Find Next", FindNext),
                item("Find Previous", FindPrev),
                item("Go to Line…", GoToLine),
                item("Insert Date/Time", InsertDateTime),
            ],
        },
        Menu {
            name: "View".into(),
            items: vec![
                item("Word Wrap", ToggleWrap),
                item("Line Numbers", ToggleLineNumbers),
                item("Toggle Dark Theme", ToggleTheme),
                MenuItem::separator(),
                item("Zoom In", ZoomIn),
                item("Zoom Out", ZoomOut),
                MenuItem::separator(),
                item("Next Tab", NextTab),
                item("Previous Tab", PrevTab),
            ],
        },
    ]
}
