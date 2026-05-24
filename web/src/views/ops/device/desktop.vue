<template>
  <div class="desktop-page">
    <div class="desktop-toolbar">
      <div class="desktop-title">
        <span>{{ title }}</span>
        <span class="desktop-state" :class="stateClass">{{ statusText }}</span>
      </div>
      <n-space align="center" :size="10">
        <n-button size="small" @click="reload">刷新</n-button>
        <n-button size="small" type="primary" @click="openStandalone">新窗口</n-button>
      </n-space>
    </div>
    <iframe
      ref="frameRef"
      class="desktop-frame"
      :src="weylusUrl"
      title="Weylus Remote Desktop"
      allow="fullscreen; clipboard-read; clipboard-write"
      @load="handleLoad"
      @error="handleError"
    />
  </div>
</template>

<script setup lang="ts">
  import { computed, ref, watchEffect } from 'vue';
  import { useRoute } from 'vue-router';
  import { NButton, NSpace } from 'naive-ui';
  import { ACCESS_TOKEN } from '@/store/mutation-types';
  import { useTabsViewStore } from '@/store/modules/tabsView';
  import { storage } from '@/utils/Storage';

  const route = useRoute();
  const tabsViewStore = useTabsViewStore();
  const frameRef = ref<HTMLIFrameElement | null>(null);
  const loaded = ref(false);
  const failed = ref(false);
  const deviceId = Number(route.query.deviceId || 0);
  const deviceName = computed(() => String(route.query.name || route.query.deviceId || '').trim());
  const title = computed(() => (deviceName.value ? `远程桌面 - ${deviceName.value}` : '远程桌面'));
  const statusText = computed(() => {
    if (failed.value) return '连接失败';
    if (loaded.value) return 'Weylus 已加载';
    return '正在连接 Weylus';
  });
  const stateClass = computed(() => ({
    'is-online': loaded.value && !failed.value,
    'is-offline': failed.value,
  }));

  watchEffect(() => {
    route.meta.title = title.value;
    document.title = title.value;
    tabsViewStore.updateTabTitle(route.fullPath, title.value);
  });

  const weylusUrl = computed(() => {
    const token = storage.get(ACCESS_TOKEN, '') || '';
    const params = new URLSearchParams();
    params.set('deviceId', String(deviceId));
    params.set('authorization', token);
    return `/admin/opsDevice/weylus/?${params.toString()}`;
  });

  function handleLoad() {
    loaded.value = true;
    failed.value = false;
  }

  function handleError() {
    failed.value = true;
  }

  function reload() {
    loaded.value = false;
    failed.value = false;
    const frame = frameRef.value;
    if (frame) {
      frame.src = weylusUrl.value;
    }
  }

  function openStandalone() {
    window.open(weylusUrl.value, '_blank', 'noopener,noreferrer');
  }
</script>

<style scoped lang="less">
  .desktop-page {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: #111418;
    color: #f5f7fa;
  }

  .desktop-toolbar {
    min-height: 48px;
    padding: 6px 12px;
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
    background: #171c22;
  }

  .desktop-title {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 14px;
    line-height: 1.25;
    white-space: nowrap;
  }

  .desktop-state {
    color: rgba(245, 247, 250, 0.62);
    font-size: 12px;
  }

  .desktop-state.is-online {
    color: #6ee7a8;
  }

  .desktop-state.is-offline {
    color: #ff8a8a;
  }

  .desktop-frame {
    flex: 1;
    width: 100%;
    min-height: calc(100vh - 48px);
    border: 0;
    background: #050607;
  }

  @media (max-width: 640px) {
    .desktop-toolbar {
      min-height: 56px;
    }

    .desktop-frame {
      min-height: calc(100vh - 56px);
    }
  }
</style>
