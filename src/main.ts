import { createApp } from 'vue';
import { createPinia } from 'pinia';
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

    // Initialize i18n with user's language preference
    await initI18n(store.settings.language);

    app.mount('#root');
}

bootstrap();
