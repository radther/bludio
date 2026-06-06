## Context

The app currently uses a single-default-sink model: all audio routes to one PulseAudio sink, and the user presses "Default" on any row to switch. PulseAudio's `module-combine-sink` can create a virtual sink that fans out to multiple real sinks, but the app has no integration for it.

The `libpulse_binding` crate exposes `Context::load_module(name, args, callback)` and `Context::unload_module(index, callback)`. The `Introspect` API provides `get_module_info_list` to inspect loaded modules. These are the only primitives needed.

## Goals / Non-Goals

**Goals:**
- Allow users to create multiple combined sinks, each mirroring audio to a chosen set of output devices
- Provide a selection-mode UI: header button toggles selection mode, checkboxes appear on rows, press button again to create
- Combined sinks are visually distinguished with an icon and have a delete button
- Combined sinks created externally (e.g., `pactl`) are detected and managed the same way
- The app treats all combined sinks uniformly, regardless of origin

**Non-Goals:**
- Input device combining (sources) — out of scope, no PA module support
- Per-app routing or channel splitting (left to one device, right to another)
- Persisting combined sink across sessions (module is session-only)
- Volume sync between slave sinks (PA handles this internally; user can adjust combined sink volume only)
- Preventing double-routing (a sink being a slave in multiple combined sinks) — PA allows this, we accept it as a user-visible edge case

## Decisions

### 1. Use `module-combine-sink` with explicit `slaves=` argument
- **Rationale:** `module-combine-sink` is a single, first-class PA module. Without an explicit `slaves=` argument, it defaults to ALL sinks, which is not what the user wants. We always specify the exact slave list.
- **Alternative:** No explicit `slaves=` and let PA include all sinks. Rejected: tested and confirmed this includes all sinks, not just the selected ones.

### 2. Multiple combined sinks supported
- **Rationale:** `module-combine-sink` can be loaded multiple times with different names. The user explicitly wants to create "as many as they want." Each combined sink is a separate virtual sink.
- **Trade-off:** Multiple virtual sinks can be confusing. Mitigation: visual icon and name distinguish them from real sinks.

### 3. Selection mode with custom checkbox component
- **Rationale:** The user wants to select multiple sinks at once and create a combined sink. A checkbox-based selection mode is clearer than the previous toggle-per-row approach. It also allows creating multiple combined sinks naturally. The checkbox uses a custom `ui::components::checkbox` component (20×20 square with rounded corners, check icon, filled with `audio_accent` when checked) that matches the app's visual style.
- **Alternative:** Toggle button on each row that adds/removes the sink from a single combined sink. Rejected: only supports one combined sink, and the user's new design is more flexible.

### 4. Discover combined sinks via module introspection, not by name
- **Rationale:** When PA rebuilds state, we need to know which sinks are combined sinks. The `get_module_info_list` callback finds `module-combine-sink` entries. We parse `sink_name` from the module argument and match it to sinks. This is authoritative and handles externally-created combined sinks.
- **Alternative:** Check if a sink name starts with "Bludio-combined." Rejected: misses externally-created combined sinks and is brittle.

### 5. Combined sink rows show "Default" button + delete button
- **Rationale:** The combined sink is a real sink in PA. The user might want to set it as default (that's the whole point). The delete button unloads the module. Both buttons can coexist.
- **Alternative:** Replace "Default" with "Delete" on combined sinks. Rejected: the user might want to set a combined sink as default without deleting it.

### 6. Auto-naming combined sinks with a counter
- **Rationale:** Each combined sink needs a unique PA name. Auto-naming (e.g., "Bludio-combined-1", "Bludio-combined-2") avoids user input and ensures uniqueness. The counter is checked against existing sink names.
- **Alternative:** Prompt user for name. Rejected: adds UI complexity and potential for duplicate names.

### 7. Do not manually set default after deleting a combined sink
- **Rationale:** PulseAudio automatically sets a new default sink when the current default is deleted. The user confirmed this behavior in testing. We trust PA to handle the default fallback.
- **Alternative:** Manually set the first slave as default. Rejected: unnecessary — PA handles it automatically.

### 8. UI state is derived from `AudioState`, not a separate flag in the app
- **Rationale:** The PA thread already rebuilds the full `AudioState` on every event. If a combined sink is active, it will appear in the sinks list. The AudioPage can inspect the state to determine which sinks are combined sinks. The selection mode flag is the only app-level state needed.
- **Benefit:** If PA is modified externally (e.g., `pactl unload-module`), the UI will reflect it automatically on the next state refresh.

## Risks / Trade-offs

- **[Risk] Audio interruption on combined sink creation** → Mitigation: This is inherent to PA's `module-combine-sink` design. The interruption is brief (sub-second). The user is explicitly initiating the action.
- **[Risk] Bluetooth latency mismatch between two BT devices in a combined sink** → Mitigation: PA does not sync clocks. This is a user-visible limitation, not a crash risk. Documented as expected behavior.
- **[Risk] A sink being a slave in multiple combined sinks** → Mitigation: PA allows this. The audio will be duplicated (one copy per combined sink). The user can see this by checking the module info. Not a crash risk.
- **[Risk] Combined sink survives app restart but UI doesn't know about it** → Mitigation: The UI derives combined state from the sink list. If a combined sink exists from a previous session, it will appear with the icon and delete button. The user can delete it.
- **[Risk] Slave sink disappears while in a combined sink** → Mitigation: PA handles this internally. The combined sink may stop routing to the missing slave. The user can delete the combined sink to restore normal behavior.

## Open Questions

- Should the newly created combined sink be automatically set as default? (Default: yes, but we should confirm.)
- Should a sink that is already a slave in another combined sink be selectable in selection mode? (PA allows it, but it might be confusing.)
- Should combined sinks themselves be selectable in selection mode (to combine into another combined sink)? (PA might allow this, but it's likely not useful.)
- What happens if the user presses the combine button with no sinks selected? (Exit selection mode without creating anything?)
- Should the delete button be styled differently (e.g., danger color) to indicate destructive action?
