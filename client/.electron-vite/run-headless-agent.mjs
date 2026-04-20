import { existsSync } from 'node:fs'
import { join } from 'node:path'
import { spawn } from 'node:child_process'

const fileName = process.platform === 'win32' ? 'auroraops-agent.exe' : 'auroraops-agent'
const platformDir = process.platform === 'win32' ? 'win32' : process.platform
const archDir = process.arch === 'x64' ? 'x64' : process.arch
const binaryPath = join(process.cwd(), 'go-client', 'bin', platformDir, archDir, fileName)

if (!existsSync(binaryPath)) {
  console.error(`agent binary not found: ${binaryPath}`)
  console.error('run `npm run dev:agent` first')
  process.exit(1)
}

const args = ['--headless', ...process.argv.slice(2)]
const child = spawn(binaryPath, args, {
  stdio: 'inherit',
})

child.on('exit', (code) => {
  process.exit(code ?? 0)
})
