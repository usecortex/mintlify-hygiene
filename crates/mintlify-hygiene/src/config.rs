use crate::finding::Severity;
use anyhow::Context;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::path::{Path, PathBuf};

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub project: ProjectSection,
    #[serde(default)]
    pub rules: RulesSection,
}

#[derive(Debug, Default, Deserialize)]
pub struct ProjectSection {
    /// Directory containing markdown pages, relative to project root.
    #[serde(default = "default_docs_dir")]
    pub docs_dir: String,
    /// Navigation file (for example docs.json), relative to project root.
    #[serde(default = "default_nav_file")]
    pub nav_file: String,
    /// Glob patterns (relative to project root) excluded from all checks.
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_docs_dir() -> String {
    "docs".to_owned()
}

fn default_nav_file() -> String {
    "docs/docs.json".to_owned()
}

#[derive(Debug, Default, Deserialize)]
pub struct RulesSection {
    #[serde(default)]
    pub unescaped_lt: Option<RuleEntry>,
    #[serde(default)]
    pub frontmatter_yaml: Option<RuleEntry>,
    #[serde(default)]
    pub filename_chars: Option<RuleEntry>,
    #[serde(default)]
    pub nav_registration: Option<RuleEntry>,
    #[serde(default)]
    pub prose_em_dash: Option<RuleEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RuleEntry {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub level: Severity,
}

#[derive(Debug)]
pub struct ResolvedConfig {
    pub root: PathBuf,
    pub docs_dir: PathBuf,
    pub nav_file: PathBuf,
    pub exclude: GlobSet,
    pub unescaped_lt: ActiveRule,
    pub frontmatter_yaml: ActiveRule,
    pub filename_chars: ActiveRule,
    pub nav_registration: ActiveRule,
    pub prose_em_dash: ActiveRule,
}

#[derive(Clone, Copy, Debug)]
pub struct ActiveRule {
    pub enabled: bool,
    pub level: Severity,
}

impl ActiveRule {
    fn merge(entry: Option<&RuleEntry>) -> Self {
        match entry {
            Some(e) => ActiveRule {
                enabled: e.enabled,
                level: e.level,
            },
            None => ActiveRule {
                enabled: true,
                level: Severity::Error,
            },
        }
    }
}

pub fn load_config_file(path: &Path) -> anyhow::Result<ConfigFile> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read config {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse TOML config {}", path.display()))
}

pub fn resolve(
    root: PathBuf,
    cfg: ConfigFile,
    config_path: &Path,
) -> anyhow::Result<ResolvedConfig> {
    let docs_dir = normalize_join(&root, &cfg.project.docs_dir).with_context(|| {
        format!(
            "resolve docs_dir {:?} from {}",
            cfg.project.docs_dir,
            config_path.display()
        )
    })?;
    let nav_file = normalize_join(&root, &cfg.project.nav_file).with_context(|| {
        format!(
            "resolve nav_file {:?} from {}",
            cfg.project.nav_file,
            config_path.display()
        )
    })?;

    let mut builder = GlobSetBuilder::new();
    for g in &cfg.project.exclude {
        let pat = if Path::new(g).is_absolute() {
            anyhow::bail!("exclude glob must be relative to project root: {g}");
        } else {
            g.as_str()
        };
        builder.add(
            Glob::new(pat).with_context(|| format!("invalid exclude glob: {pat}"))?,
        );
    }
    let exclude = builder.build()?;

    Ok(ResolvedConfig {
        root,
        docs_dir,
        nav_file,
        exclude,
        unescaped_lt: ActiveRule::merge(cfg.rules.unescaped_lt.as_ref()),
        frontmatter_yaml: ActiveRule::merge(cfg.rules.frontmatter_yaml.as_ref()),
        filename_chars: ActiveRule::merge(cfg.rules.filename_chars.as_ref()),
        nav_registration: ActiveRule::merge(cfg.rules.nav_registration.as_ref()),
        prose_em_dash: ActiveRule::merge(cfg.rules.prose_em_dash.as_ref()),
    })
}

fn normalize_join(root: &Path, rel: &str) -> anyhow::Result<PathBuf> {
    let p = Path::new(rel);
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        Ok(root.join(p))
    }
}
