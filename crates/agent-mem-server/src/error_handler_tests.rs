//! Error Handler Tests
//!
//! Tests for unified error handling system

#[cfg(test)]
mod tests {
    use super::super::error_handler::{ErrorHandler, ErrorMonitor, safe_expect, safe_unwrap};
    use super::super::error::ServerError;
    use std::io;

    #[test]
    fn test_error_handler_trait() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let server_error = io_error.to_server_error("Reading config file");

        match server_error {
            ServerError::Internal { message, .. } => {
                assert!(message.contains("Reading config file"));
                assert!(message.contains("File not found"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_error_handler_with_details() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let server_error = io_error.to_server_error_with_details(
            "Writing to file",
            "Insufficient permissions for /etc/config",
        );

        match server_error {
            ServerError::Internal { message, context, .. } => {
                assert!(message.contains("Writing to file"));
                assert!(message.contains("Access denied"));
                if let Some(ctx) = context {
                    assert!(ctx.details.is_some());
                }
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_safe_unwrap_ok() {
        let result: Result<i32, io::Error> = Ok(42);
        let server_result = safe_unwrap(result, "test operation");
        assert!(server_result.is_ok());
        assert_eq!(server_result.unwrap(), 42);
    }

    #[test]
    fn test_safe_unwrap_err() {
        let result: Result<i32, io::Error> = Err(io::Error::new(io::ErrorKind::Other, "test error"));
        let server_result = safe_unwrap(result, "test operation");
        assert!(server_result.is_err());
    }

    #[test]
    fn test_safe_expect_some() {
        let value = Some(42);
        let result = safe_expect(value, "test context", None::<String>);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_safe_expect_none() {
        let value: Option<i32> = None;
        let result = safe_expect(value, "test context", Some("value is None"));
        assert!(result.is_err());
        
        match result {
            Err(ServerError::Internal { message, .. }) => {
                assert!(message.contains("test context"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_error_monitor() {
        let monitor = ErrorMonitor::new(5);
        assert_eq!(monitor.error_count(), 0);

        let error = ServerError::Internal {
            message: "Test error".to_string(),
            source: None,
            context: None,
            backtrace: None,
        };

        for _ in 0..3 {
            monitor.record_error(&error);
        }

        assert_eq!(monitor.error_count(), 3);

        monitor.reset();
        assert_eq!(monitor.error_count(), 0);
    }

    #[test]
    fn test_error_monitor_threshold() {
        let monitor = ErrorMonitor::new(3);
        
        let error = ServerError::Internal {
            message: "Test error".to_string(),
            source: None,
            context: None,
            backtrace: None,
        };

        // Record errors up to threshold
        for _ in 0..3 {
            monitor.record_error(&error);
        }

        assert_eq!(monitor.error_count(), 3);
        // At threshold, should trigger alert (logged as warning)
    }
}
