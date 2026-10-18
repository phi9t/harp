"""Read-only-source reproduction through public CLIs in private real Git repositories."""
import argparse, fcntl, json, os, shlex, sys, traceback
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE / "frozen/tools/collab/tests"))
import test_collab as fixtures

LOG = None
class Trial(fixtures.CollabTests):
    def run_process(self, argv, cwd=None, env=None):
        result = super().run_process(argv, cwd, env)
        LOG.write(json.dumps({"cwd": str(cwd or self.repo), "argv": list(argv),
            "failpoint": (env or self.env).get("COLLAB_FAILPOINT"),
            "returncode": result.returncode, "stdout": result.stdout, "stderr": result.stderr}) + "\n")
        LOG.flush()
        return result

def trial():
    t = Trial(); t.setUp(); t.temp._finalizer.detach()
    return t

REVIEW = ["--review-json", json.dumps({"reviewer": "reproduction", "standards": "passed", "spec": "passed", "criteria": [1]})]
def corenius(t):
    t.init(); t.effort_ticket("01", "one"); t.effort_ticket("02", "two"); t.commit("tickets")
    a=t.open_corenius_attempt(); t.contribute(a); t.cli("submit", a["id"])
    return t.cli("check", a["id"])

def admission(allocation=False):
    t=trial(); a=corenius(t)
    if allocation:
        crash=t.open_corenius_attempt("02", failpoint="after_claim_record", ok=False)
        assert crash.returncode == 86
        status=t.cli("status"); b=next(row for row in status["attempts"] if row["ticket"] == "sample-effort:02")
    remote=t.add_bare_remote()
    crash=t.cli("land", a["id"], *REVIEW, failpoint="after_git_push", ok=False)
    assert crash.returncode == 86
    assert t.remote_head(remote)==a["integration"] and t.git("rev-parse", "HEAD")==a["base"]
    if allocation:
        b=t.cli("recover", b["id"], "--confirm-stopped")
    else:
        b=t.open_corenius_attempt("02")
    competing=t.git("rev-parse", "HEAD")
    assert competing not in (a["base"], a["integration"])
    blocked=t.cli("recover", a["id"], "--confirm-stopped", ok=False)
    assert "Ambiguous pushed landing" in blocked.stderr
    return {"repo": str(t.repo), "attempt": a["id"], "other_attempt": b["id"], "base": a["base"],
        "integration": a["integration"], "local_competing_claim": competing, "remote": t.remote_head(remote),
        "recovery_rc": blocked.returncode, "recovery_error": blocked.stderr}

def lane():
    t=trial(); t.init(); t.effort_ticket(); t.commit("ticket")
    with (t.repo/".git/collab/lane.lock").open("a+") as held:
        fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        a=t.open_corenius_attempt()
        after=t.git("rev-parse", "HEAD")
    return {"repo": str(t.repo), "attempt": a["id"], "state": a["state"], "admission_while_lane_held": True, "after": after}

def git_operation():
    t=trial(); a=t.prepared(); remote=t.add_bare_remote(); a=t.cli("check", a["id"])
    # Cherry-picking an existing commit is a real empty operation, not a fake marker.
    operation=t.run_process(["git", "cherry-pick", a["base"]])
    assert operation.returncode != 0 and (t.repo/".git/CHERRY_PICK_HEAD").exists()
    porcelain=t.git("status", "--porcelain"); assert porcelain == ""
    failed=t.cli("land", a["id"], ok=False)
    assert t.remote_head(remote)==a["integration"] and t.git("rev-parse", "HEAD")==a["base"]
    assert "cherry-pick" in failed.stderr
    return {"repo": str(t.repo), "attempt": a["id"], "porcelain": porcelain, "marker_exists": True,
        "state": t.cli("show", a["id"])["state"], "base": a["base"], "integration": a["integration"],
        "remote_after": t.remote_head(remote), "land_rc": failed.returncode, "land_error": failed.stderr}

def liveness(record):
    t=trial(); t.init(); t.ticket(); t.commit("ticket")
    run=t.root/"run"; run.mkdir(); (run/"run.json").write_text(json.dumps(record))
    a=t.cli("open-attempt", "T-one", "--owner", "worker", "--worker-run-dir", str(run))
    before=t.cli("inspect", a["id"])
    abandoned=t.cli("abandon", a["id"], "--confirm-stopped")
    removed=t.cli("gc", a["id"], "--confirm-stopped")
    assert abandoned["state"] == "abandoned" and not Path(a["path"]).exists()
    return {"repo": str(t.repo), "attempt": a["id"], "record": record, "exit_file_exists": (run/"exit_code").exists(),
        "inspect": before, "abandon_state": abandoned["state"], "gc": removed, "worker_workspace_exists": Path(a["path"]).exists()}

