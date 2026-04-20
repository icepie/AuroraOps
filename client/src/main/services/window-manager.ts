import config from '@config/index'
import { BrowserWindow, dialog } from 'electron'
import { winURL, loadingURL, getPreloadFile } from '../config/static-path'
import { useProcessException } from '@main/hooks/exception-hook'
import { agentService } from './agent-service'

class MainInit {
  public winURL: string = ''
  public shartURL: string = ''
  public loadWindow: BrowserWindow = null
  public mainWindow: BrowserWindow = null
  private childProcessGone = null
  private mainWindowGone = null

  constructor() {
    const { childProcessGone, mainWindowGone } = useProcessException()
    this.winURL = winURL
    this.shartURL = loadingURL
    this.childProcessGone = childProcessGone
    this.mainWindowGone = mainWindowGone
  }

  private get useStartupChart() {
    return config.UseStartupChart && process.env.NODE_ENV !== 'development'
  }

  private revealMainWindow() {
    if (!this.mainWindow || this.mainWindow.isDestroyed()) {
      return
    }
    if (!this.mainWindow.isVisible()) {
      this.mainWindow.show()
    }
    if (this.loadWindow && !this.loadWindow.isDestroyed()) {
      this.loadWindow.destroy()
    }
  }
  // 主窗口函数
  async createMainWindow() {
    console.log('[window-manager] starting agent server')
    await agentService.ensureServerStarted()
    console.log('[window-manager] creating main window', this.winURL)
    this.mainWindow = new BrowserWindow({
      titleBarOverlay: {
        color: '#fff',
      },
      titleBarStyle: config.IsUseSysTitle ? 'default' : 'hidden',
      height: 800,
      useContentSize: true,
      width: 1700,
      minWidth: 1366,
      show: false,
      frame: config.IsUseSysTitle,
      webPreferences: {
        sandbox: false,
        webSecurity: false,
        // 如果是开发模式可以使用devTools
        devTools: process.env.NODE_ENV === 'development',
        // 在macos中启用橡皮动画
        scrollBounce: process.platform === 'darwin',
        preload: getPreloadFile('preload'),
      },
    })

    let revealTimer: NodeJS.Timeout | null = setTimeout(() => {
      revealTimer = null
      this.revealMainWindow()
    }, 3000)

    const revealOnce = () => {
      if (revealTimer) {
        clearTimeout(revealTimer)
        revealTimer = null
      }
      console.log('[window-manager] revealing main window')
      this.revealMainWindow()
    }

    this.mainWindow.once('ready-to-show', revealOnce)
    this.mainWindow.webContents.once('did-finish-load', revealOnce)
    this.mainWindow.webContents.once('did-fail-load', (_, errorCode, errorDescription) => {
      if (revealTimer) {
        clearTimeout(revealTimer)
        revealTimer = null
      }
      throw new Error(`failed to load agent ui (${errorCode}): ${errorDescription}`)
    })

    // 加载主窗口
    console.log('[window-manager] loading url', this.winURL)
    await this.mainWindow.loadURL(this.winURL)
    // 开发模式下自动开启devtools
    if (process.env.NODE_ENV === 'development') {
      this.mainWindow.webContents.openDevTools({
        mode: 'undocked',
        activate: true,
      })
    }
    // 不知道什么原因，反正就是这个窗口里的页面触发了假死时执行
    this.mainWindowGone(this.mainWindow)
    /**
     * 新的gpu崩溃检测，详细参数详见：http://www.electronjs.org/docs/api/app
     * @returns {void}
     * @author zmr (umbrella22)
     * @date 2020-11-27
     */
    this.childProcessGone(this.mainWindow)
    this.mainWindow.on('closed', () => {
      if (revealTimer) {
        clearTimeout(revealTimer)
        revealTimer = null
      }
      this.mainWindow = null
    })
  }

  async openMainWindow() {
    try {
      await this.createMainWindow()
    } catch (error) {
      if (this.loadWindow && !this.loadWindow.isDestroyed()) {
        this.loadWindow.destroy()
      }
      dialog.showErrorBox(
        'AuroraOps Client',
        error instanceof Error ? error.message : String(error),
      )
      throw error
    }
  }
  // 加载窗口函数
  loadingWindow(loadingURL: string) {
    this.loadWindow = new BrowserWindow({
      width: 400,
      height: 600,
      frame: false,
      skipTaskbar: true,
      transparent: true,
      resizable: false,
      webPreferences: {
        experimentalFeatures: true,
        preload: getPreloadFile('preload'),
      },
    })

    this.loadWindow.loadURL(loadingURL)
    this.loadWindow.show()
    this.loadWindow.setAlwaysOnTop(true)
    // 延迟两秒可以根据情况后续调快，= =，就相当于个，sleep吧，就那种。 = =。。。
    setTimeout(() => {
      void this.openMainWindow()
    }, 1500)
  }
  // 初始化窗口函数
  initWindow() {
    if (this.useStartupChart) {
      return this.loadingWindow(this.shartURL)
    } else {
      return void this.openMainWindow()
    }
  }
}
export default MainInit
