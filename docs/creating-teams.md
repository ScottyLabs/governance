# Creating a team

Governance stores teams as `data/teams/<slug>.toml`. Shape matches existing teams (`slug`, optional `name` / `description`, `leads`, `members`, `repos`, optional `[[team.projects]]`, `[[team.channels]]`, …). See `schemas/team.schema.json`.

## Example team file

Save as `data/teams/<slug>.toml` where `<slug>` matches `slug` below.

```toml
# data/teams/example-team.toml — template only; not a real team.

[team]
# Human-readable title (optional but recommended).
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
[[team.repos]]
# Repository name on the forge (also used as the repo id in automation).
name = "example-app"
# Short description pushed to the forge host.
description = "Main application for Example Team."
# Which forge hosts this repo: "forgejo" (Codeberg) or "github". Omit to use org-wide `default_forge` in `data/org.toml`.
forge = "forgejo"
# "public" or "private". Omit to use the org default visibility.
visibility = "private"
# Optional topic labels on the forge.
topics = ["web", "demo"]
# If true, register Kennel deploy webhooks for this repo (Forgejo).
kennel = true
# If true, link or provision Sentry for this repo where configured.
sentry = true

[[team.repos]]
name = "example-lib"
description = "Shared library; uses org default forge and visibility when `forge` / `visibility` are omitted."

# Discord / Slack channels scoped to this team. Provisioning maps members to these channels via Keycloak-linked ids.
[[team.channels]]
# Discord channel snowflake (omit this key or leave unset if the team does not use Discord here).
discord = "1234567890123456789"
# Slack channel id (omit if unused).
slack = "C0123456789"

# Optional Figma identifiers for integrations that consume this list (omit the key if unused).
# figma_projects = ["abc123figma"]

# Sub-projects: separate repos, channels, and membership under the same team umbrella.
[[team.projects]]
# Required per project; used in paths and group names (keep stable).
slug = "subproject-alpha"
# Optional display name for humans.
name = "Alpha track"
description = "Optional project-level description."
# Project-level leads (Codeberg usernames).
leads = ["lead3-cb"]
# Members only for this sub-project (Codeberg usernames).
members = ["member2-cb"]

[[team.projects.repos]]
name = "alpha-service"
description = "Backend for the alpha track."
kennel = false

# Project-specific comms (same shape as team-level `[[team.channels]]`).
[[team.projects.channels]]
discord = "9876543210987654321"
```

1. Add `data/teams/<slug>.toml`.
2. Open a pull request on Codeberg.

**Permissions:** only people who are already a **lead** on some team or sub-project (`leads` anywhere in this repo) may add a *new* team file. The tech director and DevOps leads bypass that rule.

After merge, CI validates and applies repos, Keycloak groups, comms access, and other integrations from the file.

**Usernames:** `leads` and `members` are **Codeberg usernames**. Provisioning resolves each via **Keycloak** (Codeberg linked to your Keycloak user, then other linked IdPs). Anyone you list must have the right links in Keycloak or apply can fail for them; see `docs/adding-yourself-to-a-team.md` for what members typically need.
