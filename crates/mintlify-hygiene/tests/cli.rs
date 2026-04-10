use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    env!("CARGO_BIN_EXE_mintlify-hygiene").into()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn failing_site_exits_nonzero_and_reports_rule_ids() {
    let root = fixture("failing-site");
    let out = Command::new(bin())
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            root.join("mintlify-hygiene.toml").to_str().unwrap(),
        ])
        .output()
        .expect("run mintlify-hygiene");
    assert!(
        !out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for id in [
        "unescaped_lt",
        "frontmatter_yaml",
        "filename_chars",
        "nav_registration",
        "prose_em_dash",
    ] {
        assert!(
            stderr.contains(id),
            "expected {id} in stderr:\n{stderr}"
        );
    }
}

#[test]
fn good_site_passes() {
    let root = fixture("good-site");
    let out = Command::new(bin())
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            root.join("mintlify-hygiene.toml").to_str().unwrap(),
        ])
        .output()
        .expect("run mintlify-hygiene");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).trim().is_empty());
}

#[test]
fn deny_warnings_fails_when_only_warnings() {
    let root = fixture("warn-only-site");
    let out = Command::new(bin())
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            root.join("mintlify-hygiene.toml").to_str().unwrap(),
            "--deny-warnings",
        ])
        .output()
        .expect("run mintlify-hygiene");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("prose_em_dash"));
}

#[test]
fn deny_warnings_ok_on_clean_site() {
    let root = fixture("good-site");
    let out = Command::new(bin())
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            root.join("mintlify-hygiene.toml").to_str().unwrap(),
            "--deny-warnings",
        ])
        .output()
        .expect("run mintlify-hygiene");
    assert!(out.status.success());
}

#[test]
fn auto_fix_replaces_prose_em_dash_then_passes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let fixture = fixture("warn-only-site");
    for rel in ["mintlify-hygiene.toml", "docs/page.md", "docs/docs.json"] {
        let from = fixture.join(rel);
        let to = root.join(rel);
        std::fs::create_dir_all(to.parent().expect("parent")).expect("mkdir");
        std::fs::copy(&from, &to).expect("copy fixture file");
    }

    let out = Command::new(bin())
        .args([
            "check",
            "--root",
            root.to_str().expect("utf8 root"),
            "--config",
            root.join("mintlify-hygiene.toml").to_str().expect("utf8 config"),
            "--auto-fix",
            "--deny-warnings",
        ])
        .output()
        .expect("run mintlify-hygiene");

    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("auto-fix updated 1 file"),
        "expected autofix notice in stderr:\n{stderr}"
    );

    let page = std::fs::read_to_string(root.join("docs/page.md")).expect("read page");
    assert!(
        page.contains("One em dash - not two hyphens."),
        "expected spaced hyphen replacement, got:\n{page}"
    );
    assert!(
        !page.contains('\u{2014}'),
        "em dash should be removed from prose"
    );
}

#[test]
fn root_layout_site_passes_with_repo_local_include_and_exclude() {
    let root = fixture("root-layout-site");
    let out = Command::new(bin())
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            root.join("mintlify-hygiene.toml").to_str().unwrap(),
        ])
        .output()
        .expect("run mintlify-hygiene");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_include_overrides_config_include_set() {
    let root = fixture("root-layout-site-overrides");
    let out = Command::new(bin())
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            root.join("mintlify-hygiene.toml").to_str().unwrap(),
            "--include",
            "published/index.mdx",
        ])
        .output()
        .expect("run mintlify-hygiene");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_exclude_skips_matching_file_and_folder() {
    let root = fixture("root-layout-site-overrides");
    let out = Command::new(bin())
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            root.join("mintlify-hygiene.toml").to_str().unwrap(),
            "--exclude",
            "published/guide.mdx",
            "--exclude",
            "archive/**",
        ])
        .output()
        .expect("run mintlify-hygiene");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}
