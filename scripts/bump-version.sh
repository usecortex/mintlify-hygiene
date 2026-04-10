#!/usr/bin/env bash
#
# bump-version.sh — Bump version in Cargo.toml files, generate changelog, create git tag.
#
# Usage:
#   ./scripts/bump-version.sh <version>
#   ./scripts/bump-version.sh <version> --dry-run
#
# Examples:
#   ./scripts/bump-version.sh 0.2.0
#   ./scripts/bump-version.sh 1.0.0-rc.1 --dry-run
#
# What it does:
#   1. Validates the version string (semver, no leading "v")
#   2. Updates version in Cargo.toml files and regenerates Cargo.lock
#   3. Generates CHANGELOG.md using git-cliff with cliff.toml
#   4. Commits all changes as "chore(release): prepare for v<version>"
#   5. Creates git tag v<version> on the release commit
#
# After running locally, push with:
#   git push origin HEAD --follow-tags
#
# The pushed tag triggers the Release workflow automatically.

set -euo pipefail

# ── Helpers ──────────────────────────────────────────────────────────────────

die() { echo "error: $*" >&2; exit 1; }

# ── Args ─────────────────────────────────────────────────────────────────────

VERSION="${1:-}"
shift || true
DRY_RUN=false

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    --*)       die "unknown flag: $arg" ;;
    *)         die "unexpected argument: $arg" ;;
  esac
done

[ -z "$VERSION" ] && die "usage: $0 <version> [--dry-run]  (e.g. 0.2.0)"

# Strip leading "v" if someone passes "v0.2.0"
VERSION="${VERSION#v}"

# Validate semver (X.Y.Z with optional pre-release/build metadata)
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$'; then
  die "invalid semver: '$VERSION' — expected format like 0.2.0 or 1.0.0-rc.1"
fi

TAG="v${VERSION}"

# ── Ensure we're at repo root ────────────────────────────────────────────────

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not inside a git repository"
cd "$REPO_ROOT"

# ── Check for uncommitted changes ────────────────────────────────────────────

if ! git diff --quiet || ! git diff --cached --quiet; then
  die "working tree is dirty — commit or stash changes first"
fi

# ── Check tag doesn't already exist ──────────────────────────────────────────

if git rev-parse "$TAG" >/dev/null 2>&1; then
  die "tag '$TAG' already exists"
fi

# ── Check current version isn't the same ─────────────────────────────────────

CURRENT_VERSION=$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
if [ "$CURRENT_VERSION" = "$VERSION" ]; then
  die "version is already ${VERSION} — nothing to bump"
fi

# ── Check required tools ─────────────────────────────────────────────────────

command -v git-cliff >/dev/null 2>&1 || die "git-cliff is not installed — see https://git-cliff.org/docs/installation"
command -v cargo >/dev/null 2>&1     || die "cargo is not installed"
command -v sed >/dev/null 2>&1       || die "sed is not installed"

if $DRY_RUN; then
  echo "==> [DRY RUN] Would bump ${CURRENT_VERSION} -> ${VERSION} (tag: ${TAG})"
  echo "  -> Would update Cargo.toml (workspace)"
  echo "  -> Would update crates/mintlify-hygiene/Cargo.toml"
  echo "  -> Would update Cargo.lock"
  echo "  -> Would generate CHANGELOG.md"
  echo "  -> Would commit as: chore(release): prepare for ${TAG}"
  echo "  -> Would create tag ${TAG}"
  echo ""
  echo "Dry run complete — no changes made."
  exit 0
fi

echo "==> Bumping ${CURRENT_VERSION} -> ${VERSION} (tag: ${TAG})"

# ── 1. Update Cargo.toml version fields ──────────────────────────────────────

for toml in Cargo.toml crates/mintlify-hygiene/Cargo.toml; do
  echo "  -> Updating $toml"
  sed -i.bak -E 's/^(version *= *)"[^"]*"/\1"'"${VERSION}"'"/' "$toml"
  rm -f "${toml}.bak"
done

# ── 2. Update Cargo.lock ────────────────────────────────────────────────────

echo "  -> Updating Cargo.lock"
cargo generate-lockfile

# ── 3. Generate CHANGELOG.md ────────────────────────────────────────────────

echo "  -> Generating CHANGELOG.md"
git-cliff --config cliff.toml --tag "$TAG" --output CHANGELOG.md

# ── 4. Commit everything ────────────────────────────────────────────────────

echo "  -> Committing release"
git add Cargo.toml crates/mintlify-hygiene/Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): prepare for ${TAG}"

# ── 5. Create tag on the release commit ──────────────────────────────────────

echo "  -> Creating tag ${TAG}"
git tag "$TAG"

# ── Done ─────────────────────────────────────────────────────────────────────

echo ""
echo "Done! Version bumped: ${CURRENT_VERSION} -> ${VERSION}"
echo ""
echo "To publish the release, push the branch and tag:"
echo "  git push origin HEAD --follow-tags"
echo ""
echo "The pushed tag will trigger the Release workflow on GitHub Actions."
