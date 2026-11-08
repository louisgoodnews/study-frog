"""The package needs nothing but the standard library and exports what the README documents."""

from __future__ import annotations

import ast
import sys
import unittest
from pathlib import Path

import studyfrog

PACKAGE = Path(studyfrog.__file__).parent


class PackageTest(unittest.TestCase):
    def test_only_the_standard_library_is_imported(self) -> None:
        for path in PACKAGE.glob("*.py"):
            tree = ast.parse(path.read_text(), str(path))
            for node in ast.walk(tree):
                names = []
                if isinstance(node, ast.Import):
                    names = [a.name for a in node.names]
                elif isinstance(node, ast.ImportFrom) and node.level == 0:
                    names = [node.module or ""]
                for name in names:
                    top = name.split(".")[0]
                    self.assertIn(top, sys.stdlib_module_names, f"{path.name} imports {name}")

    def test_the_documented_names_are_exported(self) -> None:
        for name in ["connect", "Client", "Item", "Event", "LogEntry", "StudyFrogError", "trigger_event"]:
            self.assertTrue(hasattr(studyfrog, name), name)

    def test_the_trigger_event_is_read_from_the_environment(self) -> None:
        import os

        os.environ["STUDYFROG_EVENT"] = '{"kind": "item_created", "at_ms": 5, "data": {"title": "T"}}'
        try:
            ev = studyfrog.trigger_event()
        finally:
            del os.environ["STUDYFROG_EVENT"]
        self.assertEqual((ev.kind, ev.at_ms, ev.data["title"]), ("item_created", 5, "T"))
        self.assertIsNone(studyfrog.trigger_event())


if __name__ == "__main__":
    unittest.main()
