#!/usr/bin/env bash
# AMUD RAM budget check — idle server RSS should stay lean.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

IDLE_LIMIT_KB=$((55 * 1024))
PEAK_LIMIT_KB=$((160 * 1024))

echo "Building amud-server release binary..."
cargo build -p amud-server --release -q

BIN="$ROOT/target/release/amud-server"
DB="$ROOT/target/ram-bench-amud.db"
rm -f "$DB"
mkdir -p "$(dirname "$DB")"

export DB_PATH="$DB"
export PORT=18999
export BIND_ADDR=127.0.0.1
export AMUD_AGENT_SECRET=ram-bench-secret-min-32-chars-long

"$BIN" &
PID=$!
cleanup() { kill "$PID" 2>/dev/null || true; wait "$PID" 2>/dev/null || true; }
trap cleanup EXIT

sleep 2
for _ in $(seq 1 30); do
  if curl -sf "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

rss_kb() {
  ps -o rss= -p "$PID" 2>/dev/null | tr -d ' ' || echo 0
}

IDLE_RSS=$(rss_kb)
echo "Idle RSS: ${IDLE_RSS} KB (limit ${IDLE_LIMIT_KB} KB)"

# Warm cache with repeated health requests
for _ in $(seq 1 50); do
  curl -sf "http://127.0.0.1:${PORT}/health" >/dev/null || true
done
sleep 1
PEAK_RSS=$(rss_kb)
echo "Peak RSS after load: ${PEAK_RSS} KB (limit ${PEAK_LIMIT_KB} KB)"

if [[ "$IDLE_RSS" -gt "$IDLE_LIMIT_KB" ]]; then
  echo "FAIL: idle RSS ${IDLE_RSS} KB exceeds ${IDLE_LIMIT_KB} KB"
  exit 1
fi
if [[ "$PEAK_RSS" -gt "$PEAK_LIMIT_KB" ]]; then
  echo "FAIL: peak RSS ${PEAK_RSS} KB exceeds ${PEAK_LIMIT_KB} KB"
  exit 1
fi

echo "PASS: RAM within budget"
