# ScottyLabs Governance

This repository defines the organizational structure, team membership, and project ownership for ScottyLabs' Tech Committee. It serves as the source of truth for our GitHub organization's governance model.

In this document, 'ScottyLabs' will refer to the GitHub organization at https://github.com/ScottyLabs, and not the club itself.

## Repository Structure

```
.
├── contributors   # Individual contributor definitions
├── meta
│   ├── infra      # Terraform code for applying changes
│   ├── schemas    # JSON schemas for validation
│   └── validators # Rust-based validation tools
├── repos          # Team definitions with members and repos
└── teams          # Repository definitions with metadata
```

-   **Contributors** - Individuals who participate in ScottyLabs projects
-   **Teams** - Groups of contributors working on specific projects
-   **Repositories** - Code repositories owned by teams

### Joining as a contributor

Create a new TOML file in `contributors/` with your GitHub username as the filename, e.g. `your-github-username.toml`:

```toml
name = "Your Name"
github = "your-github-username"
```

> [!WARNING]
> Pull requests adding a new contributor must be submitted by the contributor themselves. This self-nomination approach promotes ownership, helps maintain the integrity of our contributor list, and encourages active participation with our governance process and the organization. PRs in violation will be automatically rejected.

### Adding a repository

Create a new TOML file in `repos/` with the repository name as the filename, e.g. `cmucourses.toml`:

```toml
name = "cmucourses"
description = "..." # Optional
website = "..." # Optional (but mutually exclusive with 'websites' if present)
websites = ["...", "..."] # Optional
```

### Adding a team

Create a new TOML file in `teams/` with the team name as the filename, e.g. `cmucourses.toml`:

```toml
name = "cmucourses"
members = [
    "your-github-username"
]
repos = [
    "cmucourses"
]
```

> [!WARNING]
> The repos and members included in this file must already exist. You may add the repos in the same PR, but members must have already been added in previous PRs due to the earlier requirement on adding contributors. For similar reasons, you must be a member of any team you create.

## Validation

We enforce [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

This repository also includes several other checks to ensure integrity:

-   File names must match the content (the `name` field for repos and teams, the `github` field for contributors)
-   Cross-references must be valid (team members must exist as contributors, team repos must exist as repos)
-   GitHub users must exist

Validation runs automatically through GitHub Actions on PRs and pushes to main. However, you can also test validators locally.

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
