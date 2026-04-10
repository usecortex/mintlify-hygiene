//! Mintlify documentation hygiene checks (config-driven).

mod config;
mod finding;
mod frontmatter;
mod lint;
mod markdown;
mod nav;

pub use config::{load_config_file, resolve, ConfigFile, ResolvedConfig};
pub use finding::{print_findings_human, print_findings_json, Finding, Severity};
pub use lint::lint_project;

use std::path::Path;

/// Load config from `config_path`, resolve paths relative to `root`, run all enabled rules.
pub fn run_lint(root: &Path, config_path: &Path) -> anyhow::Result<Vec<Finding>> {
    let cfg_file = load_config_file(config_path)?;
    let resolved = resolve(root.to_path_buf(), cfg_file, config_path)?;
    lint_project(&resolved)
}
