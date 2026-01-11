<script setup lang="ts">
import { useProgressStore } from '../stores/progressStore';
import LanguageSelector from './LanguageSelector.vue';
import './SettingsPanel.css';
import { ref, computed } from 'vue';
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

type TabType = 'general' | 'appearance';
const activeTab = ref<TabType>('general');

const themes = ['dark', 'purple', 'ocean', 'forest', 'midnight'] as const;

// App version
const appVersion = '0.1.0';

// Computed labels
const volumeLabel = computed(() => `音量: ${Math.round(store.settings.soundVolume * 100)}%`);
const fontSizeLabel = computed(() => `字体大小: ${store.settings.fontSize}px`);
const opacityLabel = computed(() => `透明度: ${Math.round(store.settings.opacity * 100)}%`);

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
</script>

<template>
  <div :class="['settings-panel', { standalone: isStandalone }]" @click.stop>
    <!-- Header for overlay mode -->
    <div v-if="!isStandalone" class="settings-header">
      <h2>设置</h2>
      <button class="close-btn" @click="emit('close')">×</button>
    </div>

    <!-- Tabs -->
    <div class="settings-tabs">
      <button :class="['tab', { active: activeTab === 'general' }]" @click="activeTab = 'general'">
        常规
      </button>
      <button :class="['tab', { active: activeTab === 'appearance' }]" @click="activeTab = 'appearance'">
        外观
      </button>
    </div>

    <!-- Content -->
    <div class="settings-content">
      <!-- General Tab -->
      <div v-if="activeTab === 'general'" class="settings-section">
        <div class="setting-item">
          <label>语言</label>
          <LanguageSelector />
        </div>
        <div class="setting-item">
          <label>始终置顶</label>
          <input type="checkbox" :checked="store.settings.alwaysOnTop" @change="store.setAlwaysOnTop(($event.target as HTMLInputElement).checked)" />
        </div>
        <div class="setting-item">
          <label>登录时自动启动</label>
          <input type="checkbox" :checked="store.settings.autoStart" @change="store.setAutoStart(($event.target as HTMLInputElement).checked)" />
        </div>
        <div class="setting-item">
          <label>{{ store.settings.windowVisible ? '隐藏窗口' : '显示窗口' }}</label>
          <button class="action-btn small" @click="handleToggleWindow">
            {{ store.settings.windowVisible ? '☾' : '☀' }}
          </button>
        </div>
        <div class="setting-item">
          <label>声音提醒</label>
          <input type="checkbox" :checked="store.settings.sound" @change="store.setSound(($event.target as HTMLInputElement).checked)" />
        </div>
        <div v-if="store.settings.sound" class="setting-item indent">
          <label>{{ volumeLabel }}</label>
          <div class="volume-control">
            <input type="range" :value="store.settings.soundVolume" @input="store.setSoundVolume(parseFloat(($event.target as HTMLInputElement).value))" min="0" max="1" step="0.1" />
            <button class="action-btn small" @click="handleTestSound">测试</button>
          </div>
        </div>
        <div class="setting-item">
          <label>HTTP 监听地址</label>
          <input type="text" :value="store.settings.httpHost" @change="store.setHttpHost(($event.target as HTMLInputElement).value)" class="host-input" placeholder="127.0.0.1" />
        </div>
        <div class="setting-item">
          <label>HTTP API 端口</label>
          <input type="number" :value="store.settings.httpPort" @change="store.setHttpPort(parseInt(($event.target as HTMLInputElement).value))" min="1024" max="65535" class="port-input" />
        </div>
        <div class="setting-item">
          <label>屏蔽插件状态上报</label>
          <input type="checkbox" :checked="store.settings.blockPluginStatus" @change="store.setBlockPluginStatus(($event.target as HTMLInputElement).checked)" />
        </div>
      </div>

      <!-- Appearance Tab -->
      <div v-if="activeTab === 'appearance'" class="settings-section">
        <div class="setting-item">
          <label>主题</label>
          <select :value="store.settings.theme" @change="store.setTheme(($event.target as HTMLSelectElement).value as any)" class="theme-select">
            <option v-for="theme in themes" :key="theme" :value="theme">
              {{ theme === 'dark' ? '深色 (默认)' : theme === 'purple' ? '紫色' : theme === 'ocean' ? '海洋' : theme === 'forest' ? '森林' : '午夜' }}
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
      </div>
    </div>

    <!-- Footer -->
    <div class="settings-footer">
      <div class="version-info">v{{ appVersion }}</div>
      <button class="reset-btn" @click="handleResetDefaults">
        重置为默认
      </button>
    </div>
  </div>
</template>
