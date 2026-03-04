#!/usr/bin/env bash
# Local CI — mirrors .github/workflows/ci.yml
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BOLD='\033[1m'
RESET='\033[0m'

pass() { echo -e "${GREEN}✓${RESET} $1"; }
fail() { echo -e "${RED}✗${RESET} $1"; exit 1; }
step() { echo -e "\n${BOLD}» $1${RESET}"; }

step "Format"
cargo fmt -- --check && pass "fmt" || fail "fmt (run: cargo fmt)"

step "Clippy"
cargo clippy -- -D warnings && pass "clippy" || fail "clippy"

step "Test"
cargo test --all-features && pass "tests" || fail "tests"

step "Audit"
if ! command -v cargo-audit &>/dev/null; then
  echo "  installing cargo-audit..."
  cargo install cargo-audit --quiet
fi
cargo audit && pass "audit" || fail "audit"

echo -e "\n${GREEN}${BOLD}All checks passed.${RESET}"
