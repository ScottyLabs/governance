# ScottyLabs Governance

This repository defines the organizational structure, team membership, and project ownership for ScottyLabs.
It serves as the source of truth for our GitHub organization's governance model,
formalizing permission as code. The specific permissions granted to contributors
and each team are detailed in [synchronizer/README.md](__meta/synchronizer/README.md).

**Repositories are registered** in `repos/` (description and canonical URL). That registry is the source of truth for display and tooling, so the system works with GitHub, Codeberg, and teams that do not have a home repository on GitHub.

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
├── repos            # Registered repos (source of truth: description, URL for GitHub/Codeberg)
├── scripts          # Utility scripts for environment secret management
└── teams            # Team definitions with members and repos (reference repos by slug)
```

## Adding a New Contributor

For each team you want to join (and you must join at least one):

0. Fork the repository and clone it to your local machine.

1. Follow the instructions in [docs/contributors.md](docs/contributors.md) to add
   your information as a new contributor.

2. Update the file of the team you want to join under the `teams/` directory:
   - If the team file has an `applicants` field, add yourself there.
   - Otherwise, add yourself to the `contributors` field.

3. Review each project's repo's `CONTRIBUTING.md` file to understand any team-specific
   contribution guidelines. For example, some teams might require you submit a Google Form.
   - Each team's repos are listed in the team file; the canonical URL for each repo is in
     the registry at `repos/<slug>.toml` (e.g. <https://github.com/ScottyLabs/governance>).

4. Submit a PR once you are ready. Familiarize yourself with the
   [PR Review Process](https://github.com/ScottyLabs/governance/wiki/PR-Review-Process)
   to understand the expectations for contributors and reviewers.

## Adding a New Team

0. Fork the repository and clone it to your local machine.

1. Follow the instructions in [docs/teams.md](docs/teams.md) to create a new team file.

2. Submit a PR once you are ready. Familiarize yourself with the
   [PR Review Process](https://github.com/ScottyLabs/governance/wiki/PR-Review-Process)
   to understand the expectations for contributors and reviewers.

## Validations

See the [validators README](__meta/validators/README.md) for more information on the checks
your PR must pass before it can be merged.

### Testing locally before you push

Run the same checks that CI runs so you can fix issues before opening a PR.

1. **From the repo root** (the directory that contains `contributors/`, `teams/`, and `repos/`).

2. **EditorConfig** (optional): install [editorconfig-checker](https://github.com/editorconfig-checker/editorconfig-checker) and run:
   ```bash
   editorconfig-checker --exclude LICENSE
   ```

3. **TOML format and schemas**: install [Taplo](https://taplo.tamasfe.dev/cli/introduction.html) (e.g. `cargo binstall taplo-cli`), then:
   ```bash
   taplo fmt --check
   taplo check
   ```

4. **Governance validator** (Rust; requires [rustup](https://rustup.rs/)):
   ```bash
   cargo run --release --bin governance
   ```
   Without `SYNC_GITHUB_TOKEN` and `SLACK_TOKEN`, GitHub/Slack checks will report **warnings** instead of failing; the rest of the rules still run. To mimic CI, set those env vars or use a `.env` file.

5. **Visualizer** (optional): build the graph and open the generated page locally:
   ```bash
   cargo run --release --bin visualizer
   # then open dist/index.html in a browser
   ```

6. **Synchronizer** (optional): only if you need to test sync logic; it needs real secrets and will talk to GitHub/Keycloak/Slack. From the repo root with dependencies installed (e.g. `uv sync`), run:
   ```bash
   uv run sync --services github
   ```
   See [__meta/synchronizer/README.md](__meta/synchronizer/README.md) for setup.

## Registering repositories

Repositories are **registered** in `repos/` so governance can list and link them regardless of host (GitHub, Codeberg) and so teams without a GitHub repo are supported. Follow [docs/repos.md](docs/repos.md) to add a repo; teams then reference it by slug in their `repos` list.

## Synchronization

See the [synchronizer README](__meta/synchronizer/README.md) for more information
on the synchronization process.

## License

This project is licensed under `Apache-2.0`, and is heavily inspired by
[Concourse's governance](https://github.com/concourse/governance).
