# Adding yourself to a team

## Before you open a PR

Everyone already has a Keycloak account. **[Link your identities in Keycloak first (remember theres a second page)](https://idp.scottylabs.org/realms/scottylabs/account/account-security/linked-accounts)**. Opening a pull request runs CI, which checks that every listed member can be resolved before the change can merge. If your identities are not linked this fails and we can't merge your PR.

Suggested links (you don't need to link them unless your team uses them):

- **CMU SSO** (`cmu-saml`), which helps give access to Google Groups
- **Codeberg** and **GitHub**, which helps give Codeberg org membership and GitHub team membership.
- **Discord** and **Slack**, which helps team channels use your ids from Keycloak if everyone has it.

Governance treats **Keycloak** as the source of truth: your row in team data is your **Codeberg username**, and automation looks up that account in Keycloak and reads your linked IdPs.

## Steps

1. Edit `data/teams/<team-slug>.toml` and add your **Codeberg username** to `members` (whole team) or to a project’s `members` under `[[team.projects]]` if membership is per-project. If your team is not yet created, [create it.](creating-teams.md)
2. Open a pull request on Codeberg.

For a full commented team file (repos, channels, etc.), see `docs/creating-teams.md`.

**Permissions:** if you are **not** a lead on that team, the PR should only add or remove **your own** username from `members` / project `members`. Changing leads, repos, channels, names, project structure, or anyone else’s membership requires being a lead on that team, or being the tech director / a DevOps lead.
