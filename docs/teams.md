# Adding a team

> [!NOTE]
> This is for Tech Leads and team members seeking to register their team under governance.

Decide on a team slug, website slug, and name. The slug will be used for internal references, the website slug will be used for the website (e.g. courses.scottylabs.org), and the name will be used for any public‑facing or display contexts (e.g. the GitHub team name and outreach posters).

Create a new TOML file in `teams/` with the team slug as the filename, e.g. `cmucourses.toml`:

```toml
name = "CMU Courses"
website-slug = "courses"
leads = [
    "your-github-username" # >= 1 lead
]
devs = [
    "your-github-username"
]
admins = [
    "andrew-id"
]
repos = [
    "cmucourses", # >= 1 repo
    "courses-backend"
]
slack-channel-ids = [
    "C0150RGAG1L" # Empty array if no associated channels
]
```

All of these fields are required; however, `devs`, `admins`, and `slack-channel-ids` are allowed to be `[]`. Admins are external individuals who need admin access to the app (e.g. orientation staffs for O-Quest) but are not involved in development.

To find a Slack channel's ID, follow these steps:

1. Right click on the channel
2. Select "View channel details"
3. Locate "Channel ID: ..." at the bottom and press copy

This value should begin with a `C` for public channels and `G` for private channels.

> [!WARNING]
> The members included in this file must already exist.
