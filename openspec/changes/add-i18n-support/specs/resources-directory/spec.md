# Spec: resources-directory

## ADDED Requirements

### REQ-001: Resources Directory Structure
The application shall have a centralized resources directory at the project root.

#### Scenario: Directory creation
Given the project is initialized
When the i18n support is added
Then a `resources/` directory shall exist at the project root
And it shall contain subdirectories for locales, icons, and other assets

#### Scenario: Resources directory contents
Given the resources directory exists
When listing its contents
Then it shall contain at least:
- `resources/locales/` - translation files
- `resources/icons/` - application icons

### REQ-002: Locales Subdirectory
The locales subdirectory shall contain translation files organized by language.

#### Scenario: Translation file organization
Given the locales directory
When viewing its contents
Then it shall contain subdirectories for each supported language
With language codes following IANA standard (en, zh-CN, zh-TW, es, fr, de, ja, ko, ru, pt, ar)

#### Scenario: Translation file format
Given a language subdirectory
When viewing translation files
Then it shall contain a `translation.json` file
And the file shall be valid JSON format

### REQ-003: Icons Subdirectory
The icons subdirectory shall contain all application icons.

#### Scenario: Icon consolidation
Given the icons directory
When viewing its contents
Then it shall contain all icons currently in `src-tauri/icons/`
Including tray icons and application icons

#### Scenario: Icon file types
Given the icons directory
When viewing file types
Then it shall support PNG, SVG, and ICO formats as needed
And icons shall be organized by size or purpose

### REQ-004: Vite Configuration
Vite shall be configured to handle resources directory.

#### Scenario: Locale file serving
Given the application is built
When the frontend requests a locale file
Then Vite shall serve the file from the build output
And locale files shall be accessible at runtime

#### Scenario: Build output structure
Given the application is built
When viewing the build output
Then the locales directory shall be copied to the distribution
With all translation files included

### REQ-005: TypeScript Support
The application shall have TypeScript support for translation keys.

#### Scenario: Type-safe translation keys
Given a TypeScript component uses i18n
When referencing a translation key
Then the IDE shall provide autocomplete for available keys
Or at minimum, allow string key access

#### Scenario: Language type definition
Given the TypeScript project
When defining supported languages
Then there shall be a type that lists all available language codes
And the type shall include language metadata (name, nativeName, direction)

### REQ-006: Git Ignore
The resources directory shall be properly managed in version control.

#### Scenario: Generated files
Given translation files are modified locally
When changes are committed to git
Then only intentional changes to translation files shall be committed
And no auto-generated cache files shall be tracked

#### Scenario: Large binary files
Given large icon files are in resources/icons
When the repository is cloned
Then the files shall be available without additional download steps

### REQ-007: Path Conventions
All resource paths shall follow consistent conventions.

#### Scenario: Absolute vs relative paths
Given a component references a resource
When the path is specified
Then it shall use a consistent path resolution strategy
Preferably relative to the resources root

#### Scenario: Icon path references
Given the application references an icon
When the icon path is specified
Then it shall reference the path in resources/icons/
And the path shall be consistent across frontend and backend

## MODIFIED Requirements

None - this is a new capability.

## REMOVED Requirements

None - this is a new capability.

## Cross-Reference

This specification depends on:
- `i18n-framework`: Translation files defined here are used by i18n framework
