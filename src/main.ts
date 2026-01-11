import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { watch } from 'vue';
import App from './App.vue';
import { i18n, initI18n } from './utils/i18n';
import { useProgressStore } from './stores/progressStore';
import './index.css';
import './App.css';

// Set default theme immediately to prevent white flash
document.documentElement.setAttribute('data-theme', 'dark');

async function bootstrap() {
    const app = createApp(App);
    const pinia = createPinia();

    app.use(pinia);
    app.use(i18n);

    // Initialize store and load settings
    const store = useProgressStore();
    await store.loadSettings();

    // Apply theme from settings
    document.documentElement.setAttribute('data-theme', store.settings.theme);
    
    // Apply font size from settings
    document.documentElement.style.setProperty('--app-font-size', `${store.settings.fontSize}px`);

    // Watch for theme changes
    watch(() => store.settings.theme, (newTheme) => {
        document.documentElement.setAttribute('data-theme', newTheme);
    });
    
    // Watch for font size changes
    watch(() => store.settings.fontSize, (newSize) => {
        document.documentElement.style.setProperty('--app-font-size', `${newSize}px`);
    });

    // Initialize i18n with user's language preference
    await initI18n(store.settings.language);

    app.mount('#root');
}

bootstrap();
