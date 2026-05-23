## Why

Bludio currently only manages Bluetooth devices. A Bluetooth audio manager needs audio device control — viewing output/input devices, switching profiles/codecs, adjusting volume, and setting defaults. This is the next foundational capability after device pairing/connection, enabling users to actually route and control audio through their Bluetooth (and other) audio hardware.

## What Changes

- Add `libpulse-binding` crate dependency for PulseAudio/PipeWire-Pulse communication
- Create `src/audio/` module with PulseAudio backend (connection, device listing, profile switching, volume control, default device management)
- Add "Output Devices" tab page: list all sinks with name, profile/codec indicator, default device badge, volume/mute controls per device
- Add "Input Devices" tab page: list all sources with name, default device badge, volume/mute controls per device
- Add new `Page` enum variants (`AudioOutputs`, `AudioInputs`) and corresponding tab bar entries
- Implement a reusable `VolumeSlider` UI component with a clickable bar, text field for precise numeric entry, and mute toggle per device
- Use dropdown selectors for choosing card profiles and codecs
- Wire audio operations through the existing Tokio bridge pattern (same as Bluetooth: `tokio_task()` for PulseAudio calls, `cx.spawn` + `this.update()` for UI updates)
- Subscribe to PulseAudio events for reactive UI updates when devices/volumes change externally
- Add `audio-video` and `mic` icons to the icons module for the new tabs

## Capabilities

### New Capabilities
- `audio-device-management`: List, inspect, and control PulseAudio sinks (outputs) and sources (inputs). Includes listing devices with properties (name, description, profile, codec, hardware port), switching card profiles and codecs, setting the default sink/source, and monitoring changes via PulseAudio subscription events.
- `audio-volume-control`: Per-device volume slider and mute toggle UI component. Combines a clickable horizontal bar with a text field for precise numeric entry. Drives PulseAudio volume/mute operations. Displays current volume as a percentage bar. Reactive to external volume changes via subscription events.

### Modified Capabilities
- `tab-bar-navigation`: Tab bar gains two new tabs — "Output Devices" and "Input Devices" — bringing the total to 3 tabs (Bluetooth, Output, Input). The "Page 2" placeholder is removed.
- `app-shell`: The `Page` enum gains `AudioOutputs` and `AudioInputs` variants and loses the `Page2` placeholder variant. The app render match arms route these to the corresponding audio page views.

## Impact

- **New dependencies**: `libpulse-binding` crate (safe Rust wrapper for libpulse)
- **New files**: `src/audio/mod.rs`, `src/audio/pulse.rs` (PulseAudio connection + operations), `src/ui/components/volume_slider.rs` (volume bar component), `src/ui/components/text_field.rs` (TextField entity), `src/ui/components/dropdown.rs` (Dropdown entity), `src/ui/audio/audio_page.rs` (AudioPage entity), `src/ui/audio/device_row.rs` (AudioDeviceRow entity)
- **Modified files**: `src/main.rs` / `src/app.rs` (add `Page` variants, tab entries, render routing), `src/ui/mod.rs` (new module declarations), `src/ui/icons.rs` (new icon helpers), `Cargo.toml` (new dependency)
- **Deleted files**: None (Page 2 placeholder code is removed, not a separate file)
- **System dependencies**: Requires `libpulse-dev` (or equivalent) installed at build time for linking against `libpulse`
