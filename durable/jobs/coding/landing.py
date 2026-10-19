"""Owner-authorized checked-out landing, journaled by the existing SQLite store."""
from __future__ import annotations

from contextlib import contextmanager
import fcntl
import hashlib
import hmac
import json
import os
from pathlib import Path
import secrets
import stat
import tempfile
import time
import uuid

from durable.common import digest, encode, fsync_dir, read_json, verify_input_bundle
from durable.estate.git import git, tracked_tree, unchanged
from durable.intent.policy import acceptance
from durable.authority import execution_roots
from durable.jobs.coding.candidates import capture, check_git_subject, ordinary, VERSION as VERIFICATION_VERSION


class LandingRefused(ValueError):
    pass


class StaleApproval(LandingRefused):
    pass


def _schema(store):
    store.db.executescript("""
    CREATE TABLE IF NOT EXISTS harp_owner (
      id INTEGER PRIMARY KEY CHECK(id=1), path TEXT NOT NULL, device INTEGER NOT NULL,
      inode INTEGER NOT NULL, secret_digest TEXT NOT NULL, exclusions TEXT NOT NULL);
    CREATE TABLE IF NOT EXISTS harp_approvals (
      id TEXT PRIMARY KEY, record TEXT NOT NULL, status TEXT NOT NULL, outcome TEXT);
    CREATE TABLE IF NOT EXISTS harp_landing_intents (
      approval_id TEXT PRIMARY KEY REFERENCES harp_approvals(id),
      created REAL NOT NULL, status TEXT NOT NULL);
    """)


def _path(value):
    path = Path(value)
    if not path.is_absolute() or ".." in path.parts:
        raise LandingRefused("use an absolute path without traversal")
    return ordinary(path)


def _exclusions(store, execution):
    roots = {str(store.root), str(Path(__file__).resolve().parent)}
    if execution is not None:
        execution.verify(require_backend=False)
        config = execution.config
        if config["state"] != str(store.root):
            raise LandingRefused("owner capability requires this project's execution")
        roots.update(execution_roots(execution))
    for row in store.db.execute("SELECT args FROM runs WHERE workflow='coding'"):
        args = json.loads(row[0])
        bundle = verify_input_bundle(store.root / "inputs", args["inputs"])
        request = read_json(bundle / "request.json")
        if "source_repository" in request:
            roots.add(request["source_repository"])
    return sorted(str(Path(root).resolve()) for root in roots)


def _private_capability(path, exclusions):
    path = _path(path)
    if any(path.is_relative_to(Path(root)) for root in exclusions):
        raise LandingRefused("owner capability must remain outside state, source and worker projections")
    parent = path.parent.stat()
    if not stat.S_ISDIR(parent.st_mode) or parent.st_uid != os.getuid() or parent.st_mode & 0o077:
        raise LandingRefused("capability parent must be an owned private directory (0700)")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        info = os.fstat(stream.fileno())
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid()
                or stat.S_IMODE(info.st_mode) != 0o600 or info.st_nlink != 1 or info.st_size > 4096):
            raise LandingRefused("owner capability must be a private, unlinked ordinary file (0600)")
        value = json.loads(stream.read(4097))
    if (not isinstance(value, dict) or set(value) != {"schema", "role", "state", "token"}
            or value["schema"] != 1 or value["role"] != "owner"
            or not isinstance(value["token"], str) or len(value["token"]) != 64
            or any(c not in "0123456789abcdef" for c in value["token"])):
        raise LandingRefused("invalid owner capability; worker/lead credentials cannot approve")
    return info, value


def owner_init(store, path, execution=None):
    """Explicit operator setup; returns a path and public identity, never a token."""
    _schema(store)
    path = _path(path)
    exclusions = _exclusions(store, execution)
    if any(path.is_relative_to(Path(root)) for root in exclusions):
        raise LandingRefused("owner capability must remain outside state, source and worker projections")
    with store.transaction() as db:
        old = db.execute("SELECT * FROM harp_owner WHERE id=1").fetchone()
        if old:
            _authenticate(store, path)
            # A newly selected profile must not expose an existing capability.
            _private_capability(path, exclusions)
            db.execute("UPDATE harp_owner SET exclusions=? WHERE id=1",
                       (encode(sorted(set(exclusions) | set(json.loads(old["exclusions"])))),))
        else:
            parent = path.parent.stat()
            if parent.st_uid != os.getuid() or parent.st_mode & 0o077:
                raise LandingRefused("create an owned private capability parent directory (0700)")
            value = {"schema": 1, "role": "owner", "state": str(store.root), "token": secrets.token_hex(32)}
            fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            with os.fdopen(fd, "wb") as stream:
                stream.write(encode(value).encode())
                stream.flush()
                os.fsync(stream.fileno())
            fsync_dir(path.parent)
            info, _ = _private_capability(path, exclusions)
            db.execute("INSERT INTO harp_owner VALUES(1,?,?,?,?,?)",
                       (str(path), info.st_dev, info.st_ino, digest(value["token"]), encode(exclusions)))
            store.event("OWNER_CAPABILITY_INITIALIZED")
    return {"registered": True, "capability": str(path), "owner_id": _authenticate(store, path)}


