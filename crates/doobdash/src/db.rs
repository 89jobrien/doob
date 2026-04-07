use anyhow::Result;

/// Domain type — doobdash's view of a todo from the DB.
/// Deliberately separate from doob's internal Todo model.
#[derive(Debug, Clone)]
pub struct DbTodo {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub status: String,
    pub project: String,
    #[allow(dead_code)]
    pub priority: String,
    #[allow(dead_code)]
    pub notes: Vec<String>,
}

/// Port: anything that can supply a list of todos.
#[allow(dead_code)]
pub trait TodoStore: Send + Sync {
    fn list_todos(&self) -> Result<Vec<DbTodo>>;
}

// ---------------------------------------------------------------------------
// SurrealKV adapter
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct SurrealKvAdapter {
    db_path: std::path::PathBuf,
}

impl SurrealKvAdapter {
    /// Use default path: ~/.ctx/doob/db/
    #[allow(dead_code)]
    pub fn default_path() -> Result<Self> {
        let home = dirs_next::home_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
        Ok(SurrealKvAdapter {
            db_path: home.join(".ctx/doob/db"),
        })
    }
}

impl TodoStore for SurrealKvAdapter {
    fn list_todos(&self) -> Result<Vec<DbTodo>> {
        // SurrealDB requires a tokio runtime — use block_in_place since doobdash
        // runs inside a tokio::main context.
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { list_todos_async(&self.db_path).await })
        })
    }
}

#[allow(dead_code)]
async fn list_todos_async(db_path: &std::path::Path) -> Result<Vec<DbTodo>> {
    use surrealdb::engine::local::SurrealKv;
    use surrealdb::Surreal;

    let db = Surreal::new::<SurrealKv>(db_path).await?;
    db.use_ns("doob").use_db("main").await?;

    // Parameterized queries silently no-op in SurrealDB 2.x (issue #6271).
    // Always use raw SQL strings.
    let mut res = db.query("SELECT * FROM todo").await?;
    let rows: Vec<serde_json::Value> = res.take(0)?;

    let todos = rows.into_iter().filter_map(parse_todo).collect();

    Ok(todos)
}

#[allow(dead_code)]
fn parse_todo(v: serde_json::Value) -> Option<DbTodo> {
    let id = v.get("id")?.as_str()?.to_string();
    let title = v.get("title")?.as_str().unwrap_or("(untitled)").to_string();
    let status = v
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("open")
        .to_string();
    let project = v
        .get("project")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let priority = v
        .get("priority")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let notes: Vec<String> = v
        .get("notes")
        .and_then(|n| n.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|n| n.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Some(DbTodo {
        id,
        title,
        status,
        project,
        priority,
        notes,
    })
}

// ---------------------------------------------------------------------------
// In-memory test double
// ---------------------------------------------------------------------------

#[cfg(test)]
pub struct InMemoryStore {
    pub todos: Vec<DbTodo>,
}

#[cfg(test)]
impl TodoStore for InMemoryStore {
    fn list_todos(&self) -> Result<Vec<DbTodo>> {
        Ok(self.todos.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_store_returns_todos() {
        let store = InMemoryStore {
            todos: vec![DbTodo {
                id: "todo:abc".into(),
                title: "Write tests".into(),
                status: "open".into(),
                project: "doob".into(),
                priority: "P1".into(),
                notes: vec![],
            }],
        };
        let result = store.list_todos().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Write tests");
    }

    #[test]
    fn test_parse_todo_missing_id_returns_none() {
        let v = serde_json::json!({ "title": "No ID todo" });
        assert!(parse_todo(v).is_none());
    }

    #[test]
    fn test_parse_todo_defaults_status_to_open() {
        let v = serde_json::json!({ "id": "todo:1", "title": "A task" });
        let t = parse_todo(v).unwrap();
        assert_eq!(t.status, "open");
    }

    #[test]
    fn test_parse_todo_full() {
        let v = serde_json::json!({
            "id": "todo:abc123",
            "title": "Ship feature",
            "status": "done",
            "project": "doob",
            "priority": "P0",
            "notes": ["First note", "Second note"]
        });
        let t = parse_todo(v).unwrap();
        assert_eq!(t.id, "todo:abc123");
        assert_eq!(t.notes.len(), 2);
        assert_eq!(t.priority, "P0");
    }
}
