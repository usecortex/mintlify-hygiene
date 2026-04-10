# Mintlify Hygiene Codebase Overview

## What this repo is

This repository contains a small Rust CLI for checking documentation hygiene in Mintlify-style docs repos. The workspace currently has one crate, `crates/mintlify-hygiene`, which exposes both a library API and a command-line interface.

The core job of the tool is:

- load a `mintlify-hygiene.toml` config
- discover markdown and MDX files in a target repo
- run a set of lint rules over filenames, frontmatter, prose, and navigation registration
- optionally auto-fix a narrow set of safe formatting issues

## Main architecture

The code is split into a few focused modules:

- `main.rs`: CLI entry point built with `clap`
- `lib.rs`: orchestration layer that loads config, resolves paths, optionally runs auto-fix, and then lints
- `config.rs`: TOML config loading and resolution of include/exclude globs, docs roots, nav path, and MDX parse mode
- `lint.rs`: filesystem walk plus per-file rule execution
- `frontmatter.rs`: frontmatter checks and body offset helpers
- `markdown.rs`: prose/body checks such as `unescaped_lt`, `prose_em_dash`, and MDX parse handling
- `nav.rs`: extracts registered slugs from `docs.json`
- `autofix.rs`: safe text rewrites, currently only `prose_em_dash`
- `finding.rs`: common finding structure plus human and JSON output rendering

At a high level, the flow is:

1. CLI parses flags.
2. Config is loaded and merged with CLI overrides.
3. The repo is scanned using include/exclude filters.
4. Each candidate file is checked for frontmatter, prose/body issues, filename issues, and nav registration.
5. Findings are printed in human and/or JSON form, and exit code is derived from severity.

## Supported rules today

The main rule set currently includes:

- `unescaped_lt`
- `frontmatter_yaml`
- `filename_chars`
- `nav_registration`
- `prose_em_dash`

There is also an `mdx_parse` finding path now, but it is tied to strict MDX validation mode rather than a fully separate configurable lint rule.

## Config model

The config is intentionally simple and repo-oriented.

Important `project` settings:

- `docs_dir`: traditional docs-root path
- `nav_file`: where Mintlify navigation lives
- `include`: globs to scope linting to published files
- `exclude`: globs to suppress repo-meta or archived content
- `mdx_parse_mode`: `loose` or `strict`

Important design choice:

- if `include` is present, the tool walks the project root and filters by glob
- otherwise it walks `docs_dir`

This makes it work for both:

- classic repos with `docs/docs.json`
- Mintlify repos with `docs.json` at repo root and pages spread across root subfolders

## Markdown and MDX behavior

The codebase currently treats `.md` and `.mdx` as first-class inputs.

There are now two MDX modes:

- `loose`: compatibility-first mode, and the default
- `strict`: validates `.mdx` input with `markdown-rs` MDX parsing and emits `mdx_parse` on malformed MDX

Current implementation detail:

- loose mode still falls back to the established GFM-based lint walk for actual prose checks
- strict mode adds MDX validation first, then still falls back to GFM for the rule walk if parsing fails

That means the tool can validate MDX syntax without regressing current Mintlify repo behavior.

## Auto-fix scope

Auto-fix is deliberately narrow. It currently supports:

- `prose_em_dash`

The auto-fix pipeline rewrites prose em dashes while avoiding fenced code, then reruns linting. It uses the same parser option selection helper as the lint path for consistency.

## Test strategy

The repo has two main test layers:

- CLI/behavior tests in `crates/mintlify-hygiene/tests/cli.rs`
- snapshot tests in `crates/mintlify-hygiene/tests/snapshots.rs`

The fixtures under `crates/mintlify-hygiene/tests/fixtures/` model several real-world cases:

- failing and clean docs sites
- root-layout Mintlify repos
- include/exclude override behavior
- representative MDX cases from `mintlify-docs`
- malformed MDX for strict parse-mode coverage

Snapshot coverage is being used to lock down:

- representative report output
- false-positive regressions around MDX component tags
- nav-registration output
- JSON output shape
- strict-mode MDX parse reporting

## Practical status of the codebase

The codebase is small, understandable, and currently optimized for practical Mintlify hygiene checking rather than full semantic MDX analysis.

Key strengths:

- focused scope
- clear module boundaries
- real fixture-driven coverage
- works on both root-layout and docs-root repos
- useful JSON output for automation

Current tradeoff:

- strict MDX validation exists, but the actual lint walk still leans on the more compatibility-friendly GFM parse path for content checks

That is a deliberate compatibility choice at the moment, not an accident.

## Likely next areas of work

If this tool grows, the most natural next steps seem to be:

- decide whether `mdx_parse` should become a fully configurable rule
- improve strict-mode semantics so more valid Mintlify-flavored MDX is understood structurally
- expand auto-fix support beyond `prose_em_dash`
- add more repo-level docs for normal usage outside the fixture README
