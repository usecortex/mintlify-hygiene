# Hygiene fixtures

These examples exist so you can see **what should fail** before the linter is fully wired. Each item maps to a rule id the CLI reports.

The CLI now lints both `.md` and `.mdx` files. The fixtures also cover two common Mintlify layouts:

- classic docs-root sites such as `docs/docs.json`
- repo-root sites with `docs.json` at the project root and `include` / `exclude` globs

## Rule snippets (one concern each)

Read these files first. They are not a full Mintlify tree; they show the pattern in isolation.


| Rule id            | File                                               | What is wrong                                                                                                                                          |
| ------------------ | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `unescaped_lt`     | `rule-snippets/unescaped-lt-in-table.md`           | Raw `<` in table text (e.g. `<200ms`). Mintlify can treat it like HTML and break the page. Prefer `<200ms` outside code fences.                        |
| `frontmatter_yaml` | `rule-snippets/frontmatter-inner-double-quotes.md` | `description: "..."` with unescaped inner `"` breaks YAML; frontmatter may fail silently or parse incorrectly. Use outer single quotes or escape `\"`. |
| `filename_chars`   | `failing-site/docs/weird-(1)-name.md`              | Parentheses in the filename. Mintlify routing can break; stick to letters, digits, `_`, `-`, and `.` (see config).                                     |
| `nav_registration` | `failing-site/docs/orphan-not-in-navigation.md`    | File exists under the docs root but is not listed in `docs.json` navigation. Readers never see it.                                                     |
| `prose_em_dash`    | `rule-snippets/em-dash-in-prose.md`                | Unicode em dash (U+2014) in body prose. Style rule to avoid “obviously generated” copy. Fenced code blocks are ignored.                                |


Safe contrast: fenced code is allowed to contain `<` and em dashes. See `rule-snippets/ok-in-fenced-code.md`. MDX component tags such as `<Tabs>` and `<Tab title="...">` should not trigger `unescaped_lt`.

## Full failing site

Directory `failing-site/` is a tiny repo layout:

- `mintlify-hygiene.toml` — config the CLI loads
- `docs/docs.json` — Mintlify-style `navigation` / `pages` entries
- Several markdown pages, each triggering at least one rule (plus one orphan page for `nav_registration`)

Run from `failing-site/`:

```bash
mintlify-hygiene check
```

You should see multiple findings with stable rule ids until everything is fixed.

## Good site

Directory `good-site/` is a minimal layout expected to produce **zero** findings with default rules enabled.

## Root-layout fixtures

Directory `root-layout-site/` models a Mintlify repo that keeps `docs.json` at the repo root and uses:

- `include = ["published/**/*.mdx"]` to scope checks to published pages
- `exclude = [...]` to skip repo-meta or snippet content

Directory `root-layout-site-overrides/` exists to exercise CLI overrides such as:

```bash
mintlify-hygiene check --include published/index.mdx
mintlify-hygiene check --exclude archive/** --exclude published/guide.mdx
```

## Auto-fix

`mintlify-hygiene check --auto-fix` currently supports `prose_em_dash` only. It rewrites em dashes in prose, skips fenced code, and then reruns checks.

## Snapshot fixtures

The `snapshot-*` directories are representative Mintlify-style fixtures based on issues seen in `mintlify-docs`, including:

- MDX component tags mixed with real prose `<...` findings
- em dashes in callouts and list prose
- repo-root nav registration failures
- human and JSON snapshot output for regression coverage