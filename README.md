# governance

Declarative configuration for ScottyLabs org structure, access, and integrations. Human-edited TOML under `data/` is validated and compiled into OpenTofu JSON (`.tf.json`) plus `.forgejo/CODEOWNERS` by the `governance` CLI.

## Repository layout

| Path | Purpose |
|------|---------|
| `data/org.toml` | Organization-wide settings (forges, IdP, comms, defaults). |
| `data/teams/*.toml` | One file per team: members, repos, projects, channels. |
| `schemas/` | JSON Schema for `org.toml` and team files (generated; keep in sync). |
| `tofu/` | OpenTofu root module (`.tf` + generated `.tf.json`). |
| `crates/governance-schema` | Serde types and schema source of truth. |
| `crates/governance-core` | Load TOML, validate, Keycloak helpers, PR policy checks. |
| `crates/governance-tfgen` | Emit Terraform JSON and CODEOWNERS. |
| `crates/governance-cli` | `governance` binary. |

## Prerequisites

- [Rust](https://www.rust-lang.org/) (stable) for building the CLI.
- [OpenTofu](https://opentofu.org/) or Terraform to apply what lives under `tofu/` (outside this README’s scope).
- Optional: [taplo](https://taplo.tamasfe.dev/) for TOML formatting and `editorconfig-checker` (used in CI).

## Building

```bash
cargo build --release -p governance
```

The binary is `./target/release/governance`.

## Data model

### `data/org.toml`

Top-level table is `[org]`. Required fields include `name`, `tech_director`, and `default_forge` (`github` or `forgejo`). Optional sections configure Forgejo, GitHub, Keycloak, Discord/Slack hub metadata, Google Groups, Vaultwarden, Figma, and default repo policies. See `schemas/org.schema.json` for the full shape.

### `data/teams/<slug>.toml`

Each file describes one team: `slug`, optional `name`/`description`, `leads`, `members`, team-level `repos` and `channels`, and optional `projects` (each project can have its own leads, members, repos, channels, and Figma project IDs). Repo entries require `name`; optional fields include `description`, `forge`, `visibility`, `topics`, and `kennel` (for deployment integration). See `schemas/team.schema.json`.

**Validation rules (high level):**

- Team `slug` values must be unique across files; repo `name` values must be unique across the whole dataset.
- Someone cannot be both a `lead` and a `member` in the same group (team or project).
- `default_forge` and any per-repo `forge` overrides must refer to a forge block that exists in `org.toml`.

## CLI

All commands accept `--data-dir <path>` (default: `data`).

| Command | Description |
|---------|-------------|
| `validate` | Parse TOML and run validation (includes live checks when credentials are set; see below). |
| `generate --output-dir <dir>` | Write generated `.tf.json` files into `<dir>` and `.forgejo/CODEOWNERS` at the repo root. Default output dir: `tofu`. |
| `schema --output-dir <dir>` | Regenerate `org.schema.json` and `team.schema.json` into `<dir>`. Default: `schemas`. |
| `check-pr --author <login> --base-ref <ref> --changed-files a,b,c` | Enforce who may change which paths on a pull request (used by CI). |
| `resolve-identity` | Read JSON from stdin with `codeberg_user`, query Keycloak identity links (needs Keycloak env vars). |
| `slack-invite` / `slack-kick` | Admin-style Slack channel membership helpers (`SLACK_TOKEN` required). |

Examples:

```bash
./target/release/governance validate
./target/release/governance generate
./target/release/governance schema
```

A `.env` file is loaded automatically when present (`dotenvy`).

### Environment variables

| Variable | When it matters |
|----------|-----------------|
| `DISCORD_TOKEN` | If any team or project references a Discord channel ID, validation calls the Discord API to confirm the channel exists. |
| `SLACK_TOKEN` | Same for Slack channel IDs. Also required for `slack-invite` / `slack-kick`. |
| `KEYCLOAK_CLIENT_ID`, `KEYCLOAK_CLIENT_SECRET` | If `[org.keycloak]` is set, validation checks that each listed member has Keycloak users and linked identities (CMU SAML, Codeberg/Forgejo, GitHub, Discord, Slack as configured). |

If Discord or Slack channels are configured but the matching token is missing, validation reports an error and skips remote channel checks. If Keycloak is configured but client credentials are missing, validation adds an error for missing credentials.

## Generated artifacts and CI

Pull requests run validation, then confirm generated output matches the tree:

1. `governance generate` to a temporary directory and `diff` against `tofu/` (excluding hand-written `*.tf` and `.gitignore`).
2. `governance schema` and `diff` against `schemas/`.
3. Compare `.forgejo/CODEOWNERS` before and after generate.

After editing `data/`, run `generate` and `schema` locally and commit the updated JSON, schemas, and CODEOWNERS so CI stays green.

## Pull request policy (`check-pr`)

`governance check-pr` encodes a simple ownership model:

- The user named in `org.toml` as `tech_director`, or any lead of the `devops` team, may change anything.
- Others may not modify `data/org.toml`, `crates/`, `tofu/`, `schemas/`, or `.forgejo/` (except via generated CODEOWNERS flow owned by maintainers).
- For `data/teams/<slug>.toml`: team leads may edit their team; any lead may add a **new** team file. Non-leads may only apply a narrow set of self-service edits (for example adding or removing themselves in certain roles); anything else is denied.

Exact rules live in `crates/governance-core/src/check_pr.rs`.

## Licenses

This project is licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE-2.0), at your option.
