# doob-surrealdb

`doob-surrealdb` is doob's default persistence adapter. It embeds SurrealDB 2 with the SurrealKV
engine and implements todo/note, HANDOFF item, and archive repository ports from `doob-core`.

## Workspace role

The default `doob` feature set selects this crate. It provides:

- `TodoRepositoryImpl` for todos, notes, search, stats, dependencies, and raw queries.
- `HandoffRepositoryImpl` for HANDOFF item synchronization and updates.
- `ArchiveRepositoryImpl` for moving old completed/cancelled todos into an archive table.
- `create_connection` and the `DbConnection` alias.

It does not implement `HandoffSessionRepository`; that implementation currently exists only in
`doob-sqlite`.

## Connection and storage

`create_connection(None)` opens or creates:

```text
$HOME/.ctx/doob/db/
```

The path is a SurrealKV directory, not a database file. A caller may override it with
`create_connection(Some(path))` or the main CLI's global `--db PATH` option.

Every connection selects namespace `doob` and database `doob`, then runs idempotent schema
initialization.

```rust
use doob_core::ports::TodoRepository;
use doob_surrealdb::{create_connection, TodoRepositoryImpl};

# async fn example() -> anyhow::Result<()> {
let db = create_connection(Some("/tmp/doob-surreal-example")).await?;
let repo = TodoRepositoryImpl::new(db);
let active = repo.list_active_todos().await?;
println!("{} active todos", active.len());
# Ok(())
# }
```

Use a unique temporary directory for tests and disposable examples.

## Schema

| Table | Mode | Stored data |
| --- | --- | --- |
| `todo` | `SCHEMAFULL` | Todo fields, optional context/metadata, tags, and dependency UUIDs |
| `note` | `SCHEMAFULL` | Note content, timestamps, optional context/metadata, and tags |
| `archive` | `SCHEMAFULL` | Preserved todo fields plus `archived_at` |
| `handoff_item` | `SCHEMALESS` | HANDOFF payloads, extras, and nullable timestamps |

Todo status and project, note project, archive project, and handoff project are indexed.
`handoff_id` has a unique index.

`handoff_item` is intentionally schemaless. HANDOFF synchronization needs nullable datetime values
and injects SurrealDB datetime literals directly rather than serializing ISO strings through a
`CONTENT` payload.

## Repository behavior

### Todos and notes

The adapter supports creation, lookup by record ID or UUID, list filters, update, complete, undo,
delete, content search, due dates, dependencies, notes, statistics, and raw SurrealQL.

Record IDs with hyphenated identifiers are backtick-quoted before raw query use. Due dates accept
`YYYY-MM-DD` and are stored at midnight UTC. Undo accepts completed or cancelled todos and returns
them to pending.

### HANDOFF items

Raw create/update methods execute SQL produced by the main crate's HANDOFF command. Structured
status changes validate `open`, `done`, `parked`, and `blocked`; moving to `done` sets a completion
timestamp. Extra entries are read, appended, serialized, and merged back into the record.

### Archive

Archive candidates are completed or cancelled todos whose `updated_at` is older than a cutoff.
Archiving copies supported fields into `archive`, then deletes the source record when it has a
record ID. Pending and in-progress todos are ignored.

## SurrealDB constraints

SurrealDB 2 behavior makes several implementation details load-bearing:

- Some raw values must be validated before string interpolation; `doob-core::query_guard` provides
  status and project allowlists.
- Datetimes in HANDOFF raw payloads use `d"..."` literals.
- HANDOFF status updates check existence before `UPDATE ... MERGE ... WHERE`, whose returned rows
  are not a reliable success signal.
- Query results containing record `Thing` values should deserialize through domain models or
  `serde_json::Value`, not a generic SurrealDB `Value` vector.

The repository operating guide also records a SurrealDB 2 issue where parameterized queries may
silently no-op. Current todo and archive code still uses `.bind(...)` in multiple paths, so changes
to those paths need real SurrealKV verification rather than compile-only confidence.

## Development and testing

```bash
cargo test -p doob-surrealdb
cargo clippy -p doob-surrealdb --all-targets -- -D warnings
cargo test -p doob --no-default-features --features surrealdb
```

This crate currently has no direct unit or integration test files. Its behavior is exercised by
the `doob` integration suite, which creates isolated SurrealKV directories and tests commands and
repository behavior through the main crate.
