# Tasks: add-i18n-support

## Phase 1: Project Setup and Resources Directory

1. **Create resources directory structure**
   - Create `resources/` directory at project root
   - Create `resources/locales/` for translation files
   - Create `resources/icons/` for application icons
   - Create `resources/fonts/` (optional, for language-specific fonts)
   - Create `resources/images/` for UI assets

2. **Install i18n dependencies**
   - Add `i18next` and `react-i18next` to package.json
   - Add `i18next-http-backend` for loading translations
   - Run `npm install`

3. **Configure Vite for public resources**
   - Update `vite.config.ts` to copy locales to build output
   - Ensure locales are accessible at runtime

## Phase 2: Translation Infrastructure

4. **Create translation file structure**
   - Create `resources/locales/en/translation.json`
   - Create base English translation file with all UI strings
   - Document translation file format and key conventions

5. **Create i18n configuration**
   - Create `src/utils/i18n.ts` for i18n initialization
   - Configure supported languages and namespaces
   - Set up language detection and fallback

6. **Create language type definitions**
   - Add supported languages to TypeScript types
   - Create language metadata (name, native name, direction)

## Phase 3: Extract and Externalize Strings

7. **Extract strings from App.tsx**
   - Identify all hardcoded strings in App.tsx
   - Create translation keys for each string
   - Replace strings with translation function calls

8. **Extract strings from SettingsPanel.tsx**
   - Identify all hardcoded strings in SettingsPanel
   - Create translation keys for settings labels, tabs, buttons
   - Replace strings with translation function calls

9. **Extract strings from StatusText.tsx**
   - Identify status messages and labels
   - Create translation keys for status text
   - Replace strings with translation function calls

10. **Handle backend strings in Rust**
    - Identify tray menu strings in main.rs
    - Create IPC endpoint for getting translated strings
    - Update Rust code to fetch translated strings from frontend

## Phase 4: Language Files

11. **Create Simplified Chinese translations**
    - Create `resources/locales/zh-CN/translation.json`
    - Translate all strings to Simplified Chinese

12. **Create Traditional Chinese translations**
    - Create `resources/locales/zh-TW/translation.json`
    - Translate all strings to Traditional Chinese

13. **Create Spanish translations**
    - Create `resources/locales/es/translation.json`
    - Translate all strings to Spanish

14. **Create French translations**
    - Create `resources/locales/fr/translation.json`
    - Translate all strings to French

15. **Create German translations**
    - Create `resources/locales/de/translation.json`
    - Translate all strings to German

16. **Create Japanese translations**
    - Create `resources/locales/ja/translation.json`
    - Translate all strings to Japanese

17. **Create Korean translations**
    - Create `resources/locales/ko/translation.json`
    - Translate all strings to Korean

18. **Create Russian translations**
    - Create `resources/locales/ru/translation.json`
    - Translate all strings to Russian

19. **Create Portuguese translations**
    - Create `resources/locales/pt/translation.json`
    - Translate all strings to Portuguese

20. **Create Arabic translations**
    - Create `resources/locales/ar/translation.json`
    - Translate all strings to Arabic
    - Add RTL direction specification

## Phase 5: Language Selection UI

21. **Add language setting to store**
    - Add `language` field to settings in progressStore.ts
    - Add `setLanguage` action to store
    - Persist language preference

22. **Add language selector to Settings panel**
    - Create language dropdown in Settings panel
    - Populate with supported languages
    - Handle language change events

23. **Implement RTL layout styles**
    - Add RTL CSS rules for Arabic
    - Test layout with RTL direction

## Phase 6: Migration and Cleanup

24. **Move icons to resources directory**
    - Copy icons from `src-tauri/icons/` to `resources/icons/`
    - Update icon references in Rust code
    - Update tray icon references

25. **Update documentation**
    - Document translation key conventions
    - Document how to add new languages
    - Update README with i18n information

## Phase 7: Testing and Validation

26. **Test all language translations**
    - Verify each language displays correctly
    - Test language switching functionality
    - Test RTL layout for Arabic

27. **Run existing tests**
    - Execute `npm test` to ensure no regressions
    - Fix any failing tests

28. **Type checking**
    - Run `npm run build` to verify TypeScript
    - Fix any type errors

## Dependencies and Parallelization

**Can be done in parallel:**
- Steps 1-3 (Project Setup)
- Steps 11-20 (Creating translation files for each language)
- Steps 7-10 (Extracting strings from different files)

**Sequential dependencies:**
- Steps 4-6 must complete before steps 7-10
- Steps 7-10 must complete before steps 11-20
- Steps 21-23 depend on having translation files ready
- Step 24 can be done in parallel with steps 21-23
- Steps 26-28 should be done last
