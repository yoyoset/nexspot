import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import en from './locales/en.json';
import zh from './locales/zh.json';

i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
        resources: {
            en: { translation: en },
            zh: { translation: zh },
            'zh-CN': { translation: zh },
        },
        lng: 'zh', // Force Chinese as requested
        fallbackLng: 'zh',
        interpolation: {
            escapeValue: false, // React already safe from XSS
        },
        detection: {
            order: ['localStorage', 'navigator'],
            lookupLocalStorage: 'i18nextLng',
            caches: ['localStorage'],
        }
    });

export default i18n;
