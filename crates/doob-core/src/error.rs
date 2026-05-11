#[derive(Debug)]
pub enum ExitCode {
    Success = 0,
    TodoNotFound = 1,
    InvalidInput = 2,
    DatabaseError = 3,
    ContextError = 4,
    IoError = 5,
    ParseError = 6,
}

impl ExitCode {
    pub fn from_error(err: &anyhow::Error) -> Self {
        let msg = err.to_string().to_lowercase();

        if msg.contains("permission denied")
            || msg.contains("no such file")
            || msg.contains("os error")
        {
            ExitCode::IoError
        } else if msg.contains("failed to parse") || msg.contains("unexpected token") {
            ExitCode::ParseError
        } else if msg.contains("not found") {
            ExitCode::TodoNotFound
        } else if msg.contains("invalid") {
            ExitCode::InvalidInput
        } else if msg.contains("context") {
            ExitCode::ContextError
        } else {
            // Covers database/connection/surreal errors and unknown errors
            ExitCode::DatabaseError
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::ExitCode;
    use anyhow::anyhow;

    #[test]
    fn exit_code__from_not_found_error() {
        let err = anyhow!("Todo not found: todo:abc");
        assert!(matches!(ExitCode::from_error(&err), ExitCode::TodoNotFound));
    }

    #[test]
    fn exit_code__from_invalid_error() {
        let err = anyhow!("Invalid date format: 'foo'");
        assert!(matches!(ExitCode::from_error(&err), ExitCode::InvalidInput));
    }

    #[test]
    fn exit_code__from_generic_error_falls_back_to_database_error() {
        let err = anyhow!("connection refused");
        assert!(matches!(
            ExitCode::from_error(&err),
            ExitCode::DatabaseError
        ));
    }

    #[test]
    fn exit_code__io_error_is_not_database_error() {
        let err = anyhow!("No such file or directory (os error 2)");
        assert!(matches!(ExitCode::from_error(&err), ExitCode::IoError));
    }

    #[test]
    fn exit_code__parse_error_is_not_database_error() {
        let err = anyhow!("failed to parse yaml: unexpected token at line 3");
        assert!(matches!(ExitCode::from_error(&err), ExitCode::ParseError));
    }

    #[test]
    fn exit_code__permission_denied_is_io_error() {
        let err = anyhow!("permission denied on /foo");
        assert!(matches!(ExitCode::from_error(&err), ExitCode::IoError));
    }

    #[test]
    fn exit_code__parse_error_is_parse_error() {
        let err = anyhow!("failed to parse yaml");
        assert!(matches!(ExitCode::from_error(&err), ExitCode::ParseError));
    }

    #[test]
    fn exit_code__os_error_is_io_error() {
        let err = anyhow!("No such file (os error 2)");
        assert!(matches!(ExitCode::from_error(&err), ExitCode::IoError));
    }
}
