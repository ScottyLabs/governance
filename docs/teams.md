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

Key configurations settable:

- Set the `remove-unlisted` field to `false` if you want to keep unlisted members in the team.
- Set the `create-oidc-clients` field to `false` if the team does not need OIDC clients.
  - In this case, you also don't need to set the `website-slug` field.
- Set the `secrets-population-layout` field to `none` if the team does not want automatic secrets population.

Visit the [team schema](../__meta/schemas/team.schema.json) to learn more about the fields.

To find a Slack channel's ID, follow these steps:

1. Right click on the channel
2. Select "View channel details"
3. Locate "Channel ID: ..." at the bottom and press copy

This value should begin with a `C` for public channels and `G` for private channels.
