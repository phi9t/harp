"""Public CLI proof that push recovery belongs to the current landing operation."""
import unittest

import test_collab as fixtures


class LandingOperationRepairTests(unittest.TestCase):
    setUp = fixtures.CollabTests.setUp
    run_process = fixtures.CollabTests.run_process
    git = fixtures.CollabTests.git
    commit = fixtures.CollabTests.commit
    cli = fixtures.CollabTests.cli
    init = fixtures.CollabTests.init
    ticket = fixtures.CollabTests.ticket
    claim = fixtures.CollabTests.claim
    contribute = fixtures.CollabTests.contribute
    prepared = fixtures.CollabTests.prepared
    add_bare_remote = fixtures.CollabTests.add_bare_remote
    remote_head = fixtures.CollabTests.remote_head

    def assert_rejected_push_does_not_poison_retry(self, no_push):
        attempt = self.prepared()
        remote = self.add_bare_remote()
        checked = self.cli("check", attempt["id"])
        hook = remote / "hooks/pre-receive"
        hook.write_text("#!/bin/sh\necho operation-rejected >&2\nexit 1\n")
        hook.chmod(0o700)
        rejected = self.cli("land", attempt["id"], ok=False)
        self.assertIn("operation-rejected", rejected.stderr)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "verified")
        hook.unlink()
        options = ("--no-push",) if no_push else ()
        crashed = self.cli("land", attempt["id"], *options, failpoint="before_git_land", ok=False)
        self.assertEqual(crashed.returncode, 86)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "landing_prepared")
        recovered = self.cli("recover", attempt["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "verified")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])
        self.assertEqual(self.remote_head(remote), checked["base"])
        landed = self.cli("land", attempt["id"], *options)
        self.assertEqual(landed["state"], "landed")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["integration"])
        expected_remote = checked["base"] if no_push else checked["integration"]
        self.assertEqual(self.remote_head(remote), expected_remote)
        self.assertEqual((self.repo / "code.txt").read_text(), "contribution\n")
        self.assertEqual(self.git("rev-parse", "refs/collab/landed/" + attempt["id"]), checked["integration"])

    def test_rejected_push_then_interrupted_local_only_retry_restores_verified(self):
        self.assert_rejected_push_does_not_poison_retry(no_push=True)

    def test_rejected_push_then_interrupted_pushing_retry_restores_verified(self):
        self.assert_rejected_push_does_not_poison_retry(no_push=False)

    def test_retry_recovers_its_new_destination_after_the_new_push(self):
        attempt = self.prepared()
        remote = self.add_bare_remote()
        checked = self.cli("check", attempt["id"])
        hook = remote / "hooks/pre-receive"
        hook.write_text("#!/bin/sh\nexit 1\n")
        hook.chmod(0o700)
        self.cli("land", attempt["id"], ok=False)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "verified")
        hook.unlink()
        self.git("--git-dir", str(remote), "update-ref", "refs/heads/staging", checked["base"])
        self.git("config", "branch.main.merge", "refs/heads/staging")
        crashed = self.cli("land", attempt["id"], failpoint="after_git_push", ok=False)
        self.assertEqual(crashed.returncode, 86)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "push_completed")
        recovered = self.cli("recover", attempt["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "landed")
        self.assertEqual(self.remote_head(remote), checked["base"])
        self.assertEqual(self.remote_head(remote, "refs/heads/staging"), checked["integration"])
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["integration"])


if __name__ == "__main__":
    unittest.main()
