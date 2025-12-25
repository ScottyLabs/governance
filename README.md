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

## Adding a new contributor or team

Please follow the instructions in the `docs/` directory to make a PR to add a new contributor or team. Make sure the PR passes the validation checks. See [validation README](__meta/validators/README.md) for more details. After the PR is merged, permissions will be automatically synchronized. See [synchronizer README](__meta/synchronizer/README.md) for more details.

## License

This project is licensed under `Apache-2.0`, and is heavily inspired by [Concourse's governance](https://github.com/concourse/governance).
