#[derive(Debug)]
pub enum ExitCode {
    Success = 0,
    TodoNotFound = 1,
    InvalidInput = 2,
    DatabaseError = 3,
    ContextError = 4,
}

impl ExitCode {
    pub fn from_error(err: &anyhow::Error) -> Self {
        let msg = err.to_string().to_lowercase();

        if msg.contains("not found") {
            ExitCode::TodoNotFound
        } else if msg.contains("invalid") {
            ExitCode::InvalidInput
        } else if msg.contains("database") {
            ExitCode::DatabaseError
        } else {
            ExitCode::DatabaseError
        }
    }
}
