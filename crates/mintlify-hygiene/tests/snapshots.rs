use std::path::{Path, PathBuf};
use std::process::Command;

use insta::assert_snapshot;

fn bin() -> PathBuf {
    env!("CARGO_BIN_EXE_mintlify-hygiene").into()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run_check(root: &Path, extra_args: &[&str]) -> std::process::Output {
    let config = root.join("mintlify-hygiene.toml");
    let mut args = vec![
        "check",
        "--root",
        root.to_str().expect("utf8 root"),
        "--config",
        config.to_str().expect("utf8 config"),
    ];
    args.extend(extra_args);
    Command::new(bin())
        .args(args)
        .output()
        .expect("run mintlify-hygiene")
}

fn normalize_snapshot_text(root: &Path, text: &str) -> String {
    text.replace(root.to_string_lossy().as_ref(), "<ROOT>")
        .replace('\\', "/")
}

#[test]
fn snapshot_check_root_layout_mdx_only_report() {
    let root = fixture("snapshot-root-layout-site");
    let out = run_check(&root, &[]);
    assert!(!out.status.success(), "expected fixture to produce findings");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_snapshot!(
        "check_root_layout_mdx_only_report",
        normalize_snapshot_text(&root, &stderr)
    );
}

#[test]
fn snapshot_unescaped_lt_in_mdx_with_mintlify_components() {
    let root = fixture("snapshot-unescaped-lt-components");
    let out = run_check(&root, &[]);
    assert!(!out.status.success(), "expected fixture to produce findings");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_snapshot!(
        "unescaped_lt_in_mdx_with_mintlify_components",
        normalize_snapshot_text(&root, &stderr)
    );
}

#[test]
fn snapshot_prose_em_dash_in_lists_and_callouts() {
    let root = fixture("snapshot-prose-em-dash-callouts");
    let out = run_check(&root, &[]);
    assert!(!out.status.success(), "expected fixture to produce findings");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_snapshot!(
        "prose_em_dash_in_lists_and_callouts",
        normalize_snapshot_text(&root, &stderr)
    );
}

#[test]
fn snapshot_nav_registration_for_root_layout_repo() {
    let root = fixture("snapshot-nav-root-layout");
    let out = run_check(&root, &[]);
    assert!(!out.status.success(), "expected fixture to produce findings");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_snapshot!(
        "nav_registration_for_root_layout_repo",
        normalize_snapshot_text(&root, &stderr)
    );
}

#[test]
fn snapshot_json_output_for_representative_mintlify_fixture() {
    let root = fixture("snapshot-root-layout-site");
    let out = run_check(&root, &["--json"]);
    assert!(!out.status.success(), "expected fixture to produce findings");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_snapshot!(
        "json_output_for_representative_mintlify_fixture",
        normalize_snapshot_text(&root, &stdout)
    );
}
