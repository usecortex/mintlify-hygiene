use crate::finding::{Finding, Severity};
use crate::finding::normalize_repo_path;
use regex::Regex;
use std::path::Path;

static FRONTMATTER_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?s)\A---\r?\n(.*?)\r?\n---\r?\n").expect("frontmatter regex")
});

pub fn split_frontmatter(src: &str) -> Option<(&str, &str)> {
    let m = FRONTMATTER_RE.captures(src)?;
    let yaml_blob = m.get(1)?.as_str();
    let full = m.get(0)?;
    let body = src.get(full.end()..)?;
    Some((yaml_blob, body))
}

pub fn check_frontmatter_yaml(
    root: &Path,
    file: &Path,
    full_source: &str,
    level: Severity,
) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some((yaml_blob, _body)) = split_frontmatter(full_source) else {
        return out;
    };
    if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(yaml_blob) {
        let path = normalize_repo_path(root, file);
        out.push(Finding {
            rule_id: "frontmatter_yaml",
            severity: level,
            path,
            line: 2,
            column: 1,
            message: format!(
                "frontmatter is not valid YAML (inner quotes in double-quoted scalars often cause this): {e}"
            ),
        });
    }
    out
}

/// Byte offset in `full_source` where the markdown body starts (after closing `---`).
pub fn body_start_byte(full_source: &str) -> usize {
    split_frontmatter(full_source)
        .and_then(|_| FRONTMATTER_RE.captures(full_source))
        .and_then(|m| m.get(0).map(|full| full.end()))
        .unwrap_or(0)
}

pub fn line_col_at_byte(s: &str, byte_off: usize) -> (usize, usize) {
    let end = byte_off.min(s.len());
    let head = &s[..end];
    let line = head.as_bytes().iter().filter(|&&b| b == b'\n').count() + 1;
    let col = head
        .rfind('\n')
        .map(|i| end - i)
        .unwrap_or(end + 1);
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_standard_frontmatter() {
        let s = "---\ntitle: x\n---\n\n# Hi\n";
        let (y, b) = split_frontmatter(s).unwrap();
        assert_eq!(y, "title: x");
        assert_eq!(b, "\n# Hi\n");
    }
}
