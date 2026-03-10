# Schemas

This directory contains the JSON schemas for the TOML files in the `contributors/`, `teams/`, and `repos/` directories.

- **contributor.schema.json** — `contributors/*.toml` (per-contributor metadata).
- **team.schema.json** — `teams/*.toml` (per-team metadata; `repos` list can be repo slugs from `repos/` or legacy `owner/repo` for GitHub).
- **repo.schema.json** — `repos/*.toml` (registered repos: source of truth for description and URL; enables GitHub, Codeberg, and teams without a home repo).

## Troubleshooting

If you are using the VSCode extension [tamasfe.even-better-toml](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml), you may need to clear the cache to see the changes after updating a schema.

Run the following command to find the cache directory:

```zsh
find ~ -type d -name "tamasfe.even-better-toml"
```

After you delete the cache directory and reload VSCode, the extension should
start using the updated schemas.
