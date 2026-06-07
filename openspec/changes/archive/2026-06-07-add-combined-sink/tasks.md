## 1. Backend: PulseAudio module commands and discovery

- [x] 1.1 Add `AudioCommand` variants: `LoadCombineSink(String /* name */, Vec<String> /* slaves */)`, `UnloadModule(u32 /* module_index */)` in `src/backend/audio/mod.rs`
- [x] 1.2 Add `ModuleInfo` struct to `AudioState` with fields: `index: u32`, `name: String`, `argument: String` to track loaded `module-combine-sink` modules
- [x] 1.3 Implement `execute_command` cases for `LoadCombineSink` and `UnloadModule` in `src/backend/audio/pulse.rs` using `Context::load_module` and `Context::unload_module`
- [x] 1.4 Add `get_module_info_list` introspection in `build_audio_state` to discover all loaded `module-combine-sink` modules and parse their `sink_name=` and `slaves=` from the argument string
- [x] 1.5 Populate `AudioState.modules` with discovered modules and cross-reference the combined sink to mark sinks in `SinkInfo`

## 2. Backend: Sink model updates for combined state

- [x] 2.1 Add `is_combined_sink: bool` and `combined_module_index: Option<u32>` to `SinkInfo` in `src/backend/audio/mod.rs`
- [x] 2.2 Set `is_combined_sink` to true when a sink's name matches any `sink_name` parsed from a `module-combine-sink` module argument
- [x] 2.3 Set `combined_module_index` to the module index when the sink is a combined sink
- [x] 2.4 Add helper function to parse `module-combine-sink` argument string: extract `sink_name` and `slaves` values
- [x] 2.5 Add auto-naming logic for new combined sinks: generate "Bludio-combined-N" name and check against existing sink names to avoid duplicates

## 3. UI: AudioPage selection mode and combine button

- [x] 3.1 Add `selection_mode: bool` and `selected_sinks: Vec<String>` fields to `AudioPage`
- [x] 3.2 Add a combine button to the `AudioPage` header (right side), styled like the Bluetooth scan button (48x48 rounded square with icon)
- [x] 3.3 The button SHALL use the `merge` icon from `icons::merge()`
- [x] 3.4 Button color SHALL be `audio_accent` when `selection_mode` is true, otherwise `element_background`
- [x] 3.5 Button click handler: when not in selection mode, enter selection mode; when in selection mode, create combined sink from selected sinks and exit selection mode
- [x] 3.6 Pass `selection_mode` state to each `AudioDeviceRow` during sync

## 4. UI: AudioDeviceRow checkbox and conditional rendering

- [x] 4.1 Add `selection_mode: bool`, `is_selected: bool`, and `is_combined_sink: bool` fields to `AudioDeviceRow`
- [x] 4.2 Update `RowParams` to accept `selection_mode: bool`, `is_combined_sink: bool`, and `combined_module_index: Option<u32>`
- [x] 4.3 Update `new_sink` and `update_from_sink` to propagate combined state and selection mode from `SinkInfo` and `AudioPage`
- [x] 4.4 In `render_title_row`, when `selection_mode` is true:
  - Real sinks: show `checkbox` component from `ui::components::checkbox` (20×20 square, `audio_accent` fill when checked, check icon) on the right side, hide "Default" button
  - Combined sinks: show disabled checkbox or no checkbox
- [x] 4.5 In `render_title_row`, when `selection_mode` is false:
  - Real sinks: show "Default" button
  - Combined sinks: show "Delete" button (danger-colored, instead of "Default")
- [x] 4.6 In `render_device_name`, when `is_combined_sink` is true, show the `merge` icon from `icons::merge()` between the name and the "Default" label
- [x] 4.7 Checkbox click handler: emit an event to the parent `AudioPage` to toggle the sink's selection state
- [x] 4.8 Delete button click handler: send `AudioCommand::UnloadModule` with the `combined_module_index`, then wake PA (PulseAudio will automatically set a new default sink)

## 5. UI: AudioPage selection event handling

- [x] 5.1 Add `AudioDeviceRowEvent` enum for `AudioDeviceRow` → `AudioPage` communication (e.g., `ToggleSelection(String /* pa_name */)`) - named AudioDeviceRowEvent
- [x] 5.2 Implement `EventEmitter` for `AudioDeviceRow`
- [x] 5.3 Subscribe to row events in `AudioPage::sync_output_rows` and update `selected_sinks` accordingly
- [x] 5.4 Implement `create_combined_sink` logic: build `AudioCommand::LoadCombineSink` with auto-generated name and selected sinks, then wake PA
- [x] 5.5 On combined sink creation, clear `selected_sinks` and set `selection_mode = false`

## 6. App state integration

- [x] 6.1 Update `AudioPage::sync_output_rows` to pass selection mode and combined state flags into new and updated rows
- [x] 6.2 Ensure the combined sink virtual row appears naturally in the sink list (it already will, since PA reports it as a sink)
- [x] 6.3 Verify that `AudioPage::sync_rows` recalculates combined sink state on every state refresh
- [x] 6.4 Test delete flow: when delete button is clicked, the `UnloadModule` command is sent, PulseAudio sets a new default automatically, and the UI updates

## 7. Testing and validation

- [x] 7.1 Verify `cargo build` passes with no errors
- [x] 7.2 Verify `cargo clippy` passes with no warnings (1 expected warning: modules field never read directly)
- [x] 7.3 Run the app and test: enter selection mode, select sinks, create combined sink, verify icon and delete button appear
- [x] 7.4 Test creating a second combined sink with a different set of sinks
- [x] 7.5 Test deleting a combined sink: verify PulseAudio automatically sets a new default sink
- [x] 7.6 Test that externally-created combined sinks (via `pactl`) show the same icon and delete button
- [x] 7.7 Test that after PA reconnect, the combined state is correctly rediscovered
- [x] 7.8 Test empty selection: enter selection mode, press button with no selection, verify no combined sink is created
