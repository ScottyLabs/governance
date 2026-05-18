# Adding yourself to a team

## Before you open a PR

Everyone already has a Keycloak account. **Link your identities in Keycloak first** — opening a pull request runs CI, which checks that every listed member can be resolved before the change can merge.

Link what the org expects (the validator checks these for listed members):

- **CMU SSO** (`cmu-saml`) — used for Google Groups
- **Codeberg** and **GitHub** — Forgejo/Codeberg org membership and mirrored GitHub team access.
- **Discord** and **Slack** — when those integrations are enabled org-wide; team channels use your ids from Keycloak.

Governance treats **Keycloak** as the source of truth: your row in team data is your **Codeberg username**, and automation looks up that account in Keycloak and reads your linked IdPs.

## Steps

1. Edit `data/teams/<team-slug>.toml` and add your **Codeberg username** to `members` (whole team) or to a project’s `members` under `[[team.projects]]` if membership is per-project.
2. Open a pull request on Codeberg.

Typical edits (your real file already has repos, channels, and other sections—change only the `members` arrays you care about):

```toml
# Team-wide membership: everyone listed gets access to team-level repos and channels.
[team]
slug = "example-team"
leads = ["lead-cb"]
members = ["member-cb", "your-codeberg-username"]  # add yourself here

# Or project-only membership: access limited to that project’s repos and channels.
[[team.projects]]
slug = "subproject-alpha"
leads = ["lead-cb"]
members = ["your-codeberg-username"]  # add yourself here instead (or in addition to team.members)
```

For a full commented team file (repos, channels, etc.), see `docs/creating-teams.md`.

**Permissions:** if you are **not** a lead on that team, the PR should only add or remove **your own** username from `members` / project `members`. Changing leads, repos, channels, names, project structure, or anyone else’s membership requires being a lead on that team, or being the tech director / a DevOps lead.
