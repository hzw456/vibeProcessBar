# Design: add-i18n-support

## Architecture Overview

The i18n implementation will use i18next with react-i18next for the React frontend and a hybrid approach for the Rust backend.

## Key Design Decisions

### 1. Library Choice: i18next

**Decision:** Use i18next with react-i18next

**Rationale:**
- Mature, widely-used library with excellent TypeScript support
- Supports interpolation, pluralization, and context
- Large ecosystem with plugins for various needs
- Lightweight enough for this use case

**Alternatives considered:**
- **FormatJS/React-Intl**: More heavyweight, requires ICU message format
- **LinguiJS**: Excellent but steeper learning curve
- **Custom solution**: Would require reinventing features, not recommended

### 2. Translation File Structure

**Decision:** Use flat JSON file per language with dot-notation keys

**Example structure:**
```
resources/locales/
├── en/translation.json
├── zh-CN/translation.json
├── zh-TW/translation.json
├── es/translation.json
├── fr/translation.json
├── de/translation.json
├── ja/translation.json
├── ko/translation.json
├── ru/translation.json
├── pt/translation.json
└── ar/translation.json
```

**Example translation.json:**
```json
{
  "app": {
    "title": "Vibe Process Bar",
    "settings": "Settings",
    "closeMenu": "Close Menu"
  },
  "status": {
    "ready": "Ready",
    "running": "Running...",
    "completed": "Done",
    "error": "Error"
  },
  "settings": {
    "general": "General",
    "appearance": "Appearance",
    "notifications": "Notifications"
  }
}
```

**Rationale:**
- Flat structure with dot-notation keys is easy to manage
- Allows namespacing to avoid naming collisions
- Each language file is independent and can be edited separately

### 3. Backend String Handling

**Decision:** Use IPC to fetch translated strings from frontend for backend UI

**Approach:**
- Rust backend exposes an IPC endpoint `get_translated_string(key: string) -> string`
- Frontend i18n engine handles the translation
- Backend calls frontend to get translated text for tray menu

**Rationale:**
- Keeps all translation logic in one place (frontend)
- Avoids duplicating translation files in Rust
- Backend only needs minimal IPC interface

**Trade-off:** Slight overhead for IPC calls, but tray menu items are rarely updated

### 4. Resources Directory Structure

**Decision:** Create centralized resources directory at project root

```
resources/
├── locales/
│   ├── en/translation.json
│   └── ...
├── icons/
│   ├── app-icon.png
│   ├── tray-icon.svg
│   └── ...
└── fonts/
    └── (optional language-specific fonts)
```

**Rationale:**
- Separates assets from source code
- Easy to manage and update
- Can be easily bundled or copied during build

### 5. RTL Support for Arabic

**Decision:** Use CSS logical properties and direction attribute

**Implementation:**
- Set `dir="auto"` on root element or `dir="rtl"` for Arabic
- Use CSS logical properties (margin-inline-start, padding-inline-end, etc.)
- Add RTL-specific CSS overrides if needed

**Rationale:**
- CSS logical properties handle both LTR and RTL automatically
- Minimal code changes required
- Modern browsers support logical properties well

## Component Changes

### New Files

1. `src/utils/i18n.ts` - i18n configuration and initialization
2. `src/hooks/useTranslation.ts` - Custom hook for translations (if needed)
3. `src/components/LanguageSelector.tsx` - Language dropdown component
4. `resources/locales/*/translation.json` - Translation files for each language

### Modified Files

1. `src/App.tsx` - Replace hardcoded strings with t() function calls
2. `src/components/SettingsPanel.tsx` - Replace hardcoded strings
3. `src/components/StatusText.tsx` - Replace status text strings
4. `src/stores/progressStore.ts` - Add language setting
5. `src-tauri/src/main.rs` - Update tray menu to use translated strings

### Removed Files (if any)

None - this is a pure addition

## Data Flow

1. User opens Settings and selects a language
2. Language preference saved to settings store
3. i18n instance updates current language
4. All components re-render with translated strings
5. For RTL languages (Arabic), HTML dir attribute changes

## Testing Strategy

1. **Unit tests**: Test translation key resolution
2. **Integration tests**: Test language switching functionality
3. **Visual regression tests**: Verify UI renders correctly in all languages
4. **Manual testing**: Test RTL layout for Arabic

## Migration Path

For existing users:
- Language preference defaults to system language on first load
- No migration needed for existing settings

## Security Considerations

- Translation files are static JSON - no injection risk
- Translation keys are validated to prevent undefined keys
- User-provided content (task names) is not translated

## Performance Considerations

- Translation files loaded on demand (lazy loading)
- Fallback to English if translation missing
- Cached in memory after first load
