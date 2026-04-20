<template>
  <div class="terminal-page">
    <div ref="terminalRef" class="terminal-container" />
  </div>
</template>

<script setup lang="ts">
  import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
  import { useRoute } from 'vue-router';
  import { useMessage } from 'naive-ui';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { ACCESS_TOKEN } from '@/store/mutation-types';
  import { storage } from '@/utils/Storage';
  import 'xterm/css/xterm.css';

  const route = useRoute();
  const message = useMessage();

  const terminalRef = ref<HTMLElement | null>(null);
  const closed = ref(false);

  const sessionId = String(route.query.sessionId || '');

  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let socket: WebSocket | null = null;
  let resizeObserver: ResizeObserver | null = null;

  function buildWsUrl() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const token = storage.get(ACCESS_TOKEN, '') || '';
    return `${protocol}//${window.location.host}/admin/opsDevice/terminal/ws?sessionId=${encodeURIComponent(sessionId)}&authorization=${encodeURIComponent(token)}`;
  }

  function writeNotice(text: string) {
    if (!terminal) {
      return;
    }
    terminal.writeln('');
    terminal.writeln(text);
  }

  function sendResize() {
    if (!socket || socket.readyState !== WebSocket.OPEN || !terminal) {
      return;
    }
    socket.send(
      JSON.stringify({
        type: 'resize',
        cols: terminal.cols,
        rows: terminal.rows,
      }),
    );
  }

  function fitTerminal() {
    if (!fitAddon || !terminalRef.value) {
      return;
    }
    fitAddon.fit();
    sendResize();
  }

  function initTerminal() {
    terminal = new Terminal({
      convertEol: true,
      cursorBlink: true,
      cursorStyle: 'block',
      fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Mono", monospace',
      fontSize: 14,
      lineHeight: 1.2,
      letterSpacing: 0.2,
      scrollback: 5000,
      theme: {
        background: '#000000',
        foreground: '#f5f5f5',
        cursor: '#f5f5f5',
        cursorAccent: '#000000',
        selectionBackground: 'rgba(10, 148, 242, 0.35)',
        black: '#000000',
        red: '#ef4b4c',
        green: '#53b449',
        yellow: '#fbb142',
        blue: '#0a94f2',
        magenta: '#975fe4',
        cyan: '#14b8a6',
        white: '#d4d4d4',
        brightBlack: '#666666',
        brightRed: '#ff6b6b',
        brightGreen: '#7ed957',
        brightYellow: '#ffd166',
        brightBlue: '#4db8ff',
        brightMagenta: '#b388ff',
        brightCyan: '#2dd4bf',
        brightWhite: '#ffffff',
      },
    });

    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(terminalRef.value!);
    fitTerminal();
    terminal.focus();

    terminal.onData((data) => {
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        return;
      }
      socket.send(JSON.stringify({ type: 'input', input: data }));
    });
  }

  function handlePayload(raw: string) {
    let payload: { type?: string; output?: string; message?: string };
    try {
      payload = JSON.parse(raw);
    } catch (error) {
      terminal?.write(raw);
      return;
    }

    if (payload.type === 'output') {
      terminal?.write(payload.output || '');
      return;
    }

    if (payload.type === 'closed') {
      closed.value = true;
      connected.value = false;
      if (payload.message) {
        writeNotice(payload.message);
      }
    }
  }

  function connect() {
    if (!sessionId) {
      message.error('缺少终端会话ID');
      return;
    }

    closed.value = false;

    socket = new WebSocket(buildWsUrl());

    socket.onopen = () => {
      fitTerminal();
    };

    socket.onmessage = (event) => {
      handlePayload(String(event.data || ''));
    };

    socket.onclose = () => {
      if (!closed.value) {
        writeNotice('终端连接已断开');
      }
    };

    socket.onerror = () => {
      writeNotice('终端连接失败');
    };
  }

  onMounted(async () => {
    await nextTick();
    initTerminal();
    connect();

    resizeObserver = new ResizeObserver(() => {
      fitTerminal();
    });

    if (terminalRef.value) {
      resizeObserver.observe(terminalRef.value);
    }
  });

  onBeforeUnmount(() => {
    resizeObserver?.disconnect();
    socket?.close();
    terminal?.dispose();
  });
</script>

<style scoped lang="less">
  .terminal-page {
    min-height: 100vh;
    background: #000000;
  }

  .terminal-container {
    width: 100vw;
    height: 100vh;
    padding: 8px;
  }

  :deep(.xterm) {
    height: 100%;
  }

  @media (max-width: 768px) {
    .terminal-container {
      padding: 4px;
    }
  }
</style>
