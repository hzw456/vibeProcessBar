# Change Proposal: add-i18n-support

## Summary

Add internationalization (i18n) support to the Vibe Process Bar application, including a centralized resources directory for icons and translation content. The application will support 11 languages: English, Simplified Chinese, Traditional Chinese, Spanish, French, German, Japanese, Korean, Russian, Portuguese, and Arabic.

## Motivation

The application currently has hardcoded English strings throughout the UI, making it difficult for non-English users to interact with the application. Adding i18n support will:
- Make the application accessible to a global audience
- Create a centralized resources directory for icons and translations
- Improve maintainability by separating text content from code
- Enable easy addition of new languages in the future

## Scope

### In Scope
1. Create a `resources/` directory structure at the project root
2. Implement i18n infrastructure using a lightweight library (i18next or react-i18next)
3. Add translation files for all 11 required languages
4. Extract and externalize all hardcoded strings from:
   - `src/App.tsx`
   - `src/components/SettingsPanel.tsx`
   - `src/components/StatusText.tsx`
   - `src-tauri/src/main.rs` (tray menu items, window titles)
5. Add language selector in Settings panel
6. Persist language preference in settings
7. Support RTL layout for Arabic

### Out of Scope
- RTL layout testing for all languages (basic RTL support only)
- Third-party plugin localization (e.g., Tauri plugins)
- Localization of error messages from external dependencies

## Current Behavior

All user-facing strings are hardcoded in English:
- UI labels, buttons, and menu items in React components
- System tray menu items in Rust backend
- Window titles
- Status messages and notifications

## Proposed Behavior

1. User can select their preferred language from Settings
2. All UI text displays in the selected language
3. Application remembers language preference across sessions
4. RTL layout automatically applied for Arabic

## Dependencies

- React i18n library (react-i18next or equivalent)
- i18next backend for loading translation files
- Right-to-left (RTL) layout support for Arabic

## Risks

- RTL layout may require additional CSS adjustments
- String interpolation/pluralization may not be needed for this application
- Backend (Rust) strings require IPC calls for dynamic updates
