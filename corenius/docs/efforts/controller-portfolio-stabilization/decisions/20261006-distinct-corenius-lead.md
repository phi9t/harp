# Give Corenius a distinct remote Lead

Recorded: 2026-10-06

The owner selected a distinct Corenius Lead rather than reusing or replacing the
current DevX/Insula implementation Lead. The Corenius Lead runs GPT-5.5 TraeCLI
on the B200 host inside the qualified developer Insula. It receives its own
RoleEpoch, provider-session identity, Corenius worktree and named tmux target.

This separation prevents infrastructure implementation context and authority
from leaking into controller-portfolio decisions. A terminal or provider loss
does not change the logical role; replacement requires a new recorded binding
and must fence the prior RoleEpoch before work resumes.
