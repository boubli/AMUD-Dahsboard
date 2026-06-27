"""Resolve AMUD repo root from maintainer-local/scripts or legacy repo/scripts."""

from __future__ import annotations

from pathlib import Path


def repo_root() -> Path:
    start = Path(__file__).resolve().parent
    for candidate in (start.parent, start.parent.parent):
        if (candidate / "amud-server").is_dir():
            return candidate
    return start.parent
