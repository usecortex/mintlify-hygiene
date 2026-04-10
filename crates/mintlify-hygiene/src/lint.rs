use crate::config::ResolvedConfig;
use crate::finding::{normalize_repo_path, Finding};
use crate::frontmatter::check_frontmatter_yaml;
use crate::markdown::check_markdown_body;
use crate::nav::nav_slugs_from_file;
use anyhow::Context;
use std::path::Path;
use walkdir::WalkDir;

pub fn lint_project(cfg: &ResolvedConfig) -> anyhow::Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let nav_slugs = if cfg.nav_registration.enabled {
        if !cfg.nav_file.is_file() {
            anyhow::bail!(
                "nav file does not exist: {} (disable nav_registration or fix nav_file)",
                cfg.nav_file.display()
            );
        }
        Some(nav_slugs_from_file(&cfg.nav_file)?)
    } else {
        None
    };

    if !cfg.walk_root.is_dir() {
        anyhow::bail!("scan root is not a directory: {}", cfg.walk_root.display());
    }

    for entry in WalkDir::new(&cfg.walk_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !is_markdown_doc_file(path) {
            continue;
        }

        let rel_root = path.strip_prefix(&cfg.root).unwrap_or(path);
        let key = rel_root.to_string_lossy().replace('\\', "/");
        if !cfg.matches_project_path(&key) {
            continue;
        }

        if cfg.filename_chars.enabled {
            check_filename_chars(cfg, path, &mut findings);
        }

        let src = std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;

        if cfg.frontmatter_yaml.enabled {
            findings.extend(check_frontmatter_yaml(
                &cfg.root,
                path,
                &src,
                cfg.frontmatter_yaml.level,
            ));
        }

        if cfg.unescaped_lt.enabled || cfg.prose_em_dash.enabled {
            findings.extend(check_markdown_body(
                &cfg.root,
                path,
                &src,
                cfg.mdx_parse_mode,
                cfg.unescaped_lt.enabled,
                cfg.unescaped_lt.level,
                cfg.prose_em_dash.enabled,
                cfg.prose_em_dash.level,
            ));
        }

        if let Some(ref slugs) = nav_slugs {
            if cfg.nav_registration.enabled {
                let slug = doc_slug(cfg, path);
                if !slugs.contains(&slug) {
                    findings.push(Finding {
                        rule_id: "nav_registration",
                        severity: cfg.nav_registration.level,
                        path: normalize_repo_path(&cfg.root, path),
                        line: 1,
                        column: 1,
                        message: format!(
                            "page not listed in nav ({}); add it to navigation so readers can open it",
                            cfg.nav_file.display()
                        ),
                    });
                }
            }
        }
    }

    findings.sort_by(|a, b| {
        (&a.path, a.line, a.column, a.rule_id, &a.message)
            .cmp(&(&b.path, b.line, b.column, b.rule_id, &b.message))
    });

    Ok(findings)
}

fn is_markdown_doc_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("md" | "mdx")
    )
}

fn doc_slug(cfg: &ResolvedConfig, path: &Path) -> String {
    let rel = path
        .strip_prefix(&cfg.docs_dir)
        .unwrap_or(path)
        .with_extension("");
    rel.to_string_lossy().replace('\\', "/")
}

fn check_filename_chars(cfg: &ResolvedConfig, path: &Path, findings: &mut Vec<Finding>) {
    let rel = match path.strip_prefix(&cfg.docs_dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for seg in rel.components() {
        let s = seg.as_os_str().to_string_lossy();
        if s.contains('(') || s.contains(')') {
            findings.push(Finding {
                rule_id: "filename_chars",
                severity: cfg.filename_chars.level,
                path: normalize_repo_path(&cfg.root, path),
                line: 1,
                column: 1,
                message: format!(
                    "path segment `{s}` contains `(` or `)`; Mintlify routing can break - use only letters, digits, `_`, `-`, and `.`"
                ),
            });
            break;
        }
    }
}
