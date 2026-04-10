use anyhow::Context;
use clap::{Parser, Subcommand};
use mintlify_hygiene::{print_findings_human, print_findings_json, run_lint, Severity};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mintlify-hygiene", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run hygiene checks for a documentation tree.
    Check {
        /// Project root (directory that contains docs and config).
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Path to mintlify-hygiene.toml (relative to root unless absolute).
        #[arg(long, default_value = "mintlify-hygiene.toml")]
        config: PathBuf,
        /// Emit findings as JSON on stdout (human messages still go to stderr).
        #[arg(long)]
        json: bool,
        /// Treat warnings as errors for exit status.
        #[arg(long)]
        deny_warnings: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check {
            root,
            config,
            json,
            deny_warnings,
        } => {
            let root = root
                .canonicalize()
                .with_context(|| format!("resolve root {}", root.display()))?;
            let config_path = if config.is_absolute() {
                config
            } else {
                root.join(&config)
            };
            let findings = run_lint(&root, &config_path)?;

            if json {
                print_findings_json(&findings)?;
            }
            print_findings_human(&findings);

            let has_error = findings
                .iter()
                .any(|f| f.severity == Severity::Error);
            let has_warn = findings
                .iter()
                .any(|f| f.severity == Severity::Warn);
            if has_error || (deny_warnings && has_warn) {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
