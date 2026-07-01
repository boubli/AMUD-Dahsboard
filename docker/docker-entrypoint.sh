#!/bin/sh
set -e

# Agent containers override entrypoint to /app/amud-agent and run as root (docker.sock).
if [ "$1" = "/app/amud-agent" ] || [ "${1##*/}" = "amud-agent" ]; then
    exec "$@"
fi

exec "$@"
