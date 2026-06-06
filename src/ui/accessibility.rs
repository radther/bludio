//! Accessibility global state.
//!
//! Provides a static atomic flag for animation disable so that animation
//! helpers (which do not have access to a GPUI `Context`) can read it.
//! The flag is set at app startup and updated whenever settings change.

use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// Static atomic holding whether animations are disabled.
static DISABLE_ANIMATIONS: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

/// Read whether animations are globally disabled.
pub(crate) fn disable_animations() -> bool {
    DISABLE_ANIMATIONS.load(Ordering::Relaxed)
}

/// Set whether animations are globally disabled.
pub(crate) fn set_disable_animations(value: bool) {
    DISABLE_ANIMATIONS.store(value, Ordering::Relaxed);
}
