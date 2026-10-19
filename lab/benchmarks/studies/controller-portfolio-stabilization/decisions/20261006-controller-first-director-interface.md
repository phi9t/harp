# Use the controller as the Director interface

Recorded: 2026-10-06

The owner selected I2's authenticated command and mailbox interface as the sole
target authority between the Director and remote Lead. Direction, delivery and
acknowledgement, attention, progress, evidence and replacement fencing must be
recorded there. This keeps authority independent of terminal attachment and
provider UI state.

DevX typed SSH and tmux control remain the bounded bootstrap, observation and
emergency path until the controller interface is qualified. They must not form
a mirrored command authority: bootstrap input is recorded or reconciled into
the controller before autonomous execution relies on it.
