use crate::finding::{Finding, Severity};
use crate::finding::normalize_repo_path;
use crate::frontmatter::line_col_at_byte;
use markdown::mdast::Node;
use markdown::{to_mdast, ParseOptions};
use std::path::Path;

pub fn check_markdown_body(
    root: &Path,
    file: &Path,
    full_source: &str,
    unescaped_lt: bool,
    unescaped_level: Severity,
    prose_em_dash: bool,
    em_dash_level: Severity,
) -> Vec<Finding> {
    let body_off = crate::frontmatter::body_start_byte(full_source);
    let body = full_source.get(body_off..).unwrap_or("");

    // GFM (no MDX): `to_mdast` does not report syntax errors.
    let tree = to_mdast(body, &ParseOptions::gfm()).expect("GFM mdast parse");

    let mut ctx = WalkCtx {
        root,
        file,
        full_source,
        body_off,
        unescaped_lt,
        unescaped_level,
        prose_em_dash,
        em_dash_level,
        out: Vec::new(),
    };
    walk_node(&tree, &mut ctx);
    ctx.out
}

struct WalkCtx<'a> {
    root: &'a Path,
    file: &'a Path,
    full_source: &'a str,
    body_off: usize,
    unescaped_lt: bool,
    unescaped_level: Severity,
    prose_em_dash: bool,
    em_dash_level: Severity,
    out: Vec<Finding>,
}

fn walk_node(node: &Node, ctx: &mut WalkCtx<'_>) {
    match node {
        Node::Text(t) => {
            scan_slice(
                ctx,
                &t.value,
                t.position.as_ref().map(|p| p.start.offset),
            );
        }
        Node::Html(h) => {
            scan_slice(
                ctx,
                &h.value,
                h.position.as_ref().map(|p| p.start.offset),
            );
        }
        // Code and math literals: do not lint inside.
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

        Node::Root(n) => walk_children(&n.children, ctx),
        Node::Blockquote(n) => walk_children(&n.children, ctx),
        Node::FootnoteDefinition(n) => walk_children(&n.children, ctx),
        Node::MdxJsxFlowElement(n) => walk_children(&n.children, ctx),
        Node::List(n) => walk_children(&n.children, ctx),
        Node::Delete(n) => walk_children(&n.children, ctx),
        Node::Emphasis(n) => walk_children(&n.children, ctx),
        Node::MdxJsxTextElement(n) => walk_children(&n.children, ctx),
        Node::Link(n) => walk_children(&n.children, ctx),
        Node::LinkReference(n) => walk_children(&n.children, ctx),
        Node::Strong(n) => walk_children(&n.children, ctx),
        Node::Heading(n) => walk_children(&n.children, ctx),
        Node::Table(n) => walk_children(&n.children, ctx),
        Node::TableRow(n) => walk_children(&n.children, ctx),
        Node::TableCell(n) => walk_children(&n.children, ctx),
        Node::ListItem(n) => walk_children(&n.children, ctx),
        Node::Paragraph(n) => walk_children(&n.children, ctx),
    }
}

fn walk_children(children: &[Node], ctx: &mut WalkCtx<'_>) {
    for c in children {
        walk_node(c, ctx);
    }
}

fn scan_slice(ctx: &mut WalkCtx<'_>, text: &str, start_offset_in_body: Option<usize>) {
    let Some(base_in_body) = start_offset_in_body else {
        return;
    };
    let path = normalize_repo_path(ctx.root, ctx.file);

    if ctx.unescaped_lt {
        let bytes = text.as_bytes();
        for i in 0..bytes.len() {
            if bytes[i] != b'<' {
                continue;
            }
            let Some(&next) = bytes.get(i + 1) else {
                continue;
            };
            if !next.is_ascii_alphanumeric() {
                continue;
            }
            let abs = ctx.body_off + base_in_body + i;
            let (line, col) = line_col_at_byte(ctx.full_source, abs);
            ctx.out.push(Finding {
                rule_id: "unescaped_lt",
                severity: ctx.unescaped_level,
                path: path.clone(),
                line,
                column: col,
                message: "raw `<` before a letter or digit; escape as `&lt;` outside code fences"
                    .to_owned(),
            });
        }
    }

    if ctx.prose_em_dash {
        for (i, ch) in text.char_indices() {
            if ch == '\u{2014}' {
                let abs = ctx.body_off + base_in_body + i;
                let (line, col) = line_col_at_byte(ctx.full_source, abs);
                ctx.out.push(Finding {
                    rule_id: "prose_em_dash",
                    severity: ctx.em_dash_level,
                    path: path.clone(),
                    line,
                    column: col,
                    message: "Unicode em dash (U+2014) in prose; use ASCII hyphen spacing instead"
                        .to_owned(),
                });
            }
        }
    }
}
