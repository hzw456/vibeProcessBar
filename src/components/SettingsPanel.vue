<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useProgressStore } from '../stores/progressStore';
import LanguageSelector from './LanguageSelector.vue';
import './SettingsPanel.css';
import { ref } from 'vue';

interface Props {
  isStandalone?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isStandalone: true,
});

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
const store = useProgressStore();

type TabType = 'general' | 'appearance' | 'notifications' | 'tasks' | 'shortcuts';
const activeTab = ref<TabType>('general');

const themes = ['dark', 'light', 'purple', 'ocean', 'forest', 'midnight'] as const;


function handleResetDefaults() {
  store.setLanguage('en');
  store.setTheme('dark');
  store.setFontSize(14);
  store.setOpacity(0.85);
  store.setAlwaysOnTop(true);
  store.setAutoStart(false);
  store.setNotifications(true);
  store.setSound(true);
  store.setSoundVolume(0.7);
  store.setHttpHost('127.0.0.1');
  store.setHttpPort(31415);
  store.setCustomColors({ primaryColor: '', backgroundColor: '', textColor: '' });
  store.setReminderThreshold(100);
  store.setDoNotDisturb(false);
  store.setDoNotDisturbStart('22:00');
  store.setDoNotDisturbEnd('08:00');
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
      <button :class="['tab', { active: activeTab === 'notifications' }]" @click="activeTab = 'notifications'">
        {{ t('settings.tabs.notifications') }}
      </button>
      <button :class="['tab', { active: activeTab === 'tasks' }]" @click="activeTab = 'tasks'">
        {{ t('settings.tabs.tasks') }}
      </button>
      <button :class="['tab', { active: activeTab === 'shortcuts' }]" @click="activeTab = 'shortcuts'">
        {{ t('settings.tabs.shortcuts') }}
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
          <label>{{ t('settings.general.httpHost') }}</label>
          <input type="text" :value="store.settings.httpHost" @change="store.setHttpHost(($event.target as HTMLInputElement).value)" class="host-input" placeholder="127.0.0.1" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.general.httpPort') }}</label>
          <input type="number" :value="store.settings.httpPort" @change="store.setHttpPort(parseInt(($event.target as HTMLInputElement).value))" min="1024" max="65535" class="port-input" />
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
          <label>{{ t('settings.appearance.fontSize', { size: store.settings.fontSize }) }}</label>
          <input type="range" :value="store.settings.fontSize" @input="store.setFontSize(parseInt(($event.target as HTMLInputElement).value))" min="10" max="24" step="1" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.appearance.opacity', { percentage: Math.round(store.settings.opacity * 100) }) }}</label>
          <input type="range" :value="store.settings.opacity" @input="store.setOpacity(parseFloat(($event.target as HTMLInputElement).value))" min="0.3" max="1" step="0.05" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.appearance.primaryColor') }}</label>
          <div class="color-input-wrapper">
            <input type="color" :value="store.settings.customColors.primaryColor || '#6366f1'" @input="store.setCustomColors({ primaryColor: ($event.target as HTMLInputElement).value })" class="color-input" />
            <input type="text" class="color-text-input" :value="store.settings.customColors.primaryColor || ''" @input="store.setCustomColors({ primaryColor: ($event.target as HTMLInputElement).value })" placeholder="#6366f1" />
          </div>
        </div>
        <div class="setting-item">
          <label>{{ t('settings.appearance.resetColors') }}</label>
          <button class="action-btn small" @click="store.setCustomColors({ primaryColor: '', backgroundColor: '', textColor: '' })">
            {{ t('settings.appearance.reset') }}
          </button>
        </div>
      </div>

      <!-- Notifications Tab -->
      <div v-if="activeTab === 'notifications'" class="settings-section">
        <div class="setting-item">
          <label>{{ t('settings.notifications.desktopNotifications') }}</label>
          <input type="checkbox" :checked="store.settings.notifications" @change="store.setNotifications(($event.target as HTMLInputElement).checked)" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.notifications.soundAlerts') }}</label>
          <input type="checkbox" :checked="store.settings.sound" @change="store.setSound(($event.target as HTMLInputElement).checked)" />
        </div>
        <div v-if="store.settings.sound" class="setting-item indent">
          <label>{{ t('settings.notifications.volume', { percentage: Math.round(store.settings.soundVolume * 100) }) }}</label>
          <input type="range" :value="store.settings.soundVolume" @input="store.setSoundVolume(parseFloat(($event.target as HTMLInputElement).value))" min="0" max="1" step="0.1" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.notifications.completionThreshold', { percentage: store.settings.reminderThreshold }) }}</label>
          <input type="range" :value="store.settings.reminderThreshold" @input="store.setReminderThreshold(parseInt(($event.target as HTMLInputElement).value))" min="50" max="100" step="5" />
        </div>
        <div class="setting-item">
          <label>{{ t('settings.notifications.doNotDisturb') }}</label>
          <input type="checkbox" :checked="store.settings.doNotDisturb" @change="store.setDoNotDisturb(($event.target as HTMLInputElement).checked)" />
        </div>
        <template v-if="store.settings.doNotDisturb">
          <div class="setting-item indent">
            <label>{{ t('settings.notifications.startTime') }}</label>
            <input type="time" :value="store.settings.doNotDisturbStart" @input="store.setDoNotDisturbStart(($event.target as HTMLInputElement).value)" class="time-input" />
          </div>
          <div class="setting-item indent">
            <label>{{ t('settings.notifications.endTime') }}</label>
            <input type="time" :value="store.settings.doNotDisturbEnd" @input="store.setDoNotDisturbEnd(($event.target as HTMLInputElement).value)" class="time-input" />
          </div>
        </template>
      </div>

      <!-- Tasks Tab -->
      <div v-if="activeTab === 'tasks'" class="settings-section">
        <div class="setting-item">
          <label>{{ t('settings.tasks.history') }}</label>
          <span class="info-text">{{ t('settings.tasks.historyCount', { count: store.history.length }) }}</span>
        </div>
        <div class="setting-item">
          <label>{{ t('settings.tasks.clearHistory') }}</label>
          <button class="action-btn danger small" @click="store.clearHistory">
            {{ t('settings.tasks.clear') }}
          </button>
        </div>
      </div>

      <!-- Shortcuts Tab -->
      <div v-if="activeTab === 'shortcuts'" class="settings-section">
        <div class="shortcuts-info">
          <h4>{{ t('settings.shortcuts.title') }}</h4>
          <p>{{ t('settings.shortcuts.description') }}</p>
          <div class="shortcut-list">
            <div class="shortcut-item">
              <span class="shortcut-key">{{ t('settings.shortcuts.rightClick') }}</span>
              <span class="shortcut-desc">Open context menu</span>
            </div>
            <div class="shortcut-item">
              <span class="shortcut-key">{{ t('settings.shortcuts.drag') }}</span>
              <span class="shortcut-desc">Move window</span>
            </div>
            <div class="shortcut-item">
              <span class="shortcut-key">{{ t('settings.shortcuts.scroll') }}</span>
              <span class="shortcut-desc">Adjust progress</span>
            </div>
            <div class="shortcut-item">
              <span class="shortcut-key">{{ t('settings.shortcuts.clickStatus') }}</span>
              <span class="shortcut-desc">Activate IDE window</span>
            </div>
          </div>
        </div>
      </div>
    </div>



    <!-- Footer -->
    <div class="settings-footer">
      <div class="version-info">{{ t('settings.footer.version', { version: '0.1.0' }) }}</div>
      <button class="reset-btn" @click="handleResetDefaults">
        {{ t('settings.footer.resetDefaults') }}
      </button>
    </div>
  </div>
</template>
