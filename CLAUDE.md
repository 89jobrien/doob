# doob

## Architecture
- Binary: `doob` / Library: `doob` (lib+bin structure, `src/main.rs` + `src/lib.rs`)
- DB: SurrealKV at `~/.ctx/doob/db/` (directory, not a file)
- `handoff_item` table is SCHEMALESS — typed tables rejected nullable datetime fields via JSON

## SurrealDB 2.x Gotchas
- **Parameterized queries silently no-op** (issue #6271) — use raw interpolated SQL strings
- **Datetime fields**: ISO strings rejected in CONTENT; inject as `d"2026-01-01T00:00:00Z"` literals
- **`UPDATE ... MERGE ... WHERE`** returns empty rows even on success — SELECT first to check existence
- **`.take::<Vec<surrealdb::Value>>(0)`** fails on rows containing `Thing` — use `serde_json::Value` or ignore

## Development
- `cargo install --path .` — reinstall binary (release by default; `--release` flag is invalid)
- `cargo test --all-features` — run tests
- `./ci.sh` — full local CI gate (fmt + clippy + test + audit)
- `doob handoff sync --file HANDOFF.doob.workspace.yaml` — sync after every HANDOFF edit
- `doob handoff update-status <id> <status>` — set status in doob (doob wins on sync conflict)
- **Sync conflict order**: call `doob handoff update-status` BEFORE editing YAML — doob DB status
  overwrites YAML on pull, reverting manual edits
- Wipe DB: `rm -rf ~/.ctx/doob/db` — safe; re-sync from HANDOFF.yaml to repopulate

## doobdash (TUI)
- Lives in `crates/doobdash/` — workspace member, separate binary
- `cargo install --path crates/doobdash` — install `doobdash` binary
- Launch: `doobdash [path/to/HANDOFF.yaml]` — auto-discovers `HANDOFF.*.yaml` walking up from CWD
- Keybindings: `j/k` nav col · `h/l` switch col · `Enter` overlay · `Space` leader (actions/tabs)
  Space leader: `s`=status · `n`=note · `w`=save · `/`=search · `1-5`=tabs · `?`=help · `Esc`=cancel
  `z` toggle strip · `z+j/k` resize strip · `q` quit · `5: DB` tab browses SurrealKV todos
- Status values: `open` · `done` · `parked` · `blocked`

## Hooks
- `HANDOFF*.yaml` is excluded from obfsck pre-commit scan — `doob_uuid` fields match container ID pattern
