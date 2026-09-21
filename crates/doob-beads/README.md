# doob-beads

`doob-beads` is a synchronous adapter from `doob-sync` to the external Beads `bd` command-line
tool. It implements the minimum provider traits needed to create an external issue from a
`SyncableTodo`.

## Status

**Experimental and not active in the `doob` CLI.** The main crate re-exports `BeadsAdapter`, but no
current command constructs `SyncService<BeadsAdapter>`. The adapter can be used directly by Rust
callers that have `bd` installed.

The `doob` integration files for Beads are gated by `bd` and `integration-tests` feature names
that are not declared in `crates/doob/Cargo.toml`; those tests are therefore inactive in normal
workspace builds. The unit tests in this crate remain active.

## Workspace role

This crate is an outbound adapter in the synchronization subsystem:

```text
doob-sync traits -> BeadsAdapter -> `bd` subprocess
```

It depends only on `doob-sync` and `chrono`. It does not read the doob database and does not persist
the returned `SyncRecord`.

## Requirements

- `bd` must be available on `PATH`.
- `bd --version` must exit successfully for the health check to return `Ok(true)`.
- The installed `bd create` output must contain an ID beginning with `bd-` or `beads-`.

If the executable is missing, `is_available` returns `SyncError::ProviderUnavailable`. A non-zero
`bd create` exit becomes `SyncError::ExternalApiError` containing stderr.

## Public API

`BeadsAdapter` provides `new` and implements:

| Trait | Behavior |
| --- | --- |
| `Provider` | Name `beads`; version from this crate's package version |
| `HealthCheck` | Executes `bd --version` |
| `IssueCreator` | Executes `bd create` and parses the created issue ID |

The blanket implementation in `doob-sync` also makes it a `MinimalIssueTracker`.

## Command mapping

For a syncable todo, the adapter constructs this shape:

```text
bd create TITLE --type=task --priority=N [--description=TEXT]
  [--external-ref=doob-ID] [--labels=tag-a,tag-b]
```

- Priority is clamped to a maximum of `4`; lower values, including `0`, pass through.
- `--description` is included only when a description exists.
- `--external-ref=doob-ID` is included only when the todo has a project.
- Tags are sent through `--labels`, joined with commas.
- Due date and file path are currently ignored.

## Usage

```rust
use doob_beads::BeadsAdapter;
use doob_sync::{SyncService, SyncableTodo, TodoStatus};

let todo = SyncableTodo {
    id: "todo-42".into(),
    title: "Document the storage adapter".into(),
    description: Some("Add source-grounded examples".into()),
    priority: 2,
    status: TodoStatus::Pending,
    tags: vec!["docs".into()],
    project: Some("doob".into()),
    file_path: None,
    due_date: None,
};

let service = SyncService::new(BeadsAdapter::new());
let record = service.sync_todo(&todo)?;
println!("created {}", record.external_id);
# Ok::<(), doob_sync::SyncError>(())
```

This performs a real external mutation. The adapter has no dry-run mode and no update, delete,
read-back, or cleanup support.

## Development and testing

```bash
cargo test -p doob-beads
cargo clippy -p doob-beads --all-targets -- -D warnings
```

Unit tests cover priority clamping, argument construction, label mapping, and parsing both accepted
ID prefixes. They do not execute `bd`.
