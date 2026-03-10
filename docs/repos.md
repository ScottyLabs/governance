# Registering a repo

> [!NOTE]
> Repositories are **registered** in `repos/` so the system can display and use them without requiring a GitHub repo. This enables Codeberg, other hosts, and teams that do not have a home repository. The registry is the source of truth for repo metadata and URLs. **Governance does not sync repo descriptions to GitHub or other hosts**—team maintainers may edit descriptions on the host as they wish.

Create a new TOML file in `repos/` with the repo slug as the filename, e.g. `cmumaps.toml`:

```toml
slug = "cmumaps"
name = "CMU Maps"
description = "Web application providing a map of the Carnegie Mellon University campus."
url = "https://github.com/ScottyLabs/cmumaps"
```

- **slug** (required): Internal identifier; must match the filename (without `.toml`). Teams reference repos by this slug in their `repos` list.
- **name** (required): Display name for the repository.
- **description** (optional): Short description for use in governance tooling (e.g. the visualizer). **Governance does not sync this to GitHub or other hosts**—team maintainers may edit the repo description on the host as they wish; requiring a PR here for every update would be unnecessary overhead.
- **url** (required): Canonical URL of the repository. Use a full URL so the host is unambiguous, e.g.:
  - GitHub: `https://github.com/ScottyLabs/cmumaps`
  - Codeberg: `https://codeberg.org/org/repo-name`

Visit the [repo schema](../__meta/schemas/repo.schema.json) for the full schema.

After registering a repo, teams can reference it by slug in their `repos` array (e.g. `repos = ["cmumaps"]`). Only repos with a GitHub URL are synced to GitHub teams; Codeberg and other hosts are listed from the registry only.
