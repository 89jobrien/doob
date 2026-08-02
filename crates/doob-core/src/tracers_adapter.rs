//! Conversions between doob's `Todo` model and `tracers_task::Task`.
//!
//! Lossy in both directions by design — see
//! docs/plans/2026-07-31-wire-tracers-task-registry.md.

use crate::models::todo::TodoStatus;
use tracers_core::TraceErr;
use tracers_task::TaskStatus;

pub fn todo_status_to_task_status(status: &TodoStatus) -> TaskStatus {
    match status {
        TodoStatus::Pending => TaskStatus::Pending,
        TodoStatus::InProgress => TaskStatus::Running,
        TodoStatus::Completed => TaskStatus::Done(tracers_core::TraceRef(uuid::Uuid::nil())),
        TodoStatus::Cancelled => TaskStatus::Failed {
            error: TraceErr::Rejected("doob todo cancelled".to_string()),
            trace: tracers_core::TraceRef(uuid::Uuid::nil()),
        },
    }
}

pub fn task_status_to_todo_status(status: &TaskStatus) -> TodoStatus {
    match status {
        TaskStatus::Pending => TodoStatus::Pending,
        TaskStatus::Running => TodoStatus::InProgress,
        TaskStatus::Done(_) => TodoStatus::Completed,
        TaskStatus::Failed { .. } => TodoStatus::Cancelled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_status_maps_to_task_status() {
        assert_eq!(
            todo_status_to_task_status(&TodoStatus::Pending),
            TaskStatus::Pending
        );
        assert_eq!(
            todo_status_to_task_status(&TodoStatus::InProgress),
            TaskStatus::Running
        );
        assert!(matches!(
            todo_status_to_task_status(&TodoStatus::Cancelled),
            TaskStatus::Failed {
                error: TraceErr::Rejected(_),
                ..
            }
        ));
    }

    #[test]
    fn task_status_maps_to_todo_status() {
        use tracers_core::TraceRef;

        assert_eq!(
            task_status_to_todo_status(&TaskStatus::Pending),
            TodoStatus::Pending
        );
        assert_eq!(
            task_status_to_todo_status(&TaskStatus::Running),
            TodoStatus::InProgress
        );
        assert_eq!(
            task_status_to_todo_status(&TaskStatus::Done(TraceRef(uuid::Uuid::nil()))),
            TodoStatus::Completed
        );
        assert_eq!(
            task_status_to_todo_status(&TaskStatus::Failed {
                error: TraceErr::Rejected("test".to_string()),
                trace: TraceRef(uuid::Uuid::nil()),
            }),
            TodoStatus::Cancelled
        );
    }
}
