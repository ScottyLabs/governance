
# Synchronizer

The [sync workflow](../../.github/workflows/sync.yml) will run the [sync script](sync.py)
on every push to the `main` branch to synchronize the teams and members from the `contributors/` and `teams/`
directories to GitHub, Keycloak, Vault, and Slack.

## Github

- Invite all contributors to the [ScottyLabs](https://github.com/ScottyLabs) Github organization.

- Create the Github teams if they do not exist.
  - The team name will be the same as the team name specified in the `teams/` directory.
  - An admin team will also be created as the subteam of the main team with the same name and the suffix " Admins".

- Add the repository to the Github team. Give team developers write access and team leads admins access to the repository.

- Add team members to the corresponding Github main team as members.

- Add team leads to the corresponding Github main team and admin team as members.
  - No one should be a maintainer of the GitHub team since membership is managed by Governance.

- Delete any unknown member from the Github teams.

## Keycloak

- Create the Keycloak oidc clients if they do not exist. The team can opt out
by setting the `create-oidc-clients` field to `false` in the team file.
  - There will be 4 clients, for local, dev, staging, and prod, named as `<team-slug>-<env>`.
  - The default whitelisted redirect URIs are:
    - Local: `http://localhost:3000/*`
    - Dev: `https://<website-slug>.slabs-dev.org/*`
    - Staging: `https://<website-slug>.slabs-staging.org/*`
    - Prod: `https://<website-slug>.scottylabs.org/*`

- Create the Keycloak groups if they do not exist.

  - A lead group will also be created with the suffix "-admins".

    - The team leads and service accounts (if the oidc clients are created) will be added to this group.

  - A developer group will also be created with the suffix "-devs".

    - The team devs will be added to this group.

  - An external admin group will be created with the suffix "-ext-admins".

    - The team external admins will be added to this group.

  - An applicant group will be created with the suffix "-applicants".

    - The team applicants will be added to this group.

### Hashicorp Vault

- Create Hashicorp groups and necessary policies and aliases to integrate with Keycloak for authentication.
  - Dev groups can read the local secrets.
  - Admin groups can read and edit all secrets.

## Slack

- Add team members to the corresponding Slack channels.

## Manual Validations

### Github

- Check that you received an email invitation to the ScottyLabs Github organization.

- Check that you are added to the corresponding Github team after accepting the invitation.

### Keycloak

Make sure you have the right permissions by logging into the [vault](https://secrets.scottylabs.org/ui/vault/auth?with=oidc) and trying to access the secrets links in the corresponding folder section.

### Slack

- Check that you are added to the corresponding Slack channels.

## Troubleshooting

- Check the [workflow output logs](https://github.com/ScottyLabs/governance/actions/workflows/sync.yml) to see if your user is added to the services.

- If your user is not found in Keycloak, try logging into the [vault](https://secrets.scottylabs.org/ui/vault/auth?with=oidc) to create your account and then ask to rerun the workflow.

### Local Development

Open in devcontainer and run the following command in the root directory:

```zsh
uv run __meta/synchronizer/sync.py
```
