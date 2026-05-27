## 1. Core UI Specs

- [x] 1.1 Update `openspec/specs/app-shell/spec.md` — add Configuration and DevTest tabs, update app identity to `"dev.toomosin.bludio"`, document keyboard shortcuts (Ctrl+1–5), add root-level theme/font application, update page entity references
- [x] 1.2 Update `openspec/specs/tab-bar-navigation/spec.md` — update from 3 tabs to 5, document `TabBar` entity and `TabBarEvent` pattern, update icon names to match SVG files, add keyboard shortcut support

## 2. Audio Specs

- [x] 2.1 Update `openspec/specs/audio-device-management/spec.md` — update device row layout to reflect slider+text field volume, profile dropdown on output rows, status_strip for default indicator, connection state handling
- [x] 2.2 Update `openspec/specs/audio-volume-control/spec.md` — align terminology with `SliderState`/`Slider`/`TextField` entities, document EventEmitter pattern, add blur-sync and canvas bounds capture behaviors

## 3. Bluetooth Specs

- [x] 3.1 Update `openspec/specs/bluetooth-device-management/spec.md` — document `BluetoothPageCommand` channel pattern, `status_strip` component, multi-step PairAndTrust dispatch model, `BluetoothPage` entity ownership pattern

## 4. Component & Theme Specs

- [x] 4.1 Update `openspec/specs/slider-component/spec.md` — document `SliderState` Entity + `Slider` RenderOnce architecture, canvas bounds capture, constructor/method API instead of builder API
- [x] 4.2 Update `openspec/specs/theme-system/spec.md` — update file path to `src/ui/theme/` module structure, change default from dark to light (Rose Pine Dawn), document Rose Pine variants, add `body2` text style, document three accent colors

## 5. Verification

- [x] 5.1 Verify all updated specs match actual codebase behavior by cross-referencing each requirement against source files
