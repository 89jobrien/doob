# doobdash

`doobdash` is a standalone Ratatui terminal dashboard for browsing and updating a HANDOFF YAML
file. It provides kanban, log, statistics, help, detail, and experimental database views.

## Workspace role

This crate builds only the `doobdash` binary; it does not expose a library target. The `doob` CLI
launches it through `doob tui`, but it remains a separate process and package.

The dashboard reads YAML directly, shells out to an installed `doob` executable for mutations,
and optionally opens SurrealKV for its DB tab.

## Status

- **Active:** HANDOFF item, log, statistics, help, search, detail, and navigation views.
- **Partial:** status and note actions shell out to `doob`; failures are currently discarded by the
  event handlers rather than shown in the TUI.
- **Experimental/inactive:** the DB tab is implemented but its current namespace and field mapping
  do not match the primary SurrealDB adapter, so it may show no rows or skip valid todos.

## Installation and startup

```bash
cargo install --path crates/doobdash
doobdash .ctx/HANDOFF.doob.doob.yaml
```

If no path is supplied, `doobdash` walks upward from the current directory and uses the first file
whose name starts with `HANDOFF.` and ends with `.yaml`, excluding `HANDOFF.state.yaml`.

The input may be a bare list or a mapping containing `items`. Mapped documents may also contain a
`log` list. Each displayed item needs `id` and `title`; priority, status, description, and extras
default when omitted.

## Views

| Tab | Contents |
| --- | --- |
| Items | Three-column Active, Waiting, and Done kanban with search and details |
| Log | Date and summary entries from the YAML `log` key |
| Stats | Status and priority bar charts computed from YAML items |
| Help | In-application key reference |
| DB | Experimental direct SurrealKV todo listing |

Status-to-column mapping is source-defined:

- Active: `open`, `blocked`, and `in-progress`.
- Waiting: `parked` and `waiting`.
- Done: `done`.

These are HANDOFF statuses, not the `pending`/`in_progress` values used by the todo database.

## Keybindings

| Keys | Action |
| --- | --- |
| `j` / `k`, arrows | Move within the active column or DB list |
| `h` / `l`, arrows | Move between kanban columns |
| `gg` / `G` | Jump to top / bottom |
| `Enter` | Open the selected item's detail overlay |
| `z` | Toggle the description strip |
| `z` then `j` / `k` | Shrink / expand the description strip |
| `Space` then `1`-`5` | Switch tabs |
| `Space s` | Pick `open`, `done`, `parked`, or `blocked` |
| `Space n` | Add a note extra to the selected item |
| `Space /` | Filter items by ID or title |
| `Space w` | Exit and run HANDOFF synchronization |
| `q` / `Esc` | Quit or leave the current mode |

Inside the detail overlay, `j`/`k` scroll, `s` changes status, `n` adds a note, and `Esc` returns.

## Mutation and save flow

Status changes execute:

```text
doob handoff update-status ID STATUS
```

Notes execute:

```text
doob handoff add-extra ID --type note --note TEXT
```

These commands use the installed `doob` binary's default database because the dashboard does not
pass the selected HANDOFF path or a database override. The UI updates its in-memory status after
the subprocess call even if that call failed.

`Space w` exits the TUI and then executes:

```text
doob handoff sync --file SELECTED_PATH
```

That sync writes database-owned statuses and extras back into the YAML file. Quitting with `q`
does not perform the final sync.

## State and DB paths

Header state is loaded from `.ctx/HANDOFF.state.yaml` beneath the HANDOFF file's parent directory.
For a HANDOFF file at the repository root this resolves as expected. If the HANDOFF file itself is
inside `.ctx`, the current path rule resolves to `.ctx/.ctx/HANDOFF.state.yaml`.

The experimental DB tab opens `$HOME/.ctx/doob/db`, selects namespace `doob` and database `main`,
then expects `title`, string `priority`, and HANDOFF-like statuses. The active SurrealDB adapter
uses database `doob` and stores todo text in `content` with integer priority. Treat this tab as a
prototype until those contracts are aligned.

## Development and testing

```bash
cargo test -p doobdash
cargo clippy -p doobdash --all-targets -- -D warnings
cargo run -p doobdash -- path/to/HANDOFF.project.project.yaml
```

Tests cover YAML loading, missing state defaults, extras, column navigation and filtering, counts,
description-strip behavior, DB filtering, and the in-memory `TodoStore`. Terminal event handling,
subprocess mutations, rendering snapshots, and real SurrealKV access are not integration-tested.
