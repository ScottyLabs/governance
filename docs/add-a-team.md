# Add a new team

Teams live as **one TOML file per team** under `data/teams/`. The file name should match the team **`slug`** (for example `design.toml` for `slug = "design"`). Each slug must be unique in the folder.

This page assumes you already have permission to open the PR (see **Who can create a new team?** below). To add people to an existing team, see [add-a-user.md](add-a-user.md).

## Link Keycloak for every person you list

Anyone in `leads` or `members` (team-wide or inside a project) must pass Keycloak checks when `[org.keycloak]` is set in `data/org.toml`.

Have each person complete **Linked accounts** before you rely on CI going green:

1. [ScottyLabs linked accounts](https://idp.scottylabs.org/realms/scottylabs/account/account-security/linked-accounts) (same `url` / `realm` as in `data/org.toml`).
2. Link every required provider; use **page 2** (and beyond) if the UI splits the list across pages.

More context: [add-a-user.md](add-a-user.md#link-keycloak-accounts-first).

## Who can create a new team?

You can add a **new** file `data/teams/<slug>.toml` only if you are already a **lead on some team or project** in this repo, or you are the **tech director** / a **DevOps lead** (same admin path as other sensitive edits). Otherwise, ask DevOps or a lead to add the team or promote you first. Details: `crates/governance-core/src/check_pr.rs`.

## What a team file looks like

Here is the real **`data/teams/devops.toml`** in full. A **new** team file should follow the same **shape** (`[team]`, optional `[[team.repos]]`, optional `[[team.channels]]`, optional `[[team.projects]]` in other teams), but with **your** `slug`, `name`, people, repos, and channels—not the `devops` slug and not duplicate repo names org-wide.

```toml
[team]
name = "DevOps"
slug = "devops"
description = "Infrastructure and deployment"
leads = ["anish", "thesuperRL"]

[[team.repos]]
name = "infrastructure"
description = "NixOS configurations for ScottyLabs' VMs"

[[team.repos]]
name = "governance"
description = "Definition and automation of the Tech Committee's governance model"

[[team.repos]]
name = "kennel"
description = "Branch-based deployment platform"
kennel = true

[[team.repos]]
name = "devenv"
description = "Shared Nix development environment for ScottyLabs projects"

[[team.channels]]
discord = "1461933322505818156"
slack = "C08K3Q77ZQF"
```

Start from a small `[team]` block with `slug`, `name`, `description`, and `leads`, then add `[[team.repos]]` and `[[team.channels]]` when you are ready. Teams with sub-units also use `[[team.projects]]`; see another file under `data/teams/` for a live project example.

The JSON schema `schemas/team.schema.json` lists every allowed field.

## Watch out for these validation gotchas

- **Repo `name`** must be unique **across every team file**, not only inside your team.
- **Leads vs members:** the same username cannot appear in both `leads` and `members` in the same group.
- **`forge` on a repo:** if you set `forge = "github"` or `"forgejo"`, the matching section must exist in `data/org.toml`.
- **`default_forge`** in `org.toml` must match a forge you actually configured.

## Before you push: what CI runs and what you should run

Your PR goes through **Validate** (`.forgejo/workflows/validate.yml`): **EditorConfig** → **TOML** → **Governance**.

| Step | What it is |
|------|------------|
| EditorConfig | Whitespace, newlines, encoding—see [`.editorconfig`](../.editorconfig). |
| Taplo | `taplo fmt --check` then `taplo check` against [`taplo.toml`](../taplo.toml) and the JSON schemas. |
| Build + governance | `cargo build --release -p governance`, then `governance validate`, and a check that generated files match the tree. |

Environment variables for Discord, Slack, and Keycloak in CI are described in the [README](../README.md#environment-variables).

### Regenerate files

```bash
cargo run -p governance -- schema
cargo run -p governance -- generate
```

Hand-written files under `tofu/` (`*.tf`, `.gitignore`) are ignored by that directory diff. Infra changes can also trigger **OpenTofu Plan / Apply** on paths under `data/**` and `tofu/**`; this repo’s Validate job does not run `cargo fmt` or `clippy`.

## After your PR merges

Merging updates **desired state**. Access and cloud resources still follow your org’s **OpenTofu** pipeline for `tofu/` (see the [README](../README.md)).
