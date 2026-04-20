// 这里定义了静态文件路径的位置
import { join } from 'path'
import config from '@config/index'
import { app } from 'electron'
import { agentService } from '../services/agent-service'

const env = app.isPackaged ? 'production' : 'development'

const filePath = {
  winURL: {
    development: agentService.getUiUrl(),
    production: agentService.getUiUrl(),
  },
  loadingURL: {
    development: `http://localhost:${process.env.PORT}/loader.html`,
    production: `file://${join(app.getAppPath(), 'dist', 'electron', 'renderer', 'loader.html')}`,
  },
  __static: {
    development: join(
      __dirname,
      '..',
      '..',
      '..',
      'src',
      'renderer',
      'public',
    ).replace(/\\/g, '\\\\'),
    production: join(app.getAppPath(), 'dist', 'electron', 'renderer').replace(
      /\\/g,
      '\\\\',
    ),
  },
  getPreloadFile(fileName: string) {
    const resolved = join(__dirname, `${fileName}.js`)
    console.log('[static-path] preload resolved', fileName, resolved)
    return resolved
  },
}

process.env.__static = filePath.__static[env]

process.env.__lib = getAppRootPath(config.DllFolder)
process.env.__updateFolder = getAppRootPath(config.HotUpdateFolder)

function getAppRootPath(path: string) {
  return env !== 'development'
    ? join(__dirname, '..', '..', '..', '..', path).replace(/\\/g, '\\\\')
    : join(__dirname, '..', '..', '..', path).replace(/\\/g, '\\\\')
}

export const winURL = filePath.winURL[env]
export const loadingURL = filePath.loadingURL[env]
export const lib = process.env.__lib
export const updateFolder = process.env.__updateFolder
export const getPreloadFile = filePath.getPreloadFile
