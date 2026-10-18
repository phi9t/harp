from strategies.shared_tree import SharedTree
from strategies.worktree_rebase import WorktreeRebaseFF
from strategies.landing_lead import LeadFollower
from strategies.merge_queue import MergeQueue
from strategies.leases import Leases, LeasesLocked
from strategies.patch_stream import PatchStream

STRATEGIES = {c.name: c for c in [
    SharedTree, WorktreeRebaseFF, LeadFollower, MergeQueue, Leases, LeasesLocked, PatchStream,
]}
