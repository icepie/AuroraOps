import { contextBridge, ipcRenderer } from 'electron';

contextBridge.exposeInMainWorld('auroraClient', {
  getRuntimeInfo: () => ipcRenderer.invoke('app:get-runtime-info'),
  openExternal: (url) => ipcRenderer.invoke('app:open-external', url),
  checkServer: (url) => ipcRenderer.invoke('server:check', url),
});
