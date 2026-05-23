//! Audio device management: data types for PulseAudio sinks, sources, and cards.

pub(crate) mod pulse;

// ── Device kind ────────────────────────────────────────────────────────────

/// Whether a device is an output (sink/speaker) or input (source/microphone).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DeviceKind {
    Output,
    Input,
}

// ── Audio state ────────────────────────────────────────────────────────────

/// Complete snapshot of the PulseAudio audio state.
#[derive(Clone, Debug, Default)]
pub(crate) struct AudioState {
    pub(crate) sinks: Vec<SinkInfo>,
    pub(crate) sources: Vec<SourceInfo>,
    /// Card info is folded into sink profiles; kept for future use.
    #[allow(dead_code)]
    pub(crate) cards: Vec<CardInfo>,
    pub(crate) connected: bool,
    pub(crate) error: Option<String>,
}

// ── Sink (output device) ──────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct SinkInfo {
    pub(crate) index: u32,
    /// PulseAudio sink name (e.g., "alsa_output.pci-0000_00_1f.3.analog-stereo").
    pub(crate) name: String,
    /// Human-readable description (e.g., "Built-in Audio Analog Stereo").
    pub(crate) description: String,
    /// Average volume across all channels, 0.0–1.0 (mapped from PA_VOLUME_NORM).
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
    /// Card hardware name (kept for future display).
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) active_profile: Option<String>,
    pub(crate) profiles: Vec<ProfileInfo>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProfileInfo {
    pub(crate) name: String,
    /// Human-readable description (kept for future tooltips).
    #[allow(dead_code)]
    pub(crate) description: String,
    /// Whether this profile is available on the hardware.
    #[allow(dead_code)]
    pub(crate) available: bool,
}

// ── Commands (UI → PA thread) ──────────────────────────────────────────────

/// Commands sent from the UI to the PulseAudio backend thread.
/// The `Set` prefix is intentional — these map to PulseAudio operations.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub(crate) enum AudioCommand {
    SetSinkVolume(u32, f64),
    SetSourceVolume(u32, f64),
    SetSinkMute(u32, bool),
    SetSourceMute(u32, bool),
    SetCardProfile(u32, String),
    SetDefaultSink(String),
    SetDefaultSource(String),
}
