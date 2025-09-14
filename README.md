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

## License

This project is licensed under `Apache-2.0`, and is heavily inspired by [Concourse's governance](https://github.com/concourse/governance).
