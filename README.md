# doob

Modern, agent-first todo CLI built with Rust and SurrealDB.

## Features

- 🚀 **Fast** - Rust + SurrealDB embedded
- 🤖 **Agent-First** - JSON output, batch operations, context detection
- 📦 **Single Binary** - No dependencies
- 🔍 **Context-Aware** - Auto-detects project and file from git

## Installation

### From Source
```bash
cargo install --path .
```

Or download binary from [releases](https://github.com/yourusername/doob/releases).

## Quick Start

```bash
# Add a todo (auto-detects project/file from git)
doob todo add "Fix auth bug"

# Add with priority and tags
doob todo add "Refactor code" --priority 1 --tags refactor,urgent

# Batch add
doob todo add "Task 1" "Task 2" "Task 3"

# List todos
doob todo list

# Filter by status
doob todo list --status pending

# JSON output for agents
doob --json todo list

# Complete a todo
doob todo complete <id>

# Batch complete
doob todo complete <id1> <id2> <id3>
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
      "tags": ["bug", "urgent"],
      ...
    }
  ]
}
```

### Exit Codes
- `0` - Success
- `1` - Todo not found
- `2` - Invalid input
- `3` - Database error

### Context Detection
Automatically detects:
- **Project**: From git remote URL
- **File**: Relative path from repo root

Override with flags:
```bash
doob todo add "Task" --project myproject --file src/main.rs
```

## Development

```bash
# Run tests
cargo test

# Build
cargo build --release

# Install locally
cargo install --path .
```

## Database

Todos stored at `~/.claude/data/doob.db` (SurrealDB file)

## License

MIT
