import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import Backend from 'i18next-http-backend';

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

const DEFAULT_LANGUAGE: SupportedLanguage = 'en';

i18n
  .use(Backend)
  .use(initReactI18next)
  .init({
    supportedLngs: SUPPORTED_LANGUAGES.map(l => l.code),
    fallbackLng: DEFAULT_LANGUAGE,
    defaultNS: 'translation',
    ns: ['translation'],
    backend: {
      loadPath: '/locales/{{lng}}/translation.json',
    },
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  });

export { i18n };
