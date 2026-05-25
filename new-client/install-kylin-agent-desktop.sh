#!/usr/bin/env bash
set -euo pipefail

SERVER_HOST="${SERVER_HOST:-192.168.200.124:8000}"
HTTP_BASE="${HTTP_BASE:-http://192.168.200.124:8000}"
TCP_ADDRESS="${TCP_ADDRESS:-192.168.200.124:8099}"
DEVICE_NAME="${DEVICE_NAME:-$(hostname)}"
BIND_ADDRESS="${BIND_ADDRESS:-127.0.0.1}"
WEB_PORT="${WEB_PORT:-0}"
MGMT_PORT="${MGMT_PORT:-18765}"
BUILD_MODE="${BUILD_MODE:-system}"
DISPLAY_VALUE="${DISPLAY_VALUE:-:0}"
LOGIN_USER="${LOGIN_USER:-${SUDO_USER:-$(logname 2>/dev/null || id -un)}}"
XAUTHORITY_VALUE="${XAUTHORITY_VALUE:-/home/$LOGIN_USER/.Xauthority}"
BUILD_ROOT="${BUILD_ROOT:-$HOME/auroraops-agent-build}"
SRC_DIR="${SRC_DIR:-$BUILD_ROOT/new-client}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${LOG_FILE:-$HOME/auroraops-agent-install-$(date +%Y%m%d-%H%M%S).log}"

exec > >(tee -a "$LOG_FILE") 2>&1

say() {
  printf '\n[%s] %s\n' "$(date '+%F %T')" "$*"
}

run_sudo() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  elif [ -n "${SUDO_PASSWORD:-}" ]; then
    printf '%s\n' "$SUDO_PASSWORD" | sudo -S "$@"
  else
    sudo "$@"
  fi
}

install_available_packages() {
  local available=()
  local missing=()
  local pkg

  for pkg in "$@"; do
    if apt-cache show "$pkg" >/dev/null 2>&1; then
      available+=("$pkg")
    else
      missing+=("$pkg")
    fi
  done

  if [ "${#missing[@]}" -gt 0 ]; then
    say "这些包当前源里没有，先跳过：${missing[*]}"
  fi

  if [ "${#available[@]}" -gt 0 ]; then
    run_sudo apt-get install -y "${available[@]}"
  fi
}

say "AuroraOps Agent 安装脚本启动"
say "日志文件：$LOG_FILE"
say "配置：SERVER_HOST=$SERVER_HOST HTTP_BASE=$HTTP_BASE TCP_ADDRESS=$TCP_ADDRESS DEVICE_NAME=$DEVICE_NAME"
say "编译模式：$BUILD_MODE"
say "桌面环境：DISPLAY=$DISPLAY_VALUE XAUTHORITY=$XAUTHORITY_VALUE LOGIN_USER=$LOGIN_USER"
say "说明：Linux 下 glibc、X11、DBus、GStreamer 等系统库仍会动态链接；脚本会使用 Rust release/LTO、rustls，并默认链接系统 FFmpeg。"
say "如需尝试内置 FFmpeg，可用 BUILD_MODE=bundled-ffmpeg 重新运行，但耗时明显更长。"

say "安装系统编译依赖"
run_sudo apt-get update
install_available_packages \
  build-essential pkg-config git curl ca-certificates file cmake make gcc g++ \
  libdbus-1-dev \
  libx11-dev libxtst-dev libxext-dev libxrandr-dev libxfixes-dev libxcomposite-dev libxi-dev \
  libxinerama-dev libxcursor-dev libxkbcommon-dev libwayland-dev \
  libxcb1-dev libxcb-dri3-dev libx11-xcb-dev libxau-dev libxdmcp-dev x11proto-dev xtrans-dev \
  libdrm-dev \
  libglib2.0-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev liborc-0.4-dev \
  libavformat-dev libavfilter-dev libavcodec-dev libavutil-dev libavdevice-dev libswscale-dev libswresample-dev libx264-dev \
  libunwind-dev libdw-dev libelf-dev liblzma-dev libzstd-dev zlib1g-dev \
  libfontconfig1-dev libpango1.0-dev libcairo2-dev \
  gstreamer1.0-plugins-base gstreamer1.0-pipewire