def recovery(records=False):
    t=trial(); a=corenius(t) if records else t.prepared(); remote=t.add_bare_remote()
    if not records: a=t.cli("check", a["id"])
    args=REVIEW if records else []
    if records:
        # Freeze after the attributed bookkeeping merge, leaving its state unfinished.
        crash=t.cli("land", a["id"], *args, failpoint="after_records_land", ok=False)
    else:
        crash=t.cli("land", a["id"], failpoint="after_git_land", ok=False)
    assert crash.returncode == 86
    before=t.git("rev-parse", "HEAD"); state=t.cli("show", a["id"])["state"]
    assert t.git("merge-base", a["integration"], before)==a["integration"]
    remote.rename(remote.with_name("offline.git"))
    failed=t.cli("recover", a["id"], "--confirm-stopped", ok=False)
    assert "Cannot inspect pushed remote" in failed.stderr
    remote.with_name("offline.git").rename(remote)
    recovered=t.cli("recover", a["id"], "--confirm-stopped")
    assert recovered["state"] == "landed"
    return {"repo": str(t.repo), "attempt": a["id"], "integration": a["integration"], "local_landed": before,
        "state_before_recover": state, "offline_recovery_rc": failed.returncode, "offline_recovery_error": failed.stderr,
        "state_after_online_recover": recovered["state"], "local_after": t.git("rev-parse", "HEAD")}

def legacy_admission():
    t=trial()
    t.cli("init", "--trunk", "main", "--check-json", json.dumps([sys.executable,"-c","print('checked')"]),
          "--workspace-root", str(t.root/"workspaces"), "--override-owners-json", '["owner"]')
    t.effort_ticket("01","one"); t.effort_ticket("02","two"); base=t.commit("tickets")
    path=t.root/"topic"; t.git("worktree","add","-b","topic",str(path))
    (path/"code.txt").write_text("legacy contribution\n"); tip=t.commit("legacy candidate",cwd=path)
    remote=t.add_bare_remote()
    command=[sys.executable,str(HERE/"frozen/tools/agents/land.py"),"topic","--repo",str(t.repo),
             "--mainline","main","--owner-mainline-override","owner","--gate","true"]
    crashed=t.run_process(command,env=dict(t.env,COLLAB_FAILPOINT="after_legacy_push"))
    assert crashed.returncode == 86 and t.remote_head(remote)==tip
    journal=next((t.repo/".git/legacy-land").glob("*.json")); op=json.loads(journal.read_text())
    a=t.open_corenius_attempt("02"); competing=t.git("rev-parse","HEAD")
    failed=t.run_process(command+["--recover-landing",op["id"]])
    assert failed.returncode != 0 and "Target moved beyond" in failed.stderr
    return {"repo":str(t.repo),"legacy_operation":op["id"],"base":base,"legacy_tip":tip,
            "remote_after":t.remote_head(remote),"competing_claim":competing,"attempt":a["id"],
            "recovery_rc":failed.returncode,"recovery_error":failed.stderr}

CASES={"legacy-admission": legacy_admission, "admission": lambda: admission(), "allocation": lambda: admission(True), "lane": lane,
    "operation": git_operation, "liveness-empty": lambda: liveness({}),
    "liveness-invalid": lambda: liveness({"pid": "broken", "process_session_id": "broken"}),
    "recovery": recovery, "records": lambda: recovery(True)}
if __name__ == "__main__":
    parser=argparse.ArgumentParser(); parser.add_argument("--case", choices=CASES); args=parser.parse_args()
    names=[args.case] if args.case else list(CASES)
    results=json.loads((HERE/"observations.json").read_text()) if (HERE/"observations.json").exists() else {}
    with (HERE/"commands.jsonl").open("a") as LOG:
        for name in names:
            try:
                results[name]={"reproduced": True, **CASES[name]()}
            except Exception:
                results[name]={"reproduced": False, "error": traceback.format_exc()}
            print(name, json.dumps(results[name]), flush=True)
    (HERE/"observations.json").write_text(json.dumps(results, indent=2)+"\n")
    sys.exit(0 if all(result["reproduced"] for result in results.values()) else 1)
