## Why

Users frequently want to play audio through multiple output devices simultaneously (e.g., wired speakers and Bluetooth earbuds). PulseAudio supports this via `module-combine-sink`, but the app currently only supports single-default-sink routing. Adding combined-sink support lets users create virtual sinks that mirror audio to multiple outputs without external tools.

## What Changes

- Add a **combine button** to the Output Devices page header (styled like the Bluetooth scan button). When pressed, it enters **selection mode** (highlighted in audio accent color).
- In selection mode, **all real sink rows display a checkbox** on the right side. The user checks which sinks to combine.
- Pressing the combine button again **creates a new combined sink** from the selected sinks as slaves. The combined sink is named automatically (e.g., "Bludio-combined-1").
- **Multiple combined sinks** are allowed. The user can create as many as they want.
- **Combined sink rows** display a small icon next to their name (between the name and the "Default" label) and a **delete button** to unload the combined sink.
- Combined sinks created **outside the app** (e.g., via `pactl`) are detected automatically and displayed the same way (icon + delete button).
- The app **does not distinguish** between combined sinks it created and those created externally — all are managed uniformly.
- **Sources page is unaffected** — this feature is gated to output devices only.

## Capabilities

### New Capabilities
- `combined-sink-output`: Multi-device audio output via PulseAudio's `module-combine-sink`. Covers selection mode, checkbox-based sink selection, combined sink creation, deletion, and visual distinction (icon + delete button).

### Modified Capabilities
- `audio-device-management`: Output device rows gain a checkbox when in selection mode. Combined sink rows display a special icon and delete button instead of the normal "Default" button. The Output Devices page header gains a combine-mode button.

## Impact

- `src/backend/audio/pulse.rs` — new command execution for `module-combine-sink` load/unload and module introspection
- `src/backend/audio/mod.rs` — new `AudioCommand` variants and `AudioState` fields to track loaded modules and combined sink state
- `src/ui/pages/audio/audio_page.rs` — combine button in header, selection mode state, checkbox management
- `src/ui/pages/audio/audio_device_row.rs` — checkbox rendering, conditional "Default" vs "Delete" button, combined sink icon
- `src/app.rs` — state wiring for selection mode and combined sink detection
- `libpulse_binding` — `Context::load_module`, `Context::unload_module`, `Introspect::get_module_info_list` usage
