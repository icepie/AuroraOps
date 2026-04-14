import { app, BrowserWindow, ipcMain, shell } from 'electron';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import os from 'node:os';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DEV_SERVER_URL = 'http://127.0.0.1:5173';
const RENDERER_HTML = path.join(__dirname, '../dist/index.html');

function isDev() {
  return !app.isPackaged;
}

function getPrimaryIp() {
  const interfaces = os.networkInterfaces();

  for (const entries of Object.values(interfaces)) {
    for (const item of entries || []) {
      if (item.family === 'IPv4' && !item.internal) {
        return item.address;
      }
    }
  }

  return '';
}

function createWindow() {
  const win = new BrowserWindow({
    width: 1320,
    height: 860,
    minWidth: 1080,
    minHeight: 720,
    backgroundColor: '#f3f4f6',
    title: 'AuroraOps Client',
    webPreferences: {
      preload: path.join(__dirname, 'preload.mjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  if (isDev()) {
    win.loadURL(DEV_SERVER_URL);
    win.webContents.openDevTools({ mode: 'detach' });
    return win;
  }

  win.loadFile(RENDERER_HTML);
  return win;
}

ipcMain.handle('app:get-runtime-info', () => {
  return {
    appVersion: app.getVersion(),
    electronVersion: process.versions.electron,
    chromeVersion: process.versions.chrome,
    nodeVersion: process.versions.node,
    platform: process.platform,
    arch: process.arch,
    hostname: os.hostname(),
    username: os.userInfo().username,
    primaryIp: getPrimaryIp(),
  };
});

ipcMain.handle('app:open-external', async (_, rawUrl) => {
  const url = new URL(rawUrl);
  if (!['http:', 'https:'].includes(url.protocol)) {
    throw new Error('仅允许打开 http/https 地址');
  }
  await shell.openExternal(url.toString());
  return true;
});

ipcMain.handle('server:check', async (_, rawUrl) => {
  const url = new URL(rawUrl);
  if (!['http:', 'https:'].includes(url.protocol)) {
    throw new Error('服务端地址必须是 http/https');
  }

  const response = await fetch(url.toString(), {
    method: 'GET',
    redirect: 'follow',
  });

  return {
    ok: response.ok,
    status: response.status,
    statusText: response.statusText,
    finalUrl: response.url,
  };
});

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
