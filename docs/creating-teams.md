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
# Set to false to exclude this repo from the docs hub, it is by default included.
# docs = false

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

With this file created, open a pull request on Codeberg. Everyone you list must have the required accounts linked. See [docs/adding-yourself-to-a-team.md](docs/adding-yourself-to-a-team.md).

## `team.projects` Note

Note that this system is defined recursively. You can repeatedly add `.projects` to these headers and each `team.projects` has all the headers available for what `team` generally has.

## Slack-Discord bridging

Discord is the source of truth, i.e. mautrix-discord creates Matrix portal rooms from Discord channels (guild bridge / channel activity) rather than creating Matrix hub rooms.

Set both channel IDs on the same `[[team.channels]]` or `[[team.projects.channels]]` entry to selectively link Slack into the existing mautrix-discord portal for that Discord channel (via `synapse_mautrix_slack_link` in OpenTofu). Channels with only Discord or only Slack are not mirrored:

```toml
[[team.channels]]
discord = "1461933322505818156"
slack = "C08K3Q77ZQF"
```

The Discord channel must already exist on Discord and have a mautrix-discord portal before apply succeeds. You can still set only `discord` or only `slack` if you do not want mirroring but still want membership permissions applied.

### Org-wide channels

The **hub** (`slack_hub_channel_id` / `discord_hub_channel_id` in `data/org.toml`) is linked the same way when communication is configured.

Other org-wide channels (e.g. open-source, merch, finance) that are not owned by a team can be declared in `data/org.toml`:

```toml
[[org.communication.channels]]
name = "Open Source"
slug = "open-source"
discord = "<discord-channel-id>"
slack = "<slack-channel-id>"
```

**Permissions:** creating new teams is restricted to the Tech Director and Tech Leads already declared in governance.
