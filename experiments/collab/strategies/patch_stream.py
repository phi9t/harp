from strategies.common import BatchLander, Strategy


class PatchStream(BatchLander, Strategy):
    """F: agents stream small patches to a coordinator that waits a short window,
    applies the batch with 3-way merge (a text conflict is auto-resolved by
    re-applying the structured edit with probability auto_p = 0.6, otherwise
    bounced), and gates ONCE per batch of up to 8 (no speculation, S=1)."""
    name = "F_patch_stream"
    S = 1
    K = 8
    window = 8.0
    auto_p = 0.6
