### Adding a repository

> [!NOTE]
> Any individual is welcome to register repositories under governance.

Create a new TOML file in `repos/` with the repository name as the filename, e.g. `cmucourses.toml`:

```toml
name = "cmucourses"
description = "..." # Empty string if no description
websites = ["..."] # Empty array if no websites
```

In addition to `name`, `description` and `websites` are required fields, so you must use `""` and `[]` respectively to denote their absence. All repos should ideally have a description, though.
