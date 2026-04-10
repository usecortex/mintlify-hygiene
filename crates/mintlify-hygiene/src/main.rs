use anyhow::Context;
use clap::{Parser, Subcommand};
use mintlify_hygiene::{
    print_findings_human, print_findings_json, run_lint, MdxParseMode, PathFilterOverrides,
    Severity,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mintlify-hygiene", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, clap::ValueEnum)]
enum MdxParseModeArg {
    Loose,
    Strict,
}

impl From<MdxParseModeArg> for MdxParseMode {
    fn from(value: MdxParseModeArg) -> Self {
        match value {
            MdxParseModeArg::Loose => MdxParseMode::Loose,
            MdxParseModeArg::Strict => MdxParseMode::Strict,
        }
    }
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
        /// Apply auto-fixes for supported rules (e.g. `prose_em_dash`), then run checks.
        #[arg(long)]
        auto_fix: bool,
        /// Glob pattern(s) to include, relative to project root. Replaces config `include`.
        #[arg(long)]
        include: Vec<String>,
        /// Glob pattern(s) to exclude, relative to project root. Appends to config `exclude`.
        #[arg(long)]
        exclude: Vec<String>,
        /// How to treat MDX parse failures: `loose` warns, `strict` errors.
        #[arg(long, value_enum)]
        mdx_parse_mode: Option<MdxParseModeArg>,
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
            auto_fix,
            include,
            exclude,
            mdx_parse_mode,
        } => {
            let root = root
                .canonicalize()
                .with_context(|| format!("resolve root {}", root.display()))?;
            let config_path = if config.is_absolute() {
                config
            } else {
                root.join(&config)
            };
            let findings = run_lint(
                &root,
                &config_path,
                auto_fix,
                PathFilterOverrides {
                    include,
                    exclude,
                    mdx_parse_mode: mdx_parse_mode.map(Into::into),
                },
            )?;

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
