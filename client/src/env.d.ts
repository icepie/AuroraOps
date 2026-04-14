/// <reference types="vite/client" />

interface AuroraRuntimeInfo {
  appVersion: string;
  electronVersion: string;
  chromeVersion: string;
  nodeVersion: string;
  platform: string;
  arch: string;
  hostname: string;
  username: string;
  primaryIp: string;
}

interface AuroraServerCheckResult {
  ok: boolean;
  status: number;
  statusText: string;
  finalUrl: string;
}

interface Window {
  auroraClient: {
    getRuntimeInfo: () => Promise<AuroraRuntimeInfo>;
    openExternal: (url: string) => Promise<boolean>;
    checkServer: (url: string) => Promise<AuroraServerCheckResult>;
  };
}
