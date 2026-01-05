
# Synchronizer

This directory contains the Python-based permission synchronizer for ScottyLabs.

## Github

### Contributor Synchronization

- A new contributor will receive an email invitation
  to join the [ScottyLabs Github organization](https://github.com/ScottyLabs) as a member.

### Team Synchronization

If the `sync-github` field is not explicitly set to `false` in the team file,
the following synchronizations will be performed for the team:

- Github teams will be created if they do not exist.
  - The team name will be the same as the team name specified in the `teams/` directory.
  - An admin team will also be created as a subteam of the main team with the name
    "`Team Name` Admins".

- Team contributors will be added to the corresponding Github main team as members.

- Team maintainers will be added to the corresponding Github main team and admin team as maintainers.

  - While this allows the team maintainers to add and remove members from the team directly,
    members who are not listed in Governance will be removed on the next Governance sync.

  - The `remove-unlisted` field can be set to `false` in the team file to keep unlisted members.
    This option is useful for quickly adding members as the team migrates to use Governance,
    but it is **recommended** to eventually list all members in the team file
    and remove the `remove-unlisted` field override.

- Any unlisted member will be removed from the Github teams. The team can keep unlisted
  members by setting the `remove-unlisted` field to `false` in the team file.

- The repos listed in the team file will be added to the Github team.

  - The GitHub admin team will be given admin access to the repos.

  - The GitHub main team will be given write access to the repos.

  - The `remove-unlisted` field can be set to `false` in the team file to give the main
    GitHub team a different permission level.

- The repos not listed in the team file will be removed from the Github team.

  - The team can keep unlisted repos by setting the `remove-unlisted` field to `false` in the team file.

## Keycloak

The groups and oidc clients created by Keycloak synchronization are used for auth
and integrate with Hashicorp Vault. However, they are not directly visible to you,
so you can skip this section if you only want to know about the permissions you can directly use.

- Create the Keycloak oidc clients if they do not exist. The team can opt out
by setting the `create-oidc-clients` field to `false` in the team file.

  - There will be 4 clients, for local, dev, staging, and prod, named
  as `<team-slug>-<env>`.

  - The default whitelisted redirect URIs are:
    - Local: `http://localhost/auth/callback`
    - Dev: `https://api.<website-slug>.slabs-dev.org/auth/callback`
    - Staging: `https://api.<website-slug>.slabs-staging.org/auth/callback`
    - Prod: `https://api.<website-slug>.scottylabs.org/auth/callback`

  - The default post-logout redirect URIs are:
    - Local: `http://localhost:3000/*`
    - Dev: `https://<website-slug>.slabs-dev.org/*`
    - Staging: `https://<website-slug>.slabs-staging.org/*`
    - Prod: `https://<website-slug>.scottylabs.org/*`

- Create the Keycloak groups if they do not exist.

  - An admin group will be created with the suffix "-admins".

    - The team maintainers and service accounts (if the oidc clients are created)
      will be added to this group.

  - A dev group will also be created with the suffix "-devs".

  - An external admin group will be created with the suffix "-ext-admins" if the
    `ext-admins` field is present in the team file.

  - An applicant group will be created with the suffix "-applicants" if the
    `applicants` field is present in the team file.

- Any unlisted members will be removed from the Keycloak groups unless the
  team opts out by setting the `remove-unlisted` field to `false` in the team file.

## Hashicorp Vault

- Hashicorp groups, policies, and aliases will be created to integrate with Keycloak groups.

  - Admin groups can read and edit all secrets.

  - Dev groups can read the secrets in the `local` folder.

  - Applicants group can read the secrets in the `applicants` folder.

- If the team did not opt out of secrets population by setting the `secrets-population-layout`
  field to `none` in the team file, the secrets will be populated in the following layout:

  - `single`: A folder for each environment (e.g. `local`, `dev`, `staging`, `prod`).

  - `multi`: A folder for each environment and a subfolder for each app
  (e.g. `local/web`, `local/server`, `dev/web`, `dev/server`, `staging/web`, `staging/server`, `prod/web`, `prod/server`).

  - The secrets will include the OIDC client secrets if the `create-oidc-clients`
    field is not explicitly set to `false` in the team file.

- See [Vault Wiki page](https://github.com/ScottyLabs/wiki/wiki/Secrets-Management)
  on how to access the Vault secrets.

## Slack

- Invite team members to the corresponding Slack channels listed in the team file.

- The Slack Governance App needs to be added to every **private** Slack channel in
  order to invite users to the channel.
  To add the app in a channel, send the following message in the channel:

  ```slack
  /invite @Governance
  ```

## CODEOWNERS

- The [`CODEOWNERS`](https://github.com/ScottyLabs/governance/blob/main/.github/CODEOWNERS)
  file will be automatically generated based on the contributors and teams.

## Leadership

- The [leadership team](https://github.com/ScottyLabs/governance/blob/main/teams/leadership.toml)
  will be added as GitHub organization owners.

- The leadership team maintainers will have all permissions in the Vault.

- The leadership team contributors will have read permissions to everything in the Vault.

## Troubleshooting

- Check out the [workflow output logs](https://github.com/ScottyLabs/governance/actions/workflows/sync.yml)
  to see the synchronization in real time!

- If your user is not found in Keycloak, try
  [log into the Vault](https://github.com/ScottyLabs/wiki/wiki/Secrets-Management)
  to create your account and then wait after the next sync workflow completes or
  ask for the workflow to be rerun.

## Development

The [sync workflow](../../.github/workflows/sync.yml) will automatically run the
[sync script](synchronizer/main.py) on every push to the `main` branch to synchronize
the permissions for the contributors and teams from the `contributors/` and `teams/`
directories, so the following instructions will not be relevant to most people.
If you are interested, you can also test synchronizer locally, but it will fail
due to the lack of environment variables.

Open in devcontainer and run the following command in the root directory to see the available options:

```zsh
uv run sync -h
```

### Linting

```zsh
uv run mypy .
uv run ruff check .
```
