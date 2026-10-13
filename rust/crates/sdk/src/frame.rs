//! Frame types and serialization for the SDK wire protocol.

use serde_json::{Map, Value};

use crate::error::{SdkError, SdkResult};

/// Protocol version implemented.
pub const PROTOCOL_VERSION: u32 = 3;

/// Maximum UTF-8 byte length for a single broker JSON frame.
pub const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;

/// Frame type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// Hello frame sent by broker on connection
    BrokerHello,
    /// Request frame from client to broker
    BrokerRequest,
    /// Response frame from broker to client
    BrokerResponse,
    /// Query request
    QueryRequest,
    /// Query response
    QueryResponse,
    /// Control request
    ControlRequest,
    /// Control response
    ControlResponse,
    /// Event replay request
    EventReplay,
    /// Event replay result
    EventReplayResult,
    /// Session activation
    SessionActivate,
    /// Session activation result
    SessionActivateResult,
}

impl FrameType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BrokerHello => "broker_hello",
            Self::BrokerRequest => "broker_request",
            Self::BrokerResponse => "broker_response",
            Self::QueryRequest => "query_request",
            Self::QueryResponse => "query_response",
            Self::ControlRequest => "control_request",
            Self::ControlResponse => "control_response",
            Self::EventReplay => "event_replay",
            Self::EventReplayResult => "event_replay_result",
            Self::SessionActivate => "session_activate",
            Self::SessionActivateResult => "session_activate_result",
        }
    }

    pub fn parse(s: &str) -> SdkResult<Self> {
        match s {
            "broker_hello" => Ok(Self::BrokerHello),
            "broker_request" => Ok(Self::BrokerRequest),
            "broker_response" => Ok(Self::BrokerResponse),
            "query_request" => Ok(Self::QueryRequest),
            "query_response" => Ok(Self::QueryResponse),
            "control_request" => Ok(Self::ControlRequest),
            "control_response" => Ok(Self::ControlResponse),
            "event_replay" => Ok(Self::EventReplay),
            "event_replay_result" => Ok(Self::EventReplayResult),
            "session_activate" => Ok(Self::SessionActivate),
            "session_activate_result" => Ok(Self::SessionActivateResult),
            _ => Err(SdkError::frame_error(format!("unknown frame type: {}", s))),
        }
    }
}

/// A frame in the SDK wire protocol.
///
/// Frames are JSON objects with a `type` field and optional `id` field for correlation.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Underlying JSON value
    pub data: Map<String, Value>,
}

impl Frame {
    /// Create a new frame from a JSON object.
    pub fn new(data: Map<String, Value>) -> Self {
        Self { data }
    }

    /// Create an empty frame.
    pub fn empty() -> Self {
        Self { data: Map::new() }
    }

