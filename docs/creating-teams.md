# Creating a team

Governance stores teams as `data/teams/<slug>.toml`. Reference existing teams for samples, and see [schemas/team.schema.json](schemas/team.schema.json) for the complete specification.

After merge, the `Apply` step will automatically configure everything specified. If `Apply` fails, it's likely because you tried to declare pre-existing resources. Ask someone in DevOps to manually import them into governance for you.

## Example team file

```toml
# data/teams/example-team.toml

[team]
# Human-readable title
name = "Example Team"

# Stable id for this team matching the filename
slug = "example-team"

# Description for documentation purposes (optional)
description = "Demonstrates common team.toml fields."

# Codeberg usernames corresponding to Tech Leads
# These are added to the "admin" subgroup under the team group in Keycloak,
# as well as given maintainer access on repositories belonging to the team
leads = ["lead1-cb", "lead2-cb"]

# Codeberg usernames for team members; mutually exclusive with the `leads` array
members = ["member1-cb"]

# Repositories belonging to the team. Each `name` must be unique across the whole org.
# Repos are created on Codeberg (Forgejo) and mirrored to GitHub.
[[team.repos]]
name = "example-app"
# Repo description
description = "Main application for Example Team."
# Repo visibility (optional, default: public)
visibility = "private"
# Repo topic labels (optional)
topics = ["web", "demo"]
# Set to true if this project uses Kennel
kennel = true
# Set to true if this project uses observability
sentry = true

[[team.repos]]
name = "example-lib"
description = "Shared library; uses org default visibility when `visibility` is omitted."

# Discord / Slack channels scoped to this team; governance adds members to these channels via Keycloak-linked ids.
[[team.channels]]
# Discord channel ID (optional)
discord = "1234567890123456789"
# Slack channel ID (optional)
slack = "C0123456789"

# Sub-projects: separate repos, channels, and membership under the same team. Check the heading below.
[[team.projects]]
# . . .
# . . .

[[team.projects.repos]]
# . . .
# . . .

[[team.projects.channels]]
# . . .
# . . .
```

After this, open a pull request on Codeberg. Everyone you list in this file must have the required accounts linked. See [docs/adding-yourself-to-a-team.md](docs/adding-yourself-to-a-team.md).

## `team.projects` Note

Note that this system is defined recursively. You can repeatedly add `.projects` to these headers and each `team.projects` has all the headers available for what `team` generally has.

**Permissions:** creating new teams is restricted to the Tech Director and Tech Leads already declared in governance.
