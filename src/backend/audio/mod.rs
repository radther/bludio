//! Audio device management: data types for `PulseAudio` sinks, sources, and cards.

pub(crate) mod pulse;

use crate::backend::subsystem::SubsystemStatus;

// ── Device kind ────────────────────────────────────────────────────────────

/// Whether a device is an output (sink/speaker) or input (source/microphone).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum DeviceKind {
    Output,
    Input,
}

// ── Audio state ────────────────────────────────────────────────────────────

/// Complete snapshot of the `PulseAudio` audio state.
#[derive(Clone, Debug)]
pub(crate) struct AudioState {
    pub(crate) sinks: Vec<SinkInfo>,
    pub(crate) sources: Vec<SourceInfo>,
    /// Card info for the Configuration page and sink profile decoration.
    pub(crate) cards: Vec<CardInfo>,
    /// Unified subsystem health status.
    pub(crate) subsystem_status: SubsystemStatus,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            sinks: Vec::new(),
            sources: Vec::new(),
            cards: Vec::new(),
            subsystem_status: SubsystemStatus::Connecting,
        }
    }
}

// ── Sink (output device) ──────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct SinkInfo {
    pub(crate) index: u32,
    /// `PulseAudio` sink name (e.g., "alsa_output.pci-0000_00_1f.3.analog-stereo").
    pub(crate) name: String,
    /// Human-readable description (e.g., "Built-in Audio Analog Stereo").
    pub(crate) description: String,
    /// Number of audio channels (e.g., 2 for stereo, 8 for 7.1).
    pub(crate) channels: u8,
    /// Average volume across all channels, 0.0–1.0 (mapped from `PA_VOLUME_NORM`).
    pub(crate) volume: f64,
    pub(crate) muted: bool,
    /// Whether this sink is the system default output.
    pub(crate) is_default: bool,
    /// Index of the owning card, if any.
    pub(crate) card_index: Option<u32>,
    /// Currently active profile name on the owning card.
    pub(crate) active_profile: Option<String>,
    /// All available profiles on the owning card.
    pub(crate) available_profiles: Vec<ProfileInfo>,
}

// ── Source (input device) ─────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct SourceInfo {
    pub(crate) index: u32,
    pub(crate) name: String,
    pub(crate) description: String,
    /// Number of audio channels (e.g., 2 for stereo, 8 for 7.1).
    pub(crate) channels: u8,
    pub(crate) volume: f64,
    pub(crate) muted: bool,
    pub(crate) is_default: bool,
    /// Whether this is a monitor source (virtual, records sink output).
    pub(crate) is_monitor: bool,
}

// ── Card (hardware) ────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct CardInfo {
    pub(crate) index: u32,
    /// Card hardware name (fallback when `description` is not set).
    pub(crate) name: String,
    /// Human-readable description from proplist `device.description`.
    pub(crate) description: Option<String>,
    pub(crate) active_profile: Option<String>,
    pub(crate) profiles: Vec<ProfileInfo>,
    /// Active Bluetooth codec name (e.g., "aac", "ldac"), if any.
    pub(crate) active_codec: Option<String>,
    /// Available Bluetooth codecs for this card.
    pub(crate) codecs: Vec<CodecInfo>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProfileInfo {
    pub(crate) name: String,
    /// Human-readable description (kept for future tooltips).
    #[allow(dead_code)]
    pub(crate) description: String,
    /// Whether this profile is available on the hardware.
    /// Kept for completeness — having the full PA state available may inform
    /// future UI decisions (e.g., greying out unavailable profiles).
    #[allow(dead_code)]
    pub(crate) available: bool,
}

// ── Bluetooth codec ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct CodecInfo {
    pub(crate) name: String,
    pub(crate) description: String,
}

// ── Commands (UI → PA thread) ──────────────────────────────────────────────

/// Commands sent from the UI to the `PulseAudio` backend thread.
/// The `Set` prefix is intentional — these map to `PulseAudio` setter operations.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub(crate) enum AudioCommand {
    SetVolume(DeviceKind, u32, u8, f64),
    SetMute(DeviceKind, u32, bool),
    SetCardProfile(u32, String),
    SetCardCodec(u32, String, String),
    SetDefaultSink(String),
    SetDefaultSource(String),
}
