# mintlify-hygiene

Rust CLI that lints [Mintlify](https://mintlify.com)-style documentation trees: filenames, YAML frontmatter, prose patterns, navigation registration in `docs.json`, and optional strict MDX parsing.

## Requirements

- Rust toolchain (2024 edition)

## Build

```bash
cargo build --release
```

The binary is `target/release/mintlify-hygiene`.

## Usage

From your docs repo root (where `mintlify-hygiene.toml` lives):

```bash
mintlify-hygiene check
```

Common flags:

- `--root <path>` — project root (default: current directory)
- `--config <path>` — config file relative to root (default: `mintlify-hygiene.toml`)
- `--json` — print findings as JSON on stdout
- `--deny-warnings` — non-zero exit if any warnings
- `--auto-fix` — apply safe fixes (e.g. em dash rule) then check
- `--mdx-parse-mode loose|strict` — override config (`loose` is default)

## Configuration

Add a `mintlify-hygiene.toml` and tune `[project]` (`docs_dir`, `nav_file`, `include` / `exclude`, `mdx_parse_mode`) plus per-rule `[rules.*]` blocks. Example:

```toml
[project]
docs_dir = "docs"
nav_file = "docs/docs.json"

[rules.unescaped_lt]
enabled = true

[rules.frontmatter_yaml]
enabled = true

[rules.filename_chars]
enabled = true

[rules.nav_registration]
enabled = true

[rules.prose_em_dash]
enabled = true
```

The crate also exposes a library API (`mintlify_hygiene`) for embedding checks in other tools.

## Snapshot testing

CLI and report output is covered by [insta](https://github.com/mitsuhiko/insta) snapshots under `crates/mintlify-hygiene/tests/snapshots/`. Run `cargo test -p mintlify-hygiene` from the repo root. When output is meant to change, refresh snapshots with `INSTA_UPDATE=always cargo test -p mintlify-hygiene`, or install [`cargo-insta`](https://crates.io/crates/cargo-insta) (`cargo install cargo-insta`) and run `cargo insta test --accept` to review or accept updates.

## License

MIT — see workspace `Cargo.toml` for repository URL.
