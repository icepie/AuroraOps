## Docker 部署服务端

本文档说明如何构建 AuroraOps 服务端镜像、推送到 Docker Hub，并在服务器上运行。镜像包含：

- GoFrame 服务端二进制 `auroraops-server`
- 已构建的 Vue 管理端静态资源
- 默认配置目录 `manifest/config`
- 初始化 SQL 文件目录 `storage/data`

镜像不内置 MySQL/PostgreSQL/Redis。生产环境建议把数据库、Redis 和服务端容器分开部署。

服务端启动时会读取数据库中的系统配置。数据库不可达或未导入初始化 SQL 时，容器会启动失败并退出，这是预期行为；请先完成数据库和 `config.yaml` 配置。

### 端口

| 端口 | 用途 |
| --- | --- |
| `8000` | 管理端 HTTP、API、WebSocket |
| `8099` | 客户端 Agent TCP 长连接通道 |

客户端需要能访问服务端的 `8099` 端口；浏览器需要能访问 `8000` 端口。

### 构建镜像

在项目根目录执行：

```bash
docker build -f Dockerfile.server -t your-dockerhub-user/auroraops-server:latest .
```

构建指定版本标签：

```bash
docker build -f Dockerfile.server -t your-dockerhub-user/auroraops-server:0.12.0 .
```

如果服务器访问 npm/go 模块较慢，可以按本机网络情况配置 Docker BuildKit 代理或镜像源。Dockerfile 使用多阶段构建，会先构建 `web`，再构建 `server`。

Dockerfile 默认使用：

- npm registry：`https://registry.npmmirror.com`
- Go proxy：`https://goproxy.cn,direct`

如需覆盖：

```bash
docker build \
  -f Dockerfile.server \
  --build-arg NPM_REGISTRY=https://registry.npmjs.org \
  --build-arg GOPROXY=https://proxy.golang.org,direct \
  -t your-dockerhub-user/auroraops-server:latest \
  .
```

### 推送到 Docker Hub

先登录 Docker Hub：

```bash
docker login
```

推送镜像：

```bash
docker push your-dockerhub-user/auroraops-server:latest
docker push your-dockerhub-user/auroraops-server:0.12.0
```

### GitHub Actions 自动发布

仓库已提供 `.github/workflows/docker-server.yml`，可以在 GitHub Actions 中构建并推送多架构镜像到 Docker Hub。

需要在 GitHub 仓库中配置 Secrets：

| Secret | 说明 |
| --- | --- |
| `DOCKERHUB_USERNAME` | Docker Hub 用户名 |
| `DOCKERHUB_TOKEN` | Docker Hub Access Token，不建议使用账号密码 |

可选配置仓库 Variables：

| Variable | 说明 |
| --- | --- |
| `DOCKERHUB_IMAGE` | 完整镜像名，例如 `your-dockerhub-user/auroraops-server` |

如果没有配置 `DOCKERHUB_IMAGE`，默认使用：

```text
${DOCKERHUB_USERNAME}/auroraops-server
```

触发方式：

- 推送到 `main` 分支：发布 `latest` 和 `main-<sha>` 标签
- 推送 `v*` 标签：发布同名版本标签，例如 `v0.12.0`
- 手动运行 workflow：可以通过输入参数覆盖镜像名

如果要同时发布多架构镜像，可以使用 `buildx`：

```bash
docker buildx create --use --name auroraops-builder
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -f Dockerfile.server \
  --build-arg NPM_REGISTRY=https://registry.npmmirror.com \
  --build-arg GOPROXY=https://goproxy.cn,direct \
  -t your-dockerhub-user/auroraops-server:latest \
  -t your-dockerhub-user/auroraops-server:0.12.0 \
  --push \
  .
```

### 准备数据库

创建数据库后导入初始化 SQL。MySQL 使用：

```bash
mysql -uroot -p -e "CREATE DATABASE auroraops DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;"
mysql -uroot -p auroraops < server/storage/data/hotgo.sql
```

PostgreSQL 使用：

```bash
createdb auroraops
psql auroraops < server/storage/data/hotgo-pg.sql
```

生产部署时建议创建独立数据库账号，不要直接使用 root 或 postgres 超级用户。

### 准备配置目录

先把默认配置复制到宿主机目录，便于后续修改和持久化：

```bash
mkdir -p ./deploy/auroraops/config ./deploy/auroraops/storage ./deploy/auroraops/logs ./deploy/auroraops/upload
cp -a server/manifest/config/. ./deploy/auroraops/config/
```

