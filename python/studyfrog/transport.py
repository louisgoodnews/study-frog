"""Line transports to a StudyFrog instance: its Unix control socket, or the standard streams of a
headless instance started as a child process (``studyfrog --control-stdio``)."""

from __future__ import annotations

import os
import select
import socket
import subprocess
from pathlib import Path
from typing import Optional, Sequence

__all__ = [
    "Transport",
    "app_home",
    "default_socket",
    "open_socket",
    "start_headless",
]


class Transport:
    """One connection: write request lines, read answer and event lines (with an optional timeout)."""

    def __init__(self, fileno: int, recv, send, closer=None, process=None) -> None:
        self._fileno = fileno
        self._recv = recv
        self._send = send
        self._closer = closer
        self._buf = bytearray()
        self.process: Optional[subprocess.Popen] = process

    def send(self, line: str) -> None:
        self._send(line.encode("utf-8") + b"\n")

    def readline(self, timeout: Optional[float] = None) -> Optional[str]:
        """The next line without its line break; ``None`` when the other side is gone. Raises
        ``TimeoutError`` when no whole line arrived within ``timeout`` seconds (the connection stays
        usable)."""
        while b"\n" not in self._buf:
            ready, _, _ = select.select([self._fileno], [], [], timeout)
            if not ready:
                raise TimeoutError("no answer from StudyFrog in time")
            chunk = self._recv(65536)
            if not chunk:
                return None
            self._buf.extend(chunk)
        line, _, rest = bytes(self._buf).partition(b"\n")
        self._buf = bytearray(rest)
        return line.decode("utf-8").rstrip("\r")

    def close(self) -> None:
        if self._closer is not None:
            self._closer()
        if self.process is not None:
            try:
                self.process.wait(timeout=30)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()


def app_home(home: Optional[str] = None) -> Path:
    """The application home: ``home``, ``$STUDYFROG_HOME``, ``$XDG_DATA_HOME/studyfrog`` or
    ``~/.local/share/studyfrog`` (the same order as the application)."""
    for value in (home, os.environ.get("STUDYFROG_HOME")):
        if value:
            return Path(value)
    data = os.environ.get("XDG_DATA_HOME")
    if data:
        return Path(data) / "studyfrog"
    return Path.home() / ".local" / "share" / "studyfrog"


def default_socket(home: Optional[str] = None) -> Path:
    """``$STUDYFROG_SOCKET`` (set for scripts run by StudyFrog), else ``<app home>/run/control.sock``."""
    named = os.environ.get("STUDYFROG_SOCKET")
    if named and home is None:
        return Path(named)
    return app_home(home) / "run" / "control.sock"


def open_socket(path: Path, timeout: Optional[float] = 10.0) -> Transport:
    """Connect to a control socket; raises ``OSError`` when nothing answers there."""
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.settimeout(timeout)
    try:
        sock.connect(str(path))
    except OSError:
        sock.close()
        raise
    sock.settimeout(None)
    return Transport(sock.fileno(), sock.recv, sock.sendall, closer=sock.close)


def start_headless(
    binary: Optional[str] = None,
    vault: Optional[str] = None,
    home: Optional[str] = None,
    flags: Sequence[str] = (),
) -> Transport:
    """Start ``studyfrog --control-stdio --no-control-socket`` and talk over its standard streams.
    The binary is ``binary``, ``$STUDYFROG_BIN`` or ``studyfrog`` on the ``PATH``."""
    exe = binary or os.environ.get("STUDYFROG_BIN") or "studyfrog"
    args = [exe, "--control-stdio", "--no-control-socket", *flags]
    if vault:
        args += ["--vault", str(vault)]
    env = dict(os.environ)
    if home:
        env["STUDYFROG_HOME"] = str(home)
    proc = subprocess.Popen(
        args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=env
    )
    assert proc.stdin is not None and proc.stdout is not None
    stdin, stdout = proc.stdin, proc.stdout

    def send(data: bytes) -> None:
        stdin.write(data)
        stdin.flush()

    def closer() -> None:
        for stream in (stdin, stdout):
            try:
                stream.close()
            except OSError:
                pass

    fd = stdout.fileno()
    return Transport(fd, lambda n: os.read(fd, n), send, closer=closer, process=proc)
