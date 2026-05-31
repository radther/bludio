//! Bludio — GPUI Bluetooth and audio manager.
//!
//! Entry point: Tokio bridge setup, font loading, theme init, window creation.

mod app;
mod audio;
mod bluetooth;
mod settings;
mod subsystem;
mod ui;

use std::sync::LazyLock;

use futures::channel::oneshot;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform_gpui_unofficial::application;

// ── Tokio bridge ───────────────────────────────────────────────────────────

/// Global Tokio runtime for all `BlueZ` D-Bus operations.
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
        // ── Load bundled fonts ────────────────────────────────────────
        let fonts: Vec<std::borrow::Cow<'static, [u8]>> = vec![
            std::borrow::Cow::Borrowed(include_bytes!("../fonts/NotoSans.ttf")),
            std::borrow::Cow::Borrowed(include_bytes!("../fonts/NotoSans-Italic.ttf")),
        ];
        cx.text_system()
            .add_fonts(fonts)
            .expect("Failed to load bundled fonts");

        // ── Init settings + theme ──────────────────────────────────
        let settings = crate::settings::Settings::load();
        cx.set_global(crate::settings::GlobalSettings::new(settings));

        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                app_id: Some("dev.toomosin.bludio".to_string()),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| app::BludioApp::new(window, cx)),
        )
        .unwrap();

        cx.activate(true);
    });
}
