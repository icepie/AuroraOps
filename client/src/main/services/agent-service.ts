import { app } from 'electron'
import { spawn, type ChildProcessWithoutNullStreams } from 'child_process'
import { existsSync } from 'fs'
import { join, resolve } from 'path'

export interface AgentConfig {
  serverHost: string
  deviceName: string
  httpBase: string
  deviceId?: number
  token?: string
  tcpAddress?: string
  hostname?: string
}

export interface AgentStatus {
  state: string
  deviceId?: number
  tcpAddress?: string
  message?: string
  updatedAt: number
}

type ControlResponse = {
  ok: boolean
  status: AgentStatus
  config: AgentConfig
  message?: string
}

class AgentService {
  private process: ChildProcessWithoutNullStreams | null = null
  private port = 18765
  private lastErrorOutput = ''

  getBaseUrl() {
    return `http://127.0.0.1:${this.port}`
  }

  getUiUrl() {
    return `${this.getBaseUrl()}/`
  }

  getConfigPath() {
    return join(app.getPath('userData'), 'agent-config.json')
  }

  private getAgentBinaryCandidates() {
    const fileName = process.platform === 'win32' ? 'auroraops-agent.exe' : 'auroraops-agent'
    const platformDir = process.platform === 'win32' ? 'win32' : process.platform
    const archDir = process.arch === 'x64' ? 'x64' : process.arch

    if (app.isPackaged) {
      if (process.platform === 'linux') {
        return [
          '/opt/auroraops/auroraops-agent',
          join(process.resourcesPath, 'go-agent', platformDir, archDir, fileName),
          join(process.resourcesPath, '..', 'go-agent', platformDir, archDir, fileName),
          join(process.resourcesPath, 'go-agent', fileName),
          join(process.resourcesPath, 'go-agent', platformDir, fileName),
          join(process.resourcesPath, '..', 'go-agent', fileName),
          join(process.resourcesPath, '..', 'go-agent', platformDir, fileName),
        ]
      }

      return [
        join(process.resourcesPath, 'go-agent', platformDir, archDir, fileName),
        join(process.resourcesPath, '..', 'go-agent', platformDir, archDir, fileName),
        join(process.resourcesPath, 'go-agent', fileName),
        join(process.resourcesPath, 'go-agent', platformDir, fileName),
        join(process.resourcesPath, '..', 'go-agent', fileName),
        join(process.resourcesPath, '..', 'go-agent', platformDir, fileName),
      ]
    }

    return [
      join(process.cwd(), 'go-client', 'bin', platformDir, archDir, fileName),
      join(process.cwd(), 'go-client', 'bin', platformDir, fileName),
      resolve(app.getAppPath(), '..', '..', '..', 'go-client', 'bin', platformDir, archDir, fileName),
      resolve(app.getAppPath(), '..', '..', '..', 'go-client', 'bin', platformDir, fileName),
      resolve(app.getAppPath(), '..', '..', '..', '..', 'go-client', 'bin', platformDir, archDir, fileName),
      resolve(app.getAppPath(), '..', '..', '..', '..', 'go-client', 'bin', platformDir, fileName),
    ]
  }

  getAgentBinaryPath() {
    const candidates = this.getAgentBinaryCandidates()
    const matched = candidates.find((candidate) => existsSync(candidate))
    return matched ?? candidates[0]
  }

  async ensureServerStarted() {
    if (await this.isAlive()) {
      console.log('[agent-service] reusing existing agent server', this.getBaseUrl())
      return
    }

    const agentPath = this.getAgentBinaryPath()
    if (!existsSync(agentPath)) {
      const candidates = this.getAgentBinaryCandidates()
      throw new Error(
        `agent binary not found. checked: ${candidates.join(', ')}. Run \`npm run build:agent\` in client first.`,
      )
    }

    this.process = spawn(
      agentPath,
      ['--config', this.getConfigPath(), '--port', String(this.port)],
      {
        stdio: ['ignore', 'pipe', 'pipe'],
      },
    )

    console.log('[agent-service] spawned agent', {
      agentPath,
      port: this.port,
      packaged: app.isPackaged,
      resourcesPath: process.resourcesPath,
    })
    this.lastErrorOutput = ''
    this.process.stdout.on('data', (chunk) => {
      const line = chunk.toString().trim()
      if (line) {
        console.log(`[agent-service] ${line}`)
      }
    })
    this.process.stderr.on('data', (chunk) => {
      this.lastErrorOutput = `${this.lastErrorOutput}\n${chunk.toString()}`.trim()
    })

    this.process.on('exit', (code, signal) => {
      console.log('[agent-service] agent exited', { code, signal })
      this.process = null
    })

    await this.waitUntilAlive()
    console.log('[agent-service] agent http server ready', this.getBaseUrl())
  }

  async saveConfig(input: { serverHost: string; deviceName: string }) {
    await this.ensureServerStarted()
    const data = await this.request('/api/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
    })
    return data.config
  }

  async start() {
    await this.ensureServerStarted()
    const data = await this.request('/api/start', {
      method: 'POST',
    })
    return data.status
  }

  async stop() {
    await this.ensureServerStarted()
    const data = await this.request('/api/stop', {
      method: 'POST',
    })
    return data.status
  }

  async getStatus() {
    await this.ensureServerStarted()
    const data = await this.request('/api/status')
    return data.status
  }

  private async request(path: string, init?: RequestInit) {
    const response = await fetch(`${this.getBaseUrl()}${path}`, init)
    const data = (await response.json()) as ControlResponse
    if (!response.ok || !data.ok) {
      throw new Error(data.message || 'agent request failed')
    }
    return data
  }

  private async isAlive() {
    try {
      const response = await fetch(`${this.getBaseUrl()}/api/status`)
      return response.ok
    } catch {
      return false
    }
  }

  private async waitUntilAlive() {
    const deadline = Date.now() + 8000
    while (Date.now() < deadline) {
      if (await this.isAlive()) {
        return
      }
      await new Promise((resolve) => setTimeout(resolve, 250))
    }
    const detail = this.lastErrorOutput
      ? `: ${this.lastErrorOutput.split('\n').slice(-5).join(' | ')}`
      : ''
    throw new Error(`agent http server did not start in time${detail}`)
  }
}

export const agentService = new AgentService()
