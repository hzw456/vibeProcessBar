import { createI18n } from 'vue-i18n';

export type SupportedLanguage =
  | 'en'
  | 'zh-CN'
  | 'zh-TW'
  | 'es'
  | 'fr'
  | 'de'
  | 'ja'
  | 'ko'
  | 'ru'
  | 'pt'
  | 'ar';

export interface LanguageInfo {
  code: SupportedLanguage;
  name: string;
  nativeName: string;
  dir: 'ltr' | 'rtl';
}

export const SUPPORTED_LANGUAGES: LanguageInfo[] = [
  { code: 'en', name: 'English', nativeName: 'English', dir: 'ltr' },
  { code: 'zh-CN', name: 'Simplified Chinese', nativeName: '简体中文', dir: 'ltr' },
  { code: 'zh-TW', name: 'Traditional Chinese', nativeName: '繁體中文', dir: 'ltr' },
  { code: 'es', name: 'Spanish', nativeName: 'Español', dir: 'ltr' },
  { code: 'fr', name: 'French', nativeName: 'Français', dir: 'ltr' },
  { code: 'de', name: 'German', nativeName: 'Deutsch', dir: 'ltr' },
  { code: 'ja', name: 'Japanese', nativeName: '日本語', dir: 'ltr' },
  { code: 'ko', name: 'Korean', nativeName: '한국어', dir: 'ltr' },
  { code: 'ru', name: 'Russian', nativeName: 'Русский', dir: 'ltr' },
  { code: 'pt', name: 'Portuguese', nativeName: 'Português', dir: 'ltr' },
  { code: 'ar', name: 'Arabic', nativeName: 'العربية', dir: 'rtl' },
];

export const getLanguageByCode = (code: string): LanguageInfo | undefined => {
  return SUPPORTED_LANGUAGES.find(lang => lang.code === code);
};

export const isRTL = (code: string): boolean => {
  const lang = getLanguageByCode(code);
  return lang?.dir === 'rtl' || false;
};

// Dynamically load locale messages
async function loadLocaleMessages(locale: string): Promise<Record<string, any>> {
  try {
    const response = await fetch(`/locales/${locale}/translation.json`);
    if (response.ok) {
      return await response.json();
    }
  } catch (e) {
    console.warn(`Failed to load locale ${locale}:`, e);
  }
  return {};
}

// Create i18n instance
export const i18n = createI18n({
  legacy: false, // Use Composition API mode
  locale: 'en',
  fallbackLocale: 'en',
  messages: {},
  globalInjection: true,
  missingWarn: false,
  fallbackWarn: false,
});

// Function to change language
export async function setLanguage(locale: SupportedLanguage): Promise<void> {
  if (!i18n.global.availableLocales.includes(locale)) {
    const messages = await loadLocaleMessages(locale);
    i18n.global.setLocaleMessage(locale, messages);
  }
  i18n.global.locale.value = locale;
  document.documentElement.setAttribute('dir', isRTL(locale) ? 'rtl' : 'ltr');
}

// Initialize with default language
export async function initI18n(locale: SupportedLanguage = 'en'): Promise<void> {
  // Load default locale
  const enMessages = await loadLocaleMessages('en');
  i18n.global.setLocaleMessage('en', enMessages);

  if (locale !== 'en') {
    await setLanguage(locale);
  }
}
