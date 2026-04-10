use anyhow::Context;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

pub fn nav_slugs_from_file(nav_path: &Path) -> anyhow::Result<HashSet<String>> {
    let raw = std::fs::read_to_string(nav_path)
        .with_context(|| format!("read nav file {}", nav_path.display()))?;
    let v: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse JSON nav {}", nav_path.display()))?;
    let mut set = HashSet::new();
    collect_pages(&v, &mut set);
    Ok(set)
}

fn collect_pages(v: &Value, set: &mut HashSet<String>) {
    match v {
        Value::Object(map) => {
            if let Some(Value::Array(pages)) = map.get("pages") {
                for p in pages {
                    match p {
                        Value::String(s) => {
                            set.insert(normalize_slug(s));
                        }
                        Value::Object(_) => {
                            collect_pages(p, set);
                        }
                        _ => {}
                    }
                }
            }
            for val in map.values() {
                collect_pages(val, set);
            }
        }
        Value::Array(arr) => {
            for x in arr {
                collect_pages(x, set);
            }
        }
        _ => {}
    }
}

fn normalize_slug(s: &str) -> String {
    let t = s.trim().trim_start_matches("./");
    let without_ext = t
        .strip_suffix(".mdx")
        .or_else(|| t.strip_suffix(".md"))
        .unwrap_or(t);
    without_ext.replace('\\', "/")
}
