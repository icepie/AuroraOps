# AuroraOps

AuroraOps 是一个面向运维设备管理、远程终端和远程桌面控制的管理平台。项目由 GoFrame2 服务端、Vue3/Naive UI 管理前端和 Rust 客户端组成，目标是在服务端统一管理多台设备，并通过客户端提供终端、桌面代理和设备上报能力。

## 功能范围

- 设备管理：设备注册、在线状态、分组和基础信息维护。
- 远程终端：通过服务端转发到客户端终端会话，支持多设备标签页和重连。
- 远程桌面：集成 Weylus 协议能力，通过服务端入口打开，不暴露固定公网桌面端口。
- 硬件管理：采集和展示设备硬件信息。
- 系统管理：用户、角色、菜单、日志、字典、定时任务、附件等后台基础能力。

## 项目结构

- `server/`：GoFrame 服务端，默认 HTTP 端口 `8000`，TCP 设备通道端口 `8099`。
- `web/`：Vue3 管理后台前端，构建后资源嵌入或同步到服务端静态目录。
- `new-client/`：Rust 客户端，负责设备常驻服务、远程终端、远程桌面代理和本机配置。
- `docs/`：中文使用文档和开发文档。

## 本地开发

服务端：

```bash
cd server
go mod download
go run main.go
```

前端：

```bash
cd web
pnpm install
pnpm run dev
```

客户端：

```bash
cd new-client
cargo build --release
```

## 构建

服务端默认构建产物命名为 `auroraops-server`：

```bash
cd server
go build -o temp/auroraops-server main.go
```

前端生产构建：

```bash
cd web
pnpm run build
```

## 说明

本项目基于 HotGo/GoFrame 生态做二次开发，保留上游源码授权和必要署名。面向用户展示的产品名称统一为 AuroraOps。

## License

[MIT](./LICENSE)
