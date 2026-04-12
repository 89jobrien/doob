use crate::models::{Note, Todo};
use crate::ports::TodoRepository;
use anyhow::Result;

pub struct SearchResults {
    pub todos: Vec<Todo>,
    pub notes: Vec<Note>,
    pub query: String,
}

pub async fn execute(
    repo: &dyn TodoRepository,
    query: String,
    search_type: String,
    project: Option<String>,
) -> Result<SearchResults> {
    let todos = if search_type != "note" {
        repo.search_todos(&query, project.as_deref()).await?
    } else {
        vec![]
    };

    let notes = if search_type != "todo" {
        repo.search_notes(&query, project.as_deref()).await?
    } else {
        vec![]
    };

    Ok(SearchResults {
        todos,
        notes,
        query,
    })
}
