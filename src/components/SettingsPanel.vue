<script setup lang="ts">
import { useProgressStore } from '../stores/progressStore';
import LanguageSelector from './LanguageSelector.vue';
import './SettingsPanel.css';
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { playSound } from '../utils/notifications';

interface Props {
  isStandalone?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isStandalone: true,
});

const emit = defineEmits<{
  close: [];
}>();

const store = useProgressStore();
const { t } = useI18n();

type TabType = 'general' | 'appearance';
const activeTab = ref<TabType>('general');

const themes = ['dark', 'purple', 'ocean', 'forest', 'midnight'] as const;

// App version
const appVersion = '1.0.1';

// Computed labels
const volumeLabel = computed(() => t('settings.notifications.volume', { percentage: Math.round(store.settings.soundVolume * 100) }));
const fontSizeLabel = computed(() => t('settings.appearance.fontSize', { size: store.settings.fontSize }));
const opacityLabel = computed(() => t('settings.appearance.opacity', { percentage: Math.round(store.settings.opacity * 100) }));

function handleResetDefaults() {
  store.setLanguage('en');
  store.setTheme('dark');
  store.setFontSize(14);
  store.setOpacity(0.85);
  store.setAlwaysOnTop(true);
  store.setAutoStart(false);
  store.setSound(true);
  store.setSoundVolume(0.7);
  store.setHttpHost('127.0.0.1');
  store.setHttpPort(31415);
  store.setBlockPluginStatus(true);
}

function handleToggleWindow() {
  store.setWindowVisible(!store.settings.windowVisible);
}

function handleTestSound() {
  playSound(store.settings.soundVolume);
}

function handlePositionChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const axis = input.dataset.axis;
  const value = parseInt(input.value) || 0;
  
  const x = axis === 'x' ? value : (store.settings.windowX ?? 0);
  const y = axis === 'y' ? value : (store.settings.windowY ?? 0);
  
  store.setWindowPosition(x, y);
}
</script>

<template>
  <div :class="['settings-panel', { standalone: isStandalone }]" @click.stop>
    <!-- Header for overlay mode -->
    <div v-if="!isStandalone" class="settings-header">
      <h2>{{ t('settings.title') }}</h2>
      <button class="close-btn" @click="emit('close')">×</button>
    </div>

    <!-- Tabs -->
    <div class="settings-tabs">
      <button :class="['tab', { active: activeTab === 'general' }]" @click="activeTab = 'general'">
        {{ t('settings.tabs.general') }}
      </button>
      <button :class="['tab', { active: activeTab === 'appearance' }]" @click="activeTab = 'appearance'">
        {{ t('settings.tabs.appearance') }}
      </button>
    </div>

    <!-- Content -->
    <div class="settings-content">
      <!-- General Tab -->
      <div v-if="activeTab === 'general'" class="settings-section">
        <div class="setting-item">
          <label>{{ t('settings.general.language') }}</label>
          <LanguageSelector />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.general.alwaysOnTop') }}</label>
          <input type="checkbox" :checked="store.settings.alwaysOnTop" @change="store.setAlwaysOnTop(($event.target as HTMLInputElement).checked)" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.general.autoStart') }}</label>
          <input type="checkbox" :checked="store.settings.autoStart" @change="store.setAutoStart(($event.target as HTMLInputElement).checked)" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.notifications.soundAlerts') }}</label>
          <input type="checkbox" :checked="store.settings.sound" @change="store.setSound(($event.target as HTMLInputElement).checked)" />
        </div>
        <div v-if="store.settings.sound" class="setting-item indent">
          <label>{{ volumeLabel }}</label>
          <div class="volume-control">
            <input type="range" :value="store.settings.soundVolume" @input="store.setSoundVolume(parseFloat(($event.target as HTMLInputElement).value))" min="0" max="1" step="0.1" />
            <button class="action-btn small" @click="handleTestSound">{{ t('settings.notifications.test') }}</button>
          </div>
        </div>
        <div class="setting-item">
          <label>{{ t('settings.general.httpHost') }}</label>
          <input type="text" :value="store.settings.httpHost" @change="store.setHttpHost(($event.target as HTMLInputElement).value)" class="host-input" placeholder="127.0.0.1" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.general.httpPort') }}</label>
          <input type="number" :value="store.settings.httpPort" @change="store.setHttpPort(parseInt(($event.target as HTMLInputElement).value))" min="1024" max="65535" class="port-input" />
        </div>
        <div class="setting-hint">{{ t('settings.general.httpRestartHint') }}</div>
        <div class="setting-item">
          <label>{{ t('settings.general.blockPluginStatus') }}</label>
          <input type="checkbox" :checked="store.settings.blockPluginStatus" @change="store.setBlockPluginStatus(($event.target as HTMLInputElement).checked)" />
        </div>
      </div>

      <!-- Appearance Tab -->
      <div v-if="activeTab === 'appearance'" class="settings-section">
        <div class="setting-item">
          <label>{{ t('settings.appearance.theme') }}</label>
          <select :value="store.settings.theme" @change="store.setTheme(($event.target as HTMLSelectElement).value as any)" class="theme-select">
            <option v-for="theme in themes" :key="theme" :value="theme">
              {{ t(`settings.appearance.themes.${theme}`) }}
            </option>
          </select>
        </div>
        <div class="setting-item">
          <label>{{ fontSizeLabel }}</label>
          <input type="range" :value="store.settings.fontSize" @input="store.setFontSize(parseInt(($event.target as HTMLInputElement).value))" min="12" max="18" step="1" />
        </div>
        <div class="setting-item">
          <label>{{ opacityLabel }}</label>
          <input type="range" :value="store.settings.opacity" @input="store.setOpacity(parseFloat(($event.target as HTMLInputElement).value))" min="0.5" max="1" step="0.05" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.appearance.windowPosition') }}</label>
          <div class="position-inputs">
            <label class="position-label">{{ t('settings.appearance.positionX') }}</label>
            <input type="number" :value="store.settings.windowX ?? 0" @change="handlePositionChange" class="position-input" data-axis="x" />
            <label class="position-label">{{ t('settings.appearance.positionY') }}</label>
            <input type="number" :value="store.settings.windowY ?? 0" @change="handlePositionChange" class="position-input" data-axis="y" />
          </div>
        </div>
        <div class="setting-hint">{{ t('settings.appearance.windowPositionHint') }}</div>
      </div>
    </div>

    <!-- Footer -->
    <div class="settings-footer">
      <div class="version-info">v{{ appVersion }}</div>
      <button class="reset-btn" @click="handleResetDefaults">
        {{ t('settings.footer.resetDefaults') }}
      </button>
    </div>
  </div>
</template>
