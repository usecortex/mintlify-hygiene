use serde::Serialize;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warn,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Error
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Serialize)]
struct FindingJson<'a> {
    rule_id: &'a str,
    severity: Severity,
    path: &'a str,
    line: usize,
    column: usize,
    message: &'a str,
}

pub fn print_findings_human(findings: &[Finding]) {
    for f in findings {
        let sev = match f.severity {
            Severity::Error => "error",
            Severity::Warn => "warning",
        };
        eprintln!(
            "{}: {} [{}] {}:{}:{}",
            sev, f.rule_id, f.message, f.path, f.line, f.column
        );
    }
}

pub fn print_findings_json(findings: &[Finding]) -> anyhow::Result<()> {
    let v: Vec<FindingJson> = findings
        .iter()
        .map(|f| FindingJson {
            rule_id: f.rule_id,
            severity: f.severity,
            path: &f.path,
            line: f.line,
            column: f.column,
            message: &f.message,
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

pub fn normalize_repo_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}
