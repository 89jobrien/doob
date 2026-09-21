# doob-core

`doob-core` owns doob's backend-independent models, repository ports, context helpers, query
validation, record-ID helpers, exit-code classification, and shell status cache.

## Workspace role

This is the domain and inbound-port crate in the workspace's hexagonal layout:

```text
doob commands -> doob-core ports <- doob-surrealdb / doob-sqlite
```

It does not choose a backend or provide a CLI. Storage crates implement its async repository
traits, and the main `doob` crate re-exports its modules.

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `surrealdb-backend` | Yes | Deserializes database IDs from SurrealDB `Thing` values |

Disable default features for adapters whose serialized IDs are ordinary optional strings:

```toml
doob-core = { path = "../doob-core", default-features = false }
```

`doob-sqlite` uses this configuration. With the default feature, `Todo`, `Note`, `ArchivedTodo`,
and `HandoffItem` expect SurrealDB record IDs during deserialization.

## Models

### Todos, notes, and archives

`Todo` contains a backend record ID, stable UUID, content, status, priority, timestamps, project
and file context, tags, metadata, due date, and dependency UUID lists. `TodoStatus` has
`Pending`, `InProgress`, `Completed`, and `Cancelled` variants serialized as snake case.

`Note` stores content, UUID, timestamps, optional project/file context, tags, and metadata.
`ArchivedTodo` preserves todo fields and adds `archived_at` while storing status as a string.

```rust
use doob_core::models::TodoStatus;

assert_eq!(TodoStatus::InProgress.as_str(), "in_progress");
```

### HANDOFF models

Two model families are public:

- `models::handoff_item`: persisted `HandoffItem`, `ExtraEntry`, and `ExtraType` values.
- `models::handoff`: YAML/session models, reconciliation plans, handup reports, validation warnings,
  and title/priority helpers.

`CommitRef` accepts either a bare YAML scalar SHA or an object containing `sha` and optional
`branch`. `Handoff::active_items` returns only items whose status is `open` or `blocked`.

Reconciliation compares title variants between active HANDOFF items and todo snapshots. In sync
mode it creates missing todos; in audit mode it reports them without producing creates.

## Repository ports

| Port | Responsibility | Current implementation |
| --- | --- | --- |
| `TodoRepository` | Todo, note, dependency, search, stats, raw queries | Both backends |
| `HandoffRepository` | Persisted HANDOFF items, status, and extras | Both backends |
| `HandoffSessionRepository` | Session log, state, and handup checkpoints | SQLite only |
| `ArchiveRepository` | Candidate selection, move, and archive listing | SurrealDB only |

All ports are async and require `Send + Sync`. `TodoRepository::execute_raw_query` and the raw
HANDOFF create/update methods exist for SurrealDB-specific workflows; a backend may return an
unsupported error.

The current tuple contract used by the command layer for `create_todos` is:

```text
(uuid, content, priority, project, file_path, tags)
```

The tuple is not named in the trait signature. The SQLite adapter currently interprets its first
two fields in the opposite order, so consumers should avoid assuming backend parity until that
contract is made explicit.

## Context and cache

`context::detect_project` discovers the current Git repository, reads its `origin` remote, and
uses the last URL component as the project name. `context::detect_file_path` returns the current
directory relative to the worktree root.

`cache::refresh` rebuilds `$HOME/.cache/doob/status.json` from active todos. Cache failures are
reported to stderr and deliberately do not fail the caller. The JSON contains:

```json
{
  "updated_at": "RFC3339 timestamp",
  "pending_total": 4,
  "overdue_total": 1,
  "overdue_by_repo": { "doob": 1 }
}
```

Only pending and in-progress todos count as active. Overdue grouping uses the todo's `project`.

## Validation and IDs

- `validate_status` accepts todo statuses plus `open`, `done`, `parked`, and `blocked`.
- `validate_project` allows alphanumerics, hyphens, underscores, dots, spaces, and `/`.
- `normalize_id` prefixes an unqualified value with `todo:`.
- `quote_record_id` backtick-quotes record IDs whose ID component contains a hyphen.
- `ExitCode::from_error` classifies user-facing errors by matching their rendered text.

Query guards exist because the SurrealDB adapter must interpolate values in some raw SQL paths.
They do not automatically validate every port call; adapters must invoke them.

## Development and testing

```bash
cargo test -p doob-core
cargo test -p doob-core --no-default-features
cargo clippy -p doob-core --all-targets --all-features -- -D warnings
```

Inline tests cover HANDOFF parsing and reconciliation helpers, query-injection guards, ID
normalization, context-free utility behavior, and exit-code classification. Backend contracts are
exercised primarily by adapter and `doob` integration tests.
