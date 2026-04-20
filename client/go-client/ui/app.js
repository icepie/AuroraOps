const els = {
  serverHost: document.getElementById('serverHost'),
  deviceName: document.getElementById('deviceName'),
  stateText: document.getElementById('stateText'),
  deviceIdText: document.getElementById('deviceIdText'),
  tcpText: document.getElementById('tcpText'),
  messageText: document.getElementById('messageText'),
  previewBtn: document.getElementById('previewBtn'),
  assetPreview: document.getElementById('assetPreview'),
  diagnosticPreview: document.getElementById('diagnosticPreview'),
  refreshBtn: document.getElementById('refreshBtn'),
  stopBtn: document.getElementById('stopBtn'),
  form: document.getElementById('configForm'),
}

function render(payload) {
  const state = payload.status || {}
  const config = payload.config || {}
  els.serverHost.value = config.serverHost || ''
  els.deviceName.value = config.deviceName || ''
  els.stateText.textContent = state.state || 'idle'
  els.deviceIdText.textContent = '设备ID: ' + (state.deviceId || '-')
  els.tcpText.textContent = 'TCP: ' + (state.tcpAddress || '-')
  els.messageText.textContent = state.message || payload.message || ''
}

function renderAssets(localAssets, serverAssets) {
  if ((!localAssets || localAssets.length === 0) && (!serverAssets || serverAssets.length === 0)) {
    els.assetPreview.className = 'asset-preview empty'
    els.assetPreview.textContent = '当前没有可展示的资产'
    return
  }
  const serverMap = new Map((serverAssets || []).map((item) => [`${item.assetType}::${item.uniqueKey}`, item]))
  const localMap = new Map((localAssets || []).map((item) => [`${item.assetType}::${item.uniqueKey}`, item]))
  const cards = []

  for (const item of localAssets || []) {
    const key = `${item.assetType}::${item.uniqueKey}`
    const serverItem = serverMap.get(key)
    let action = 'create'
    if (serverItem) {
      action = serverItem.syncHash === item.syncHash ? 'keep' : 'update'
    }
    const meta = [item.brand, item.model, item.serialNo, item.specification]
      .filter(Boolean)
      .join(' | ')
    cards.push(`
      <article class="asset-card">
        <div class="asset-row">
          <div class="asset-type">${item.assetType}</div>
          <div class="asset-action ${action}">${action}</div>
        </div>
        <strong>${item.assetName || '-'}</strong>
        <p>${meta || '-'}</p>
        <span>${item.uniqueKey || '-'}</span>
      </article>
    `)
  }

  for (const item of serverAssets || []) {
    const key = `${item.assetType}::${item.uniqueKey}`
    if (localMap.has(key)) continue
    const meta = [item.brand, item.model, item.serialNo, item.specification, item.source]
      .filter(Boolean)
      .join(' | ')
    cards.push(`
      <article class="asset-card">
        <div class="asset-row">
          <div class="asset-type">${item.assetType}</div>
          <div class="asset-action server-only">server-only</div>
        </div>
        <strong>${item.assetName || '-'}</strong>
        <p>${meta || '-'}</p>
        <span>${item.uniqueKey || '-'}</span>
      </article>
    `)
  }

  els.assetPreview.className = 'asset-preview'
  els.assetPreview.innerHTML = cards.join('')
}

function renderDiagnostics(diagnostics) {
  if (!diagnostics || diagnostics.length === 0) {
    els.diagnosticPreview.className = 'diagnostic-preview empty'
    els.diagnosticPreview.textContent = '当前没有可展示的诊断信息'
    return
  }

  const cards = diagnostics.map((item) => `
    <article class="diagnostic-card ${item.ok ? 'ok' : 'error'}">
      <div class="asset-row">
        <div class="asset-type">${item.name || '-'}</div>
        <div class="diagnostic-state ${item.ok ? 'ok' : 'error'}">${item.ok ? 'ok' : 'error'}</div>
      </div>
      <strong>数量: ${item.count ?? 0}</strong>
      <p>${item.message || (item.ok ? '采集成功' : '未返回详细错误')}</p>
    </article>
  `)

  els.diagnosticPreview.className = 'diagnostic-preview'
  els.diagnosticPreview.innerHTML = cards.join('')
}

async function request(url, options) {
  const response = await fetch(url, options)
  const data = await response.json()
  if (!response.ok || data.ok === false) {
    throw new Error(data.message || 'request failed')
  }
  return data
}

async function refresh() {
  const data = await request('/api/status')
  render(data)
}

els.form.addEventListener('submit', async (event) => {
  event.preventDefault()
  try {
    await request('/api/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        serverHost: els.serverHost.value,
        deviceName: els.deviceName.value,
      }),
    })
    const data = await request('/api/start', { method: 'POST' })
    render(data)
  } catch (error) {
    els.messageText.textContent = error.message
  }
})

els.previewBtn.addEventListener('click', async () => {
  try {
    await request('/api/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        serverHost: els.serverHost.value || '127.0.0.1:8000',
        deviceName: els.deviceName.value || 'preview-node',
      }),
    })
    const data = await request('/api/assets/preview')
    renderAssets(data.assets || [], data.serverAssets || [])
    renderDiagnostics(data.diagnostics || [])
    els.messageText.textContent = `本地 ${data.assets?.length || 0} 条 / 服务端 ${data.serverAssets?.length || 0} 条`
  } catch (error) {
    els.messageText.textContent = error.message
  }
})

els.refreshBtn.addEventListener('click', async () => {
  try {
    await refresh()
  } catch (error) {
    els.messageText.textContent = error.message
  }
})

els.stopBtn.addEventListener('click', async () => {
  try {
    const data = await request('/api/stop', { method: 'POST' })
    render(data)
  } catch (error) {
    els.messageText.textContent = error.message
  }
})

refresh().catch((error) => {
  els.messageText.textContent = error.message
})

setInterval(refresh, 5000)
