## MODIFIED Requirements

### Requirement: Tab bar supports six pages
The system SHALL render the application window as a horizontal flex row comprising a fixed-width left tab bar and a flexible right content area. The left tab bar SHALL span the full height of the window. The right content area SHALL fill the remaining horizontal space. The root element SHALL set the font family from the active theme.

#### Scenario: Layout is two-column horizontal split
- **WHEN** the application window is open
- **THEN** the left column SHALL be a tab bar entity (`TabBar`) with fixed width of 48 pixels
- **THEN** the right column SHALL fill the remaining window width
- **THEN** both columns SHALL span the full window height
- **THEN** the root element SHALL apply `font_family` from the active theme

#### Scenario: Content area renders the active page
- **WHEN** a tab is selected
- **THEN** the right content area SHALL render the page entity corresponding to the active tab
- **THEN** switching tabs SHALL instantly swap the content area contents
- **THEN** the active page entity SHALL be one of: `BluetoothPage`, `AudioPage` (output), `AudioPage` (input), `ConfigurationPage`, `DevTestPage`, or `SettingsPage`
