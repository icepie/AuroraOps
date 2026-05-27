<template>
  <div class="terminal-page">
    <div class="terminal-toolbar">
      <div class="terminal-title">
        <span>{{ title }}</span>
        <span class="terminal-state" :class="stateClass">{{ statusText }}</span>
      </div>
      <n-space align="center" :size="8" class="terminal-actions">
        <n-button
          class="terminal-action terminal-action-secondary"
          size="tiny"
          quaternary
          @mousedown.prevent
          @click="fitTerminal"
        >
          适配
        </n-button>
        <n-button
          class="terminal-action terminal-action-primary"
          size="tiny"
          type="primary"
          @mousedown.prevent
          @click="reconnect"
        >
          重连
        </n-button>
      </n-space>
    </div>
    <div
      ref="terminalRef"
      class="terminal-container"
      tabindex="0"
      @pointerdown="focusTerminal"
      @click="focusTerminal"
    />
  </div>
</template>

<script setup lang="ts">
  import {
    computed,
    nextTick,
    onActivated,
    onBeforeUnmount,
    onDeactivated,
    onMounted,
    ref,
    watch,
    watchEffect,
  } from 'vue';
  import { useRoute } from 'vue-router';
  import { NButton, NSpace, useMessage } from 'naive-ui';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { CreateTerminal } from '@/api/opsDevice';
  import { ACCESS_TOKEN } from '@/store/mutation-types';
  import { useTabsViewStore } from '@/store/modules/tabsView';
  import { storage } from '@/utils/Storage';
  import 'xterm/css/xterm.css';

  const route = useRoute();
  const message = useMessage();
  const tabsViewStore = useTabsViewStore();

  const terminalRef = ref<HTMLElement | null>(null);
  const closed = ref(false);
  const connected = ref(false);
  const opened = ref(false);
  const statusText = ref('准备连接');

  const sessionId = ref(String(route.query.sessionId || ''));
  const deviceId = computed(() => Number(route.query.deviceId || 0));
  const deviceName = computed(() => String(route.query.name || route.query.deviceId || '').trim());
  const title = computed(() => (deviceName.value ? `远程终端 - ${deviceName.value}` : '远程终端'));
  const stateClass = computed(() => ({
    'is-online': connected.value && opened.value,
    'is-offline': !connected.value,
  }));

  watchEffect(() => {
    route.meta.title = title.value;
    document.title = title.value;
    tabsViewStore.updateTabTitle(route.fullPath, title.value);
  });

  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let socket: WebSocket | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let pendingOutput = '';
  let outputFlushRaf = 0;
  let reconnectTimer = 0;
  let reconnecting = false;
  let manualClosing = false;
  let active = true;

  function buildWsUrl() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const token = storage.get(ACCESS_TOKEN, '') || '';
    return `${protocol}//${window.location.host}/admin/opsDevice/terminal/ws?sessionId=${encodeURIComponent(sessionId.value)}&authorization=${encodeURIComponent(token)}`;
  }

  function writeNotice(text: string) {
    terminal?.writeln('');
    terminal?.writeln(`\x1b[33m${text}\x1b[0m`);
  }

  function socketReady() {
    return socket?.readyState === WebSocket.OPEN;
  }

  function focusTerminal() {
    if (!active) return;
    window.requestAnimationFrame(() => {
      terminal?.focus();
    });
  }

  function sendPayload(payload: Record<string, unknown>) {
    if (!socketReady()) return false;
    socket?.send(JSON.stringify(payload));
    return true;
  }

  function resetTerminal() {
    pendingOutput = '';
    if (outputFlushRaf) {
      window.cancelAnimationFrame(outputFlushRaf);
      outputFlushRaf = 0;
    }
    terminal?.reset();
  }

  async function refreshSession() {
    if (!deviceId.value) return false;
    const res = await CreateTerminal({ deviceId: deviceId.value });
    if (!res?.sessionId) return false;
    sessionId.value = res.sessionId;
    return true;
  }

  function flushOutput() {
    outputFlushRaf = 0;
    if (!pendingOutput) return;
    const text = pendingOutput;
    pendingOutput = '';
    terminal?.write(text);
  }

  function queueOutput(text: string) {
    if (!text) return;
    pendingOutput += text;
    if (!outputFlushRaf) {
      outputFlushRaf = window.requestAnimationFrame(flushOutput);
    }
  }

  function sendOpen() {
    if (!terminal || !socketReady()) return;
    if (opened.value) {
      sendResize();
      return;
    }
    const ok = sendPayload({
      type: 'open',
      cols: terminal.cols,
      rows: terminal.rows,
    });
    if (ok) {
      opened.value = true;
      statusText.value = '已连接';
    }
  }

  function sendResize() {
    if (!terminal || !socketReady()) return;
    const ok = sendPayload({
      type: opened.value ? 'resize' : 'open',
      cols: terminal.cols,
      rows: terminal.rows,
    });
    if (ok) {
      opened.value = true;
    }
  }

  function fitTerminal(notifyServer = true) {
    if (!fitAddon || !terminalRef.value) return;
    try {
      fitAddon.fit();
      if (notifyServer) sendResize();
    } catch {
      // xterm can throw while the container is hidden during route/tab switches.
    } finally {
      focusTerminal();
    }
  }

  function initTerminal() {
    terminal = new Terminal({
      convertEol: false,
      cursorBlink: true,
      cursorStyle: 'block',
      fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Mono", monospace',
      fontSize: 14,
      lineHeight: 1.2,
      scrollback: 5000,
      drawBoldTextInBrightColors: true,
      allowTransparency: false,
      theme: {
        background: '#0c0c0c',
        foreground: '#cccccc',
        cursor: '#ffffff',
        cursorAccent: '#0c0c0c',
        selectionBackground: 'rgba(51, 153, 255, 0.35)',
        black: '#0c0c0c',
        red: '#c50f1f',
        green: '#13a10e',
        yellow: '#c19c00',
        blue: '#0037da',
        magenta: '#881798',
        cyan: '#3a96dd',
        white: '#cccccc',
        brightBlack: '#767676',
        brightRed: '#e74856',
        brightGreen: '#16c60c',
        brightYellow: '#f9f1a5',
        brightBlue: '#3b78ff',
        brightMagenta: '#b4009e',
        brightCyan: '#61d6d6',
        brightWhite: '#f2f2f2',
      },
    });

    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(terminalRef.value!);
    focusTerminal();

    terminal.onData((data) => {
      if (!socketReady()) return;
      if (!opened.value) sendOpen();
      const input = data.replace(/\n/g, '\r');
      sendPayload({ type: 'input', input, cols: terminal?.cols || 120, rows: terminal?.rows || 32 });
    });
  }

  function handlePayload(raw: string) {
    let payload: { type?: string; output?: string; message?: string };
    try {
      payload = JSON.parse(raw);
    } catch {
      terminal?.write(raw);
      return;
    }

    if (payload.type === 'output') {
      queueOutput(payload.output || '');
      return;
    }

    if (payload.type === 'closed') {
      closed.value = true;
      opened.value = false;
      connected.value = false;
      statusText.value = payload.message || '终端已关闭';
      if (payload.message) {
        writeNotice(payload.message);
      }
      if (!manualClosing) {
        writeNotice('终端进程已结束，请点击“重连”重新打开。');
      }
    }
  }

  function closeSocket() {
    if (socket) {
      socket.onclose = null;
      socket.close();
    }
    socket = null;
  }

  function stopReconnectTimer() {
    if (reconnectTimer) {
      window.clearTimeout(reconnectTimer);
      reconnectTimer = 0;
    }
  }

  async function connect(options: { recreateSession?: boolean; silent?: boolean } = {}) {
    if (!active) return;
    if (reconnecting) return;
    reconnecting = true;
    try {
      stopReconnectTimer();

      if (options.recreateSession || !sessionId.value) {
        statusText.value = '正在创建终端会话';
        resetTerminal();
        const ok = await refreshSession();
        if (!ok) {
          statusText.value = '创建终端会话失败';
          if (!options.silent) message.error('创建远程终端失败');
          return;
        }
      }

      if (!sessionId.value) {
        message.error('缺少终端会话ID');
        return;
      }

      manualClosing = false;
      closeSocket();
      closed.value = false;
      connected.value = false;
      opened.value = false;
      statusText.value = '连接中';

      const ws = new WebSocket(buildWsUrl());
      socket = ws;

      ws.onopen = () => {
        if (socket !== ws) return;
        connected.value = true;
        statusText.value = '正在打开终端';
        nextTick(() => {
          fitTerminal(false);
          sendOpen();
          focusTerminal();
        });
      };

      ws.onmessage = (event) => {
        if (socket !== ws) return;
        handlePayload(String(event.data || ''));
      };

      ws.onclose = () => {
        if (socket !== ws) return;
        socket = null;
        connected.value = false;
        opened.value = false;
        if (!closed.value && !manualClosing) {
          statusText.value = '连接已断开，正在重连';
          writeNotice('终端连接已断开，正在重新创建会话');
          reconnectTimer = window.setTimeout(() => {
            if (!active) return;
            connect({ recreateSession: true, silent: true });
          }, 800);
        }
      };

      ws.onerror = () => {
        if (socket !== ws) return;
        statusText.value = '连接失败，正在重连';
        writeNotice('终端连接失败，正在重新创建会话');
      };
    } finally {
      reconnecting = false;
    }
  }

  async function reconnect() {
    focusTerminal();
    await connect({ recreateSession: true });
    focusTerminal();
  }

  function disconnectBrowserSocket() {
    stopReconnectTimer();
    manualClosing = true;
    closeSocket();
    connected.value = false;
    opened.value = false;
    statusText.value = '已暂停';
  }

  watch(
    () => route.query.sessionId,
    (value) => {
      const nextSessionId = String(value || '');
      if (nextSessionId && nextSessionId !== sessionId.value) {
        sessionId.value = nextSessionId;
        connect({ recreateSession: false, silent: true });
      }
    }
  );

  watch(
    () => route.query.deviceId,
    (value, oldValue) => {
      if (value === oldValue) return;
      sessionId.value = '';
      connect({ recreateSession: true, silent: true });
    }
  );

  onMounted(async () => {
    await nextTick();
    initTerminal();
    focusTerminal();
    active = true;
    connect({ recreateSession: true, silent: true });

    resizeObserver = new ResizeObserver(() => {
      fitTerminal();
    });

    if (terminalRef.value) {
      resizeObserver.observe(terminalRef.value);
    }
  });

  onActivated(() => {
    active = true;
    manualClosing = false;
    nextTick(() => {
      fitTerminal(false);
      focusTerminal();
    });
    if (!socketReady()) {
      connect({ recreateSession: !sessionId.value, silent: true });
    }
  });

  onDeactivated(() => {
    active = false;
    disconnectBrowserSocket();
  });

  onBeforeUnmount(() => {
    active = false;
    manualClosing = true;
    sendPayload({ type: 'close' });
    stopReconnectTimer();
    if (outputFlushRaf) window.cancelAnimationFrame(outputFlushRaf);
    resizeObserver?.disconnect();
    closeSocket();
    terminal?.dispose();
  });
