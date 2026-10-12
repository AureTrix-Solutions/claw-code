//! SDK error types and result type.

use std::fmt;

/// SDK wire protocol error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Invalid JSON or malformed frame
    InvalidInput,
    /// Unknown broker operation
    UnknownOperation,
    /// Payload too large (>4 MiB)
    PayloadTooLarge,
    /// Authentication token mismatch
    Unauthorized,
    /// Broker or session unavailable
    Unavailable,
    /// WebSocket connection error
    ConnectionError,
    /// Request timeout
    Timeout,
    /// Frame parsing or validation error
    FrameError,
    /// Request/response correlation failure
    CorrelationError,
    /// Other unclassified error
    Internal,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => write!(f, "invalid_input"),
            Self::UnknownOperation => write!(f, "unknown_operation"),
            Self::PayloadTooLarge => write!(f, "payload_too_large"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Unavailable => write!(f, "unavailable"),
            Self::ConnectionError => write!(f, "connection_error"),
            Self::Timeout => write!(f, "timeout"),
            Self::FrameError => write!(f, "frame_error"),
            Self::CorrelationError => write!(f, "correlation_error"),
            Self::Internal => write!(f, "internal_error"),
        }
    }
}

/// SDK wire protocol error.
#[derive(Debug)]
pub struct SdkError {
    pub code: ErrorCode,
    pub message: String,
}

impl SdkError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    pub fn unknown_operation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::UnknownOperation, message)
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PayloadTooLarge, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unauthorized, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unavailable, message)
    }

    pub fn connection_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ConnectionError, message)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout, message)
    }

    pub fn frame_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::FrameError, message)
    }

    pub fn correlation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::CorrelationError, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SdkError {}

/// Result type for SDK operations.
pub type SdkResult<T> = Result<T, SdkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_formats_code_and_message() {
        let err = SdkError::invalid_input("test message");
        assert_eq!(err.to_string(), "invalid_input: test message");
    }

    #[test]
    fn error_code_display_works() {
        assert_eq!(ErrorCode::InvalidInput.to_string(), "invalid_input");
        assert_eq!(ErrorCode::PayloadTooLarge.to_string(), "payload_too_large");
        assert_eq!(ErrorCode::Unauthorized.to_string(), "unauthorized");
    }
}
