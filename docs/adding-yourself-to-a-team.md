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

Instead of building JSON by hand like it tells you to, you can copy a Slack API request as cURL and paste the command into the bot DM which is a bit easier.

1. In DevTools (F12 or right-click → Inspect) → **Network**, click any request to `https://scottylabs.slack.com/api/...` (or similar)
1. Right-click the request → **Copy** → **Copy as cURL** (Chrome) or **Copy as cURL** (Firefox)
1. Paste the entire cURL command into the DM

The bridge extracts `auth_token` and `cookie_token`.

#### Alternative: Get tokens from Chrome or Firefox DevTools

This is a bit more effortful.

Do this in a normal browser where you are **already logged into ScottyLabs Slack** ([https://scottylabs.slack.com](https://scottylabs.slack.com) or the workspace you use).

**Cookie token (`cookie_token`)**

1. Open DevTools (F12 or right-click → Inspect)
1. Go to **Application** (Chrome) or **Storage** (Firefox) → **Cookies**
1. Select `https://app.slack.com` or `https://scottylabs.slack.com`
1. Find the cookie named `d`
1. Copy its **Value** (starts with `xoxd-`). This is your `cookie_token`

**Auth token (`auth_token`) — pick one method**

*From localStorage (often easiest):*

1. In DevTools, open the **Console** tab while Slack is open
1. Run:

```javascript
JSON.parse(localStorage.localConfig_v2).teams   
```

1. Expand the workspace entry and copy the `token` field (starts with `xoxc-`). This is your `auth_token`

*From the Network tab:*

1. Open DevTools (F12 or right-click → Inspect) → **Network**
1. Reload Slack or click a channel so requests appear
1. Filter by `api` or look for requests to `*.slack.com/api/...`
1. Click a request and find `token` in the **Request payload** or **Form data** (starts with `xoxc-`)

**Paste into Matrix**

Build the JSON and send it in the DM with `@slack:doggylabs.org`. It is suggested to copy it before sending because the bot would delete your message even if it errors (likely for security purposes) meaning you would lose it otherwise.

```json
{"auth_token":"xoxc-PASTE_HERE","cookie_token":"xoxd-PASTE_HERE"}
```

Tech Lead setup to enable the shared relay is documented in [creating-teams.md](creating-teams.md#infrastructure-prerequisites).
