<template>
  <div class="desktop-page">
    <div class="desktop-toolbar">
      <div class="desktop-title">
        <span>{{ title }}</span>
        <span class="desktop-state" :class="stateClass">{{ statusText }}</span>
      </div>
      <n-space align="center" :size="8">
        <n-button class="desktop-action desktop-action-secondary" size="tiny" quaternary @click="reload">
          刷新
        </n-button>
        <n-button
          class="desktop-action desktop-action-primary"
          size="tiny"
          type="primary"
          @click="openStandalone"
        >
          新窗口
        </n-button>
      </n-space>
    </div>
    <div class="desktop-frame-wrap">
      <iframe
        ref="frameRef"
        class="desktop-frame"
        :src="weylusUrl"
        title="AuroraOps 远程桌面"
        allow="fullscreen; clipboard-read; clipboard-write"
        @load="handleLoad"
        @error="handleError"
      ></iframe>
    </div>
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
    if (loaded.value) return '远程桌面已连接';
    return '正在连接远程桌面';
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
    height: calc(100vh - 96px);
    height: calc(100dvh - 96px);
    min-height: 320px;
    width: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 0;
    background: #0b0f14;
    color: #f6f8fa;
  }

  .desktop-toolbar {
    flex: 0 0 34px;
    min-height: 0;
    box-sizing: border-box;
    padding: 3px 10px 3px 12px;
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    background: #111820;
  }

  .desktop-title {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    line-height: 1.2;
    white-space: nowrap;
  }

  .desktop-state {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: rgba(246, 248, 250, 0.68);
    font-size: 10px;
  }

  .desktop-state::before {
    content: '';
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }

  .desktop-state.is-online {
    color: #37d67a;
  }

  .desktop-state.is-offline {
    color: #ff6b6b;
  }

  .desktop-action {
    min-width: 48px;
  }

  :deep(.desktop-action-secondary) {
    --n-text-color: #f6f8fa !important;
    --n-text-color-hover: #ffffff !important;
    --n-text-color-pressed: #ffffff !important;
    --n-border: 1px solid rgba(246, 248, 250, 0.38) !important;
    --n-border-hover: 1px solid rgba(246, 248, 250, 0.7) !important;
    --n-border-pressed: 1px solid rgba(246, 248, 250, 0.8) !important;
    --n-color: rgba(255, 255, 255, 0.08) !important;
    --n-color-hover: rgba(255, 255, 255, 0.16) !important;
    --n-color-pressed: rgba(255, 255, 255, 0.2) !important;
  }

  :deep(.desktop-action-primary) {
    --n-text-color: #ffffff !important;
    --n-text-color-hover: #ffffff !important;
    --n-text-color-pressed: #ffffff !important;
    --n-color: #2f7cf6 !important;
    --n-color-hover: #4a8dff !important;
    --n-color-pressed: #246be0 !important;
    --n-border: 1px solid #2f7cf6 !important;
    --n-border-hover: 1px solid #4a8dff !important;
    --n-border-pressed: 1px solid #246be0 !important;
  }

  .desktop-frame-wrap {
    width: 100%;
    min-height: 0;
    box-sizing: border-box;
    flex: 1;
    overflow: hidden;
    background: #050607;
  }

  .desktop-frame {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    border: 0;
    background: #050607;
    display: block;
  }

  @media (max-width: 640px) {
    .desktop-toolbar {
      flex-basis: 40px;
      padding: 4px 8px;
      gap: 8px;
      flex-wrap: wrap;
    }

    .desktop-title {
      font-size: 11px;
      gap: 6px;
    }
  }
</style>
