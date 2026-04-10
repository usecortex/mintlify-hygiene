use crate::config::MdxParseMode;
use crate::finding::{Finding, Severity};
use crate::finding::normalize_repo_path;
use crate::frontmatter::line_col_at_byte;
use markdown::mdast::Node;
use markdown::message::Place;
use markdown::{to_mdast, Constructs, MdxSignal, ParseOptions};
use std::path::Path;

pub fn check_markdown_body(
    root: &Path,
    file: &Path,
    full_source: &str,
    mdx_parse_mode: MdxParseMode,
    unescaped_lt: bool,
    unescaped_level: Severity,
    prose_em_dash: bool,
    em_dash_level: Severity,
) -> Vec<Finding> {
    let body_off = crate::frontmatter::body_start_byte(full_source);
    let body = full_source.get(body_off..).unwrap_or("");
    if file.extension().and_then(|s| s.to_str()) == Some("mdx")
        && matches!(mdx_parse_mode, MdxParseMode::Strict)
    {
        if let Err(message) = to_mdast(body, &parse_options_for(file)) {
            return vec![mdx_parse_finding(
                root,
                file,
                full_source,
                body_off,
                &message,
                mdx_parse_mode,
            )];
        }
    }

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

pub(crate) fn parse_options_for(path: &Path) -> ParseOptions {
    let is_mdx = path.extension().and_then(|s| s.to_str()) == Some("mdx");
    if !is_mdx {
        return ParseOptions::gfm();
    }

    ParseOptions {
        constructs: Constructs {
            autolink: false,
            code_indented: false,
            html_flow: false,
            html_text: false,
            mdx_esm: true,
            mdx_expression_flow: true,
            mdx_expression_text: true,
            mdx_jsx_flow: true,
            mdx_jsx_text: true,
            ..Constructs::gfm()
        },
        mdx_esm_parse: Some(Box::new(|_value| MdxSignal::Ok)),
        ..ParseOptions::gfm()
    }
}

fn mdx_parse_finding(
    root: &Path,
    file: &Path,
    full_source: &str,
    body_off: usize,
    message: &markdown::message::Message,
    mode: MdxParseMode,
) -> Finding {
    let prefix_line_count = full_source[..body_off]
        .bytes()
        .filter(|b| *b == b'\n')
        .count();
    let (line, column) = match message.place.as_deref() {
        Some(Place::Point(point)) => (point.line + prefix_line_count, point.column),
        Some(Place::Position(position)) => (
            position.start.line + prefix_line_count,
            position.start.column,
        ),
        None => (1, 1),
    };
    Finding {
        rule_id: "mdx_parse",
        severity: mode.severity(),
        path: normalize_repo_path(root, file),
        line,
        column,
        message: format!("MDX parse error: {}", message.reason),
    }
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
                true,
            );
        }
        Node::Html(h) => {
            scan_slice(
                ctx,
                &h.value,
                h.position.as_ref().map(|p| p.start.offset),
                false,
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

fn scan_slice(
    ctx: &mut WalkCtx<'_>,
    text: &str,
    start_offset_in_body: Option<usize>,
    scan_unescaped_lt: bool,
) {
    let Some(base_in_body) = start_offset_in_body else {
        return;
    };
    let path = normalize_repo_path(ctx.root, ctx.file);

    if ctx.unescaped_lt && scan_unescaped_lt {
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

#[cfg(test)]
mod tests {
    use super::check_markdown_body;
    use super::parse_options_for;
    use crate::config::MdxParseMode;
    use markdown::to_mdast;
    use markdown::mdast::Node;
    use crate::finding::Severity;
    use std::path::Path;

    #[test]
    fn ignores_mdx_tags_but_flags_prose_less_than() {
        let src = "### Examples\n\n<Tabs>\n<Tab title=\"API Request\" />\n</Tabs>\n\nCreate responses under <200ms.\n";
        let findings = check_markdown_body(
            Path::new("."),
            Path::new("page.mdx"),
            src,
            MdxParseMode::Loose,
            true,
            Severity::Error,
            false,
            Severity::Warn,
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "unescaped_lt");
        assert_eq!(findings[0].line, 7);
    }

    #[test]
    fn mdx_parse_options_support_esm_jsx_and_expressions() {
        let src = "import Foo from './foo.js'\n\n<Tabs>{value}</Tabs>\n";
        let tree = to_mdast(src, &parse_options_for(Path::new("page.mdx"))).expect("mdx parse");
        match tree {
            Node::Root(root) => {
                assert!(
                    root.children.iter().any(|node| matches!(node, Node::MdxjsEsm(_))),
                    "expected mdx esm node"
                );
                assert!(
                    root.children
                        .iter()
                        .any(|node| matches!(node, Node::MdxJsxFlowElement(_))),
                    "expected mdx jsx flow node"
                );
            }
            other => panic!("unexpected root node: {other:?}"),
        }
    }
}
