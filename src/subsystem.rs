//! Unified subsystem health status for Bluetooth and Audio backends.

/// Health status of a backend subsystem (Bluetooth or Audio).
///
/// This is the single source of truth for whether a subsystem is operational.
/// Pages match on this enum to decide what to render — no booleans, no
/// `.is_none()` checks.
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) enum SubsystemStatus {
    /// Initial app startup — subsystem is connecting for the first time.
    #[default]
    Connecting,
    /// Connected and operational.
    Connected,
    /// Subsystem is down. Contains a user-facing error message.
    Disconnected(String),
    /// Attempting to reconnect after a disconnect.
    Reconnecting,
}
