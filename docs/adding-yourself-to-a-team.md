# Adding yourself to a team

## Before you open a PR

1. **[Link your identities in Keycloak first](https://idp.scottylabs.org/realms/scottylabs/account/account-security/linked-accounts)**, and remember to check the second page.
1. If any member listed in the team file lacks the required linked accounts, CI will fail.

- **Codeberg**, **GitHub**, **Discord**, and **Slack** must all be linked. Codeberg and GitHub are needed so governance can add you to team repositories. Discord and Slack are needed even if your team has no channels of its own: governance gives every member the org-wide "Tech" Discord role and invites everyone to the Slack hub channel.

## Steps

1. Edit `data/teams/<team-slug>.toml` and add your **Codeberg username** to `members` (to join the whole team) or to a project's `members` under `[[team.projects]]` if membership is per-project. If your team is not yet created, follow the [team creation docs](creating-teams.md) instead.
1. [Set up your Matrix account and bridge logins](#set-up-your-matrix-account-and-bridge-logins)
1. Open a pull request on Codeberg.

**Permissions:** if you are not a lead on a team, a PR editing it should only involve you adding or removing yourself from `members` / project `members`. Changing any other field is restricted to the Tech Director and Tech Leads already declared in governance.

## Set up your Matrix account and bridge logins

### Matrix account

1. Go to [Element](https://app.element.io)
1. Edit your homeserver to be `doggylabs.org`
1. Register using your **Codeberg username in all lowercase** as your Matrix username (Matrix does not allow uppercase letters).

### Discord bridge

1. Open a DM with `@discord:doggylabs.org`
1. Send `login`
1. Follow the OAuth link the bot sends you and authorize Discord

### Slack bridge

1. Open a DM with `@slack:doggylabs.org`
1. Send `login token`
1. The bot asks for your Slack session. Paste one of the following into the DM:
   - A **Copy as cURL** command from browser DevTools (easier), or
   - A JSON object with `auth_token` and `cookie_token`

#### Paste a cURL command

Instead of building JSON by hand, you can copy a Slack API request as cURL and paste the whole command into the bot DM.

1. In DevTools → **Network**, click any request to `https://scottylabs.slack.com/api/...` (or similar)
1. Right-click the request → **Copy** → **Copy as cURL** (Chrome) or **Copy as cURL** (Firefox)
1. Paste the entire cURL command into the DM

The bridge extracts `auth_token` and `cookie_token`.

Tech Lead setup to enable the shared relay is documented in [creating-teams.md](creating-teams.md#infrastructure-prerequisites).
