# Governance

This repository is the source of truth for the Tech Committee's governance model. It declaratively manages teams, repositories, and membership using [OpenTofu](https://opentofu.org/) and [Atlantis](https://www.runatlantis.io/).

## Joining a team

1. [Link all available accounts](https://idp.scottylabs.org/realms/scottylabs/account/account-security/linked-accounts) in Keycloak.
1. Add your Codeberg username to the `members` array in the desired team `.toml` file under `data/`.
1. Open a PR using a [conventional](https://www.conventionalcommits.org/en/v1.0.0/) PR title.

Note that only team leads are allowed to modify other people's memberships.

## Creating a team

Teams are groups of leads, members, repositories, and channels. They can nest sub-projects recursively, each with the same shape. Copy an existing file in `data/teams/` for a working starting point.

Reference the [team schema](./schemas/team.schema.json) for an authoritative list of fields and their constraints.

### Features

Each repository opts into capabilities through its `features` array:

- `kennel` adds a Forgejo webhook that connects the repository to kennel for builds and deployments
- `sentry` creates a Sentry project and writes its DSN to Vault
- `oidc_client` provisions prod and staging Keycloak OIDC clients with a fixed redirect URI and writes their credentials to Vault per profile
- `admin_client` provisions a Keycloak service-account client with user-management roles and writes its credentials to Vault

How a project declares and consumes what these provision lives in the kennel docs: [Deploying a Project](https://docs.kennel.scottylabs.org/guides/deploying.html) and [Secrets](https://docs.kennel.scottylabs.org/guides/secrets.html).

## Description

The following is a non-exhaustive list of platforms Governance manages:

1. Keycloak
   - Members are added to their team's Keycloak groups, which gives them permission to access environment variables and other project-specific resources
   - Team leads are further added to the team's admins subgroup, which gives additional access
   - For projects with it enabled, OIDC clients are provisioned
1. OpenBao
   - Keycloak groups are given the appropriate access to secret paths on OpenBao
1. Codeberg
   - Members are added to their Codeberg teams, which gives them appropriate access to the team's repositories
   - Codeberg repositories are set up to automatically sync to GitHub for visibility
1. Google
   - Members are automatically added to ScottyLabs' and Tech's mailing lists (Google Groups)
1. Sentry
   - Projects are provisioned under [Sentry](https://scottylabs.sentry.io) and registered under ScottyLabs' [observability stack](https://codeberg.org/scottylabs/observability)
1. Kennel
   - Repositories automatically receive a deploy webhook that authorizes them to be deployed by [kennel](https://codeberg.org/scottylabs/kennel)
1. Discord and Slack
   - Members are added to the appropriate channels on both platforms
   - On Discord, members are assigned the Tech role and their team's roles
   - Bidirectional sync is established between registered Discord and Slack channels via Matrix
1. Vaultwarden
   - Members are given the appropriate access to account credentials on Vaultwarden

Here, "appropriate access" serves to delineate between member permissions and team lead permissions.
