import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useProgressStore } from '../stores/progressStore';
import { SUPPORTED_LANGUAGES, isRTL } from '../utils/i18n';

export function LanguageSelector() {
  const { t, i18n } = useTranslation();
  const { settings, setLanguage } = useProgressStore();

  const handleLanguageChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newLanguage = e.target.value as typeof SUPPORTED_LANGUAGES[number]['code'];
    setLanguage(newLanguage);
    i18n.changeLanguage(newLanguage);
  };

  useEffect(() => {
    if (settings.language && settings.language !== i18n.language) {
      i18n.changeLanguage(settings.language);
    }
  }, [settings.language, i18n]);

  useEffect(() => {
    const direction = isRTL(settings.language) ? 'rtl' : 'ltr';
    document.documentElement.dir = direction;
  }, [settings.language]);

  return (
    <div className="setting-item">
      <label>{t('language.selector')}</label>
      <select
        value={settings.language}
        onChange={handleLanguageChange}
      >
        {SUPPORTED_LANGUAGES.map((lang) => (
          <option key={lang.code} value={lang.code}>
            {lang.nativeName}
          </option>
        ))}
      </select>
    </div>
  );
}
