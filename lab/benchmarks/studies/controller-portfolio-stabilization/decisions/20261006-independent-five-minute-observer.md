# Monitor autonomous work with an independent Observer

Recorded: 2026-10-06

The owner requires an independent read-only Observer while the remote Lead and
workers operate autonomously. It samples controller events, process activity,
artifact freshness and repository progress every five minutes. Two consecutive
intervals without meaningful progress, or an explicit `STUCK` signal, alert the
Director.

Lead and workers emit `STUCK: <exact blocker>; NEED: <specific help>` rather
than waiting silently. The Observer can report and preserve evidence, but cannot
steer, kill, restart, accept or land work. Bootstrap may use a Luna monitoring
agent; I2 must eventually own the durable observations and attention state.
