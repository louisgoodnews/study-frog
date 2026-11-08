"""The client over the standard streams of a headless instance it starts itself."""

from __future__ import annotations

import zipfile
import unittest

import studyfrog
from helpers import BIN, CARD_ID, Scratch, needs_binary


class StdioClientTest(unittest.TestCase):
    def setUp(self) -> None:
        needs_binary(self)
        self.s = Scratch()
        self.sf = studyfrog.connect(
            home=self.s.home, vault=self.s.vault, binary=BIN
        )

    def tearDown(self) -> None:
        self.sf.close()
        self.s.close()

    def test_state_lists_the_vault_and_its_stacks(self) -> None:
        state = self.sf.state()
        self.assertEqual(state["vault"]["root"], str(self.s.vault))
        self.assertEqual([s["title"] for s in self.sf.stacks()], ["French"])

    def test_items_are_created_queried_edited_and_moved(self) -> None:
        uid = self.sf.create_item("card", "Hund", text="dog", answer="der Hund", tags=["animals"], stack="fr")
        self.assertIsNotNone(uid)
        self.assertEqual(len(uid), 26)
        cards = self.sf.query("type = card order by title")
        self.assertEqual([c.title for c in cards], ["Chat", "Hund"])
        hund = next(c for c in cards if c.title == "Hund")
        self.assertEqual((hund.uid, hund.kind, hund.tags), (uid, "card", ["animals"]))
        self.sf.edit_item(CARD_ID, "back", "le chat")
        self.assertIn("le chat", (self.s.vault / "fr" / "a.md").read_text())
        page = self.sf.open_item(CARD_ID)["editor"]
        self.assertEqual(page["uid"] if "uid" in page else CARD_ID, CARD_ID)
        self.sf.create_stack("Bio")
        self.sf.move_item(CARD_ID, "bio")
        self.assertTrue((self.s.vault / "bio" / "a.md").exists())

    def test_refused_and_invalid_requests_raise(self) -> None:
        with self.assertRaises(studyfrog.StudyFrogError) as e:
            self.sf.delete_item(CARD_ID)
        self.assertIn("destructive", str(e.exception))
        with self.assertRaises(studyfrog.StudyFrogError):
            self.sf.query("type = = card")
        with self.assertRaises(studyfrog.StudyFrogError):
            self.sf.call("nonsense")
        self.assertEqual(self.sf.nav("calendar")["screen"], "calendar")

    def test_a_rehearsal_is_set_up_stepped_and_finished(self) -> None:
        setup = self.sf.rehearse_setup(preset="all", stacks=["fr"])
        self.assertEqual(setup["rehearse"]["preview"], {"status": "count", "items": 1})
        self.sf.rehearse("start")
        self.assertEqual(self.sf.rehearse("flip")["rehearse"]["run"]["answer"], "chat")
        self.sf.rehearse("rate", "good")
        self.assertEqual(self.sf.run()["state"], "finished")

    def test_export_writes_a_file_and_logs_are_read(self) -> None:
        out = self.s.root / "out"
        out.mkdir()
        self.sf.export("json", out)
        files = list(out.iterdir())
        self.assertEqual(len(files), 1, files)
        with zipfile.ZipFile(files[0]) as z:
            text = " ".join(z.read(n).decode("utf-8") for n in z.namelist() if n.endswith(".json"))
        self.assertIn("Chat", text)
        with self.assertRaises(studyfrog.StudyFrogError):
            self.sf.call("item", "edit", "NO_SUCH_1", "title", "x")
        entries = self.sf.logs(tail=5, level="warn")
        self.assertTrue(entries)
        self.assertEqual(entries[-1].source, "control")


if __name__ == "__main__":
    unittest.main()
