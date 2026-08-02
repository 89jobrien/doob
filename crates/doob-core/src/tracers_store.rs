//! `CheckpointStore` backed by doob's `TodoRepository`.
//!
//! Bridges tracers-task's synchronous `CheckpointStore` trait to doob's
//! async `TodoRepository` via `block_on`. Must not be constructed from
//! inside code already holding a tokio runtime — see
//! docs/plans/2026-07-31-wire-tracers-task-registry.md Risk section.

use crate::ports::TodoRepository;
use tracers_core::TraceErr;
use tracers_task::checkpoint::CheckpointStore;

pub struct DoobCheckpointStore<R: TodoRepository> {
    repo: R,
    registry_uuid: String,
}

impl<R: TodoRepository> DoobCheckpointStore<R> {
    pub fn new(repo: R, registry_uuid: String) -> Self {
        Self {
            repo,
            registry_uuid,
        }
    }
}

impl<R: TodoRepository> CheckpointStore for DoobCheckpointStore<R> {
    fn load(&self) -> Result<String, TraceErr> {
        let handle = tokio::runtime::Handle::current();
        let todo = handle
            .block_on(self.repo.get_todo_by_uuid(&self.registry_uuid))
            .map_err(|e| TraceErr::Other(miette::miette!("{e}").to_string()))?
            .ok_or_else(|| {
                TraceErr::Other(
                    miette::miette!("no checkpoint todo found for uuid {}", self.registry_uuid)
                        .to_string(),
                )
            })?;
        Ok(todo.content)
    }

    fn save(&self, data: &str) -> Result<(), TraceErr> {
        let handle = tokio::runtime::Handle::current();
        handle
            .block_on(async {
                if self
                    .repo
                    .get_todo_by_uuid(&self.registry_uuid)
                    .await?
                    .is_some()
                {
                    self.repo
                        .update_todo(&self.registry_uuid, None, None, None, None, Some(data))
                        .await
                        .map(|_| ())
                } else {
                    self.repo
                        .create_todos(vec![(
                            data.to_string(),
                            self.registry_uuid.clone(),
                            0,
                            None,
                            None,
                            Vec::new(),
                        )])
                        .await
                        .map(|_| ())
                }
            })
            .map_err(|e| TraceErr::Other(miette::miette!("{e}").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::InMemoryTodoRepository;
    use tracers_task::checkpoint::CheckpointStore;
    use tracers_task::{Task, TaskRegistry};

    #[test]
    fn save_load_roundtrip() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("building a current-thread runtime cannot fail");
        let _guard = rt.enter();

        let repo = InMemoryTodoRepository::new();
        let store = super::DoobCheckpointStore::new(repo, "tracers-registry".to_string());

        let mut registry = TaskRegistry::new();
        registry.insert(Task::new("fetch requirements"));
        registry.insert(Task::new("plan architecture"));

        let json =
            serde_json::to_string(&registry).expect("TaskRegistry always serializes to JSON");
        store
            .save(&json)
            .expect("save must succeed on a fresh in-memory store");
        let loaded = TaskRegistry::load(&store).expect("load must succeed immediately after save");

        assert_eq!(loaded.total(), registry.total());
    }

    #[test]
    fn doob_checkpoint_store_conforms_to_checkpoint_store_contract() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("building a current-thread runtime cannot fail");
        let _guard = rt.enter();

        let repo = InMemoryTodoRepository::new();
        let store = super::DoobCheckpointStore::new(repo, "conformance-check".to_string());
        tracers_task::checkpoint::conformance::assert_checkpoint_store_contract(&store);
    }
}
