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
