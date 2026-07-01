#!/bin/sh
set -e

# Agent containers override entrypoint to /app/amud-agent and stay UID 0 (Docker socket).
if [ "$1" = "/app/amud-agent" ] || [ "${1##*/}" = "amud-agent" ]; then
    exec "$@"
fi

PUID="${PUID:-99}"
PGID="${PGID:-100}"

if [ "$(id -u)" = "0" ]; then
    exec su-exec "${PUID}:${PGID}" "$@"
fi

exec "$@"
