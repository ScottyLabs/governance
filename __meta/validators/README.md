# Validators

We enforce [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

We also include several other checks to ensure integrity:

- The `github-username` field for contributors must match the filename.
- Cross-references must be valid (team members must exist as contributors)
- GitHub users must exist
- Slack member IDs and channel IDs must be valid

## Development

Validation runs automatically through GitHub Actions on PRs and pushes to main, so the following instructions will not be relevant to most people. If you are interested, you can also test validators locally:

1. Make sure you are in the root of the repository.

2. Install Rust with `rustup`, if you do not already have it installed:

    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

3. Install `cargo-binstall` (enables use of `cargo binstall`):

    ```sh
    cargo install cargo-binstall
    ```

4. Install Taplo:

    ```sh
    cargo binstall taplo-cli
    ```

5. Check TOML files for proper formatting and/or against the schemas:

    ```sh
    taplo fmt --check # for formatting
    taplo check # against the schemas
    ```

6. Run the other checks specified above:

    ```sh
    cargo run --bin governance
    ```
