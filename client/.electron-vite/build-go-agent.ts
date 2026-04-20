import { mkdirSync } from 'fs'
import { join } from 'path'
import { spawn } from 'child_process'
import minimist from 'minimist'

type PlatformTarget = {
  os: string
  arch: string
  binDir: string
  output: string
}

const root = join(__dirname, '..')
const goRoot = join(root, 'go-client')
const binRoot = join(goRoot, 'bin')
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
  mkdirSync(join(target.output, '..'), { recursive: true })
  await runGoBuild(target)
}

function runGoBuild(target: PlatformTarget) {
  return new Promise<void>((resolve, reject) => {
    const child = spawn(
      'go',
      ['build', '-o', target.output, './...'],
      {
        cwd: goRoot,
        env: {
          ...process.env,
          GOOS: target.os,
          GOARCH: target.arch,
          CGO_ENABLED: '0',
        },
        stdio: 'inherit',
      },
    )

    child.on('exit', (code) => {
      if (code === 0) {
        resolve()
        return
      }
      reject(new Error(`go build failed for ${target.os}/${target.arch}`))
    })
  })
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
