"""Shared set-up of the client tests: a scratch application home and vault, and the binary."""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import time
import unittest
from pathlib import Path

BIN = os.environ.get("STUDYFROG_BIN", "")

STACK = "---\nid: 01J00000000000000000000001\nkind: stack\ntitle: French\n---\n"
CARD = "---\nid: 01J00000000000000000000010\nkind: card\ntitle: Chat\n---\ncat\n???\nchat\n"
CARD_ID = "01J00000000000000000000010"


def needs_binary(test: unittest.TestCase) -> None:
    if not BIN or not Path(BIN).exists():
        test.skipTest("STUDYFROG_BIN does not name the studyfrog binary")


class Scratch:
    """A temporary home and vault, removed again by ``close``."""

    def __init__(self) -> None:
        self.root = Path(tempfile.mkdtemp(prefix="sfpy-"))
        self.home = self.root / "home"
        self.vault = self.root / "vault"
        (self.vault / "fr").mkdir(parents=True)
        (self.vault / "fr" / "_stack.md").write_text(STACK)
        (self.vault / "fr" / "a.md").write_text(CARD)
        self.home.mkdir()

    def close(self) -> None:
        shutil.rmtree(self.root, ignore_errors=True)


def serve(scratch: Scratch, *flags: str) -> subprocess.Popen:
    """A headless instance serving ``run/control.sock`` in the scratch home."""
    env = dict(os.environ, STUDYFROG_HOME=str(scratch.home))
    proc = subprocess.Popen(
        [BIN, "--control-stdio", "--vault", str(scratch.vault), *flags],
        stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env,
    )
    meta = scratch.home / "run" / "control.json"
    for _ in range(400):
        if meta.exists():
            return proc
        time.sleep(0.05)
    proc.kill()
    raise AssertionError("the headless instance did not publish its socket")


def stop(proc: subprocess.Popen) -> None:
    assert proc.stdin is not None
    proc.stdin.close()
    proc.wait(timeout=30)
