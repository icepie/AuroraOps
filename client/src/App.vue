<template>
  <div class="shell">
    <section class="hero">
      <div class="hero__content">
        <p class="eyebrow">AuroraOps Desktop Client</p>
        <h1>配置服务端并自动注册设备</h1>
        <p class="hero__desc">
          客户端不需要登录。填写服务端地址和设备名称后，系统会检查服务端连通性，并自动把当前设备注册到 AuroraOps。
        </p>
      </div>
      <div class="hero__actions">
        <button class="ghost-button" @click="refreshRuntimeInfo">刷新设备信息</button>
        <button class="primary-button" :disabled="submitting" @click="submitRegistration">
          {{ submitting ? '连接中...' : '检查连接并注册' }}
        </button>
      </div>
    </section>

    <section class="panel-grid">
      <article class="panel">
        <header class="panel__header">
          <div>
            <p class="panel__label">接入配置</p>
            <h2>服务端与设备</h2>
          </div>
        </header>

        <div class="form-grid">
          <label class="field">
            <span>服务端地址</span>
            <input
              v-model.trim="form.serverUrl"
              type="text"
              placeholder="例如 127.0.0.1:9500 或 http://192.168.1.10:9500"
            />
          </label>
          <label class="field">
            <span>设备名称</span>
            <input v-model.trim="form.deviceName" type="text" placeholder="请输入设备名称" />
          </label>
        </div>

        <div class="kv-list">
          <div class="kv-item">
            <span>连接状态</span>
            <strong :class="serverStatusClass">{{ serverStatusText }}</strong>
          </div>
          <div class="kv-item">
            <span>心跳状态</span>
            <strong :class="heartbeatStatusClass">
              {{ heartbeatStatus }}
            </strong>
          </div>
          <div class="kv-item">
            <span>已保存地址</span>
            <strong>{{ savedServerUrl || '-' }}</strong>
          </div>
          <div v-if="heartbeatAt" class="kv-item">
            <span>最近心跳</span>
            <strong>{{ heartbeatAt }}</strong>
          </div>
          <div v-if="serverStatusDetail" class="notice">
            {{ serverStatusDetail }}
          </div>
        </div>
      </article>

      <article class="panel">
        <header class="panel__header">
          <div>
            <p class="panel__label">桌面环境</p>
            <h2>运行时信息</h2>
          </div>
        </header>

        <div class="kv-list">
          <div class="kv-item">
            <span>主机名</span>
            <strong>{{ runtimeInfo.hostname || '-' }}</strong>
          </div>
          <div class="kv-item">
            <span>用户</span>
            <strong>{{ runtimeInfo.username || '-' }}</strong>
          </div>
          <div class="kv-item">
            <span>平台</span>
            <strong>{{ runtimeInfo.platform || '-' }} / {{ runtimeInfo.arch || '-' }}</strong>
          </div>
          <div class="kv-item">
            <span>主 IP</span>
            <strong>{{ runtimeInfo.primaryIp || '-' }}</strong>
          </div>
          <div class="kv-item">
            <span>Electron</span>
            <strong>{{ runtimeInfo.electronVersion || '-' }}</strong>
          </div>
          <div class="kv-item">
            <span>Node.js</span>
            <strong>{{ runtimeInfo.nodeVersion || '-' }}</strong>
          </div>
        </div>
      </article>
    </section>

    <section v-if="registrationInfo.id" class="roadmap">
      <p class="panel__label">注册结果</p>
      <div class="roadmap__items roadmap__items--result">
        <div class="roadmap__item">设备ID：{{ registrationInfo.id }}</div>
        <div class="roadmap__item">设备名称：{{ registrationInfo.name }}</div>
        <div class="roadmap__item">
          {{ registrationInfo.created ? '本次为新注册' : '本次为更新现有设备' }}
        </div>
        <div class="roadmap__item">主机名：{{ registrationInfo.hostname }}</div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
  import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue';

  const DEFAULT_SERVER_URL = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:9500';
  const STORAGE_KEY = 'auroraops-client-config';
  const HEARTBEAT_INTERVAL = 30_000;

  const runtimeInfo = reactive({
    appVersion: '',
    electronVersion: '',
    chromeVersion: '',
    nodeVersion: '',
    platform: '',
    arch: '',
    hostname: '',
    username: '',
    primaryIp: '',
  });

  const form = reactive({
    serverUrl: DEFAULT_SERVER_URL,
    deviceName: '',
  });

  const submitting = ref(false);
  const serverStatusText = ref('未检测');
  const serverStatusDetail = ref('');
  const savedServerUrl = ref('');
  const registrationInfo = reactive({
    id: 0,
    name: '',
    hostname: '',
    ip: '',
    created: false,
    createdAt: '',
  });
  const heartbeatStatus = ref('未启动');
  const heartbeatAt = ref('');
  let heartbeatTimer: number | null = null;

  const serverStatusClass = computed(() => {
    if (serverStatusText.value === '连接成功') {
      return 'status-success';
    }
    if (serverStatusText.value === '连接失败') {
      return 'status-error';
    }
    return 'status-pending';
  });

  const heartbeatStatusClass = computed(() => {
    if (heartbeatStatus.value === '正常') {
      return 'status-success';
    }
    if (heartbeatStatus.value === '异常') {
      return 'status-error';
    }
    return 'status-pending';
  });

  async function refreshRuntimeInfo() {
    const info = await window.auroraClient.getRuntimeInfo();
    Object.assign(runtimeInfo, info);

    if (!form.deviceName) {
      form.deviceName = info.hostname || '';
    }
  }

  function normalizeServerUrl(value: string) {
    const raw = value.trim();
    if (!raw) {
      return '';
    }

    const url = raw.startsWith('http://') || raw.startsWith('https://') ? raw : `http://${raw}`;
    return url.replace(/\/+$/, '');
  }

  function saveConfig() {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        serverUrl: form.serverUrl,
        deviceName: form.deviceName,
      })
    );
    savedServerUrl.value = form.serverUrl;
  }

  function loadConfig() {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return;
    }

    try {
      const data = JSON.parse(raw) as { serverUrl?: string; deviceName?: string };
      if (data.serverUrl) {
        form.serverUrl = data.serverUrl;
        savedServerUrl.value = data.serverUrl;
      }
      if (data.deviceName) {
        form.deviceName = data.deviceName;
      }
    } catch {
      localStorage.removeItem(STORAGE_KEY);
    }
  }

  async function requestJson(url: string, payload: Record<string, any>) {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    const rawText = await response.text();
    let result: any = null;

    try {
      result = rawText ? JSON.parse(rawText) : null;
    } catch {
      throw new Error(rawText || `请求失败: HTTP ${response.status}`);
    }

    if (!response.ok || !result || result.code !== 0) {
      throw new Error(result?.message || `请求失败: HTTP ${response.status}`);
    }

    return result.data;
  }

  function stopHeartbeat() {
    if (heartbeatTimer !== null) {
      window.clearInterval(heartbeatTimer);
      heartbeatTimer = null;
    }
  }

  async function sendHeartbeat() {
    if (!registrationInfo.id) {
      return;
    }

    try {
      const data = await requestJson(`${form.serverUrl}/admin/client/heartbeat`, {
        id: registrationInfo.id,
        hostname: registrationInfo.hostname || runtimeInfo.hostname,
        ip: runtimeInfo.primaryIp,
        osName: `${runtimeInfo.platform}/${runtimeInfo.arch}`,
      });

      heartbeatStatus.value = '正常';
      heartbeatAt.value = data.aliveAt;
      serverStatusText.value = '连接成功';
    } catch (error) {
      heartbeatStatus.value = '异常';
      serverStatusText.value = '连接失败';
      serverStatusDetail.value = error instanceof Error ? error.message : '心跳失败';
    }
  }

  function startHeartbeat() {
    stopHeartbeat();
    heartbeatStatus.value = '启动中';
    sendHeartbeat();
    heartbeatTimer = window.setInterval(() => {
      sendHeartbeat();
    }, HEARTBEAT_INTERVAL);
  }

  async function submitRegistration() {
    const serverUrl = normalizeServerUrl(form.serverUrl);
    const deviceName = form.deviceName.trim();

    if (!serverUrl) {
      serverStatusText.value = '连接失败';
      serverStatusDetail.value = '请输入服务端地址';
      return;
    }

    if (!deviceName) {
      serverStatusText.value = '连接失败';
      serverStatusDetail.value = '请输入设备名称';
      return;
    }

    form.serverUrl = serverUrl;
    submitting.value = true;
    serverStatusDetail.value = '';

    try {
      const data = await requestJson(`${serverUrl}/admin/client/register`, {
        name: deviceName,
        hostname: runtimeInfo.hostname,
        ip: runtimeInfo.primaryIp,
        deviceType: 'physical',
        osName: `${runtimeInfo.platform}/${runtimeInfo.arch}`,
        location: 'AuroraOps Client',
      });

      serverStatusText.value = '连接成功';
      serverStatusDetail.value = data.created
        ? `设备已自动注册，设备ID ${data.id}`
        : `设备已存在，已更新基础信息，设备ID ${data.id}`;

      Object.assign(registrationInfo, data);
      saveConfig();
      startHeartbeat();
    } catch (error) {
      stopHeartbeat();
      heartbeatStatus.value = '未启动';
      serverStatusText.value = '连接失败';
      serverStatusDetail.value = error instanceof Error ? error.message : '未知错误';
    } finally {
      submitting.value = false;
    }
  }

  onMounted(async () => {
    loadConfig();
    await refreshRuntimeInfo();
  });

  onBeforeUnmount(() => {
    stopHeartbeat();
  });
</script>
