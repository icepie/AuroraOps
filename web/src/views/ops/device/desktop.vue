<template>
  <div class="desktop-page">
    <div class="desktop-toolbar">
      <div class="desktop-title">
        <span class="desktop-title__text">{{ title }}</span>
        <span class="desktop-state" :class="stateClass">{{ statusText }}</span>
      </div>
      <div class="desktop-actions">
        <n-button
          class="desktop-action desktop-action-secondary"
          size="tiny"
          quaternary
          @mousedown.prevent
          @click="reload"
        >
          刷新
        </n-button>
        <n-button
          class="desktop-action desktop-action-primary"
          size="tiny"
          type="primary"
          @mousedown.prevent
          @click="openStandalone"
        >
          新窗口
        </n-button>
      </div>
    </div>
    <div class="desktop-frame-wrap" @pointerdown="focusFrame" @click="focusFrame">
      <iframe
        ref="frameRef"
        class="desktop-frame"
        :src="weylusUrl"
        tabindex="0"
        title="AuroraOps 远程桌面"
        allow="fullscreen; clipboard-read; clipboard-write"
        @load="handleLoad"
        @error="handleError"
      ></iframe>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { computed, nextTick, onBeforeUnmount, onMounted, ref, watchEffect } from 'vue';
  import { useRoute } from 'vue-router';
  import { NButton } from 'naive-ui';
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
    focusFrame();
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
    focusFrame();
  }

  function openStandalone() {
    window.open(weylusUrl.value, '_blank', 'noopener,noreferrer');
  }

  function focusFrame() {
    window.requestAnimationFrame(() => {
      const frame = frameRef.value;
      if (!frame) return;
      try {
        frame.focus();
      } catch {
        // ignore focus failures for browsers that block iframe focus during navigation.
      }
      try {
        frame.contentWindow?.focus();
      } catch {
        // ignore cross-frame focus restrictions.
      }
    });
  }

  function shouldKeepKeyboardLocal(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    if (!target) return false;
    if (target.isContentEditable) return true;
    const tagName = target.tagName;
    return tagName === 'INPUT' || tagName === 'TEXTAREA' || tagName === 'SELECT';
  }

  function forwardKeyboardToFrame(event: KeyboardEvent) {
    if (shouldKeepKeyboardLocal(event)) return;
    const frame = frameRef.value;
    const frameWindow = frame?.contentWindow;
    const frameDocument = frame?.contentDocument;
    if (!frame || !frameWindow || !frameDocument) return;

    try {
      frame.focus();
      frameWindow.focus();
      const forwarded = new KeyboardEvent(event.type, {
        key: event.key,
        code: event.code,
        location: event.location,
        repeat: event.repeat,
        altKey: event.altKey,
        ctrlKey: event.ctrlKey,
        shiftKey: event.shiftKey,
        metaKey: event.metaKey,
        bubbles: true,
        cancelable: true,
      });
      frameDocument.dispatchEvent(forwarded);
      event.preventDefault();
      event.stopPropagation();
    } catch {
      // The iframe is same-origin in normal deployments. Ignore transient focus
      // errors while it is navigating or reloading.
    }
  }

  nextTick(() => {
    focusFrame();
  });

  onMounted(() => {
    window.addEventListener('keydown', forwardKeyboardToFrame, true);
    window.addEventListener('keyup', forwardKeyboardToFrame, true);
    window.addEventListener('keypress', forwardKeyboardToFrame, true);
    focusFrame();
  });

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', forwardKeyboardToFrame, true);
    window.removeEventListener('keyup', forwardKeyboardToFrame, true);
    window.removeEventListener('keypress', forwardKeyboardToFrame, true);
  });
</script>

<style scoped lang="less">
  .desktop-page {
    height: calc(100vh - 84px);
    height: calc(100dvh - 84px);
    max-height: calc(100dvh - 84px);
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
    --desktop-toolbar-height: 36px;
    --desktop-toolbar-text-offset: 2px;
    --desktop-toolbar-button-offset: 2px;
    flex: 0 0 auto;
    height: var(--desktop-toolbar-height);
    min-height: var(--desktop-toolbar-height);
    box-sizing: border-box;
    padding: 0 10px;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 12px;
    align-items: center;
    line-height: var(--desktop-toolbar-height);
    border-bottom: 1px solid rgba(255, 255, 255, 0.16);
    background: #111820;
  }

  .desktop-title {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 100%;
    font-size: 12px;
    line-height: var(--desktop-toolbar-height);
    transform: translateY(var(--desktop-toolbar-text-offset));
    white-space: nowrap;
  }

  .desktop-title__text {
    min-width: 0;
    overflow: hidden;
    color: #f6f8fa;
    font-weight: 600;
    line-height: var(--desktop-toolbar-height);
    text-overflow: ellipsis;
  }

  .desktop-state {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: #dbeafe;
    font-size: 10px;
    line-height: var(--desktop-toolbar-height);
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

  .desktop-actions {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .desktop-action {
    min-width: 44px;
    transform: translateY(var(--desktop-toolbar-button-offset));
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
    .desktop-page {
      height: calc(100dvh - 68px);
      max-height: calc(100dvh - 68px);
      min-height: 260px;
    }

    .desktop-toolbar {
      height: auto;
      min-height: 36px;
      padding: 4px 8px;
      gap: 8px;
    }

    .desktop-title {
      font-size: 11px;
      gap: 6px;
    }
  }
</style>
