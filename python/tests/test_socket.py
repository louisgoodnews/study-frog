"""The client attached to a running instance's socket, and lifecycle events."""

from __future__ import annotations

import unittest

import studyfrog
from helpers import Scratch, needs_binary, serve, stop


class SocketClientTest(unittest.TestCase):
    def setUp(self) -> None:
        needs_binary(self)
        self.s = Scratch()
        self.server = serve(self.s)

    def tearDown(self) -> None:
        stop(self.server)
        self.s.close()

    def test_two_clients_share_one_instance_and_events_arrive(self) -> None:
        with studyfrog.connect(home=self.s.home, start=False) as listener, studyfrog.connect(
            home=self.s.home, start=False
        ) as writer:
            self.assertEqual(listener.subscribe("item_created"), ["item_created"])
            uid = writer.create_item("note", "Notiz", text="Hallo", stack="fr")
            events = []
            for ev in listener.events(timeout=20):
                events.append(ev)
                break
            self.assertEqual(len(events), 1)
            self.assertEqual(events[0].kind, "item_created")
            self.assertEqual(events[0].data["uid"], uid)
            self.assertEqual(events[0].data["title"], "Notiz")
            self.assertEqual(listener.state()["vault"]["root"], str(self.s.vault))
            self.assertEqual(list(listener.events(timeout=0.2)), [], "nothing else happened")

    def test_unknown_event_kinds_and_a_missing_instance_are_errors(self) -> None:
        with studyfrog.connect(home=self.s.home, start=False) as sf:
            with self.assertRaises(studyfrog.StudyFrogError):
                sf.subscribe("bogus")
        with self.assertRaises(studyfrog.StudyFrogError):
            studyfrog.connect(home=self.s.root / "elsewhere", start=False)


if __name__ == "__main__":
    unittest.main()
