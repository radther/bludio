use gpui::{
    App, Bounds, Context, FocusHandle, Focusable, IntoElement, Render, Window, WindowBounds,
    WindowOptions, div, hsla, prelude::*, px, size,
};
use gpui_platform_gpui_unofficial::application;

struct BludioApp {
    _focus_handle: FocusHandle,
}

impl Focusable for BludioApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self._focus_handle.clone()
    }
}

impl Render for BludioApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 1.0))
            .text_color(hsla(0.0, 0.0, 1.0, 1.0))
            .child("bludio")
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                app_id: Some("com.bludio.app".to_string()),
                ..Default::default()
            },
            |_window, cx| {
                cx.new(|cx| BludioApp {
                    _focus_handle: cx.focus_handle(),
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
