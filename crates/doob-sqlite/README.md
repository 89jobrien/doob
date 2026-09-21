# doob-sqlite

`doob-sqlite` is an optional SQLite adapter for the repository ports in `doob-core`. It uses
`rusqlite` with a bundled SQLite library and stores structured collections as JSON text.

## Status

**Partial and experimental as a complete CLI backend.** Todo, note, handoff item, session log,
session state, and checkpoint operations are implemented. Archive persistence, raw query
execution, and the raw HANDOFF synchronization operations are not implemented.

The adapter has focused integration tests, but current backend contracts are not fully aligned:
`TodoRepositoryImpl::create_todos` interprets tuple fields as `(content, uuid, ...)`, while the
`doob` command layer and SurrealDB adapter pass `(uuid, content, ...)`. Until reconciled, direct
library callers should follow this adapter's current order and the SQLite-backed `doob todo add`
path should be considered unsafe for real data.

## Workspace role

The crate implements:

- `TodoRepository` through `TodoRepositoryImpl`.
- `HandoffRepository` through `HandoffRepositoryImpl`.
- `HandoffSessionRepository` through `HandoffSessionRepositoryImpl`.

It deliberately depends on `doob-core` with default features disabled so record IDs deserialize as
ordinary strings rather than SurrealDB `Thing` values.

## Connection API

`create_connection(Some(path))` opens the requested file and initializes the schema.
`create_connection(None)` uses:

```text
$HOME/.ctx/doob/doob.db
```

The connection enables WAL journal mode and SQLite foreign-key enforcement. `SqliteConnection` is
a cloneable `Arc<Mutex<rusqlite::Connection>>`; `with_conn` provides exclusive closure-based
access to the underlying connection.

```rust
use doob_core::ports::TodoRepository;
use doob_sqlite::{create_connection, TodoRepositoryImpl};

# async fn example() -> anyhow::Result<()> {
let db = create_connection(Some("/tmp/doob-example.db"))?;
let repo = TodoRepositoryImpl::new(db);
let todos = repo.list_todos(Some("pending"), None, Some(20)).await?;
println!("{} pending todos", todos.len());
# Ok(())
# }
```

Applications should use an isolated temporary directory in tests rather than the fixed example
path.

## Schema

Initialization creates these tables and indexes if absent:

| Table | Purpose | Important constraints |
| --- | --- | --- |
| `todo` | Todos and dependencies | Unique UUID; indexed status, project, and UUID |
| `note` | Free-form notes | Unique UUID; indexed project |
| `archive` | Archived todo rows | Unique UUID; indexed project |
| `handoff_item` | HANDOFF items | Unique UUID and handoff ID; indexed project |
| `handoff_log` | Project session log | Indexed project |
| `handoff_state` | Latest project state | Project is the primary key |
| `metadata` | Checkpoint and initialization values | String key/value store |

Timestamps are stored as text. Row decoding accepts RFC 3339 and SQLite's
`YYYY-MM-DD HH:MM:SS` format. Tags, metadata, dependency lists, handoff files/extras, commits, and
checkpoint payloads are encoded as JSON text.

Although the `archive` table exists, this crate does not export or implement an
`ArchiveRepositoryImpl`.

## Repository behavior

### Todos and notes

The adapter supports CRUD, status/project filtering, limits, content search with `LIKE`, due dates,
dependency lists, UUID lookup, stats, and active/all-todo listing. Numeric row IDs are exposed as
`todo:N` and `note:N`.

`execute_raw_query` always returns an unsupported error. Due-date values are written as supplied;
unlike the SurrealDB adapter, this layer does not validate `YYYY-MM-DD` itself.

### HANDOFF items

Listing, status changes, and extra-entry append are implemented. Status changes accept only
`open`, `done`, `parked`, and `blocked`; moving to `done` sets `completed_at`.

`create_handoff_raw` and `update_handoff_raw` return unsupported errors. The current `doob handoff
sync` command depends on those methods, so file synchronization does not work with SQLite even
though existing handoff rows can be listed and updated.

### Session data

Session logs are append-only and queried newest first. Project state uses an upsert and persists
branch, build, tests, notes, and update time. Fields such as touched files and last log are not
stored. Handup checkpoints are written as two `metadata` entries.

The main `doob` binary does not currently construct or expose the session repository.

## Building the SQLite CLI

```bash
cargo build -p doob --no-default-features --features sqlite
```

If both `sqlite` and `surrealdb` are enabled, the binary selects SQLite. Archive commands then
return an unsupported error.

## Development and testing

```bash
cargo test -p doob-sqlite
cargo clippy -p doob-sqlite --all-targets -- -D warnings
```

Integration tests use a unique temporary database and cover todo CRUD, search, handoff status and
filters, session log ordering, and session state upserts. There is no archive test because no
archive repository implementation exists.
