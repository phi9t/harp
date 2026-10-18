import unittest
from datetime import datetime, timezone
from operational_state import check_projection


class ProjectionTests(unittest.TestCase):
    def test_retired_worktree_is_not_current(self):
        self.assertIn("unregistered worktree", " ".join(check_projection({"worktree": "/gone"}, {"/main"})))

    def test_registered_worktree_is_current(self):
        self.assertEqual([], check_projection({"worktree": "/main"}, {"/main"}))

    def test_historical_pointer_is_allowed_when_quiescent(self):
        self.assertEqual([], check_projection({"execution_state": "quiescent", "worktree": None, "history": "history/old.json"}, {"/main"}))

    def test_quiescence_cannot_carry_launch_command(self):
        self.assertTrue(check_projection({"execution_state": "quiescent", "next_launch": {"command": "start"}}, set()))

    def test_quiescence_cannot_carry_writer(self):
        self.assertTrue(check_projection({"execution_state": "quiescent", "writer": "actor"}, set()))

    def test_expired_active_claim_fails(self):
        now = datetime(2026, 10, 7, tzinfo=timezone.utc)
        claim = {"state": "active", "expires": "2026-10-05T19:11:38Z"}
        self.assertIn("active claim has expired", check_projection({"claim": claim}, set(), now))

    def test_historical_claim_does_not_admit_authority(self):
        claim = {"state": "expired historical claim", "expires": "2026-10-05T19:11:38Z"}
        self.assertEqual([], check_projection({"claim": claim}, set()))

    def test_future_active_claim_is_valid(self):
        now = datetime(2026, 10, 7, tzinfo=timezone.utc)
        claim = {"state": "active", "expires": "2026-10-08T19:11:38Z"}
        self.assertEqual([], check_projection({"claim": claim}, set(), now))


if __name__ == "__main__":
    unittest.main()
