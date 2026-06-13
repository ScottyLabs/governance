# Adding yourself to a team

## Before you open a PR

1. **[Link your identities in Keycloak first](https://idp.scottylabs.org/realms/scottylabs/account/account-security/linked-accounts)**, and remember to check the second page.
1. If any member listed in the team file lacks the required linked accounts, CI will fail.

- **Codeberg**, **GitHub**, **Discord**, and **Slack** must all be linked. Codeberg and GitHub are needed so governance can add you to team repositories. Discord and Slack are needed even if your team has no channels of its own: governance gives every member the org-wide "Tech" Discord role and invites everyone to the Slack hub channel.

## Steps

1. Edit `data/teams/<team-slug>.toml` and add your **Codeberg username** to `members` (to join the whole team) or to a project's `members` under `[[team.projects]]` if membership is per-project. If your team is not yet created, follow the [team creation docs](creating-teams.md) instead.
1. [Set up your Matrix account](#set-up-your-matrix-login-on-httpsmatrixdoggylabsorg)
1. Open a pull request on Codeberg.

**Permissions:** if you are not a lead on a team, a PR editing it should only involve you adding or removing yourself from `members` / project `members`. Changing any other field is restricted to the Tech Director and Tech Leads already declared in governance.

### Set up Your Matrix Login on doggylabs.org

1. Go to [Element](https://app.element.io)
1. Edit your homeserver to be doggylabs.org
1. Register using your **Codeberg username in all lowercase** as your Matrix username (Matrix does not allow uppercase letters)
1. Open a DM with `@discord:doggylabs.org` and send `login`. Use `login token` in DMs with `@slack:doggylabs.org`.

Tech Lead setup to enable the shared relay is documented in [creating-teams.md](creating-teams.md#infrastructure-prerequisites).