    /// Get the frame type.
    pub fn frame_type(&self) -> SdkResult<FrameType> {
        let frame_type_str = self
            .data
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| SdkError::frame_error("missing 'type' field"))?;
        FrameType::parse(frame_type_str)
    }

    /// Get the optional request/response ID for correlation.
    pub fn id(&self) -> Option<String> {
        self.data
            .get("id")
            .and_then(Value::as_str)
            .map(String::from)
    }

    /// Get the `ok` boolean field from a response frame.
    pub fn ok(&self) -> SdkResult<bool> {
        self.data
            .get("ok")
            .and_then(Value::as_bool)
            .ok_or_else(|| SdkError::frame_error("missing or invalid 'ok' field"))
    }

    /// Get the error object from a response frame.
    pub fn error(&self) -> Option<SdkResult<(String, String)>> {
        let error_obj = self.data.get("error")?.as_object()?;
        let code = error_obj.get("code")?.as_str()?.to_string();
        let message = error_obj.get("message")?.as_str()?.to_string();
        Some(Ok((code, message)))
    }

    /// Get the result field from a response frame.
    pub fn result(&self) -> Option<Value> {
        self.data.get("result").cloned()
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> SdkResult<String> {
        serde_json::to_string(&self.data)
            .map_err(|e| SdkError::frame_error(format!("JSON serialization error: {}", e)))
    }

    /// Parse from JSON string.
    pub fn from_json(json_str: &str) -> SdkResult<Self> {
        let data: Map<String, Value> = serde_json::from_str(json_str)
            .map_err(|e| SdkError::frame_error(format!("JSON parsing error: {}", e)))?;
        Ok(Self::new(data))
    }

    /// Create a broker hello frame.
    pub fn broker_hello() -> Self {
        let mut data = Map::new();
        data.insert(
            "type".to_string(),
            Value::String("broker_hello".to_string()),
        );
        data.insert(
            "protocolVersion".to_string(),
            Value::Number(PROTOCOL_VERSION.into()),
        );
        Self::new(data)
    }

    /// Create a broker request frame.
    pub fn broker_request(id: String, operation: String, input: Value) -> Self {
        let mut data = Map::new();
        data.insert(
            "type".to_string(),
            Value::String("broker_request".to_string()),
        );
        data.insert("id".to_string(), Value::String(id));
        data.insert("operation".to_string(), Value::String(operation));
        data.insert("input".to_string(), input);
        Self::new(data)
    }

    /// Create a broker response frame.
    pub fn broker_response_ok(id: String, result: Value) -> Self {
        let mut data = Map::new();
        data.insert(
            "type".to_string(),
            Value::String("broker_response".to_string()),
        );
        data.insert("id".to_string(), Value::String(id));
        data.insert("ok".to_string(), Value::Bool(true));
        data.insert("result".to_string(), result);
        Self::new(data)
    }

    /// Create a broker error response frame.
    pub fn broker_response_error(id: String, code: String, message: String) -> Self {
        let mut error_data = Map::new();
        error_data.insert("code".to_string(), Value::String(code));
        error_data.insert("message".to_string(), Value::String(message));

        let mut data = Map::new();
        data.insert(
            "type".to_string(),
            Value::String("broker_response".to_string()),
        );
        data.insert("id".to_string(), Value::String(id));
        data.insert("ok".to_string(), Value::Bool(false));
        data.insert("error".to_string(), Value::Object(error_data));
        Self::new(data)
    }

    /// Get the payload size in bytes.
    pub fn byte_size(&self) -> usize {
        self.to_json().map(|s| s.len()).unwrap_or(0)
    }

    /// Get the operation name from a broker request frame.
    pub fn operation(&self) -> Option<String> {
        self.data
            .get("operation")
            .and_then(Value::as_str)
            .map(String::from)
    }

    /// Get the input value from a broker request frame.
    pub fn input(&self) -> Option<Value> {
        self.data.get("input").cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_type_round_trip() {
        assert_eq!(FrameType::BrokerHello.as_str(), "broker_hello");
        assert_eq!(
            FrameType::parse("broker_hello").unwrap(),
            FrameType::BrokerHello
        );
    }

    #[test]
    fn frame_parsing_extracts_type() {
        let frame = Frame::broker_hello();
        assert_eq!(frame.frame_type().unwrap(), FrameType::BrokerHello);
    }

    #[test]
    fn frame_with_id_can_be_recovered() {
        let frame = Frame::broker_request(
            "test-id".to_string(),
            "test.operation".to_string(),
            Value::Object(Map::new()),
        );
        assert_eq!(frame.id(), Some("test-id".to_string()));
    }

    #[test]
    fn frame_json_round_trip() {
        let frame = Frame::broker_hello();
        let json_str = frame.to_json().unwrap();
        let parsed = Frame::from_json(&json_str).unwrap();
        assert_eq!(parsed.frame_type().unwrap(), FrameType::BrokerHello);
    }

    #[test]
    fn broker_response_error_contains_code_and_message() {
        let frame = Frame::broker_response_error(
            "id1".to_string(),
            "invalid_input".to_string(),
            "test error".to_string(),
        );
        let (code, message) = frame.error().unwrap().unwrap();
        assert_eq!(code, "invalid_input");
        assert_eq!(message, "test error");
    }

    #[test]
    fn frame_extracts_operation_from_broker_request() {
        let frame = Frame::broker_request(
            "test-id".to_string(),
            "query.list_sessions".to_string(),
            Value::Object(Map::new()),
        );
        assert_eq!(frame.operation(), Some("query.list_sessions".to_string()));
    }

    #[test]
    fn frame_extracts_input_from_broker_request() {
        let input = Value::Object({
            let mut m = Map::new();
            m.insert("session_id".to_string(), Value::String("123".to_string()));
            m
        });
        let frame = Frame::broker_request(
            "test-id".to_string(),
            "query.get_session".to_string(),
            input.clone(),
        );
        assert_eq!(frame.input(), Some(input));
    }

    #[test]
    fn frame_without_operation_returns_none() {
        let frame = Frame::broker_hello();
        assert_eq!(frame.operation(), None);
    }
}