</script>

<style scoped lang="less">
  .terminal-page {
    height: calc(100vh - 96px);
    height: calc(100dvh - 96px);
    min-height: 320px;
    width: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: #050607;
    color: #f5f5f5;
  }

  .terminal-toolbar {
    flex: 0 0 44px;
    box-sizing: border-box;
    padding: 6px 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
    background: #171c22;
  }

  .terminal-title {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 14px;
    line-height: 1.2;
  }

  .terminal-state {
    color: rgba(245, 245, 245, 0.62);
    font-size: 12px;
  }

  .terminal-state.is-online {
    color: #6ee7a8;
  }

  .terminal-state.is-offline {
    color: #ff8a8a;
  }

  .terminal-actions {
    flex: 0 0 auto;
  }

  .terminal-action {
    min-width: 48px;
  }

  :deep(.terminal-action-secondary) {
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

  :deep(.terminal-action-primary) {
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

  .terminal-container {
    width: 100%;
    flex: 1;
    min-height: 0;
    box-sizing: border-box;
    padding: 8px;
    overflow: hidden;
  }

  :deep(.xterm) {
    height: 100%;
  }

  :deep(.xterm-viewport) {
    overflow-y: auto;
  }

  @media (max-width: 768px) {
    .terminal-toolbar {
      gap: 8px;
      padding: 6px 8px;
    }

    .terminal-title {
      font-size: 12px;
    }

    .terminal-container {
      padding: 4px;
    }
  }
</style>
