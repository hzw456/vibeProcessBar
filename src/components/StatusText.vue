<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import './StatusText.css';

interface Props {
  name: string;
  status: 'armed' | 'running' | 'completed' | 'idle';
  isFocused?: boolean;
  tokens?: number;
  ide?: string;
  elapsedTime?: string;
  showIcon?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  tokens: 0,
  showIcon: true,
  isFocused: false,
});

const emit = defineEmits<{
  activate: [];
}>();

const { t } = useI18n();

const statusIcon = computed(() => {
  // Focused window shows eye icon
  if (props.isFocused) return '◈';
  switch (props.status) {
    case 'idle': return '○';
    case 'armed': return '◎';
    case 'running': return '◉';
    case 'completed': return '✓';
    default: return '○';
  }
});

const statusText = computed(() => {
  switch (props.status) {
    case 'idle': return t('status.idle');
    case 'armed': return props.name || t('status.armed');
    case 'running': return props.name || t('status.running');
    case 'completed':
      return props.elapsedTime
        ? t('status.completedWithTime', { taskName: props.name, elapsedTime: props.elapsedTime })
        : props.name || t('status.completed');
    default: return t('status.idle');
  }
});

function formatTokens(num: number): string {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
  return num.toString();
}

function handleClick() {
  if (props.ide) {
    emit('activate');
  }
}

function getTranslatedIdeName(ideName: string): string {
  const key = ideName.toLowerCase().replace(/\s+/g, '-');
  const translation = t(`ide.${key}`);
  return translation !== `ide.${key}` ? translation : ideName;
}
</script>

<template>
  <div
    class="status-container"
    :style="{ cursor: ide ? 'pointer' : 'default' }"
    @click="handleClick"
  >
    <span v-if="showIcon" :class="['status-icon', `status-${status}`]">{{ statusIcon }}</span>
    <span :class="['status-text', `status-${status}`]">{{ statusText }}</span>
    <span v-if="tokens > 0" class="token-count">{{ formatTokens(tokens) }}</span>
    <span v-if="ide" class="ide-badge" :title="`Click to activate ${ide}`">
      {{ getTranslatedIdeName(ide) }}
    </span>
  </div>
</template>
