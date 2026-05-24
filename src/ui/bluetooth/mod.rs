//! Bluetooth UI components: page entity, device row entity, and shared types.
//!
//! `BluetoothPageCommand` and `DeviceRowAction` live here so they can be
//! imported by both `bluetooth_page` and `device_row` without circular deps.

use bluer::Address;

pub(crate) mod bluetooth_page;
pub(crate) mod device_row;

// ── Shared command types ───────────────────────────────────────────────────

/// Commands sent from the Bluetooth page (or its rows) to the app for
/// async execution on the Tokio runtime.
#[derive(Clone, Debug)]
pub(crate) enum BluetoothPageCommand {
    /// Toggle discovery on/off.
    ToggleScan,
    /// Execute a device action (connect, disconnect, forget, pair+trust).
    DeviceAction { addr: Address, action: DeviceRowAction },
}

/// Action types for device buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeviceRowAction {
    Connect,
    Disconnect,
    Forget,
    PairAndTrust,
}
