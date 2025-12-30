# ScottyLabs Governance

This repository defines the organizational structure, team membership, and project ownership for ScottyLabs. It serves as the source of truth for our GitHub organization's governance model.

In this document, 'ScottyLabs' will refer to the GitHub organization at <https://github.com/ScottyLabs>, and not the club itself.

## Repository Structure

```py
.
├── __meta
│   ├── schemas      # JSON schemas for validation
│   ├── synchronizer # Python-based permission synchronizer
│   ├── validators   # Rust-based validation tools
│   └── visualizer   # Force graph for visualizing relationships
├── contributors     # Individual contributor definitions
├── docs             # Specific instructions for all file types
├── scripts          # Utility scripts for environment secret management
└── teams            # Team definitions with members and repos
```

## Adding a New Contributor

1. Follow the instructions in [docs/contributors.md](docs/contributors.md) to submit
a PR adding yourself as a new contributor.

2. In the same PR, include yourself in the team(s) you want to join under the `teams/` directory:
   - If the team file has an `applicants` field, add yourself there.
   - Otherwise, add yourself to the `devs` field.

3. Review each project's repo's `CONTRIBUTING.md` file to understand any team-specific
contribution guidelines (for example, required Google Form submissions).

4. Request the first tech lead of each team you want to join as a reviewer to the PR.

5. If no response:
   - After 24 hours: ping the reviewer.
   - After 48 hours: ping the tech director (@Yuxiang-Huang).

## Adding a New Team

1. Follow the instructions in [docs/teams.md](docs/teams.md) to submit a PR creating the new team.

2. Request the tech director (@Yuxiang-Huang) as a reviewer to the PR.

3. If there is no response within 24 hours, ping the tech director.

## License

This project is licensed under `Apache-2.0`, and is heavily inspired by [Concourse's governance](https://github.com/concourse/governance).
