# ScottyLabs Governance

This repository defines the organizational structure, team membership, and project ownership for ScottyLabs' Tech Committee. It serves as the source of truth for our GitHub organization's governance model.

In this document, 'ScottyLabs' will refer to the GitHub organization at <https://github.com/ScottyLabs>, and not the club itself.

## Repository Structure

```py
.
├── __meta
│   ├── infra      # Terraform code for applying changes
│   ├── schemas    # JSON schemas for validation
│   ├── validators # Rust-based validation tools
│   └── visualizer # Force graph for visualizing relationships
├── contributors   # Individual contributor definitions
├── docs           # Specific instructions for all three file types
├── repos          # Team definitions with members and repos
└── teams          # Repository definitions with metadata
```

- **Contributors** - Individuals who participate in ScottyLabs projects
- **Teams** - Groups of contributors working on specific projects
- **Repositories** - Code repositories owned by teams

Depending on what you are trying to register, follow the respective guide under `docs/`.

## Validation

We enforce [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/). In addition, the extensions listed in `.vscode/extensions.json` are essential in making sure you abide by style guidelines and other rules.

This repository also includes several other checks to ensure integrity:

- File names must match the content (the `name` field for repos and teams, the `github-username` field for contributors)
- Cross-references must be valid (team members must exist as contributors, team repos must exist as repos)
- GitHub users must exist
- Slack member IDs and channel IDs must be valid

## Development

Validation runs automatically through GitHub Actions on PRs and pushes to main, so the following instructions will not be relevant to most people. If you are interested, you can also test validators locally:

1. Make sure you are in the root of the repository.

2. Install Rust with `rustup`, if you do not already have it installed:

    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

3. Install `cargo-binstall` (enables use of `cargo binstall`):

    ```sh
    cargo install cargo-binstall
    ```

4. Install Taplo:

    ```sh
    cargo binstall taplo-cli
    ```

5. Check TOML files for proper formatting and/or against the schemas:

    ```sh
    taplo fmt --check # for formatting
    taplo check # against the schemas
    ```

6. Run the other checks specified above:

    ```sh
    cargo run --bin governance
    ```

## Python Sync Script

The [sync workflow](../.github/workflows/sync.yml) will run the [sync script](scripts/sync.py) on every push to the `main` branch to sync the teams and members from the `contributors/` and `teams/` directories to GitHub and Keycloak.

### Github

- Invite all contributors to the [ScottyLabs](https://github.com/ScottyLabs) Github organization.

- Create the Github teams if they do not exist.
  - The team name will be the same as the team name in the `teams/` directory.
  - An admin team will also be created as the subteam of the main team with the same name and the suffix "-admins".

- Add the repository to the Github team. Give team developers write access and team leads admins access to the repository.

- Add team members to the corresponding Github main team as members.

- Add team leads to the corresponding Github admin team as members.
  - No one should be a maintainer of the GitHub team since membership is managed by Governance.

- Delete any unknown member from the Github team.

### Keycloak

#### Clients

- Create the Keycloak clients if they do not exist.
  - There will be 4 clients, for local, dev, staging, and prod.

#### Hashicorp Vault

- Create the Keycloak groups if they do not exist.
  - An admin group will be created with the suffix "-admins".
  - A developer group will also be created with the suffix "-devs".

- Add team members to the Keycloak cmumaps-devs team.
  - Able to view any local secrets.

- Add team leads to the Keycloak cmumaps-admins team.
  - Able to view and edit any secrets.

- Create Hashicorp groups and necessary policies for secrets management.

## Validations

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
python3 scripts/sync.py
```

## License

This project is licensed under `Apache-2.0`, and is heavily inspired by [Concourse's governance](https://github.com/concourse/governance).