def _authenticate(store, path, destination=None):
    row = store.db.execute("SELECT * FROM harp_owner WHERE id=1").fetchone()
    if row is None:
        raise LandingRefused("owner capability is not initialized")
    exclusions = json.loads(row["exclusions"])
    if destination is not None:
        exclusions.append(str(destination))
    info, value = _private_capability(path, exclusions)
    if (str(path) != row["path"] or (info.st_dev, info.st_ino) != (row["device"], row["inode"])
            or value["state"] != str(store.root)
            or not hmac.compare_digest(digest(value["token"]), row["secret_digest"])):
        raise LandingRefused("owner capability identity changed; refusing worker or substituted credentials")
    return row["secret_digest"]


def _verification(store, run_id, verification_id):
    captured = capture(store, run_id)
    run = store.status(verification_id)
    proof = run["result"]
    args = run["args"]
    bundle = verify_input_bundle(store.root / "inputs", captured["inputs"])
    checks = acceptance((bundle / "acceptance.json").read_bytes())["checks"]
    binding = read_json(bundle / "verification.json")
    if (run["workflow"] != "verification" or run["version"] != VERIFICATION_VERSION
            or run["status"] != "SUCCEEDED" or not isinstance(proof, dict)
            or type(proof.get("schema")) is not int or proof["schema"] != 1
            or proof.get("passed") is not True or proof.get("subject") != captured["subject"]
            or proof.get("subject_digest") != captured["subject_digest"]
            or proof.get("inputs") != captured["inputs"] or args.get("inputs") != captured["inputs"]
            or proof.get("checks_digest") != binding["checks_digest"]
            or args.get("checks_digest") != binding["checks_digest"]
            or args.get("subject_digest") != captured["subject_digest"]
            or proof.get("execution_contract") != run["contract"]
            or args.get("execution_contract") != run["contract"]):
        raise LandingRefused("a current successful exact-subject verification is required")
    attempts = [a for a in run["attempts"] if a["id"] == proof.get("verification_attempt")]
    if len(attempts) != 1:
        raise LandingRefused("verification has no authenticated current attempt")
    attempt = attempts[0]
    if len(attempt["id"]) != 32 or any(c not in "0123456789abcdef" for c in attempt["id"]):
        raise LandingRefused("invalid verification attempt identity")
    operations = [o for o in run["operations"] if o["id"] == attempt["operation_id"]]
    if (attempt["status"] != "SUCCEEDED" or not attempt["receipt_digest"] or len(operations) != 1
            or operations[0]["key"] != "verify" or operations[0]["kind"] != "ACTIVITY"
            or operations[0]["status"] != "SUCCEEDED" or operations[0]["active_attempt"] is not None
            or operations[0]["attempt_count"] != attempt["number"]
            or json.loads(operations[0]["result"]) != proof):
        raise LandingRefused("verification is stale or was not accepted by the guardian")
    directory = ordinary(store.root / "attempts" / attempt["id"])
    if str(directory) != attempt["directory"]:
        raise LandingRefused("verification attempt directory changed")
    receipt = read_json(directory / "receipt.json")
    if (digest(receipt) != attempt["receipt_digest"] or receipt.get("result") != proof
            or type(receipt.get("protocol")) is not int or receipt["protocol"] != 1
            or receipt.get("result_digest") != digest(proof) or receipt.get("status") != "SUCCEEDED"
            or receipt.get("cleanup") is not True or receipt.get("started") is not True
            or receipt.get("attempt_id") != attempt["id"]
            or receipt.get("operation_id") != attempt["operation_id"]
            or receipt.get("spec_digest") != attempt["spec_digest"]):
        raise LandingRefused("verification guardian receipt is missing or changed")
    if not isinstance(proof.get("checks"), list) or len(proof["checks"]) != len(checks):
        raise LandingRefused("verification does not contain every required check")
    for index, (configured, outcome) in enumerate(zip(checks, proof["checks"])):
        if (outcome.get("argv") != configured or type(outcome.get("returncode")) is not int
                or outcome["returncode"] != 0 or outcome.get("tracked_source_unchanged") is not True):
            raise LandingRefused("required check failed or its verified source changed")
        for stream in ("stdout", "stderr"):
            log = outcome["logs"][stream]
            if log["path"] != f"checks/{index:03d}.{stream}.log":
                raise LandingRefused("verification log locator changed")
            path = ordinary(directory / "payload" / log["path"])
            fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
            with os.fdopen(fd, "rb") as source:
                info = os.fstat(source.fileno())
                if not stat.S_ISREG(info.st_mode) or info.st_size > 1 << 20:
                    raise LandingRefused("verification log is not a bounded ordinary file")
                actual = hashlib.sha256(source.read((1 << 20) + 1)).hexdigest()
            if info.st_size != log["retained_bytes"] or actual != log["sha256"]:
                raise LandingRefused("verification logs changed after acceptance")
    evidence = {"run_id": verification_id, "attempt_id": attempt["id"],
                "receipt_digest": attempt["receipt_digest"], "proof_digest": digest(proof),
                "execution_contract": run["contract"], "checks_digest": binding["checks_digest"]}
    return captured, evidence


