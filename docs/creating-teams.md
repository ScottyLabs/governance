# Creating a team

Governance stores teams as `data/teams/<slug>.toml`. Reference existing teams for some samples. See [schemas/team.schema.json](schemas/team.schema.json) for more detail.

## Example team file

Save as `data/teams/<slug>.toml` where `<slug>` matches `slug` below.

```toml
# data/teams/example-team.toml — template only; not a real team.

[team]
# Human-readable title (required).
name = "Example Team"

# Stable id for this team: must match the filename (<slug>.toml), used in Terraform resources and integrations.
slug = "example-team"

# Longer blurb shown in tooling (optional).
description = "Demonstrates common team.toml fields."

# Codeberg usernames with lead privileges (Keycloak admin subgroup, GitHub maintainer where applicable, etc.).
leads = ["lead1-cb", "lead2-cb"]

# Codeberg usernames for regular members. Do not repeat anyone who is already in `leads` (validator rejects overlap).
members = ["member1-cb"]

# Repositories belonging to the team. Each `name` must be unique across the whole org.
# Repos are created on Codeberg (Forgejo) and mirrored to GitHub.
[[team.repos]]
# Repository name on Codeberg
name = "example-app"
# Short description for the repo
description = "Main application for Example Team."
# Repo visibility (defaults to public)
visibility = "private"
# Optional topic labels on the forge.
topics = ["web", "demo"]
# If true, register Kennel deploy webhooks for this repo.
kennel = true
# If true, link or provision Sentry for this repo where configured.
sentry = true

[[team.repos]]
name = "example-lib"
description = "Shared library; uses org default visibility when `visibility` is omitted."

# Discord / Slack channels scoped to this team. Provisioning maps members to these channels via Keycloak-linked ids.
[[team.channels]]
# Discord channel snowflake (omit this key or leave unset if the team does not use Discord here).
discord = "1234567890123456789"
# Slack channel id (omit if unused).
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

After this, open a pull request on Codeberg (Note everyone you list on this doc must already have the required Keycloak links. See [docs/adding-yourself-to-a-team.md](docs/adding-yourself-to-a-team.md)).

**Permissions:** only people who are already a **lead** on some team or sub-project (`leads` anywhere in this repo) may add a *new* team file. The tech director and DevOps leads can also do this.

After merge, permissions are automatically applied.

## `team.projects` Note

Note that this system is defined recursively. What this means is that you can add more and more `.projects` to these headers and each `team.projects` has all the headers available for what `team` generally has.
