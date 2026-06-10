## ADDED Requirements

### Requirement: JSON theme files are parsed into Theme structs
The system SHALL provide a theme loader that reads a JSON theme file and produces a `Theme` struct. The loader SHALL parse the `id`, `display_name`, `colors`, and `theme` fields. The loader SHALL convert hex color values into GPUI `Hsla` values.

#### Scenario: Load a valid theme JSON
- **WHEN** the loader reads a JSON file with valid `id`, `display_name`, `colors`, and `theme` fields
- **THEN** it SHALL return a `Theme` with the correct `id`, `appearance`, and populated `ThemeColors`

#### Scenario: Hex colors are converted to Hsla
- **WHEN** a color value in the `colors` dictionary is `"#RRGGBB"`
- **THEN** the loader SHALL parse the hex string into an `Hsla` with the correct RGB values and alpha 1.0

#### Scenario: Theme mapping references palette colors
- **WHEN** the `theme` dictionary contains `"background": "base"`
- **THEN** the loader SHALL resolve `"base"` to the color defined in the `colors` dictionary under the key `"base"`
- **THEN** the resulting `ThemeColors.background` SHALL equal that `Hsla` value

### Requirement: Theme loader validates semantic mappings
The loader SHALL verify that every value referenced in the `theme` dictionary exists as a key in the `colors` dictionary. If a reference is missing, the loader SHALL reject the theme file and return an error.

#### Scenario: All mappings resolve
- **WHEN** a theme JSON contains a `theme` dictionary where every value matches a key in `colors`
- **THEN** the loader SHALL successfully construct the `Theme`

#### Scenario: Missing palette reference
- **WHEN** a theme JSON contains `"theme": { "background": "nonexistent" }` and `colors` does not contain `"nonexistent"`
- **THEN** the loader SHALL return an error indicating the missing reference

### Requirement: Theme loader supports all required semantic tokens
The loader SHALL populate every field of `ThemeColors` from the `theme` dictionary. The required tokens are: `background`, `surface`, `background_hover`, `element_background`, `element_hover`, `input_background`, `text`, `text_secondary`, `text_muted`, `text_colored_button`, `border`, `bluetooth_accent`, `audio_accent`, `dev_accent`, `danger`, `warning`, `success`.

#### Scenario: All semantic tokens present
- **WHEN** a theme JSON defines all required semantic tokens in its `theme` dictionary
- **THEN** the loader SHALL produce a `ThemeColors` with every field populated

#### Scenario: Missing semantic token
- **WHEN** a theme JSON omits a required semantic token from its `theme` dictionary
- **THEN** the loader SHALL return an error indicating the missing token
