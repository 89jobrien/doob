use crate::db::DbConnection;
use crate::models::{Note, Todo};
use anyhow::Result;

pub struct SearchResults {
    pub todos: Vec<Todo>,
    pub notes: Vec<Note>,
    pub query: String,
}

pub async fn execute(
    db: &DbConnection,
    query: String,
    search_type: String,
    project: Option<String>,
) -> Result<SearchResults> {
    let q = query.to_lowercase();

    let todos = if search_type != "note" {
        fetch_todos(db, &q, project.as_deref()).await?
    } else {
        vec![]
    };

    let notes = if search_type != "todo" {
        fetch_notes(db, &q, project.as_deref()).await?
    } else {
        vec![]
    };

    Ok(SearchResults {
        todos,
        notes,
        query,
    })
}

async fn fetch_todos(db: &DbConnection, query: &str, project: Option<&str>) -> Result<Vec<Todo>> {
    let mut q = String::from(
        "SELECT * FROM todo WHERE string::contains(string::lowercase(content), $query)",
    );
    if project.is_some() {
        q.push_str(" AND project = $project");
    }
    q.push_str(" ORDER BY created_at DESC");

    let mut builder = db.query(&q).bind(("query", query.to_string()));
    if let Some(p) = project {
        builder = builder.bind(("project", p.to_string()));
    }

    let mut result = builder.await?;
    Ok(result.take(0)?)
}

async fn fetch_notes(db: &DbConnection, query: &str, project: Option<&str>) -> Result<Vec<Note>> {
    let mut q = String::from(
        "SELECT * FROM note WHERE string::contains(string::lowercase(content), $query)",
    );
    if project.is_some() {
        q.push_str(" AND project = $project");
    }
    q.push_str(" ORDER BY created_at DESC");

    let mut builder = db.query(&q).bind(("query", query.to_string()));
    if let Some(p) = project {
        builder = builder.bind(("project", p.to_string()));
    }

    let mut result = builder.await?;
    Ok(result.take(0)?)
}
