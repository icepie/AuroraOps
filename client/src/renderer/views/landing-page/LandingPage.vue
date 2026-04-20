<template>
  <section class="page">
    <div class="hero">
      <div>
        <p class="eyebrow">AuroraOps Client</p>
        <h1>设备绑定与资产采集</h1>
        <p class="summary">
          输入设备名称和服务端地址后，客户端会自动注册设备、建立长连接，
          并持续同步 CPU、内存、磁盘等硬件资产。
        </p>
      </div>
      <div class="status-card" :data-state="status.state">
        <span class="status-label">当前状态</span>
        <strong>{{ statusText }}</strong>
        <span v-if="status.deviceId">设备ID: {{ status.deviceId }}</span>
        <span v-if="status.tcpAddress">TCP: {{ status.tcpAddress }}</span>
        <span v-if="status.message">{{ status.message }}</span>
      </div>
    </div>

    <div class="content">
      <form class="panel" @submit.prevent="saveAndStart">
        <h2>连接配置</h2>
        <label>
          <span>设备名称</span>
          <input v-model.trim="form.deviceName" placeholder="例如：北京机房-01" />
        </label>
        <label>
          <span>服务端地址</span>
          <input
            v-model.trim="form.serverHost"
            placeholder="例如：192.168.1.20:8000 或 http://192.168.1.20:8000"
          />
        </label>

        <div class="actions">
          <button type="submit">保存并连接</button>
          <button type="button" class="ghost" @click="refreshStatus">刷新状态</button>
          <button type="button" class="danger" @click="stopAgent">停止客户端</button>
        </div>
      </form>

      <div class="panel info">
        <h2>运行信息</h2>
        <dl>
          <div>
            <dt>平台</dt>
            <dd>{{ systemInfo.platform }} / {{ systemInfo.arch }}</dd>
          </div>
          <div>
            <dt>内核版本</dt>
            <dd>{{ systemInfo.release }}</dd>
          </div>
          <div>
            <dt>最后刷新</dt>
            <dd>{{ lastUpdated }}</dd>
          </div>
        </dl>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'

const { ipcRendererChannel, systemInfo } = window

const form = reactive({
  deviceName: '',
  serverHost: '',
})

const status = ref<AgentStatus>({
  state: 'idle',
  updatedAt: Date.now(),
})

const statusText = computed(() => {
  const map: Record<AgentStatus['state'], string> = {
    idle: '未启动',
    starting: '启动中',
    registered: '已注册',
    connected: '已连接',
    reconnecting: '重连中',
    stopped: '已停止',
    error: '异常',
  }
  return map[status.value.state]
})

const lastUpdated = computed(() =>
  new Date(status.value.updatedAt).toLocaleString(),
)

async function saveAndStart() {
  await ipcRendererChannel.SaveAgentConfig.invoke({
    serverHost: form.serverHost,
    deviceName: form.deviceName,
  })
  status.value = await ipcRendererChannel.StartAgent.invoke()
}

async function refreshStatus() {
  status.value = await ipcRendererChannel.GetAgentStatus.invoke()
}

async function stopAgent() {
  status.value = await ipcRendererChannel.StopAgent.invoke()
}

onMounted(async () => {
  const [nextStatus, nextConfig] = await Promise.all([
    ipcRendererChannel.GetAgentStatus.invoke(),
    ipcRendererChannel.GetAgentConfig.invoke(),
  ])
  status.value = nextStatus
  form.deviceName = nextConfig.deviceName || ''
  form.serverHost = nextConfig.serverHost || ''
})
</script>

<style scoped lang="scss">
.page {
  min-height: calc(100vh - 30px);
  padding: 48px;
  background:
    radial-gradient(circle at top left, rgba(13, 148, 136, 0.16), transparent 34%),
    radial-gradient(circle at bottom right, rgba(251, 146, 60, 0.18), transparent 30%),
    linear-gradient(135deg, #f8fafc, #eef2ff 52%, #fdf2f8);
  color: #0f172a;
}

.hero {
  display: grid;
  grid-template-columns: 1.7fr 1fr;
  gap: 24px;
  margin-bottom: 28px;
}

.eyebrow {
  margin-bottom: 12px;
  color: #0f766e;
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

h1 {
  margin: 0 0 14px;
  font-size: 42px;
  line-height: 1.04;
}

.summary {
  max-width: 760px;
  color: #334155;
  font-size: 16px;
  line-height: 1.7;
}

.status-card,
.panel {
  border: 1px solid rgba(148, 163, 184, 0.22);
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.72);
  backdrop-filter: blur(20px);
  box-shadow: 0 20px 60px rgba(15, 23, 42, 0.08);
}

.status-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 24px;
}

.status-card strong {
  font-size: 28px;
}

.status-label {
  color: #475569;
  font-size: 13px;
}

.content {
  display: grid;
  grid-template-columns: 1.3fr 0.9fr;
  gap: 24px;
}

.panel {
  padding: 24px;
}

.panel h2 {
  margin: 0 0 18px;
  font-size: 22px;
}

label {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

label span,
dt {
  color: #475569;
  font-size: 13px;
  font-weight: 700;
}

input {
  border: 1px solid rgba(100, 116, 139, 0.24);
  border-radius: 14px;
  padding: 14px 16px;
  background: rgba(255, 255, 255, 0.92);
  color: #0f172a;
  font-size: 15px;
}

.actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

button {
  border: 0;
  border-radius: 999px;
  padding: 12px 18px;
  background: linear-gradient(135deg, #0f766e, #1d4ed8);
  color: #fff;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
}

button.ghost {
  background: #e2e8f0;
  color: #0f172a;
}

button.danger {
  background: linear-gradient(135deg, #dc2626, #f97316);
}

dl {
  display: grid;
  gap: 16px;
}

dl div {
  display: grid;
  gap: 6px;
}

dd {
  margin: 0;
  font-size: 15px;
}

@media (max-width: 980px) {
  .page {
    padding: 28px 18px 32px;
  }

  .hero,
  .content {
    grid-template-columns: 1fr;
  }

  h1 {
    font-size: 34px;
  }

  .actions {
    flex-direction: column;
  }
}
</style>
