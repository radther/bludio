mod actions;
mod app;
mod bluetooth;
mod ui;

use std::sync::LazyLock;

use futures::channel::oneshot;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform_gpui_unofficial::application;

// ── Tokio bridge ───────────────────────────────────────────────────────────

/// Global Tokio runtime for all BlueZ D-Bus operations.
///
/// gpui-unofficial's executor is not Tokio-based, but `bluer` requires a
/// Tokio 1.x reactor. We spin up a dedicated runtime and route all Bluetooth
/// work through `tokio_task()`.
static TOKIO: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"));

pub(crate) fn tokio_task<F, T>(f: F) -> oneshot::Receiver<T>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    TOKIO.spawn(async move {
        let _ = tx.send(f.await);
    });
    rx
}

// ── Entry point ────────────────────────────────────────────────────────────

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                app_id: Some("com.bludio.app".to_string()),
                ..Default::default()
            },
            |_window, cx| cx.new(app::BludioApp::new),
        )
        .unwrap();

        cx.activate(true);
    });
}
