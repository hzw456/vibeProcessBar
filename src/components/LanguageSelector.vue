<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { SUPPORTED_LANGUAGES, type SupportedLanguage } from '../utils/i18n';
import { useProgressStore } from '../stores/progressStore';

const { t } = useI18n();
const store = useProgressStore();

function handleLanguageChange(event: Event) {
  const target = event.target as HTMLSelectElement;
  store.setLanguage(target.value as SupportedLanguage);
}
</script>

<template>
  <select 
    class="language-selector" 
    :value="store.settings.language" 
    @change="handleLanguageChange"
  >
    <option 
      v-for="lang in SUPPORTED_LANGUAGES" 
      :key="lang.code" 
      :value="lang.code"
    >
      {{ lang.nativeName }}
    </option>
  </select>
</template>

<style scoped>
.language-selector {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  padding: 6px 10px;
  color: var(--text-color);
  font-size: 13px;
  cursor: pointer;
  min-width: 120px;
}

.language-selector:focus {
  outline: none;
  border-color: var(--primary-color);
}

.language-selector option {
  background: var(--bg-color);
  color: var(--text-color);
}
</style>
