# Adding yourself to a team

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

For a full commented team file (repos, channels, `forge`, etc.), see `docs/creating-teams.md`.

**Permissions:** if you are **not** a lead on that team, the PR should only add or remove **your own** username from `members` / project `members`. Changing leads, repos, channels, names, project structure, or anyone else’s membership requires being a lead on that team, or being the tech director / a DevOps lead.

## Keycloak

Governance treats **Keycloak** as the source of truth for identities. Your row uses your **Codeberg** handle; automation looks up that account in Keycloak and reads your linked IdPs.

If you are not yet on Keycloak, ask someone with the permissions to add you first.

Link what the org expects so membership actually grants access (the repo validator checks these for listed members):

- **CMU SSO** (`cmu-saml`), **Codeberg**, and **GitHub** — baseline (e.g. Google Groups via CMU identity, Forgejo/Codeberg and GitHub team membership).
- **Discord** and **Slack** — when those integrations are enabled org-wide; team Slack/Discord channels use your ids from Keycloak.

If a required link is missing, you can still merge a governance change, but provisioning may fail or skip part of your access.