if ! command -v cargo >/dev/null 2>&1; then
  if [ "${USE_SYSTEM_RUST:-0}" = "1" ]; then
    say "安装系统 rustc/cargo"
    run_sudo apt-get install -y rustc cargo
  else
    say "安装 rustup stable，使用 USTC 镜像"
    export RUSTUP_DIST_SERVER="${RUSTUP_DIST_SERVER:-https://mirrors.ustc.edu.cn/rust-static}"
    export RUSTUP_UPDATE_ROOT="${RUSTUP_UPDATE_ROOT:-https://mirrors.ustc.edu.cn/rust-static/rustup}"
    curl --proto '=https' --tlsv1.2 -sSf "$RUSTUP_UPDATE_ROOT/dist/$(uname -m)-unknown-linux-gnu/rustup-init" -o /tmp/rustup-init
    chmod +x /tmp/rustup-init
    /tmp/rustup-init -y --profile minimal --default-toolchain stable
  fi
fi

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi
say "Rust 版本：$(rustc --version)"
say "Cargo 版本：$(cargo --version)"

say "配置 Cargo 国内源"
mkdir -p "$HOME/.cargo"
cat > "$HOME/.cargo/config.toml" <<'EOF'
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"

[net]
git-fetch-with-cli = true
EOF

say "准备源码目录：$SRC_DIR"
mkdir -p "$BUILD_ROOT"
if [ -f "$SCRIPT_DIR/auroraops-new-client-full.tgz" ]; then
  if [ -d "$SRC_DIR/target" ]; then
    mv "$SRC_DIR/target" "$BUILD_ROOT/.new-client-target-cache"
  fi
  rm -rf "$SRC_DIR"
  tar -xzf "$SCRIPT_DIR/auroraops-new-client-full.tgz" -C "$BUILD_ROOT"
  if [ -d "$BUILD_ROOT/.new-client-target-cache" ]; then
    rm -rf "$SRC_DIR/target"
    mv "$BUILD_ROOT/.new-client-target-cache" "$SRC_DIR/target"
  fi
elif [ -f "/tmp/auroraops-new-client-full.tgz" ]; then
  if [ -d "$SRC_DIR/target" ]; then
    mv "$SRC_DIR/target" "$BUILD_ROOT/.new-client-target-cache"
  fi
  rm -rf "$SRC_DIR"
  tar -xzf /tmp/auroraops-new-client-full.tgz -C "$BUILD_ROOT"
  if [ -d "$BUILD_ROOT/.new-client-target-cache" ]; then
    rm -rf "$SRC_DIR/target"
    mv "$BUILD_ROOT/.new-client-target-cache" "$SRC_DIR/target"
  fi
elif [ -f "/tmp/auroraops-full-build/new-client/Cargo.toml" ]; then
  SRC_DIR="/tmp/auroraops-full-build/new-client"
elif [ ! -f "$SRC_DIR/Cargo.toml" ]; then
  echo "找不到 new-client 源码。请把 auroraops-new-client-full.tgz 放到脚本同目录后重试。" >&2
  exit 1
fi

cd "$SRC_DIR"
if [ ! -f Cargo.toml ]; then
  echo "源码目录无 Cargo.toml：$SRC_DIR" >&2
  exit 1
fi

if [ -f www/static/lib.js ] && [ -f ts/lib.ts ]; then
  touch -r www/static/lib.js ts/lib.ts || true
fi

say "检查 pkg-config 依赖"
for pkg in dbus-1 x11 xtst xext xrandr xfixes xcomposite xi libdrm xcb xcb-dri3 x11-xcb gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0; do
  pkg-config --exists "$pkg" || { echo "缺少 pkg-config 包：$pkg" >&2; exit 1; }
done
if [ "$BUILD_MODE" = "system" ]; then
  for pkg in libavformat libavfilter libavcodec libavutil libavdevice libswscale libswresample; do
    pkg-config --exists "$pkg" || { echo "缺少 pkg-config 包：$pkg" >&2; exit 1; }
  done
fi

say "停止旧服务并释放本轮调试进程"
run_sudo systemctl stop auroraops-agent.service >/dev/null 2>&1 || true
pkill -u "$(id -u)" -f 'auroraops-agent-service.*--config' >/dev/null 2>&1 || true
pkill -u "$(id -u)" -f '/tmp/auroraops-build/new-client/agent-service-lite' >/dev/null 2>&1 || true

