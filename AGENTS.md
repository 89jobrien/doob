# doob — Agent Operating Guide

This file instructs AI coding agents (Codex, Claude, Copilot, or any shell-capable
LLM agent) how to work effectively in the doob codebase.

## Identity

You are working on **doob**, a Rust task/todo tracker with handoff workflows,
SurrealDB/SQLite backends, and GitHub sync. The codebase is a Rust workspace
(edition 2024, 8 member crates) with a companion TUI (`doobdash`).

## Primary Toolkit

All development workflows use `cargo`. doob does not have a `justfile` or `xtask`.

### Build & Quality

| Command                      | Purpose                            |
| ---------------------------- | ---------------------------------- |
| `cargo install --path <path>` | Install binary (release by default) |
| `cargo test --all-features`  | Run all unit + integration tests   |
| `cargo clippy`               | Lint checks (warnings must pass)   |
| `cargo fmt --check`          | Check code formatting              |
| `./ci.sh`                    | Full pre-push quality gate         |

### Testing

| Command                      | Purpose                            |
| ---------------------------- | ---------------------------------- |
| `cargo test --all-features`  | Full test suite with all features  |
| `cargo test --lib`           | Library tests only                 |
| `cargo test --test '*'`      | Integration tests only             |

## Workspace Layout

```
doob/
├── crates/
│   ├── doob/            # Main binary + lib (CLI entry, lib re-exports)
│   ├── doob-core/       # Core domain types (Task, Status, etc.)
│   ├── doob-sqlite/     # SQLite backend (feature-gated)
│   ├── doob-surrealdb/  # SurrealDB backend (feature-gated)
│   ├── doob-sync/       # HANDOFF.yaml sync and conflict resolution
│   ├── doob-beads/      # Experimental subsystem (not in active use)
│   ├── doob-gh/         # GitHub integration (issues, PRs, syncing)
│   └── doobdash/        # TUI dashboard (separate binary)
├── Cargo.toml           # Workspace config
└── ci.sh                # Pre-push quality gate script
```

## Code Conventions

### Rust

- **Toolchain**: Latest stable (check `rust-toolchain.toml` if present)
- **Line width**: 100 characters
- **Linting**: `cargo clippy -- -D warnings`
- **Error handling**: `anyhow::Result<T>`, propagate with `?`, no `unwrap()` in
  production code
- **Naming**: PascalCase structs/enums, snake_case functions/variables
- **Imports**: group by external crate, then std
- **Tests**: unit tests in `mod tests {}`, integration tests in `tests/`
- **Test isolation**: no `std::env::set_var` without restoration; avoid shared
  filesystem assumptions

### SurrealDB Gotchas (Critical)

When writing SurrealDB queries:

- **Parameterized queries are broken** (SurrealDB issue #6271) — use raw
  interpolated SQL strings instead of `surrealdb::sql::Param`
- **Datetime fields**: ISO strings are rejected in CONTENT; inject as
  `d"2026-01-01T00:00:00Z"` literals instead
- **UPDATE...MERGE...WHERE returns empty** even on success — SELECT first to
  check existence before updating
- **`.take::<Vec<Value>>(0)` fails on rows containing `Thing`** — use
  `serde_json::Value` instead or ignore the issue if `Thing` fields are present

### Database Selection

- **Default**: SQLite (in-memory for tests, disk at `~/.ctx/doob/db/` for local development)
- **Feature flag**: `--all-features` enables both backends; feature-gate backend-specific code with
  `#[cfg(feature = "surrealdb")]` / `#[cfg(feature = "sqlite")]`
- **Runtime selection**: Typically driven by build/deploy context; see
  `doob-core/src/backend/mod.rs` for the trait abstraction

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

- **Types**: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, `quality:`
- **Scope**: optional (crate name, subsystem, or domain)
- **Good**: `feat(sync): add HANDOFF.yaml conflict detection`
- **Good**: `fix(sqlite): handle nullable datetime fields`

## Key Development Workflows

### Pre-Push Quality Gate

Before pushing, run the full gate:

```bash
./ci.sh
```

This covers:

- `cargo fmt --all --check` — code formatting
- `cargo clippy` — linting with `-D warnings`
- `cargo test --all-features` — tests with all features
- `cargo deny check` — audit dependencies and licenses

If any check fails, fix it locally before pushing.

### Local Development Setup

1. **Install binaries** (for local use):

   ```bash
   cargo install --path crates/doob
   cargo install --path crates/doobdash
   ```

2. **Wipe and resync DB** (if state is corrupted):

   ```bash
   rm -rf ~/.ctx/doob/db
   doob handoff sync --file HANDOFF.doob.workspace.yaml
   ```

3. **Test with all features**:

   ```bash
   cargo test --all-features
   ```

### Handoff Sync Workflow

After editing a HANDOFF file:

```bash
doob handoff sync --file HANDOFF.doob.workspace.yaml
```

**Important**: doob DB status always wins on conflict. If you edit HANDOFF.yaml
manually after using doob, the DB value will overwrite your changes on the next
sync. Use `doob handoff update-status <id> <status>` to change status in doob
first, then sync.

### Using doobdash (TUI)

```bash
# Auto-discover HANDOFF.*.yaml walking up from CWD
doobdash

# Or point explicitly
doobdash /path/to/HANDOFF.doob.workspace.yaml
```

**Keybindings**:

- `j/k` — navigate rows
- `h/l` — switch columns
- `Enter` — open overlay
- `Space` — leader key (then `s`=status, `n`=note, `w`=save, `/`=search,
  `1-5`=tabs, `?`=help, `Esc`=cancel)
- `z` — toggle strip; `z+j/k` — resize
- `q` — quit

**Status values**: `open`, `done`, `parked`, `blocked`

## Environment & Storage

### Database Location

- **Default**: `~/.ctx/doob/db/` (SurrealKV directory, not a file)
- **Wipe**: `rm -rf ~/.ctx/doob/db` — safe; re-sync from HANDOFF.yaml to repopulate

### GitHub Token (for sync)

GitHub sync requires a personal access token. Store in 1Password or `.env`:

```bash
export GITHUB_TOKEN="ghp_..."
doob handoff sync --file HANDOFF.doob.workspace.yaml
```

## Git Hooks

- `HANDOFF*.yaml` files are excluded from `obfsck` pre-commit scans — `doob_uuid`
  fields match container ID patterns and trigger false positives. No manual
  exclusion needed — already configured.

## Key Dependencies

- **CLI**: `clap` (derive)
- **Serialization**: `serde`, `serde_json`
- **Database**: `surrealdb` (feature: `surrealdb`), `sqlx` (feature: `sqlite`)
- **Error handling**: `anyhow`
- **Time**: `chrono`
- **Testing**: `tempfile` for test directories
- **GitHub**: `octocrab`

## Testing Patterns

### Unit Tests

Place in `mod tests {}` block within the source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_some_behavior() -> Result<()> {
        // arrange
        // act
        // assert
        Ok(())
    }
}
```

### Integration Tests

Place in `tests/` directory at crate root. Use `tempfile::TempDir` for isolated
filesystem state — never use shared `/tmp` or hardcoded paths.

## Troubleshooting

| Issue | Cause | Fix |
|-------|-------|-----|
| `rm -rf ~/.ctx/doob/db` produces empty state | DB is stale | Resync: `doob handoff sync --file ...` |
| SurrealDB datetime errors | ISO strings in CONTENT | Use `d"..."` literal syntax |
| Tests fail with "db locked" | Parallel test contention | Use `tempfile::TempDir` per test, not shared dir |
| `cargo clippy` warnings | Not run locally | Run `cargo clippy` before pushing |
