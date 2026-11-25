mod actions;
mod buffer;
mod editor;
mod files;
mod find;
mod notepad;
mod settings;
mod theme;
mod ui;
mod update;

use gpui::{prelude::*, App, Application, Bounds, WindowBounds, WindowOptions, px, size};
use notepad::Notepad;
use settings::Settings;

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.set_global(Settings::load());
        cx.bind_keys(actions::bindings());
        cx.set_menus(actions::menus());
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(900.), px(620.)), cx);
        let window = cx
            .open_window(
                WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
                |window, cx| {
                    let np = cx.new(|cx| Notepad::new(window, cx));
                    let guard = np.clone();
                    window.on_window_should_close(cx, move |window, cx| {
                        guard.update(cx, |np, cx| np.request_close(window, cx, false))
                    });
                    np
                },
            )
            .unwrap();
        let _ = window;
        cx.activate(true);
    });
}
