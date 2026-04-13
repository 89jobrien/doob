use crate::commands::quote_record_id;
use crate::db::DbConnection;
use crate::models::Todo;
use anyhow::Result;

pub struct DepsView {
    pub root: Todo,
    /// Todos that block this one (from blocked_by list)
    pub blockers: Vec<Todo>,
    /// Todos that this one blocks (from blocks list)
    pub dependents: Vec<Todo>,
}

pub async fn execute(db: &DbConnection, id: String) -> Result<DepsView> {
    // Look up root by uuid or record ID
    let root = fetch_by_id(db, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Todo not found: {}", id))?;

    let blockers = fetch_by_uuids(db, &root.blocked_by).await?;
    let dependents = fetch_by_uuids(db, &root.blocks).await?;

    Ok(DepsView {
        root,
        blockers,
        dependents,
    })
}

/// Set blocks/blocked_by on an existing todo by UUID.
///
/// SurrealDB 2.x parameterized queries silently no-op (issue #6271), so the
/// UUID filter and list values are interpolated directly. UUIDs are validated
/// to be hex+hyphen only before interpolation. Dep UUIDs are also validated.
pub async fn link(
    db: &DbConnection,
    uuid: &str,
    blocks: &[String],
    blocked_by: &[String],
) -> Result<()> {
    // Validate UUID is safe to interpolate (hex chars and hyphens only)
    if !uuid.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        anyhow::bail!("invalid UUID for dep link: {uuid:?}");
    }

    fn surreal_string_list(uuids: &[String]) -> anyhow::Result<String> {
        for u in uuids {
            if !u.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
                anyhow::bail!("invalid dep UUID: {u:?}");
            }
        }
        let items: Vec<String> = uuids.iter().map(|u| format!("'{u}'")).collect();
        Ok(format!("[{}]", items.join(", ")))
    }

    let blocks_str = surreal_string_list(blocks)?;
    let blocked_by_str = surreal_string_list(blocked_by)?;

    let query = format!(
        "UPDATE todo SET blocks = {blocks_str}, blocked_by = {blocked_by_str} WHERE uuid = '{uuid}'"
    );
    db.query(&query).await?;
    Ok(())
}

async fn fetch_by_id(db: &DbConnection, id: &str) -> Result<Option<Todo>> {
    // Try uuid lookup first, then record ID
    let mut result = db
        .query("SELECT * FROM todo WHERE uuid = $id LIMIT 1")
        .bind(("id", id.to_string()))
        .await?;
    let todos: Vec<Todo> = result.take(0)?;
    if let Some(todo) = todos.into_iter().next() {
        return Ok(Some(todo));
    }

    // Try as record ID (todo:xxx)
    let record_id = if id.contains(':') {
        id.to_string()
    } else {
        format!("todo:{}", id)
    };
    let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
    let mut result = db.query(&query).await?;
    let todos: Vec<Todo> = result.take(0)?;
    Ok(todos.into_iter().next())
}

async fn fetch_by_uuids(db: &DbConnection, uuids: &[String]) -> Result<Vec<Todo>> {
    if uuids.is_empty() {
        return Ok(vec![]);
    }
    let mut todos = Vec::new();
    for uuid in uuids {
        let mut result = db
            .query("SELECT * FROM todo WHERE uuid = $uuid LIMIT 1")
            .bind(("uuid", uuid.clone()))
            .await?;
        let found: Vec<Todo> = result.take(0)?;
        todos.extend(found);
    }
    Ok(todos)
}
