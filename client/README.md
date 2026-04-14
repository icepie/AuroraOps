# AuroraOps Client

AuroraOps 桌面客户端骨架，使用 `Electron + Vite + Vue 3`。

当前目标：

- 作为设备侧客户端承载桌面能力
- 与 AuroraOps 服务端做状态同步、设备注册、心跳和远程控制联动
- 为后续托盘、自动启动、采集器、隧道代理等能力预留入口

## 目录

```text
client/
  electron/         Electron 主进程和 preload
  src/              Vite 渲染层
  package.json
  vite.config.ts
```

## 启动

```bash
cd client
pnpm install
cp .env.example .env
pnpm dev
```

## 构建

```bash
cd client
pnpm build
```

## 当前已预留能力

- `window.auroraClient.getRuntimeInfo()`
- `window.auroraClient.openExternal(url)`
- `window.auroraClient.checkServer(url)`

后续如果要继续做设备联动，可以在这个骨架上继续加：

- 设备指纹采集
- 客户端注册和鉴权
- WebSocket / SSE 长连接
- 托盘常驻
- 开机自启
- 本地命令执行与审计