修改 `./deploy/auroraops/config/config.yaml` 中的数据库连接：

```yaml
database:
  default:
    link: "mysql:auroraops:your-password@tcp(mysql-host:3306)/auroraops?loc=Local&parseTime=true&charset=utf8mb4"
```

如果使用 Redis，也需要修改 `redis.default.address`、`redis.default.pass` 等配置，并按需把 `cache.adapter` 或 `queue.driver` 改为 `redis`。

生产环境还建议调整：

```yaml
system:
  debug: false
  mode: "product"
token:
  secretKey: "请替换为足够随机的生产密钥"
```

### docker run 启动

启动前确认 `./deploy/auroraops/config/config.yaml` 里的数据库地址在容器内可访问。注意：容器内的 `127.0.0.1` 指向容器自己，不是宿主机；如果数据库跑在宿主机上，请使用宿主机局域网 IP，或 Docker 的 `host.docker.internal`/自定义网络地址。

```bash
docker run -d \
  --name auroraops-server \
  --restart unless-stopped \
  -p 8000:8000 \
  -p 8099:8099 \
  -v "$PWD/deploy/auroraops/config:/app/manifest/config" \
  -v "$PWD/deploy/auroraops/storage:/app/storage" \
  -v "$PWD/deploy/auroraops/logs:/app/logs" \
  -v "$PWD/deploy/auroraops/upload:/app/resource/public/upload" \
  your-dockerhub-user/auroraops-server:latest
```

查看日志：

```bash
docker logs -f auroraops-server
```

访问管理端：

```text
http://服务器IP:8000/admin
```

默认账号密码以初始化 SQL 为准；当前开发环境默认通常为：

```text
admin / 123456
```

首次上线后请立即修改默认密码。

### docker compose 示例

下面示例只演示服务端容器，数据库使用外部 MySQL：

```yaml
services:
  auroraops-server:
    image: your-dockerhub-user/auroraops-server:latest
    container_name: auroraops-server
    restart: unless-stopped
    ports:
      - "8000:8000"
      - "8099:8099"
    volumes:
      - ./deploy/auroraops/config:/app/manifest/config
      - ./deploy/auroraops/storage:/app/storage
      - ./deploy/auroraops/logs:/app/logs
      - ./deploy/auroraops/upload:/app/resource/public/upload
```

启动：

```bash
docker compose up -d
```

停止：

```bash
docker compose down
```

### 客户端连接配置

客户端 Agent 的服务端地址需要指向 Docker 宿主机的 TCP 端口：

```text
服务器IP:8099
```

如果前面还有防火墙、安全组或反向代理，需要放通：

- `8000/tcp`：浏览器访问管理端和 API
- `8099/tcp`：客户端 Agent 接入服务端

### Nginx 反向代理

如果使用域名访问管理端，可以把 `8000` 放到 Nginx 后面。WebSocket 必须保留升级头：

```nginx
location / {
    proxy_pass http://127.0.0.1:8000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_read_timeout 600s;
    proxy_send_timeout 600s;
}
```

`8099` 是 Agent TCP 通道，不建议用普通 HTTP 反向代理转发；可以直接暴露端口，或使用 Nginx stream/TCP 代理。

### 常用维护命令

升级镜像：

```bash
docker pull your-dockerhub-user/auroraops-server:latest
docker stop auroraops-server
docker rm auroraops-server
docker run -d \
  --name auroraops-server \
  --restart unless-stopped \
  -p 8000:8000 \
  -p 8099:8099 \
  -v "$PWD/deploy/auroraops/config:/app/manifest/config" \
  -v "$PWD/deploy/auroraops/storage:/app/storage" \
  -v "$PWD/deploy/auroraops/logs:/app/logs" \
  -v "$PWD/deploy/auroraops/upload:/app/resource/public/upload" \
  your-dockerhub-user/auroraops-server:latest
```

进入容器：

```bash
docker exec -it auroraops-server sh
```

健康检查接口：

```bash
curl http://127.0.0.1:8000/api.json
```

### 注意事项

- 容器内默认以非 root 用户 `auroraops` 运行。
- `/app/manifest/config`、`/app/storage`、`/app/logs`、`/app/resource/public/upload` 建议持久化。
- 如果挂载了空的 `/app/manifest/config`，服务端会找不到配置文件。首次运行前请先复制默认配置。
- 修改 `config.yaml` 后需要重启容器。
- 生产环境不要使用默认 `token.secretKey`、默认数据库密码和默认管理员密码。
