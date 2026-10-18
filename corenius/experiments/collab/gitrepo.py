"""A throwaway REAL git repository driven mostly through plumbing.

* Objects (blobs/trees/commits) are written as genuine loose git objects, with
  virtual-time commit timestamps, so commit ids are deterministic.
* Every 3-way merge / rebase step / conflict decision is made by real git:
  `git merge-tree --write-tree --merge-base=<base> <ours> <theirs>` is the very
  operation `git rebase` (ort) performs per replayed commit.
* Refs are moved with `git update-ref <ref> <new> <old>` (compare-and-swap).

File model: f000.txt ... each with H hunk lines `H<k>: <tokens>` separated by
FILL filler lines, so edits to different hunks merge cleanly and edits to the
same hunk line conflict textually, exactly as in real code.
"""
import hashlib
import os
import subprocess
import zlib

EPOCH = 1_700_000_000
FILL = 5
AUTHOR = "sim <sim@example.invalid>"

_ENV = dict(os.environ)
_ENV.update(
    GIT_CONFIG_GLOBAL="/dev/null",
    GIT_CONFIG_NOSYSTEM="1",
    GIT_OPTIONAL_LOCKS="0",
    GIT_TERMINAL_PROMPT="0",
)


def fname(f):
    return "f%03d.txt" % f


_parse_memo = {}


def parse_tokens(text):
    """-> (finished ids, wip ids, recon pairs) found in a file's hunk lines."""
    hit = _parse_memo.get(text)
    if hit is not None:
        return hit
    fin, wip, rec = set(), set(), set()
    for line in text.split("\n"):
        if line[:1] != "H":
            continue
        for tok in line.split()[1:]:
            c = tok[0]
            if c == "t":
                if tok.endswith("~"):
                    wip.add(int(tok[1:-1]))
                else:
                    fin.add(int(tok[1:]))
            elif c == "r":
                a, b = tok[1:].split("-")
                rec.add((int(a), int(b)))
    out = (frozenset(fin), frozenset(wip), frozenset(rec))
    _parse_memo[text] = out
    return out


