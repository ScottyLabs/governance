
# Synchronizer

The [sync workflow](../../.github/workflows/sync.yml) will run the [sync script](sync.py) on every push to the `main` branch to synchronize the teams and members from the `contributors/` and `teams/` directories to GitHub and Keycloak.

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

- Create the Keycloak clients if they do not exist.
  - There will be 4 clients, for local, dev, staging, and prod.

- Create the Keycloak groups if they do not exist.
  - A lead group will also be created with the suffix "-leads".
  - A developer group will also be created with the suffix "-devs".
  - An admin group will be created with the suffix "-admins".

- Add team leads to the Keycloak lead group.

- Add team devs to the Keycloak developer group.

- Add team admins to the Keycloak admin group.

### Hashicorp Vault

- Create Hashicorp groups and necessary policies and aliases to integrate with Keycloak for authentication.
  - Dev groups can read the local secrets.
  - Lead groups can read and edit all secrets.

## Manual Validations

### Github

Check that you are added to the [CMU Maps](https://github.com/orgs/ScottyLabs/teams/cmu-maps) Github team.

### Keycloak

Make sure you have the right permissions by logging into the [vault](https://secrets.scottylabs.org/ui/vault/auth?with=oidc) and trying to access the secrets links in the corresponding folder section.

## Troubleshooting

- Check the [workflow output logs](https://github.com/ScottyLabs/governance/actions/workflows/sync.yml) to see if your user is added to the services.

- If your user is not found in Keycloak, try logging into the [vault](https://secrets.scottylabs.org/ui/vault/auth?with=oidc) to create your account and then ask to rerun the workflow.

### Local Development

Open in devcontainer and run the following command:

```zsh
python3 __meta/synchronizer/sync.py
```
