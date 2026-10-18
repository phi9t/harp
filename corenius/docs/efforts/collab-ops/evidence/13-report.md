# Ticket 13: manual sweep skill

Status: implemented; independent acceptance remains with the lead.

The user-invoked `.claude/skills/sweep-collab/SKILL.md` sits alongside run-worker.
It invokes the CLI with the host temporary-root wrapper, presents the dry-run
candidates and reasons, then applies the same scope only with explicit user
authorization. It reports removed, archived, and skipped results from the CLI.
It introduces no cleanup logic or separate eligibility rules.

The writing-for-agents, skill mechanics, and skill-creator instructions were read.
Manual inspection confirmed YAML frontmatter, discoverability, invocation choice,
and the default dry-run command. The specification explicitly excludes tests
of the skill itself; its CLI is covered by the eight public cleanup tests.
