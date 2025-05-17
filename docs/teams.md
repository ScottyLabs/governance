### Adding a team

> [!NOTE]
> This is for Tech Leads and team members seeking to register their team under governance.

Create a new TOML file in `teams/` with the team name as the filename, e.g. `cmucourses.toml`:

```toml
name = "cmucourses"
members = [
    "your-github-username" # >= 1 member (yourself)
]
repos = [
    "cmucourses", # >= 1 repo
    "courses-backend"
]
slack-channel-ids = [
    "C0150RGAG1L" # Empty array if no associated channels
]
```

All of these fields are required; however, `slack-channel-ids` is allowed to be `[]` if the team has no channels on the Slack.

To find a Slack channel's ID, follow these steps:

1. Right click on the channel
2. Select "View channel details"
3. Locate "Channel ID: ..." at the bottom and press copy

This value should begin with a `C` for public channels and `G` for private channels.

> [!WARNING]
> The repos and members included in this file must already exist. You may add the repos in the same PR, but members must have already been added in previous PRs due to the earlier requirement on adding contributors. For similar reasons, you must be a member of any team you create.