def _destination(path):
    path = _path(path)
    if Path(git(path, "rev-parse", "--show-toplevel").decode().strip()).resolve() != path:
        raise LandingRefused("destination must be its physical Git working-tree root")
    ordinary(path / ".git")
    gitdir = _path(git(path, "rev-parse", "--absolute-git-dir").decode().strip())
    for name in ("HEAD", "index", "config"):
        ordinary(gitdir / name)
    common = Path(git(path, "rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip())
    _path(common)
    branch = git(path, "symbolic-ref", "-q", "HEAD").decode().strip()
    if not branch.startswith("refs/heads/"):
        raise LandingRefused("destination must have an attached local branch")
    identity = {"path": str(path), "branch": branch, "gitdir": str(gitdir), "common": str(common)}
    for prefix, target in (("worktree", path), ("gitdir", gitdir), ("common", common)):
        info = target.stat()
        identity[prefix + "_device"], identity[prefix + "_inode"] = info.st_dev, info.st_ino
    return identity


@contextmanager
def _repository_lock(destination):
    common = Path(destination["common"])
    lock = ordinary(common / "harp-landing.lock")
    directory = os.open(common, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    info = os.fstat(directory)
    if (info.st_dev, info.st_ino) != (destination["common_device"], destination["common_inode"]):
        os.close(directory)
        raise StaleApproval("destination repository was substituted; new approval is required")
    fd = None
    try:
        fd = os.open(lock.name, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600, dir_fd=directory)
        if not stat.S_ISREG(os.fstat(fd).st_mode):
            raise LandingRefused("repository landing lock must be an ordinary file")
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        if fd is not None:
            os.close(fd)
        os.close(directory)


def _clean(path, expected, tree=None):
    _checkout_policy(path)
    head = git(path, "rev-parse", "HEAD^{commit}").decode().strip()
    if head != expected:
        raise StaleApproval("destination advanced; create a new candidate, verification and owner approval")
    flags = git(path, "ls-files", "-v", "-z").split(b"\0")
    if any(entry and entry[:1] != b"H" for entry in flags):
        raise LandingRefused("destination has special index flags or conflicts; inspect it before landing")
    if git(path, "status", "--porcelain=v1", "-z", "--untracked-files=all"):
        raise LandingRefused("destination is dirty; preserve and resolve its changes before retrying")
    index_tree = git(path, "write-tree").decode().strip()
    expected_tree = tree or git(path, "rev-parse", "HEAD^{tree}").decode().strip()
    if index_tree != expected_tree:
        raise LandingRefused("destination index does not match its approved revision")
    if not unchanged(path, tracked_tree(path, expected)):
        raise LandingRefused("destination tracked bytes differ; preserve and resolve changes before retrying")


def _checkout_policy(path):
    """Refuse destination-local transformations before even running Git status.

    Verification used an independent raw checkout. Local filters/attributes can
    otherwise change newly introduced files only after the branch has advanced.
    """
    # Every effective worktree attribute file must have an admitted Git identity.
    # Status omits ignored files, so ask Git for all untracked attribute paths.
    if git(path, "ls-files", "--others", "-z", "--", ":(glob)**/.gitattributes"):
        raise LandingRefused("destination-only attributes are unqualified; preserve and resolve before landing")
    config = {}
    for item in git(path, "config", "--includes", "--null", "--list").split(b"\0"):
        if item:
            key, _, value = item.partition(b"\n")
            config[key.decode().lower()] = value.decode()
    false = {"false", "no", "off", "0"}
    if config.get("core.autocrlf", "false").lower() not in false:
        raise LandingRefused("destination autocrlf is unqualified; configure raw LF checkout before approval/landing")
    if config.get("core.eol", "lf").lower() not in {"lf", "native"}:
        raise LandingRefused("destination eol policy can transform candidate bytes; use LF checkout")
    if config.get("core.symlinks", "true").lower() in false:
        raise LandingRefused("destination must preserve Git symlinks before approval/landing")
    if config.get("core.sparsecheckout", "false").lower() not in false:
        raise LandingRefused("sparse destination checkout is not qualified for physical landing")
    if any(name.startswith("filter.") for name in config):
        raise LandingRefused("destination checkout filters are unqualified; inspect them before approval/landing")
    if config.get("core.attributesfile", os.devnull) != os.devnull:
        raise LandingRefused("destination external attributes are unqualified; inspect them before approval/landing")
    identity = _destination(path)
    for root in {identity["gitdir"], identity["common"]}:
        attributes = ordinary(Path(root) / "info/attributes")
        if not attributes.exists():
            continue
        fd = os.open(attributes, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as source:
            info = os.fstat(source.fileno())
            if not stat.S_ISREG(info.st_mode) or info.st_size > 65536:
                raise LandingRefused("destination info/attributes must be an ordinary bounded file")
            lines = source.read(65537).splitlines()
        if any(line.strip() and not line.lstrip().startswith(b"#") for line in lines):
            raise LandingRefused("destination info/attributes overrides are unqualified; inspect before landing")


def approve(store, run_id, verification_id, destination, capability_path):
    _schema(store)
    owner = _authenticate(store, capability_path, _path(destination))
    captured, evidence = _verification(store, run_id, verification_id)
    identity = _destination(destination)
    with _repository_lock(identity):
        if _destination(destination) != identity:
            raise LandingRefused("destination identity changed")
        _clean(Path(destination), captured["subject"]["base"])
        record = {"schema": 1, "decision": uuid.uuid4().hex, "approved_at": time.time(),
                  "owner_id": owner, "coding_run": run_id, "captured": captured,
                  "verification": evidence, "destination": identity,
                  "expected_base": captured["subject"]["base"]}
        approval_id = "approval-" + digest(record)
        with store.transaction() as db:
            db.execute("INSERT INTO harp_approvals VALUES(?,?,?,NULL)", (approval_id, encode(record), "APPROVED"))
            store.event("OWNER_APPROVED", run_id, approval_id=approval_id,
                        subject_digest=captured["subject_digest"], verification_id=verification_id)
    return {"approval_id": approval_id, **record}


def _collisions(store, captured, destination):
    path = Path(destination["path"])
    tracked = {os.fsdecode(p) for p in git(path, "ls-files", "-z").split(b"\0") if p}
    inputs = verify_input_bundle(store.root / "inputs", captured["inputs"])
    with tempfile.TemporaryDirectory(dir=store.root, prefix="landing-") as temporary:
        repo = Path(temporary) / "candidate.git"
        check_git_subject(inputs / "candidate.bundle", captured["subject"], repo, bare=True)
        names = git(repo, "ls-tree", "-r", "--name-only", "-z", captured["subject"]["commit"])
        for raw in names.split(b"\0"):
            if not raw:
                continue
            name = os.fsdecode(raw)
            target = path / name
            if name not in tracked and (target.exists() or target.is_symlink()):
                raise LandingRefused("untracked or ignored destination path conflicts with candidate: " + name)
            for parent in target.parents:
                if parent == path:
                    break
                if parent.is_symlink() or (parent.exists() and not parent.is_dir()):
                    raise LandingRefused("destination path ancestry conflicts with candidate: " + name)


@contextmanager
def _landing_lock(store, approval_id, record, destination):
    try:
        with _repository_lock(destination):
            yield
    except (ValueError, OSError) as exc:
        reason = str(exc)
        with store.transaction() as db:
            db.execute("UPDATE harp_approvals SET status=?,outcome=? WHERE id=?",
                       ("STALE" if isinstance(exc, StaleApproval) else "REFUSED",
                        encode({"reason": reason}), approval_id))
            store.event("LANDING_REFUSED", record["coding_run"], approval_id=approval_id, reason=reason)
        raise LandingRefused(reason) from exc


def land(store, approval_id, capability_path, failpoint=None):
    _schema(store)
    row = store.db.execute("SELECT * FROM harp_approvals WHERE id=?", (approval_id,)).fetchone()
    if row is None:
        raise LandingRefused("unknown owner approval")
    record = json.loads(row["record"])
    if "approval-" + digest(record) != approval_id:
        raise LandingRefused("owner approval record changed")
    destination = record["destination"]
    owner = _authenticate(store, capability_path, Path(destination["path"]))
    if owner != record["owner_id"]:
        raise LandingRefused("approval belongs to a different owner capability")
    point = failpoint or (lambda _name, _detail=None: None)
    path = Path(destination["path"])
    with _landing_lock(store, approval_id, record, destination):
        if _destination(path) != destination:
            raise StaleApproval("destination was substituted; new owner approval is required")
        captured, evidence = _verification(store, record["coding_run"], record["verification"]["run_id"])
        if captured != record["captured"] or evidence != record["verification"]:
            raise StaleApproval("candidate or verification changed; new owner approval is required")
        status = store.db.execute("SELECT status FROM harp_approvals WHERE id=?", (approval_id,)).fetchone()[0]
        if status == "STALE":
            raise StaleApproval("approval is stale; create a new candidate, verification and approval")
        intent = store.db.execute("SELECT * FROM harp_landing_intents WHERE approval_id=?", (approval_id,)).fetchone()
        candidate = captured["subject"]["commit"]
        head = git(path, "rev-parse", "HEAD^{commit}").decode().strip()
        if intent and intent["status"] == "LANDED" and head != candidate:
            raise StaleApproval("approval was consumed; a different destination revision requires new approval")
        if intent and head == candidate:
            _clean(path, candidate, captured["subject"]["tree"])
            result = {"approval_id": approval_id, "status": "LANDED", "commit": candidate,
                      "destination": str(path), "reconciled": True}
            with store.transaction() as db:
                db.execute("UPDATE harp_approvals SET status='LANDED',outcome=? WHERE id=?", (encode(result), approval_id))
                db.execute("UPDATE harp_landing_intents SET status='LANDED' WHERE approval_id=?", (approval_id,))
                store.event("LANDING_RECONCILED", record["coding_run"], approval_id=approval_id, commit=candidate)
            return result
        _clean(path, record["expected_base"])
        _collisions(store, captured, destination)
        with store.transaction() as db:
            db.execute("INSERT OR IGNORE INTO harp_landing_intents VALUES(?,?,'INTENT')", (approval_id, time.time()))
            db.execute("UPDATE harp_approvals SET status='LANDING' WHERE id=?", (approval_id,))
            store.event("LANDING_INTENT", record["coding_run"], approval_id=approval_id,
                        base=record["expected_base"], commit=candidate)
        point("after_intent", {"approval_id": approval_id})
        # The transaction and repository lock serialize this owner operation.
        # The earlier committed intent survives either process or acknowledgment loss.
        with store.transaction() as db:
            _authenticate(store, capability_path, path)
            if _destination(path) != destination:
                raise LandingRefused("destination identity changed after landing intent")
            _clean(path, record["expected_base"])
            inputs = verify_input_bundle(store.root / "inputs", captured["inputs"])
            git(path, "-c", "gc.auto=0", "-c", "maintenance.auto=false", "fetch", "--no-tags", "--no-write-fetch-head",
                str(inputs / "candidate.bundle"), "HEAD")
            git(path, "merge", "--ff-only", "--no-squash", "--no-edit", "--no-autostash", "--no-overwrite-ignore", candidate)
            point("after_git", {"approval_id": approval_id})
            _clean(path, candidate, captured["subject"]["tree"])
            result = {"approval_id": approval_id, "status": "LANDED", "commit": candidate,
                      "destination": str(path), "reconciled": False}
            db.execute("UPDATE harp_approvals SET status='LANDED',outcome=? WHERE id=?", (encode(result), approval_id))
            db.execute("UPDATE harp_landing_intents SET status='LANDED' WHERE approval_id=?", (approval_id,))
            store.event("LANDING_COMPLETED", record["coding_run"], approval_id=approval_id, commit=candidate)
        return result
