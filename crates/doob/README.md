# doob

`doob` is the workspace's primary library and command-line application. It combines the domain
ports from `doob-core`, a selected storage adapter, HANDOFF YAML synchronization, GitHub issue
synchronization, rendering, and command dispatch.

## Contents

- [Workspace role](#workspace-role)
- [Status](#status)
- [Installation](#installation)
- [Storage and context](#storage-and-context)
- [CLI usage](#cli-usage)
- [Library API](#library-api)
- [Backend constraints](#backend-constraints)
- [Development and testing](#development-and-testing)

## Workspace role

The crate is the composition root for the other workspace crates:

| Dependency | Role in `doob` |
| --- | --- |
| `doob-core` | Domain models, repository ports, context detection, cache, and validation |
| `doob-surrealdb` | Default todo, handoff, and archive persistence adapter |
| `doob-sqlite` | Optional, partial SQLite persistence adapter |
| `doob-sync` | Provider-neutral issue-tracker synchronization domain |
| `doob-beads` | Experimental `bd` CLI adapter, re-exported but not called by the CLI |
| `doob-gh` | GitHub issue synchronization used by `todo gh-sync` |
| `doobdash` | Separate TUI binary launched by `doob tui` |

The library re-exports the domain modules under `doob::{models, ports, context, cache}` and the
default SurrealDB adapter under `doob::db` and `doob::adapters`.

## Status

- **Active:** the SurrealDB-backed CLI, todo and note commands, kanban/search/stats output,
  HANDOFF synchronization, GitHub synchronization, and archive commands.
- **Partial:** SQLite supports todo, note, handoff status, and handoff session persistence, but
  several CLI paths are not compatible with it yet.
- **Experimental/inactive:** the Beads adapter is exposed as a library API but has no CLI command.
  The machine-readable `schema` output is a partial manifest rather than a complete mirror of the
  Clap command tree.

## Installation

Install the default SurrealDB build from the workspace root:

```bash
cargo install --path crates/doob
```

Build a SQLite-only binary explicitly:

```bash
cargo install --path crates/doob --no-default-features --features sqlite
```

The crate has two backend features:

| Feature | Default | Effect |
| --- | --- | --- |
| `surrealdb` | Yes | Enables the SurrealKV adapter and archive support |
| `sqlite` | No | Enables and selects the SQLite adapter in the binary |

If both features are enabled, the binary selects SQLite. Therefore `--all-features` is useful for
compilation and tests, but it does not produce a SurrealDB-backed CLI at runtime.

## Storage and context

The global `--db PATH` option overrides the selected backend's default path.

| Backend | Default path | Shape |
| --- | --- | --- |
| SurrealDB | `$HOME/.ctx/doob/db/` | SurrealKV directory, namespace/database `doob/doob` |
| SQLite | `$HOME/.ctx/doob/doob.db` | SQLite file using WAL mode |

Todo mutations refresh `$HOME/.cache/doob/status.json` on a best-effort basis. The cache contains
pending and overdue totals for shell integrations.

When `--project` or `--file` is omitted from an add command, the command attempts to discover the
current Git repository. Project detection requires an `origin` remote; file context is the current
directory relative to the repository worktree.

## CLI usage

Global options may appear with any command:

```text
--json       Request JSON output where the dispatch path implements it
--db PATH    Override the backend's database path
```

### Todos

```bash
doob todo add "Write crate documentation" --priority 3 --project doob --tags docs,rust
doob todo add "Unblock release" --blocks UUID_A,UUID_B
doob todo list --status pending --project doob --limit 20
doob todo update todo:42 --status in_progress --priority 4
doob todo due todo:42 2026-10-01
doob todo due todo:42 clear
doob todo deps UUID
doob todo complete todo:42 todo:43
doob todo undo todo:42
doob todo remove todo:43
```

Statuses stored for todos are `pending`, `in_progress`, `completed`, and `cancelled`. The update
command currently accepts only the first three. Priorities default to `0` on add; update validates
explicit priorities in the range `1..=5`.

When a batch add includes `--blocks` or `--blocked-by`, dependency links are attached only to the
first newly created todo.

### Notes and views

```bash
doob note add "Investigate SurrealDB issue 6271" --project doob --tags database
doob note list --project doob --limit 10
doob note remove note:7

doob kan --project doob --status pending,in_progress
doob watch --project doob --interval 5
doob search "SurrealDB" --type all --project doob
doob stats --project doob --window 14
```

`search --type` recognizes `todo` and `note`; every other value behaves like `all`. `watch` clears
and redraws the kanban view until Ctrl-C.

### HANDOFF synchronization

```bash
doob handoff sync --file .ctx/HANDOFF.doob.doob.yaml
doob handoff list --project doob --status open
doob handoff update-status doob-12 blocked
doob handoff add-extra doob-12 --type blocker --note "Waiting on upstream release"
```

`handoff sync` accepts either a top-level YAML list or a mapping with an `items` key. New items are
inserted in the database and receive a `doob_uuid` in the YAML file. For existing items, the
database status wins and is written back to YAML. Extra entries merge in both directions and are
deduplicated by type, date, and note.

The sync command rewrites the YAML document's `items` value. It derives the project name from the
input file's immediate parent directory, not from the YAML `project` field.

Valid handoff statuses are `open`, `done`, `parked`, and `blocked`. Valid extra types are `note`,
`blocker`, `decision`, `discovery`, and `escalation`.

### GitHub issues

```bash
doob todo gh-sync
doob todo gh-sync --action complete
doob todo gh-sync --uuid 4fc2d014-aaaa-bbbb-cccc-7e6f6c332f88
doob todo gh-sync --execute
```

GitHub synchronization is a dry run unless `--execute` is supplied. It requires the `gh` CLI and
`$HOME/.config/doob/gh-sync.toml`; see the `doob-gh` crate README for configuration and state
semantics.

### Archive, TUI, and command schema

```bash
doob archive run --older-than 30
doob archive run --older-than 30 --apply --project doob
doob archive list --project doob --limit 50
doob tui --file .ctx/HANDOFF.doob.doob.yaml
doob schema
```

Archive execution is a dry run unless `--apply` is present. Archive operations are available only
with the SurrealDB backend. `doob tui` shells out to an installed `doobdash` executable.

`doob schema` emits JSON for agent discovery. Its current manifest omits some live commands and
describes several positional Clap arguments as flags, so consumers should treat it as provisional.

## Library API

Application code can call command handlers against any implementation of the core ports:

```rust
use doob::commands;
use doob::ports::TodoRepository;

async fn pending(repo: &dyn TodoRepository) -> anyhow::Result<()> {
    let todos = commands::list::execute(repo, Some("pending".into()), None, Some(20)).await?;
    for todo in todos {
        println!("{}: {}", todo.uuid, todo.content);
    }
    Ok(())
}
```

Important public surfaces are:

- `doob::commands`: command-level validation and orchestration.
- `doob::output`: human, JSON, kanban, statistics, search, dependency, and archive renderers.
- `doob::cli`: Clap parser types (`Cli`, `Commands`, and action enums).
- `doob::adapters`: SurrealDB repository implementations when `surrealdb` is enabled.
- `doob::adapters_sqlite`: SQLite repository implementations when `sqlite` is enabled.
- `doob::sync::domain`: re-export of `doob-sync`.
- `doob::sync::adapters::BeadsAdapter`: experimental Beads provider.

## Backend constraints

- SQLite does not implement `ArchiveRepository`; archive commands return an unsupported error.
- HANDOFF file sync uses the raw-query methods of `HandoffRepository`. Those methods intentionally
  return unsupported errors in the SQLite adapter.
- The current SQLite `create_todos` implementation interprets the first tuple field as content,
  while the command layer and SurrealDB adapter pass UUID first. Treat SQLite CLI creation as
  experimental until that contract is reconciled.
- JSON output is implemented for list/reporting paths. Mutation commands still print human status
  lines even when the global `--json` option is accepted.

## Development and testing

From the workspace root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
./ci.sh
```

Run only this package's tests with:

```bash
cargo test -p doob --all-features
```

The integration suite exercises command handlers against isolated SurrealKV databases and covers
todo/note CRUD, context detection, dependencies, due dates, search, stats, archive behavior,
rendering, HANDOFF sync, and the provider-neutral sync domain.
