# doob-gh

`doob-gh` synchronizes doob todos to GitHub issues by invoking the authenticated `gh` CLI. It owns
project-to-repository mapping, local sync state, issue body construction, and create/close/comment
actions.

## Workspace role

The main `doob` crate re-exports this crate as `doob::gh_sync` and uses it from
`doob todo gh-sync`. This integration is separate from the generic traits in `doob-sync` and does
not implement `Provider`, `IssueCreator`, or the other provider interfaces.

## Status

GitHub issue creation, close-on-complete, tombstone comments, dry runs, repository allowlisting,
and local idempotency state are implemented. Synchronization is push-only; it does not read issue
state back into doob and does not update existing issue titles or bodies.

## Requirements

- `gh` must be installed, on `PATH`, and authenticated for the target owner.
- Todos must have a `project`; todos without one are skipped.
- `$HOME/.config/doob/gh-sync.toml` must exist for synchronization to run.

The library does not read `GITHUB_TOKEN` directly. Authentication is delegated entirely to `gh`.

## Configuration

```toml
[github]
owner = "89jobrien"
allowlist = ["doob", "minibox"]

[sync]
close_on_complete = true
tombstone_on_remove = true
```

Configuration is loaded from `$HOME/.config/doob/gh-sync.toml`.

- `github.owner` is required.
- `github.allowlist` is optional. If absent, every derived repository name is allowed.
- `sync.close_on_complete` and `sync.tombstone_on_remove` both default to `true`.
- A leading `dev/` is stripped from a doob project before mapping it to `owner/repository`.

A missing config file returns `Ok(None)` and causes synchronization to skip without an error. A
present but malformed file is an error.

## State and idempotency

Created issue references are stored at:

```text
$HOME/.config/doob/gh-sync-state.json
```

The JSON object maps each doob UUID to `{ "repo": "owner/name", "issue_number": N }`. Saves use
a temporary file followed by a same-directory rename.

An `add` action skips a UUID already present in state. Complete and remove actions skip UUIDs that
have no state entry. State entries are retained after closing or tombstoning an issue.

## Actions

`sync_todo(todo, action, dry_run)` supports:

| Action | GitHub operation | Configuration gate |
| --- | --- | --- |
| `add` | `gh issue create` | UUID must not already exist in state |
| `complete` | `gh issue close` | `close_on_complete` |
| `remove` | Add a tombstone comment | `tombstone_on_remove` |

Issue bodies contain the todo content and a footer with its stable doob UUID. A dry run returns a
`SyncPlan` without executing `gh` or writing state.

```rust
use doob_core::models::Todo;

fn preview(todo: &Todo) -> anyhow::Result<()> {
    if let Some(plan) = doob_gh::sync_todo(todo, "add", true)? {
        println!("would {} in {}", plan.action, plan.repo);
    }
    Ok(())
}
```

Lower-level public helpers expose config loading, repository mapping, state load/save/upsert, and
the direct `gh` operations.

## CLI usage

The main crate provides:

```bash
doob todo gh-sync                         # bulk pending preview
doob todo gh-sync --action complete       # bulk completed preview
doob todo gh-sync --action remove         # bulk cancelled preview
doob todo gh-sync --execute               # execute selected bulk action
doob todo gh-sync --uuid UUID --execute   # derive action from this todo's status
doob --json todo gh-sync
```

For a single UUID, the action is derived from status: active becomes `add`, completed becomes
`complete`, and cancelled becomes `remove`. The `--action` hint applies only to bulk mode.

`--force` currently bypasses the command's bulk pre-filter, but `sync_todo` still skips an `add`
whose UUID is present in state. It therefore does not create a duplicate issue in the current
implementation.

## Development and testing

```bash
cargo test -p doob-gh
cargo clippy -p doob-gh --all-targets -- -D warnings
```

Tests cover TOML defaults and parsing, repository mapping, state serialization, issue URL parsing,
and issue body construction. They do not execute `gh` or mutate GitHub.
