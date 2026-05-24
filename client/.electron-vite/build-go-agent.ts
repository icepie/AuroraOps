import { mkdirSync } from 'fs'
import { join, resolve } from 'path'
import { spawn } from 'child_process'
import minimist from 'minimist'

type PlatformTarget = {
  os: string
  arch: string
  binDir: string
  output: string
}

const root = join(__dirname, '..')
const newClientRoot = resolve(root, '..', 'new-client')
const binRoot = join(root, 'go-client', 'bin')
const argv = minimist(process.argv.slice(2))
const targetArg = String(argv.target || '').trim().toLowerCase()
const archArg = String(argv.arch || '').trim().toLowerCase()

const targets: PlatformTarget[] = [
  {
    os: 'windows',
    arch: 'amd64',
    binDir: 'x64',
    output: join(binRoot, 'win32', 'x64', 'auroraops-agent.exe'),
  },
  {
    os: 'windows',
    arch: 'arm64',
    binDir: 'arm64',
    output: join(binRoot, 'win32', 'arm64', 'auroraops-agent.exe'),
  },
  {
    os: 'linux',
    arch: 'amd64',
    binDir: 'x64',
    output: join(binRoot, 'linux', 'x64', 'auroraops-agent'),
  },
  {
    os: 'linux',
    arch: 'arm64',
    binDir: 'arm64',
    output: join(binRoot, 'linux', 'arm64', 'auroraops-agent'),
  },
]

const filteredTargets = targets.filter((target) => {
  if (targetArg === 'dev' && target.os !== normalizeDevPlatform(process.platform)) {
    return false
  }
  if (archArg !== '' && target.binDir !== archArg && target.arch !== archArg) {
    return false
  }
  if (targetArg !== '' && targetArg !== 'dev' && target.os !== normalizeTargetPlatform(targetArg)) {
    return false
  }
  return true
})

async function buildTarget(target: PlatformTarget) {
  if (target.os !== normalizeDevPlatform(process.platform)) {
    throw new Error(`new-client Rust build only supports the host platform for now: requested ${target.os}, host ${process.platform}`)
  }
  mkdirSync(join(target.output, '..'), { recursive: true })
  await runCargoBuild(target)
}

function runCargoBuild(target: PlatformTarget) {
  return new Promise<void>((resolve, reject) => {
    const child = spawn('cargo', ['build', '--release', '--features', 'ffmpeg-system', '--bin', 'auroraops-agent'], {
      cwd: newClientRoot,
      env: process.env,
      stdio: 'inherit',
    })

    child.on('exit', (code) => {
      if (code === 0) {
        copyBuiltBinary(target).then(resolve, reject)
      } else {
        reject(new Error(`cargo build failed for ${target.os}/${target.arch}`))
      }
    })
  })
}

async function copyBuiltBinary(target: PlatformTarget) {
  const { copyFile, chmod } = await import('fs/promises')
  const builtName = process.platform === 'win32' ? 'auroraops-agent.exe' : 'auroraops-agent'
  const builtPath = join(newClientRoot, 'target', 'release', builtName)
  await copyFile(builtPath, target.output)
  if (target.os !== 'windows') {
    await chmod(target.output, 0o755)
  }
}

Promise.all(filteredTargets.map(buildTarget)).catch((error) => {
  console.error(error)
  process.exit(1)
})

function normalizeDevPlatform(platform: NodeJS.Platform) {
  if (platform === 'win32') {
    return 'windows'
  }
  if (platform === 'linux') {
    return 'linux'
  }
  return platform
}

function normalizeTargetPlatform(platform: string) {
  if (platform === 'win32') {
    return 'windows'
  }
  return platform
}
