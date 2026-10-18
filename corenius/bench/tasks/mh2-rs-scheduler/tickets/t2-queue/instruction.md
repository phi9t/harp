Implement the fair ready queue in `src/queue.rs`.

Required behavior:

- `ReadyQueue::push` accepts ready `JobSpec` values in any order.
- `ReadyQueue::pop` normally chooses the highest priority tier, but if the
  previous pop used that same tier and another non-empty tier exists, it must
  pop from the next highest tier. This is the fairness rule.
- Within a priority tier, order jobs by id ascending so dispatch is
  deterministic.
- `drain_ids` repeatedly pops until the queue is empty and returns the popped
  ids.

Also update the implemented component registry in `src/lib.rs` by adding
`queue`. Preserve the top-level canonical ordering invariant; do not use
iteration order from an unordered map.
