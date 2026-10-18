import os

from gitrepo import EPOCH, fname, parse_tokens
from strategies.common import Strategy

WIP_RETRY = 3.0  # minutes an agent waits before re-running a gate broken by someone's half-edit


class SharedTree(Strategy):
    """A (baseline): every agent edits ONE real checkout.  Edits appear in the
    tree the moment an agent starts (as half-finished `t<id>~`), the gate runs
    on whatever is on disk, and `git add -A && git commit` sweeps in anyone's
    uncommitted work.  No isolation, no rebase, no merge."""
    name = "A_shared_tree"

    def start(self):
        w = self.w
        self.dir = w.repo.path
        for a in range(w.scn.n_agents):
            w.spawn(self.agent(a))

    # -- the working tree (real files on disk) --------------------------------
    def _tokens(self):
        fin, wip, rec = set(), set(), set()
        for name in sorted(os.listdir(self.dir)):
            if name.startswith("f") and name.endswith(".txt"):
                with open(os.path.join(self.dir, name)) as fh:
                    a, b, c = parse_tokens(fh.read())
                fin |= a
                wip |= b
                rec |= c
        return frozenset(fin), frozenset(wip), frozenset(rec)

    def _edit_line(self, f, h, fn):
        path = os.path.join(self.dir, fname(f))
        with open(path) as fh:
            lines = fh.read().split("\n")
        for k, line in enumerate(lines):
            if line.startswith("H%d:" % h):
                lines[k] = fn(line)
                break
        with open(path, "w") as fh:
            fh.write("\n".join(lines))

    def _write_edit(self, t):
        for f, h in t.edits:
            self._edit_line(f, h, lambda l: l + " t%d~" % t.id)

    def _finish_edit(self, t):
        for f, h in t.edits:
            self._edit_line(f, h, lambda l: l.replace(" t%d~" % t.id, " t%d" % t.id))

    def _recon(self, t, pairs):
        f, h = t.edits[0]
        for p in pairs:
            self._edit_line(f, h, lambda l: l if (" r%d-%d" % p) in l else l + " r%d-%d" % p)

    def _commit(self, t):
        w = self.w
        head_tok = w.tokens(w.main)
        disk = self._tokens()
        if (disk[0] | disk[1]) - (head_tok[0] | head_tok[1]) - {t.id}:
            w.swept += 1
        env = {"GIT_AUTHOR_DATE": "%d +0000" % (EPOCH + int(w.now * 60)),
               "GIT_COMMITTER_DATE": "%d +0000" % (EPOCH + int(w.now * 60))}
        w.repo.run("add", "-A")
        if w.repo.run("diff", "--cached", "--quiet", check=False).returncode == 0:
            return
        w.repo.run("commit", "-q", "-m", "task %d" % t.id, env=env)
        w.land(w.repo.head(), moved=True)

    def agent(self, a):
        w = self.w
        yield w.sim.timeout(w.offsets[a])
        for t in w.agent_tasks[a]:
            w.mark_start(t)
            self._write_edit(t)
            yield from w.work(a, t.edit_dur)
            self._finish_edit(t)
            w.mark_ready(t)
            while t.id not in w.landed:
                res = yield from w.gate(self._tokens, [t.id], t.id)
                if t.id in w.landed:      # someone's `git add -A` already committed it
                    break
                if res.ok:
                    self._commit(t)
                    res.mark_useful()
                elif res.reason == "semantic":
                    yield from w.work(a, w.rework_dur(t))
                    self._recon(t, res.pairs)
                else:                      # half-edited file broke the build: wait, retry
                    yield w.sim.timeout(WIP_RETRY)
