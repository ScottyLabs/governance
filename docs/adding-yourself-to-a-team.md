# Adding yourself to a team

## Before you open a PR

1. **[Link your identities in Keycloak first](https://idp.scottylabs.org/realms/scottylabs/account/account-security/linked-accounts)**, and remember to check the second page.
1. If any member listed in the team file lacks the linked accounts necssary for that team, CI will fail.

- Linking **Codeberg** and **GitHub** is necessary for everyone. These allow governance to add you to the team's repositories on Codeberg and GitHub.
- **Discord** and **Slack** are only necessary if your team has these communication channels defined in the team file.

## Steps

1. Edit `data/teams/<team-slug>.toml` and add your **Codeberg username** to `members` (to join the whole team) or to a project’s `members` under `[[team.projects]]` if membership is per-project. If your team is not yet created, follow the [team creation docs](creating-teams.md) instead.
2. Open a pull request on Codeberg.

**Permissions:** if you are not a lead on a team, a PR editing it should only involve you adding or removing yourself from `members` / project `members`. Changing any other field is restricted to the Tech Director and Tech Leads already declared in governance.
