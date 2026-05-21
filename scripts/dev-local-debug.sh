#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
APP_SUPPORT_DIR=${SRIGHT_APP_SUPPORT_DIR:-"$HOME/Library/Application Support/sRight"}
CLI_PATH="$ROOT_DIR/target/debug/sright-cli"
DESKTOP_PATH="$ROOT_DIR/target/debug/sright-desktop"
BRIDGE_PATH="$APP_SUPPORT_DIR/sright-cli-debug.sh"
ACTION_LOG="$APP_SUPPORT_DIR/actions.jsonl"
FINDER_TRACE_LOG="$APP_SUPPORT_DIR/finder-sync-trace.log"
START_TAURI=1
RESTART_FINDER=1
DEV_PORT=1420

usage() {
  cat <<'EOF'
sRight 本地调试一键启动脚本

Usage:
  scripts/dev-local-debug.sh [--no-tauri] [--no-finder-restart]

默认会执行：
  1. 按需执行 pnpm install
  2. 构建 debug 版 sright-cli
  3. 初始化本地 config.json
  4. 写入 FinderSync debug bridge
  5. 通过 launchctl 设置 SRIGHT_CLI_PATH
  6. 重启 Finder，让 FinderSync 优先调用 target/debug/sright-cli
  7. 启动 Tauri dev 偏好设置 app

Options:
  --no-tauri            只准备 Finder/CLI 调试环境，不启动 Tauri dev
  --no-finder-restart   不重启 Finder
  -h, --help            显示帮助
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --no-tauri)
      START_TAURI=0
      ;;
    --no-finder-restart)
      RESTART_FINDER=0
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

free_dev_port() {
  if ! command -v lsof >/dev/null 2>&1; then
    return
  fi

  pids=$(lsof -ti "tcp:$DEV_PORT" 2>/dev/null || true)
  if [ -z "$pids" ]; then
    return
  fi

  echo "停止占用 127.0.0.1:$DEV_PORT 的旧 dev server: $pids"
  for pid in $pids; do
    kill "$pid" >/dev/null 2>&1 || true
  done
  sleep 1
}

stop_debug_app() {
  pids=$(pgrep -f "$DESKTOP_PATH" 2>/dev/null || true)
  if [ -z "$pids" ]; then
    return
  fi

  echo "停止旧的 sRight debug app: $pids"
  for pid in $pids; do
    kill "$pid" >/dev/null 2>&1 || true
  done
  sleep 1
}

write_bridge() {
  mkdir -p "$APP_SUPPORT_DIR"
  cat > "$BRIDGE_PATH" <<EOF
#!/bin/sh
set -eu
exec "$CLI_PATH" "\$@"
EOF
  chmod 755 "$BRIDGE_PATH"
}

print_next_steps() {
  cat <<EOF

本地调试环境已准备好：
  CLI: $CLI_PATH
  Bridge: $BRIDGE_PATH
  Config: $APP_SUPPORT_DIR/config.json
  Action log: $ACTION_LOG
  FinderSync trace: $FINDER_TRACE_LOG

常用观察命令：
  tail -f "$ACTION_LOG"
  tail -f "$FINDER_TRACE_LOG"
  log stream --style compact --predicate 'eventMessage CONTAINS "sRight FinderSync"'

验证 CLI：
  cargo run -p sright-cli -- action run --id debug.echo --path "\$PWD/docs/requirements.md"
EOF
}

cd "$ROOT_DIR"

require_command cargo
require_command pnpm

if [ ! -d "$ROOT_DIR/node_modules/.pnpm" ]; then
  pnpm install
fi

cargo build -p sright-cli
cargo run -p sright-cli -- config init >/dev/null
write_bridge

if command -v launchctl >/dev/null 2>&1; then
  launchctl setenv SRIGHT_CLI_PATH "$CLI_PATH"
fi

if [ "$RESTART_FINDER" -eq 1 ]; then
  killall Finder >/dev/null 2>&1 || true
fi

print_next_steps

if [ "$START_TAURI" -eq 1 ]; then
  stop_debug_app
  free_dev_port
  exec pnpm --filter @sright/desktop tauri dev
fi
