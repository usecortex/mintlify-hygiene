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
    /// Glob patterns (relative to project root) explicitly included in checks.
    #[serde(default)]
    pub include: Vec<String>,
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
    pub walk_root: PathBuf,
    pub nav_file: PathBuf,
    pub include: Option<GlobSet>,
    pub exclude: GlobSet,
    pub unescaped_lt: ActiveRule,
    pub frontmatter_yaml: ActiveRule,
    pub filename_chars: ActiveRule,
    pub nav_registration: ActiveRule,
    pub prose_em_dash: ActiveRule,
}

#[derive(Debug, Default)]
pub struct PathFilterOverrides {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
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
    overrides: &PathFilterOverrides,
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

    let include_patterns = if overrides.include.is_empty() {
        cfg.project.include.clone()
    } else {
        overrides.include.clone()
    };
    let mut exclude_patterns = cfg.project.exclude.clone();
    exclude_patterns.extend(overrides.exclude.iter().cloned());

    let include = if include_patterns.is_empty() {
        None
    } else {
        Some(build_globset(&include_patterns, "include")?)
    };
    let exclude = build_globset(&exclude_patterns, "exclude")?;
    let walk_root = if include.is_some() {
        root.clone()
    } else {
        docs_dir.clone()
    };

    Ok(ResolvedConfig {
        root,
        docs_dir,
        walk_root,
        nav_file,
        include,
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

fn build_globset(patterns: &[String], kind: &str) -> anyhow::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for g in patterns {
        let pat = if Path::new(g).is_absolute() {
            anyhow::bail!("{kind} glob must be relative to project root: {g}");
        } else {
            g.as_str()
        };
        builder.add(Glob::new(pat).with_context(|| format!("invalid {kind} glob: {pat}"))?);
    }
    builder.build().with_context(|| format!("build {kind} globset"))
}

impl ResolvedConfig {
    pub fn matches_project_path(&self, key: &str) -> bool {
        if let Some(include) = &self.include {
            if !include.is_match(key) {
                return false;
            }
        }
        !self.exclude.is_match(key)
    }
}