say "开始编译完整 agent"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$(nproc)}"
features=()
if [ "$BUILD_MODE" = "system" ]; then
  features+=(ffmpeg-system)
elif [ "$BUILD_MODE" = "bundled-ffmpeg" ]; then
  say "使用内置 FFmpeg 构建，首次编译会比较久"
else
  echo "未知 BUILD_MODE：$BUILD_MODE，可用 system 或 bundled-ffmpeg" >&2
  exit 1
fi

if [ "${#features[@]}" -gt 0 ]; then
  cargo build --release --features "${features[*]}" --bin auroraops-agent
else
  cargo build --release --bin auroraops-agent
fi

say "编译产物依赖"
ldd target/release/auroraops-agent || true

say "安装二进制、配置和 systemd 服务"
run_sudo install -d /usr/local/bin /opt/auroraops /etc/auroraops /etc/systemd/system
run_sudo install -m 0755 target/release/auroraops-agent /usr/local/bin/auroraops-agent
if [ -f "$SRC_DIR/auroraops-uinput-setup" ]; then
  run_sudo install -m 0755 "$SRC_DIR/auroraops-uinput-setup" /opt/auroraops/auroraops-uinput-setup
  run_sudo /opt/auroraops/auroraops-uinput-setup || true
fi

if command -v xhost >/dev/null 2>&1 && [ -n "${DISPLAY:-}" ]; then
  say "授权 root 访问当前 X server"
  xhost +SI:localuser:root || true
fi

if [ ! -f "$XAUTHORITY_VALUE" ] && [ -f "/run/user/$(id -u "$LOGIN_USER" 2>/dev/null || echo 1000)/gdm/Xauthority" ]; then
  XAUTHORITY_VALUE="/run/user/$(id -u "$LOGIN_USER")/gdm/Xauthority"
fi

if [ -f /etc/auroraops/agent-config.json ]; then
  run_sudo cp -f /etc/auroraops/agent-config.json "/etc/auroraops/agent-config.json.bak.$(date +%Y%m%d-%H%M%S)" || true
fi

tmp_config="$(mktemp)"
cat > "$tmp_config" <<JSON
{
  "serverHost": "$SERVER_HOST",
  "deviceName": "$DEVICE_NAME",
  "httpBase": "$HTTP_BASE",
  "bindAddress": "$BIND_ADDRESS",
  "webPort": $WEB_PORT,
  "tcpAddress": "$TCP_ADDRESS",
  "controlDisplayManager": true
}
JSON
run_sudo install -m 0644 "$tmp_config" /etc/auroraops/agent-config.json
rm -f "$tmp_config"

tmp_service="$(mktemp)"
cat > "$tmp_service" <<SERVICE
[Unit]
Description=AuroraOps Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStartPre=-/opt/auroraops/auroraops-uinput-setup
Environment=DISPLAY=$DISPLAY_VALUE
Environment=XAUTHORITY=$XAUTHORITY_VALUE
Environment=XDG_RUNTIME_DIR=/run/user/$(id -u "$LOGIN_USER" 2>/dev/null || echo 1000)
Environment=WAYLAND_DISPLAY=
ExecStart=/usr/local/bin/auroraops-agent --service --config /etc/auroraops/agent-config.json --port $MGMT_PORT
Restart=always
RestartSec=5
User=root
WorkingDirectory=/usr/local/bin

[Install]
WantedBy=multi-user.target
SERVICE
run_sudo install -m 0644 "$tmp_service" /etc/systemd/system/auroraops-agent.service
rm -f "$tmp_service"

say "清理 KARE 包装服务残留"
run_sudo rm -f /opt/kare/etc/systemd/system/auroraops-agent.service /opt/kare/lib/systemd/system/auroraops-agent.service.bin || true

say "启动服务"
run_sudo systemctl daemon-reload
run_sudo systemctl enable auroraops-agent.service
run_sudo systemctl restart auroraops-agent.service
sleep 3

say "服务状态"
systemctl --no-pager --full status auroraops-agent.service || true
ss -lntp | grep -E ":($MGMT_PORT|$WEB_PORT)\\b" || true
curl -fsS "http://127.0.0.1:$MGMT_PORT/api/status" || true
printf '\n'

say "安装完成"
say "本机管理接口：http://127.0.0.1:$MGMT_PORT/"
say "Weylus 桌面接口：http://127.0.0.1:$WEB_PORT/"
say "日志文件：$LOG_FILE"
