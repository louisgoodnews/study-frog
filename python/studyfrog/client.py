"""The StudyFrog API: typed functions over the control protocol (``studyfrog ctl help`` lists the
commands). Every call waits until the instance has settled and returns its answer."""

from __future__ import annotations

import collections
import json
from pathlib import Path
from typing import Any, Deque, Dict, Iterable, Iterator, List, Optional, Sequence

from .transport import Transport, default_socket, open_socket, start_headless
from .types import Event, Item, LogEntry, StudyFrogError

__all__ = ["Client", "connect"]


class Client:
    """One connection to a StudyFrog instance. Use it as a context manager or call ``close``."""

    def __init__(self, transport: Transport) -> None:
        self._t = transport
        self._next_id = 0
        self._events: Deque[Dict[str, Any]] = collections.deque()

    # -- the protocol -------------------------------------------------------------------------
    def call(self, cmd: str, *args: Any) -> Any:
        """Send one command with its arguments and return its ``result``; raises ``StudyFrogError``
        when the answer is an error. Event lines that arrive meanwhile are kept for ``events``."""
        self._next_id += 1
        rid = self._next_id
        words = [str(a) for a in args if a is not None]
        self._t.send(json.dumps({"id": rid, "cmd": cmd, "args": words}))
        while True:
            line = self._t.readline()
            if line is None:
                raise StudyFrogError("the StudyFrog instance closed the connection")
            msg = json.loads(line)
            if "event" in msg:
                self._events.append(msg)
                continue
            if msg.get("id") != rid:
                continue
            if not msg.get("ok"):
                raise StudyFrogError(msg.get("error", "unknown error"))
            return msg.get("result")

    def events(self, timeout: Optional[float] = None) -> Iterator[Event]:
        """Lifecycle events on this connection (after ``subscribe``), as they arrive; ends when the
        instance goes away or no event came for ``timeout`` seconds (then it can be called again)."""
        while True:
            while self._events:
                msg = self._events.popleft()
                if msg.get("event") == "lifecycle":
                    yield Event.from_json(msg.get("data") or {})
            line = self._read_with_timeout(timeout)
            if line is None:
                return
            msg = json.loads(line)
            if "event" in msg:
                self._events.append(msg)

    def _read_with_timeout(self, timeout: Optional[float]) -> Optional[str]:
        try:
            return self._t.readline(timeout)
        except TimeoutError:
            return None

    def close(self) -> None:
        self._t.close()

    def __enter__(self) -> "Client":
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()

    # -- reading ------------------------------------------------------------------------------
    def state(self) -> Dict[str, Any]:
        """The snapshot: screen, vault and stacks, item page, rehearsal, library, notices."""
        return self.call("state")

    def help(self) -> Dict[str, Any]:
        return self.call("help")

    def stacks(self) -> List[Dict[str, Any]]:
        vault = self.state().get("vault") or {}
        return list(vault.get("stacks") or [])

    def query(self, expr: str) -> List[Item]:
        """Items matching a library query such as ``type = card and tag = verbs order by title``."""
        result = self.call("query", expr)
        return [Item.from_json(r) for r in result.get("rows", [])]

    def search(self, text: str) -> List[Dict[str, Any]]:
        result = self.call("search", text)
        return list(((result.get("search") or {}).get("results")) or [])

    def notices(self) -> List[Dict[str, Any]]:
        return self.call("notices")

    def logs(self, tail: int = 50, level: Optional[str] = None) -> List[LogEntry]:
        args: List[str] = ["--tail", str(tail)]
        if level:
            args += ["--level", level]
        result = self.call("logs", *args)
        return [LogEntry.from_json(e) for e in result.get("entries", [])]

    def subscribe(self, *kinds: str) -> List[str]:
        """Ask for lifecycle events (all kinds when none are named); read them with ``events``."""
        return self.call("subscribe", *kinds).get("subscribed", [])

    # -- navigation ---------------------------------------------------------------------------
    def nav(self, screen: str) -> Dict[str, Any]:
        return self.call("nav", screen)

    def back(self) -> Dict[str, Any]:
        return self.call("back")

    def forward(self) -> Dict[str, Any]:
        return self.call("forward")

    def settle(self) -> Dict[str, Any]:
        return self.call("settle")

    # -- vault, items and stacks --------------------------------------------------------------
    def open_vault(self, folder: str | Path) -> Dict[str, Any]:
        return self.call("vault", "open", str(folder))

    def close_vault(self) -> Dict[str, Any]:
        return self.call("vault", "close")

    def create_item(
        self,
        kind: str,
        title: str,
        text: Optional[str] = None,
        answer: Optional[str] = None,
        tags: Optional[Iterable[str]] = None,
        stack: Optional[str] = None,
    ) -> Optional[str]:
        """Create a ``note``, ``card`` or ``question``; returns the new item's id."""
        args: List[Any] = ["create", kind, title]
        for flag, value in (("--text", text), ("--answer", answer), ("--stack", stack)):
            if value is not None:
                args += [flag, value]
        if tags:
            args += ["--tags", ",".join(tags)]
        return self.call("item", *args).get("notice_item")

    def open_item(self, item: str) -> Dict[str, Any]:
        """Open an item (by ULID or key) and return the snapshot with its page in ``editor``."""
        return self.call("item", "open", item)

    def edit_item(self, item: str, field: str, text: str) -> Dict[str, Any]:
        """Change ``title``, ``tags``, ``front``, ``back``, ``body``, ``prompt``, ``reference`` or
        ``fieldN`` and save."""
        return self.call("item", "edit", item, field, text)

    def color_item(self, item: str, color: Optional[str]) -> Dict[str, Any]:
        return self.call("item", "color", item, color or "none")

    def move_item(self, item: str, stack: str) -> Dict[str, Any]:
        return self.call("item", "move", item, stack)

    def delete_item(self, item: str) -> Dict[str, Any]:
        """Destructive: needs an instance or script entry that allows destructive commands."""
        return self.call("item", "delete", item)

    def create_stack(self, title: str, parent: Optional[str] = None) -> Dict[str, Any]:
        args: List[Any] = ["create", title]
        if parent:
            args += ["--parent", parent]
        return self.call("stack", *args)

    def open_stack(self, folder: str) -> Dict[str, Any]:
        return self.call("stack", "open", folder)

    def stack_settings(self, folder: str) -> Dict[str, Any]:
        return self.call("stack", "settings", folder)

    def move_stack(self, folder: str, into: Optional[str] = None) -> Dict[str, Any]:
        return self.call("stack", "move", folder, into)

    # -- rehearsal ----------------------------------------------------------------------------
    def rehearse_setup(
        self,
        preset: Optional[str] = None,
        mode: Optional[str] = None,
        limit: Optional[int] = None,
        stacks: Sequence[str] = (),
    ) -> Dict[str, Any]:
        """Set up a run; ``result["rehearse"]["preview"]`` says how many items it would ask."""
        args: List[Any] = ["setup"]
        for flag, value in (("--preset", preset), ("--mode", mode), ("--limit", limit)):
            if value is not None:
                args += [flag, value]
        for s in stacks:
            args += ["--stack", s]
        return self.call("rehearse", *args)

    def rehearse(self, step: str, *args: Any) -> Dict[str, Any]:
        """One step: ``start``, ``flip``, ``skip``, ``pause``, ``resume``, ``stop``, ``leave``,
        ``answer TEXT``, ``rate again|hard|good|easy``, ``mark easy|medium|difficult``, and in exams
        ``type TEXT``, ``submit``, ``finish``."""
        return self.call("rehearse", step, *args)

    def run(self) -> Optional[Dict[str, Any]]:
        """The run on screen, if any."""
        return ((self.state().get("rehearse") or {}).get("run")) or None

    # -- transfer -----------------------------------------------------------------------------
    def export(
        self, fmt: str, dest: str | Path, stack: Optional[str] = None, scheduling: bool = False
    ) -> Dict[str, Any]:
        """Export as ``json``, ``toml``, ``ron``, ``csv`` or ``markdown`` into the folder ``dest``."""
        args: List[Any] = [fmt, str(dest)]
        if stack:
            args += ["--stack", stack]
        if scheduling:
            args.append("--scheduling")
        return self.call("export", *args)

    def import_file(self, path: str | Path, stack: Optional[str] = None) -> Dict[str, Any]:
        args: List[Any] = [str(path)]
        if stack:
            args += ["--stack", stack]
        return self.call("import", *args)


def connect(
    socket: Optional[str | Path] = None,
    *,
    home: Optional[str | Path] = None,
    start: bool = True,
    vault: Optional[str | Path] = None,
    binary: Optional[str] = None,
    read_only: bool = False,
    allow_destructive: bool = False,
) -> Client:
    """Connect to a running StudyFrog, else (with ``start``) start a headless one.

    The socket is ``socket``, else ``$STUDYFROG_SOCKET`` (scripts run by StudyFrog get one), else
    ``<app home>/run/control.sock``. A started instance opens ``vault`` (else the last one) and
    stops when the client is closed."""
    path = Path(socket) if socket else default_socket(str(home) if home else None)
    if path.exists():
        try:
            return Client(open_socket(path))
        except OSError:
            if socket:
                raise
    if not start:
        raise StudyFrogError(f"no StudyFrog answers on {path}")
    flags = []
    if read_only:
        flags.append("--control-read-only")
    if allow_destructive:
        flags.append("--control-allow-destructive")
    t = start_headless(binary, str(vault) if vault else None, str(home) if home else None, flags)
    return Client(t)
