from __future__ import annotations

import json
import os
import sys
import tempfile
import threading
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import state  # noqa: E402
import stream  # noqa: E402

CMD_START = '{"type":"item.started","item":{"id":"i6","type":"command_execution","command":"rg -n \\"OrderId\\" src Cargo.toml","aggregated_output":"","exit_code":null,"status":"in_progress"}}'
CMD_DONE = '{"type":"item.completed","item":{"id":"i6","type":"command_execution","command":"rg","aggregated_output":"src/lib.rs:1:use std::fmt;\\nsrc/lib.rs:4:pub struct OrderId(u32);\\n","exit_code":0,"status":"completed"}}'
CMD_FAIL = '{"type":"item.completed","item":{"id":"i7","type":"command_execution","command":"false","aggregated_output":"","exit_code":1,"status":"completed"}}'
EDIT = '{"type":"item.completed","item":{"id":"i11","type":"file_change","changes":[{"path":"/workspace/src/lib.rs","kind":"update"}],"status":"completed"}}'
MSG = '{"type":"item.completed","item":{"id":"i5","type":"agent_message","text":"I will inspect the crate layout\\nthen fix it."}}'
THINK = '{"type":"item.completed","item":{"id":"i4","type":"reasoning","text":"**Planning code modifications**\\n\\nI need to look."}}'
ERR = '{"type":"item.completed","item":{"id":"i0","type":"error","message":"plugin repo not reachable"}}'
TURN = '{"type":"turn.completed","usage":{"input_tokens":46299,"cached_input_tokens":42752,"output_tokens":2548}}'


class StateTest(unittest.TestCase):
    def test_update_merges_and_stamps(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "u" / "state.json"
            state.update(p, phase="queued", unit="u1")
            data = state.update(p, phase="agent", pane_id="%3")
            self.assertEqual((data["unit"], data["phase"], data["pane_id"]), ("u1", "agent", "%3"))
            self.assertIn("updated", data)

    def test_readers_never_see_a_partial_file_under_concurrent_writes(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "state.json"
            state.write_atomic(p, {"n": 0})
            stop, bad = threading.Event(), []

            def writer():
                i = 0
                while not stop.is_set():
                    state.write_atomic(p, {"n": i, "pad": "x" * 5000})
                    i += 1

            t = threading.Thread(target=writer)
            t.start()
            try:
                for _ in range(400):
                    if state.read(p) is None:
                        bad.append(1)
            finally:
                stop.set()
                t.join()
            self.assertEqual(bad, [])

    def test_read_of_missing_or_corrupt_is_none(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertIsNone(state.read(Path(d) / "nope.json"))
            (Path(d) / "bad.json").write_text("{not json")
            self.assertIsNone(state.read(Path(d) / "bad.json"))

    def test_collect_snapshot_sorted_and_tolerant(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            state.update(state.run_state_path(root, "r2"), run_id="r2", created=2)
            state.update(state.run_state_path(root, "r1"), run_id="r1", created=1)
            state.update(state.unit_state_path(root, "u1"), unit="u1", created=1)
            (root / "units" / "broken").mkdir(parents=True)
            (root / "units" / "broken" / "state.json").write_text("{")
            snap = state.collect(root)
            self.assertEqual([r["run_id"] for r in snap["runs"]], ["r1", "r2"])
            self.assertEqual([u["unit"] for u in snap["units"]], ["u1"])


class StreamTest(unittest.TestCase):
    def fe(self, line, **kw):
        return stream.format_event(line, color=False, **kw)

    def test_command_lifecycle(self):
        text, up = self.fe(CMD_START)
        self.assertTrue(text.startswith("$ rg -n"))
        self.assertTrue(up["last"].startswith("$ rg"))
        text, up = self.fe(CMD_DONE)
        self.assertIn("rc=0", text)
        self.assertEqual(up["commands_add"], 1)
        text, _ = self.fe(CMD_FAIL)
        self.assertIn("rc=1", text)

    def test_edit_message_reasoning_error_and_usage(self):
        self.assertIn("/workspace/src/lib.rs", self.fe(EDIT)[0])
        self.assertEqual(self.fe(EDIT)[1]["files_add"], 1)
        self.assertTrue(self.fe(MSG)[0].startswith("» I will inspect"))
        self.assertIn("Planning code modifications", self.fe(THINK)[0])
        self.assertIn("plugin repo not reachable", self.fe(ERR)[0])
        self.assertEqual(self.fe(ERR)[1]["errors_add"], 1)
        text, up = self.fe(TURN)
        self.assertEqual((up["tokens_in_add"], up["tokens_out_add"]), (46299, 2548))

    def test_noise_and_non_json(self):
        self.assertEqual(self.fe("")[0], None)
        self.assertEqual(self.fe('{"type":"item.started","item":{"type":"file_change","changes":[]}}')[0], None)
        text, up = self.fe("stderr: something odd")
        self.assertEqual(text, "stderr: something odd")
        self.assertEqual(self.fe("[1,2,3]")[0], "[1,2,3]")

    def test_merge_update_accumulates(self):
        s = {}
        for line in (CMD_DONE, CMD_DONE, TURN, TURN, EDIT):
            s = stream.merge_update(s, self.fe(line)[1])
        self.assertEqual((s["commands"], s["tokens_out"], s["tokens_in"], s["files"]), (2, 5096, 92598, 1))

    def test_color_and_clipping(self):
        colored, _ = stream.format_event(CMD_FAIL, color=True)
        self.assertIn("\033[31m", colored)
        long = json.dumps({"type": "item.completed", "item": {"type": "agent_message", "text": "word " * 200}})
        text, _ = stream.format_event(long, color=False, width=60)
        self.assertLessEqual(len(text), 62)


if __name__ == "__main__":
    unittest.main()
