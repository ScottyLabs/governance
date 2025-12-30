# Adding a team

> [!NOTE]
> This is for tech leads and team members seeking to register their team under governance.

Create a new TOML file in `teams/` with the slug as the filename, e.g. `cmucourses.toml`:

```toml
slug = "cmucourses"
name = "CMU Courses"
website-slug = "courses"
leads = [
    "your-github-username" # >= 1 lead
]
devs = [
    "your-github-username" # empty array if no developers
]
applicants = [
    "your-github-username" # empty array if no applicants
]
ext-admins = [
    "andrew-id" # empty array if no external admins
]
repos = [
    "cmucourses", # >= 1 repo
    "courses-backend"
]
slack-channel-ids = [
    "C0150RGAG1L" # Empty array if no associated channels
]
```

Visit the [team schema](../__meta/schemas/team.schema.json) to learn more about the fields.

To find a Slack channel's ID, follow these steps:

1. Right click on the channel
2. Select "View channel details"
3. Locate "Channel ID: ..." at the bottom and press copy

This value should begin with a `C` for public channels and `G` for private channels.

> [!WARNING]
> The members included in this file must already exis as contributors in the `contributors/` directory.
