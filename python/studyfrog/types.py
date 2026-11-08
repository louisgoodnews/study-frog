"""Plain records the API returns."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

__all__ = ["StudyFrogError", "Item", "Event", "LogEntry"]


class StudyFrogError(Exception):
    """A request StudyFrog answered with an error (refused, invalid, or failed)."""


@dataclass(frozen=True)
class Item:
    """One row of a query: an item of the vault."""

    uid: str
    title: str
    kind: str
    status: str
    path: str
    key: Optional[str] = None
    tags: List[str] = field(default_factory=list)
    color: Optional[str] = None

    @staticmethod
    def from_json(row: Dict[str, Any]) -> "Item":
        return Item(
            uid=row["uid"],
            title=row.get("title", ""),
            kind=row.get("kind", ""),
            status=row.get("status", ""),
            path=row.get("path", ""),
            key=row.get("key"),
            tags=list(row.get("tags") or []),
            color=row.get("color"),
        )


@dataclass(frozen=True)
class Event:
    """A lifecycle event: ``kind`` is one of ``vault_opened``, ``vault_closed``, ``item_created``,
    ``item_saved``, ``rehearsal_finished`` or ``script_finished``."""

    kind: str
    at_ms: int
    data: Dict[str, Any]

    @staticmethod
    def from_json(value: Dict[str, Any]) -> "Event":
        return Event(kind=value.get("kind", ""), at_ms=int(value.get("at_ms", 0)), data=value.get("data") or {})


@dataclass(frozen=True)
class LogEntry:
    """One entry of the application log."""

    at_ms: int
    level: str
    source: str
    message: str

    @staticmethod
    def from_json(value: Dict[str, Any]) -> "LogEntry":
        return LogEntry(
            at_ms=int(value.get("at_ms", 0)),
            level=value.get("level", "info"),
            source=value.get("source", ""),
            message=value.get("message", ""),
        )
