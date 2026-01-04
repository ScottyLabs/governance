# ScottyLabs Governance

This repository defines the organizational structure, team membership, and project ownership for ScottyLabs.
It serves as the source of truth for our GitHub organization's governance model.

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

1. Follow the instructions in [docs/contributors.md](docs/contributors.md) to add
   your information as a new contributor.

2. Update the file of the team(s) you want to join under the `teams/` directory:
   - If the team file has an `applicants` field, add yourself there.
   - Otherwise, add yourself to the `contributors` field.

3. Review each project's repo's `CONTRIBUTING.md` file to understand any team-specific
   contribution guidelines (for example, some teams might require you submit a Google Form).
   The link to each repo is in the format of `https://github.com/<organization>/<repo>`,
   where `<organization>/<repo>` is listed in the `repos` field of the team file.
   (e.g. `https://github.com/ScottyLabs/governance`).

4. Submit a PR once you are ready. Familiarize yourself with the
   [PR Review Process](https://github.com/ScottyLabs/governance/wiki/PR-Review-Process)
   to understand the expectations for contributors and reviewers.

## Adding a New Team

1. Follow the instructions in [docs/teams.md](docs/teams.md) to create a new team file.

2. Submit a PR once you are ready. Familiarize yourself with the
   [PR Review Process](https://github.com/ScottyLabs/governance/wiki/PR-Review-Process)
   to understand the expectations for contributors and reviewers.

## Validations

Check the [validators README](__meta/validators/README.md) for more information on the checks
your PR must pass before it can be merged.

## Synchronization

Check the [synchronizer README](__meta/synchronizer/README.md) for more information
on the synchronization process, including what permissions are granted.

## License

This project is licensed under `Apache-2.0`, and is heavily inspired by
[Concourse's governance](https://github.com/concourse/governance).
