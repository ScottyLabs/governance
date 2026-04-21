# Add someone to a team

If you are trying to get **yourself** or a teammate onto a team (Forgejo access, groups, and the rest flow from this repo), you are in the right place. You will edit a small TOML file under `data/teams/` and open a pull request.

**Usernames** are the short logins your forge shows on pull requests (for example your Codeberg username). Use that same spelling in TOML so automation and Keycloak line up.

## Link Keycloak accounts first

CI runs `governance validate`, which checks that people listed in team data exist in Keycloak and have the right **linked accounts** (CMU login, Codeberg, GitHub, Discord, Slack—whatever your org has turned on).

**Ask anyone you are adding** to sign in and link everything **before** your PR lands:

1. Open **Account security → Linked accounts** (ScottyLabs):  
   [https://idp.scottylabs.org/realms/scottylabs/account/account-security/linked-accounts](https://idp.scottylabs.org/realms/scottylabs/account/account-security/linked-accounts)  
   The base URL and realm also live in `data/org.toml` under `[org.keycloak]` if you need to double-check.
2. On that screen, link **every** provider that applies. If the list spans more than one page, open **page 2** (and any further pages) so nothing is skipped.

If links are missing, your PR may fail validation with messages like `MissingIdentityLink` or `KeycloakUserNotFound`.

## Where to edit

- **Team-wide** roster: the `[team]` section in `data/teams/<team>.toml` (`leads` and `members`).
- **Project-only** roster: some teams use `[[team.projects]]` blocks with their own `leads` / `members`. The DevOps file below does not define projects; if your team does, mirror the pattern used in that team’s file.

Do not put the same person in **both** `leads` and `members` for the same team or project block.

## Example: how DevOps lists people today

Below is copied from `data/teams/devops.toml`. Your team file will look similar at the top; add teammates under `members = [...]` (you can add that key if it is not there yet). Keep the same style and quoting.

```toml
[team]
name = "DevOps"
slug = "devops"
description = "Infrastructure and deployment"
leads = ["anish", "thesuperRL"]

```

The rest of the same file lists repos and channels (you usually do not change these when you are only adding a person):

```toml
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

For every field you can set, see `schemas/team.schema.json`.

## Who is allowed to open the PR?

Roughly:

- **Tech director** (named in `data/org.toml`) or a **DevOps team lead** can edit any team file.
- **Leads for that team** can update their own team’s file (including adding or removing people), subject to validation above.
- **Everyone else** is limited to self-service: add or remove **only yourself** from `members` on the team or on a project.

Anything else—adding someone else’s username, changing `leads`, repos, channels, or project layout—needs a lead or the people above. The rules in code live in `crates/governance-core/src/check_pr.rs` if you want the exact logic.

## Before you push: what CI runs and what you should run

Your PR goes through **Validate** (`.forgejo/workflows/validate.yml`): **EditorConfig** → **TOML** → **Governance**.

| Step | What it is |
|------|------------|
| EditorConfig | Whitespace, newlines, encoding—see [`.editorconfig`](../.editorconfig). |
| Taplo | `taplo fmt --check` then `taplo check` against [`taplo.toml`](../taplo.toml) and the JSON schemas. |
| Build + governance | `cargo build --release -p governance`, then `governance validate`, and a check that generated files match the tree. |

Environment variables for Discord, Slack, and Keycloak in CI are described in the [README](../README.md#environment-variables).

Commit **`data/`** together with regenerated **`tofu/*.tf.json`**, **`schemas/*.json`**, and **`.forgejo/CODEOWNERS`**.

### Regenerate files

```bash
cargo run -p governance -- schema
cargo run -p governance -- generate
```

## Adding a new team?

See [add-a-team.md](add-a-team.md).
