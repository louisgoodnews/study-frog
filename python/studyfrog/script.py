"""What a script started by StudyFrog's runner is told through its environment."""

from __future__ import annotations

import json
import os
from typing import Optional

from .types import Event

__all__ = ["trigger_event", "job_name", "access"]


def trigger_event() -> Optional[Event]:
    """The lifecycle event that started this script (``$STUDYFROG_EVENT``), or ``None`` when it was
    started by its schedule, by hand or from the console."""
    raw = os.environ.get("STUDYFROG_EVENT")
    if not raw:
        return None
    return Event.from_json(json.loads(raw))


def job_name() -> Optional[str]:
    """The name of the ``scripts.toml`` entry that runs this script (``$STUDYFROG_JOB``)."""
    return os.environ.get("STUDYFROG_JOB") or None


def access() -> str:
    """What the script's socket lets through: ``read``, ``write`` or ``destructive``."""
    return os.environ.get("STUDYFROG_ACCESS", "read")
