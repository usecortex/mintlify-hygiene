## [0.2.0] - 2026-04-10

### 🚀 Features

- Add platform-aware install script for downloading release binaries
- Add version bump workflow and local script

### 🐛 Bug Fixes

- Harden install.sh — executable bit, grep -F, precise chmod, jq fallback, find grouping
- Handle null tag_name in jq fallback for version resolution
## [0.1.0] - 2026-04-10

### 🚀 Features

- Scaffold workspace crate structure with cliff.toml
- Lite the fire
- *(cli)* Add --auto-fix for prose_em_dash
- *(cli)* Add Mintlify include filters and snapshots
- *(parser)* Add strict MDX parse mode

### 🐛 Bug Fixes

- Snapshot testing
- *(parser)* Keep fallback findings
- Add MIT LICENSE file and pin all action SHAs in release workflow

### 📚 Documentation

- README

### 🧪 Testing

- *(mintlify-hygiene)* Add MDX ESM skip and parse-fallback snapshots

### ⚙️ Miscellaneous Tasks

- Keep clean
- Add release workflow for cross-platform builds
