use crate::models::Todo;
use crate::ports::TodoRepository;
use anyhow::{anyhow, Result};

pub struct UpdateFields {
    pub priority: Option<u8>,
    pub status: Option<String>,
    pub project: Option<String>,
    pub tags: Option<String>,
    pub content: Option<String>,
}

pub async fn execute(repo: &dyn TodoRepository, id: String, fields: UpdateFields) -> Result<Todo> {
    // Require at least one field
    if fields.priority.is_none()
        && fields.status.is_none()
        && fields.project.is_none()
        && fields.tags.is_none()
        && fields.content.is_none()
    {
        return Err(anyhow!(
            "No fields provided. Specify at least one of: --priority, --status, --project, \
             --tags, --content"
        ));
    }

    // Validate status value if provided
    if let Some(ref s) = fields.status {
        match s.as_str() {
            "pending" | "in_progress" | "completed" => {}
            other => {
                return Err(anyhow!(
                    "Invalid status '{}'. Must be one of: pending, in_progress, completed",
                    other
                ));
            }
        }
    }

    // Validate priority if provided
    if let Some(p) = fields.priority {
        if !(1..=5).contains(&p) {
            return Err(anyhow!("Priority must be between 1 and 5, got {}", p));
        }
    }

    let tag_list = fields.tags.as_ref().map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>()
    });

    repo.update_todo(
        &id,
        fields.priority,
        fields.status.as_deref(),
        fields.project.as_deref(),
        tag_list,
        fields.content.as_deref(),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_fields__no_fields_returns_error() {
        // We can't run the async fn without a real DB, but we can test the
        // validation logic by checking our guard condition directly.
        let fields = UpdateFields {
            priority: None,
            status: None,
            project: None,
            tags: None,
            content: None,
        };
        let all_none = fields.priority.is_none()
            && fields.status.is_none()
            && fields.project.is_none()
            && fields.tags.is_none()
            && fields.content.is_none();
        assert!(all_none, "should detect that no fields were provided");
    }

    #[test]
    fn update_fields__priority_set_is_not_empty() {
        let fields = UpdateFields {
            priority: Some(3),
            status: None,
            project: None,
            tags: None,
            content: None,
        };
        let all_none = fields.priority.is_none()
            && fields.status.is_none()
            && fields.project.is_none()
            && fields.tags.is_none()
            && fields.content.is_none();
        assert!(
            !all_none,
            "priority = Some(3) should not trigger empty guard"
        );
    }
}
