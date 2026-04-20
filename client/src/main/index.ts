'use strict'

import { app, Menu, Tray, nativeImage, session } from 'electron'
import { existsSync } from 'fs'
import { join } from 'path'
import InitWindow from './services/window-manager'
import { useDisableButton } from './hooks/disable-button-hook'
import { useProcessException } from '@main/hooks/exception-hook'
import { useMenu } from '@main/hooks/menu-hook'
import { useMainDefaultIpc } from './services/ipc-main'

let mainApp: InitWindow | null = null
let tray: Tray | null = null
let isQuitting = false

function resolveTrayIcon() {
  const candidates = [
    join(process.cwd(), 'src', 'renderer', 'assets', 'logo.png'),
    join(__dirname, '..', '..', 'renderer', 'logo.png'),
    join(process.resourcesPath, 'app.asar.unpacked', 'dist', 'electron', 'renderer', 'logo.png'),
    join(process.resourcesPath, 'dist', 'electron', 'renderer', 'logo.png'),
  ]
  for (const iconPath of candidates) {
    if (!existsSync(iconPath)) {
      continue
    }
    const image = nativeImage.createFromPath(iconPath)
    if (!image.isEmpty()) {
      return image
    }
  }
  return nativeImage.createEmpty()
}

function createTray() {
  if (tray) {
    return tray
  }
  tray = new Tray(resolveTrayIcon())
  tray.setToolTip('AuroraOps Client')
  tray.setContextMenu(
    Menu.buildFromTemplate([
      {
        label: '显示主窗口',
        click: () => mainApp?.showMainWindow(),
      },
      {
        label: '隐藏到托盘',
        click: () => mainApp?.hideMainWindow(),
      },
      {
        label: '退出',
        click: () => {
          isQuitting = true
          mainApp?.prepareQuit()
          app.quit()
        },
      },
    ]),
  )
  tray.on('double-click', () => mainApp?.showMainWindow())
  tray.on('click', () => mainApp?.showMainWindow())
  return tray
}

function onAppReady() {
  const { disableF12 } = useDisableButton()
  const { renderProcessGone } = useProcessException()
  const { defaultIpc } = useMainDefaultIpc()
  const { creactMenu } = useMenu()
  disableF12()
  renderProcessGone()
  defaultIpc()
  creactMenu()
  mainApp = new InitWindow()
  createTray()
  mainApp.initWindow()
  if (process.env.NODE_ENV === 'development') {
    const { VUEJS_DEVTOOLS } = require('electron-devtools-vendor')
    session.defaultSession.extensions.loadExtension(VUEJS_DEVTOOLS, {
      allowFileAccess: true,
    })
    console.log('已安装: vue-devtools')
  }
}

app.whenReady().then(onAppReady)
// 由于9.x版本问题，需要加入该配置关闭跨域问题
app.commandLine.appendSwitch('disable-features', 'OutOfBlinkCors')

app.on('window-all-closed', () => {
  if (isQuitting) {
    app.quit()
  }
})
app.on('browser-window-created', () => {
  console.log('window-created')
})
app.on('before-quit', () => {
  isQuitting = true
  mainApp?.prepareQuit()
})

if (process.defaultApp) {
  if (process.argv.length >= 2) {
    app.removeAsDefaultProtocolClient('electron-vue-template')
    console.log('由于框架特殊性开发环境下无法使用')
  }
} else {
  app.setAsDefaultProtocolClient('electron-vue-template')
}