class GitRepo:
    def __init__(self, path):
        self.path = path
        os.makedirs(path, exist_ok=True)
        self.run("init", "-q", "--initial-branch=main")
        self.run("config", "gc.auto", "0")
        self.run("config", "user.name", "sim")
        self.run("config", "user.email", "sim@example.invalid")
        self._cat = subprocess.Popen(
            ["git", "cat-file", "--batch"], cwd=path, env=_ENV,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        )
        self._mem = {}
        self._merge_memo = {}
        self._tok_memo = {}
        self.merge_calls = 0

    # -- process helpers ---------------------------------------------------
    def run(self, *args, env=None, check=True):
        e = _ENV if env is None else {**_ENV, **env}
        r = subprocess.run(["git", *args], cwd=self.path, env=e,
                           capture_output=True, text=True)
        if check and r.returncode != 0:
            raise RuntimeError("git %s failed: %s%s" % (" ".join(args), r.stdout, r.stderr))
        return r

    def close(self):
        try:
            self._cat.stdin.close()
            self._cat.wait(timeout=5)
        except Exception:
            self._cat.kill()

    # -- object layer ------------------------------------------------------
    def _write(self, typ, data):
        raw = ("%s %d\0" % (typ, len(data))).encode() + data
        oid = hashlib.sha1(raw).hexdigest()
        if oid in self._mem:
            return oid
        d = os.path.join(self.path, ".git", "objects", oid[:2])
        p = os.path.join(d, oid[2:])
        if not os.path.exists(p):
            os.makedirs(d, exist_ok=True)
            tmp = p + ".tmp"
            with open(tmp, "wb") as fh:
                fh.write(zlib.compress(raw))
            os.replace(tmp, p)
        self._mem[oid] = (typ, data)
        return oid

    def _read(self, oid):
        hit = self._mem.get(oid)
        if hit is not None:
            return hit
        self._cat.stdin.write(oid.encode() + b"\n")
        self._cat.stdin.flush()
        hdr = self._cat.stdout.readline().split()
        if len(hdr) < 3:
            raise KeyError(oid)
        data = self._cat.stdout.read(int(hdr[2]))
        self._cat.stdout.read(1)
        self._mem[oid] = (hdr[1].decode(), data)
        return self._mem[oid]

    def write_blob(self, text):
        return self._write("blob", text.encode())

    def blob_text(self, oid):
        return self._read(oid)[1].decode()

    def write_tree(self, entries):
        body = b"".join(
            b"100644 " + name.encode() + b"\0" + bytes.fromhex(oid)
            for name, oid in sorted(entries.items())
        )
        return self._write("tree", body)

    def tree_entries(self, tree):
        data = self._read(tree)[1]
        out, i = {}, 0
        while i < len(data):
            sp = data.index(b" ", i)
            nul = data.index(b"\0", sp)
            out[data[sp + 1:nul].decode()] = data[nul + 1:nul + 21].hex()
            i = nul + 21
        return out

    def write_commit(self, tree, parents, msg, t):
        ts = EPOCH + int(t * 60)
        lines = ["tree " + tree] + ["parent " + p for p in parents]
        lines += ["author %s %d +0000" % (AUTHOR, ts), "committer %s %d +0000" % (AUTHOR, ts), "", msg, ""]
        return self._write("commit", "\n".join(lines).encode())

    def commit_info(self, oid):
        """-> (tree, [parents], msg)"""
        text = self._read(oid)[1].decode()
        head, _, msg = text.partition("\n\n")
        tree, parents = None, []
        for line in head.split("\n"):
            if line.startswith("tree "):
                tree = line[5:]
            elif line.startswith("parent "):
                parents.append(line[7:])
        return tree, parents, msg.rstrip("\n")

    def tokens_of_tree(self, tree):
        hit = self._tok_memo.get(tree)
        if hit is None:
            fin, wip, rec = set(), set(), set()
            for name, boid in self.tree_entries(tree).items():
                a, b, c = parse_tokens(self.blob_text(boid))
                fin |= a
                wip |= b
                rec |= c
            hit = (frozenset(fin), frozenset(wip), frozenset(rec))
            self._tok_memo[tree] = hit
        return hit

    def tokens(self, commit):
        return self.tokens_of_tree(self.commit_info(commit)[0])

    # -- repo construction ---------------------------------------------------
    def init_files(self, n_files, hunks):
        entries = {}
        for f in range(n_files):
            lines = []
            for h in range(hunks):
                lines.append("H%d: -" % h)
                lines += ["# f%d h%d pad%d" % (f, h, j) for j in range(FILL)]
            entries[fname(f)] = self.write_blob("\n".join(lines) + "\n")
        c = self.write_commit(self.write_tree(entries), [], "initial", 0.0)
        self.run("update-ref", "refs/heads/main", c)
        self.run("reset", "-q", "--hard")
        return c

    def edit(self, parent, edits, task_id, recon, wip, t):
        """New commit on `parent` appending this task's token to each (file,hunk)
        line (and recon tokens on the first line)."""
        tree, _, _ = self.commit_info(parent)
        entries = self.tree_entries(tree)
        tok = "t%d%s" % (task_id, "~" if wip else "")
        extra = ["r%d-%d" % p for p in recon]
        for i, (f, h) in enumerate(edits):
            name = fname(f)
            lines = self.blob_text(entries[name]).split("\n")
            for k, line in enumerate(lines):
                if line.startswith("H%d:" % h):
                    have = line.split()[1:]
                    add = [x for x in ([tok] + (extra if i == 0 else [])) if x not in have]
                    lines[k] = " ".join([line] + add)
                    break
            entries[name] = self.write_blob("\n".join(lines))
        return self.write_commit(self.write_tree(entries), [parent], "task %d" % task_id, t)

    # -- merging (real git) --------------------------------------------------
    def merge(self, base, ours, theirs):
        """3-way merge by real git. -> (tree_oid, conflicted_paths)"""
        key = (base, ours, theirs)
        hit = self._merge_memo.get(key)
        if hit is not None:
            return hit
        self.merge_calls += 1
        r = self.run("merge-tree", "--write-tree", "--name-only", "--no-messages",
                     "--merge-base=" + base, ours, theirs, check=False)
        if r.returncode not in (0, 1):
            raise RuntimeError("merge-tree failed: " + r.stderr)
        out = r.stdout.split("\n")
        conflicts = [x for x in out[1:] if x] if r.returncode == 1 else []
        hit = (out[0].strip(), conflicts)
        self._merge_memo[key] = hit
        return hit

    def cherry_pick(self, commit, onto, t):
        """Replay `commit` (single-parent) onto `onto`. -> new commit or None on
        a real text conflict.  Same operation `git rebase` does per commit."""
        _, parents, msg = self.commit_info(commit)
        if parents[0] == onto:
            return commit
        tree, conflicts = self.merge(parents[0], onto, commit)
        if conflicts:
            return None
        return self.write_commit(tree, [onto], msg, t)

    def is_ancestor(self, a, b):
        c = b
        while c is not None:
            if c == a:
                return True
            parents = self.commit_info(c)[1]
            c = parents[0] if parents else None
        return False

    def update_ref(self, new, old):
        self.run("update-ref", "refs/heads/main", new, old)

    def head(self):
        return self.run("rev-parse", "refs/heads/main").stdout.strip()
