# doob-sync

`doob-sync` defines the provider-neutral domain for pushing doob todos to external issue trackers.
It contains serializable sync types, capability-oriented provider traits, and `SyncService`.

## Workspace role

This crate is the synchronization port layer. It contains no database, HTTP, subprocess, or async
runtime dependency. Provider adapters such as `doob-beads` depend on it, while the primary `doob`
crate re-exports it as `doob::sync::domain`.

The GitHub integration in `doob-gh` is a separate implementation and currently does not implement
these provider traits.

## Status

The domain and Beads integration are implemented and unit-tested. The provider framework is not
wired to a `doob` CLI command, and there are no active GitHub, Linear, Jira, or bidirectional
provider implementations in this workspace.

## Domain types

| Type | Purpose |
| --- | --- |
| `SyncableTodo` | Provider-facing todo with title, description, priority, tags, and context |
| `TodoStatus` | Active sync states: `Pending` and `InProgress` |
| `SyncRecord` | External ID, optional URL, provider name, and sync timestamp |
| `SyncError` | Provider, configuration, API, duplicate, database, and serialization failures |

`TodoStatus` serializes with Rust variant names (`"Pending"` and `"InProgress"`), not the
snake-case values used by `doob-core::TodoStatus`.

## Provider traits

Traits are deliberately segregated so adapters implement only their actual capabilities:

| Trait | Responsibility |
| --- | --- |
| `Provider` | Name, version, and declared capabilities |
| `HealthCheck` | Availability and optional health details |
| `IssueCreator` | Create one issue |
| `IssueUpdater` | Update an existing issue |
| `IssueDeleter` | Delete an issue |
| `ExternalIssueReader` | Read one issue or list changed issues |
| `BatchIssueCreator` | Create several issues; blanket implementation is sequential |

Composite marker traits describe useful provider levels:

- `MinimalIssueTracker`: provider identity, health, and creation.
- `StandardIssueTracker`: minimal plus update and delete.
- `FullIssueTracker`: standard plus external reads.
- `BatchIssueTracker`: minimal plus batch creation.

`IssueTracker` remains as a deprecated compatibility alias for the minimal trait set.

`ProviderCapabilities` advertises update, delete, bidirectional, batch, and webhook support. The
default marks every capability false. The service does not currently inspect these flags.

## SyncService

`SyncService<T: MinimalIssueTracker>` owns one provider and exposes:

- `new`: construct a service.
- `sync_todo`: check availability and create one issue.
- `sync_todos`: check availability once, then process every todo sequentially.
- `provider_name` and `is_available`: provider introspection.

Batch results preserve input order and a creation failure does not stop later items. If the health
check returns `false` or errors, `sync_todos` returns `ProviderUnavailable` for every input.

## Implementing a provider

```rust
use doob_sync::{
    HealthCheck, IssueCreator, Provider, SyncError, SyncRecord, SyncService, SyncableTodo,
};

struct Tracker;

impl Provider for Tracker {
    fn name(&self) -> &str { "example" }
    fn version(&self) -> &str { "1" }
}

impl HealthCheck for Tracker {
    fn is_available(&self) -> Result<bool, SyncError> { Ok(true) }
}

impl IssueCreator for Tracker {
    fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
        Ok(SyncRecord {
            external_id: format!("example-{}", todo.id),
            external_url: None,
            provider: self.name().into(),
            synced_at: "2026-09-19T00:00:00Z".into(),
        })
    }
}

let service = SyncService::new(Tracker);
assert_eq!(service.provider_name(), "example");
```

Implementing `Provider + HealthCheck + IssueCreator` automatically implements
`MinimalIssueTracker`; no explicit marker implementation is needed.

## Constraints

- The API is synchronous even when an adapter performs network or subprocess I/O.
- Only active statuses exist in this crate; conversion from `doob-core::Todo` is caller-owned.
- `SyncRecord` persistence and duplicate detection are adapter or application responsibilities.
- `sync_todos` uses repeated `create_issue` calls rather than `BatchIssueCreator`.
- Provider capability flags are descriptive and are not enforced by `SyncService`.

## Development and testing

```bash
cargo test -p doob-sync
cargo clippy -p doob-sync --all-targets -- -D warnings
```

Unit tests use in-memory mock providers to cover successful creation, provider availability,
per-item failures, result ordering, and the single availability check used by batch sync.
