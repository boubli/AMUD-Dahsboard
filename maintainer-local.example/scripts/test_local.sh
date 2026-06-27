#!/bin/bash
set -e

echo "=== AMUD Rust Integration Test Script ==="

# Clean up previous runs
cleanup() {
    echo "Cleaning up test processes and sockets..."
    if [[ -n "$AGENT_PID" ]]; then
        kill "$AGENT_PID" 2>/dev/null || true
    fi
    if [[ -n "$SERVER_PID" ]]; then
        kill "$SERVER_PID" 2>/dev/null || true
    fi
    rm -f /tmp/amud.sock
}
trap cleanup EXIT

# 1. Build Rust workspace
echo "Compiling Rust workspace projects in release mode..."
cargo build --release

# 2. Start Rust web server
echo "Starting AMUD Rust web server on port 8000..."
export AMUD_SOCKET_PATH="/tmp/amud.sock"
export AMUD_AGENT_SECRET="test-local-secret"
export PORT="8000"
./target/release/amud-server > server.log 2>&1 &
SERVER_PID=$!
sleep 2

# Verify server started
if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "Error: Rust web server failed to start. Logs:" >&2
    cat server.log >&2
    exit 1
fi

# 3. Start Rust telemetry agent
echo "Starting Rust telemetry agent..."
export AMUD_AGENT_SECRET="test-local-secret"
./target/release/amud-agent > agent.log 2>&1 &
AGENT_PID=$!
sleep 3

# Verify agent is running
if ! kill -0 "$AGENT_PID" 2>/dev/null; then
    echo "Error: Rust agent failed to start. Logs:" >&2
    cat agent.log >&2
    exit 1
fi

# 4. Query home page to verify rendering
echo "Verifying web page rendering..."
RESPONSE=$(curl -s http://localhost:8000/)

if echo "$RESPONSE" | grep -q "AMUD"; then
    echo "Success: Home page responded with AMUD branding variables!"
else
    echo "Error: Render verification failed. Web page response:" >&2
    echo "$RESPONSE" >&2
    exit 1
fi

echo "All tests passed successfully!"
