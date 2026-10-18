from strategies.common import BatchLander, Strategy


class MergeQueue(BatchLander, Strategy):
    """D: Zuul/bors-style queue.  Up to S=3 cumulative batches of K=3 changes are
    tested speculatively in parallel (each stacked on its predecessors); failing
    batches are bisected.  No text auto-resolution: conflicts bounce."""
    name = "D_merge_queue"
    S = 3
    K = 3
    window = 0.0
    auto_p = 0.0
