//! Auto-fixes mirror lint rules: only safe, deterministic edits. Each fix is tied to a `rule_id`
//! and applies in the same regions as the corresponding check (e.g. prose only, not code fences).

use crate::config::ResolvedConfig;
use crate::frontmatter::body_start_byte;
use markdown::mdast::Node;
use markdown::{to_mdast, ParseOptions};
use std::path::Path;
use walkdir::WalkDir;

/// Rule IDs that support `--auto-fix` (for docs / future CLI help).
pub const AUTOFIX_RULE_IDS: &[&str] = &["prose_em_dash"];

fn is_markdown_doc_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("md" | "mdx")
    )
}

/// Apply all enabled auto-fixes to markdown files under `docs_dir`. Returns count of files written.
pub fn autofix_project(cfg: &ResolvedConfig) -> anyhow::Result<usize> {
    let mut fixed_files = 0usize;

    for entry in WalkDir::new(&cfg.docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || !is_markdown_doc_file(path) {
            continue;
        }
        let rel_root = path.strip_prefix(&cfg.root).unwrap_or(path);
        let key = rel_root.to_string_lossy().replace('\\', "/");
        if cfg.exclude.is_match(&key) {
            continue;
        }

        let src = std::fs::read_to_string(path)?;
        let Some(new_src) = apply_markdown_autofixes(&src, cfg) else {
            continue;
        };
        std::fs::write(path, new_src)?;
        fixed_files += 1;
    }

    Ok(fixed_files)
}

/// Returns new full document source if any enabled auto-fix changed it.
pub fn apply_markdown_autofixes(full_source: &str, cfg: &ResolvedConfig) -> Option<String> {
    let mut out = full_source.to_string();
    let mut changed = false;

    if cfg.prose_em_dash.enabled {
        if let Some(patched) = replace_prose_em_dashes_in_body(&out) {
            out = patched;
            changed = true;
        }
    }

    changed.then_some(out)
}

/// Replace U+2014 with ` - ` only in mdast text/html nodes (same scope as `prose_em_dash` lint).
fn replace_prose_em_dashes_in_body(full_source: &str) -> Option<String> {
    let body_off = body_start_byte(full_source);
    let body = full_source.get(body_off..)?;
    let tree = to_mdast(body, &ParseOptions::gfm()).expect("GFM mdast parse");

    let mut patches: Vec<(usize, usize, String)> = Vec::new();
    collect_em_dash_patches(&tree, &mut patches);

    if patches.is_empty() {
        return None;
    }

    // Apply from the end so offsets stay valid.
    patches.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));

    let mut new_body = body.to_string();
    for (start, end, replacement) in patches {
        debug_assert!(new_body.is_char_boundary(start) && new_body.is_char_boundary(end));
        new_body.replace_range(start..end, &replacement);
    }

    let mut out = String::with_capacity(full_source.len().saturating_sub(body.len()) + new_body.len());
    out.push_str(&full_source[..body_off]);
    out.push_str(&new_body);
    Some(out)
}

fn collect_em_dash_patches(node: &Node, patches: &mut Vec<(usize, usize, String)>) {
    match node {
        Node::Text(t) => {
            if !t.value.contains('\u{2014}') {
                return;
            }
            let Some(pos) = t.position.as_ref() else {
                return;
            };
            let new_val = t.value.replace('\u{2014}', " - ");
            if new_val == t.value {
                return;
            }
            patches.push((pos.start.offset, pos.end.offset, new_val));
        }
        Node::Html(h) => {
            if !h.value.contains('\u{2014}') {
                return;
            }
            let Some(pos) = h.position.as_ref() else {
                return;
            };
            let new_val = h.value.replace('\u{2014}', " - ");
            if new_val == h.value {
                return;
            }
            patches.push((pos.start.offset, pos.end.offset, new_val));
        }
        Node::Code(_)
        | Node::InlineCode(_)
        | Node::Math(_)
        | Node::InlineMath(_)
        | Node::Break(_)
        | Node::FootnoteReference(_)
        | Node::Image(_)
        | Node::ImageReference(_)
        | Node::Definition(_)
        | Node::ThematicBreak(_)
        | Node::Toml(_)
        | Node::Yaml(_)
        | Node::MdxjsEsm(_)
        | Node::MdxFlowExpression(_)
        | Node::MdxTextExpression(_) => {}

        Node::Root(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::Blockquote(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::FootnoteDefinition(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::MdxJsxFlowElement(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::List(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::Delete(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::Emphasis(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::MdxJsxTextElement(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::Link(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::LinkReference(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::Strong(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::Heading(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::Table(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::TableRow(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::TableCell(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::ListItem(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
        Node::Paragraph(n) => {
            for c in &n.children {
                collect_em_dash_patches(c, patches);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn em_dash_in_prose_replaced_not_in_fenced_code() {
        let src = "---\ntitle: T\n---\n\nHello—world.\n\n```\nx—y\n```\n";
        let out = replace_prose_em_dashes_in_body(src).expect("changed");
        assert!(out.contains("Hello - world."));
        assert!(out.contains("x—y"), "code fence should keep em dash");
    }
}
