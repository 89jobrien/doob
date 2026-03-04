# doob

Modern, agent-first todo CLI built with Rust and SurrealDB.

## Features

- **Fast** - Rust + SurrealDB embedded
- **Agent-First** - JSON output, batch operations, context detection
- **Single Binary** - No dependencies
- **Context-Aware** - Auto-detects project and file from git

## Installation

```bash
cargo install --path .
```

## Commands

```bash
# Add a todo (auto-detects project/file from git)
doob todo add "Fix auth bug"

# Add with priority and tags
doob todo add "Refactor code" --priority 1 --tags refactor,urgent

# Batch add
doob todo add "Task 1" "Task 2" "Task 3"

# List todos
doob todo list

# Filter by status or project
doob todo list --status pending
doob todo list --project myproject --limit 10

# Complete todo(s)
doob todo complete <id>
doob todo complete <id1> <id2> <id3>

# Undo completion (mark as pending)
doob todo undo <id>

# Set or clear a due date
doob todo due <id> 2026-03-15
doob todo due <id> clear

# Remove todo(s)
doob todo remove <id>
doob todo remove <id1> <id2>
```

## Agent Integration

Perfect for Claude Code, Cursor, Aider, and other AI coding assistants.

### JSON Output

```bash
doob --json todo list
```

Returns:

```json
{
  "count": 1,
  "todos": [
    {
      "id": "todo:abc123",
      "content": "Fix auth bug",
      "status": "pending",
      "priority": 1,
      "tags": ["bug", "urgent"]
    }
  ]
}
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Todo not found |
| `2` | Invalid input |
| `3` | Database error |

### Context Detection

Automatically detects from git:
- **Project** — from remote URL
- **File** — relative path from repo root

Override with flags:

```bash
doob todo add "Task" --project myproject --file src/main.rs
```

## Development

```bash
cargo test
cargo build --release
cargo install --path .
```

## Database

Todos stored at `~/.claude/data/doob.db` (SurrealDB/RocksDB).

## License

MIT
