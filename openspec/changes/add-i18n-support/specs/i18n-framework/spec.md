# Spec: i18n-framework

## ADDED Requirements

### REQ-001: Core i18n Infrastructure
The application shall implement i18n infrastructure using i18next and react-i18next.

#### Scenario: Initialize i18next with default language
Given the application starts for the first time
When i18n is initialized
Then it shall detect the system language
And load the appropriate translation file
And fall back to English if the system language is not supported

#### Scenario: Switch language at runtime
Given the user has selected a language in Settings
When the language is changed
Then the application shall immediately display all UI text in the new language
And persist the language preference for future sessions

### REQ-002: Translation File Format
The application shall use JSON translation files with dot-notation keys.

#### Scenario: Translation key lookup
Given a component requests translation for key "settings.general"
When the translation file is loaded
Then it shall return the value from settings.general in the translation file
Or return the key itself if the translation is missing

#### Scenario: Interpolation in translations
Given a translation contains interpolation like "Hello, {{name}}"
When the translation is rendered
Then it shall replace {{name}} with the provided value

### REQ-003: Supported Languages
The application shall support 11 languages: English, Simplified Chinese, Traditional Chinese, Spanish, French, German, Japanese, Korean, Russian, Portuguese, and Arabic.

#### Scenario: Available language list
Given the user opens the language selector
When the dropdown is displayed
Then it shall show all 11 supported languages
And display each language in its native name (e.g., "日本語" for Japanese)

#### Scenario: RTL layout for Arabic
Given the user selects Arabic as the language
When the UI is rendered
Then it shall set the document direction to RTL
And all layout shall adapt to right-to-left orientation

### REQ-004: String Externalization
All user-facing strings shall be externalized to translation files.

#### Scenario: App.tsx strings
Given App.tsx component
When rendering UI elements
Then it shall use translation keys for all visible text
Including menu items, labels, and status messages

#### Scenario: SettingsPanel.tsx strings
Given SettingsPanel component
When rendering settings interface
Then it shall use translation keys for all labels, tabs, buttons, and section headers

#### Scenario: StatusText.tsx strings
Given StatusText component
When displaying status information
Then it shall use translation keys for status labels (Ready, Running, Done, Error)

### REQ-005: Backend String Localization
The Rust backend shall support localized strings for system tray menu items.

#### Scenario: Tray menu text
Given the system tray menu is displayed
When the menu is rendered
Then it shall show translated text for menu items ("Settings", "Quit")
In the currently selected language

#### Scenario: Window titles
Given settings window is opened
When the window title is displayed
Then it shall show the translated title based on current language

### REQ-006: Language Persistence
The application shall persist language preference.

#### Scenario: Save language preference
Given the user changes the language
When the settings are saved
Then the language preference shall be stored in the application settings

#### Scenario: Restore language preference
Given the application is restarted
When settings are loaded
Then the previously selected language shall be restored

### REQ-007: Fallback Behavior
The application shall handle missing translations gracefully.

#### Scenario: Missing translation for a key
Given a translation key does not exist in the current language
When the key is requested
Then it shall fall back to the English translation
Or display the key itself if English is also missing

#### Scenario: Missing language file
Given a language file is corrupted or missing
When the language is selected
Then the application shall fall back to English
And log a warning message

### REQ-008: Performance Requirements
The application shall maintain responsive performance with i18n.

#### Scenario: Translation file loading
Given the application starts
When a language is selected
Then translation files shall load within 100ms

#### Scenario: Language switching
Given the user switches language
When the UI updates
Then the update shall complete within 50ms without blocking interaction

## MODIFIED Requirements

None - this is a new capability.

## REMOVED Requirements

None - this is a new capability.
