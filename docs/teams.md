# Adding a team

> [!NOTE]
> This is for tech leads and team members seeking to register their team under governance.

Create a new TOML file in `teams/` with the slug as the filename, e.g. `cmucourses.toml`:

```toml
slug = "cmumaps"
name = "CMU Maps"
description = """
CMU Maps is a web application that provides a map of the Carnegie Mellon University
campus, allowing users to easily access information about campus locations, live at cmumaps.com.
Read about our contributing guide at https://github.com/ScottyLabs/cmumaps/blob/main/.github/CONTRIBUTING.md.
"""
website-slug = "maps" # used in website URLs (e.g. maps.scottylabs.org)
maintainers = [
    "your-github-username" # >= 1 maintainer
]
contributors = [
    "your-github-username" # all maintainers must also be listed as contributors
]
applicants = [
    "applicant-github-username" # optional array of applicants
]
ext-admins = [
    "ext-admin-andrew-id" # optional array of external admins
]
repos = [
    "ScottyLabs/cmumaps", # >= 1 repo, in the format of "ScottyLabs/<repo-name>"
]

# The Slack Governance App needs to be added to every private Slack channel.
slack-channel-ids = [
    "C9999999999" # Empty array if no associated channels
]

# Set the `remove-unlisted` field to `false` if you want to keep unlisted members and repos
# in the GitHub team and unlisted members in the Keycloak groups. This setting is useful
# when not all members have been added to Governance yet. However, it is **recommended**
# to remove this override once everyone has been added to the team through Governance.
remove-unlisted = true # Default to true if not specified

# Set the `sync-github` field to `false` if the team does not want to synchronize with GitHub.
sync-github = true # Default to true if not specified

# Set the `create-oidc-clients` field to `false` if the team does not need OIDC clients.
# In this case, you also don't need to set the `website-slug` field.
create-oidc-clients = true # Default to true if not specified

# The `secrets-population-layout` field can be one of "single", "multi", or "none".
#
# Set the `secrets-population-layout` field to `none` if the team does not want automatic secrets population.
#
# Set the `secrets-population-layout` field to `single` if the team has only one app,
# such as a scripting project.
#
# Set the `secrets-population-layout` field to `multi` if the team has multiple apps,
# such as a frontend web client and a backend server.
secrets-population-layout = "multi" # Default to "multi" if not specified
```

Visit the [team schema](../__meta/schemas/team.schema.json) and
[synchronizer README](../__meta/synchronizer/README.md) to learn more about the fields.

To find a Slack channel's ID, see
[Slack Wiki page](https://github.com/ScottyLabs/governance/wiki/Slack).
