# Triage roles

Use the Matt Pocock skillset's default role names as `Status:` values:

| Role | Meaning |
| --- | --- |
| `needs-triage` | Requires scope or priority assessment |
| `needs-info` | Missing information prevents specification |
| `ready-for-agent` | Specified and ready for agent execution |
| `ready-for-human` | Requires human implementation |
| `wontfix` | Intentionally declined, with a recorded reason |

Execution states and optional wayfinding states are defined in [the tracker contract](issue-tracker.md). A ready status does not grant authority beyond the user's request.
