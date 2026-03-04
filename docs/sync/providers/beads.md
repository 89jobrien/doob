# Beads Provider

## Overview

The Beads provider syncs doob todos to [beads](https://github.com/joeobrien/beads) issues using the `bd` CLI.

## Prerequisites

1. Install `bd` CLI:

   ```bash
   cargo install --git https://github.com/joeobrien/beads bd
   ```

2. Initialize beads in your project:

   ```bash
   bd init
   ```

3. Verify bd is available:

   ```bash
   bd --version
   ```

## Configuration

Enable the beads provider:

```bash
doob sync config beads --enable
```

Configuration file (`~/.doob/sync_providers.toml`):

```toml
[beads]
enabled = true
auto_sync = false
sync_filter = { statuses = ["pending", "in_progress"], projects = [], min_priority = null }
[beads.custom]
# No custom settings needed for beads
```

## Usage

```bash
# Sync todos to beads
doob sync to --provider beads

# Dry run
doob sync to --provider beads --dry-run

# Sync specific project
doob sync to --provider beads --project dotfiles
```

## Priority Mapping

| Doob Priority | Beads Priority | Description |
| ------------- | -------------- | ----------- |
| 0             | 0 (P0)         | Critical    |
| 1             | 1 (P1)         | High        |
| 2             | 2 (P2)         | Medium      |
| 3             | 3 (P3)         | Low         |
| 4-5           | 4 (P4)         | Backlog     |

## Field Mapping

- **Title** → bd issue title
- **Description** → bd issue description (if present)
- **Priority** → bd priority (0-4 scale)
- **Tags** → bd issue notes (formatted as "tags: tag1, tag2")
- **Project** → bd external-ref (formatted as "doob-{id}")

## Limitations

- One-way sync only (doob → beads)
- No update support (Phase 1)
- Cannot sync completed todos
- Requires bd CLI installed and accessible

## Troubleshooting

### "bd CLI not found"

Ensure bd is installed and in your PATH:

```bash
which bd
bd --version
```

### "Created issue bd-42 not parsed"

The bd CLI output format may have changed. Check bd version compatibility.

### Permission denied

Ensure you have write access to the `.beads/` directory in your project.
