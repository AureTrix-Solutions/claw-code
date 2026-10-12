//! WebSocket transport implementation for the SDK wire protocol.

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use crate::error::{SdkError, SdkResult};
use crate::frame::{Frame, FrameType, MAX_FRAME_BYTES};

/// Configuration for SDK transport.
#[derive(Debug, Clone)]
pub struct SdkTransportConfig {
    /// Server URL (e.g., "ws://127.0.0.1:8080")
    pub url: String,
    /// Authentication token
    pub token: String,
    /// Maximum payload size in bytes (default: 4 MiB)
    pub max_payload_bytes: usize,
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
}

impl Default for SdkTransportConfig {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:8080".to_string(),
            token: String::new(),
            max_payload_bytes: MAX_FRAME_BYTES,
            timeout_secs: 30,
        }
    }
}

/// Pending request tracking for correlation.
struct PendingRequest {
    response_tx: oneshot::Sender<SdkResult<Frame>>,
}

/// SDK WebSocket transport.
pub struct SdkTransport {
    /// Channel for sending messages to the writer task
    tx: mpsc::UnboundedSender<Message>,
    /// Pending requests by ID
    pending: Arc<RwLock<HashMap<String, PendingRequest>>>,
    /// Configuration
    config: SdkTransportConfig,
    /// Background task handle
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl SdkTransport {
    /// Create a new SDK transport and connect to the broker.
    pub async fn connect(config: SdkTransportConfig) -> SdkResult<Self> {
        // Add token to query params
        let url_with_token = if config.url.contains('?') {
            format!("{}&token={}", config.url, config.token)
        } else {
            format!("{}?token={}", config.url, config.token)
        };

        // Create WebSocket connection
        let (ws, _) = connect_async(&url_with_token).await.map_err(|e| {
            SdkError::connection_error(format!("WebSocket connection failed: {}", e))
        })?;

        let (ws_tx, ws_rx) = tokio::sync::mpsc::unbounded_channel();
        let pending = Arc::new(RwLock::new(HashMap::new()));
        let pending_clone = pending.clone();

        // Spawn background task for handling the WebSocket
        let task_handle = tokio::spawn(async move {
            Self::background_task(ws, ws_rx, pending_clone).await;
        });

        let transport = Self {
            tx: ws_tx,
            pending,
            config,
            task_handle: Arc::new(Mutex::new(Some(task_handle))),
        };

        Ok(transport)
    }

    /// Send a request and wait for response.
    pub async fn request(&self, operation: String, input: Value) -> SdkResult<Frame> {
        // Generate correlation ID
        let id = Uuid::new_v4().to_string();

        // Create request frame
        let request = Frame::broker_request(id.clone(), operation, input);

        // Check payload size
        if request.byte_size() > self.config.max_payload_bytes {
            return Err(SdkError::payload_too_large(format!(
                "frame size {} exceeds maximum {}",
                request.byte_size(),
                self.config.max_payload_bytes
            )));
        }

        // Create response channel
        let (response_tx, response_rx) = oneshot::channel();
        self.pending
            .write()
            .await
            .insert(id.clone(), PendingRequest { response_tx });

        // Send request
        let json = request.to_json()?;
        self.tx
            .send(Message::Text(json))
            .map_err(|e| SdkError::connection_error(format!("send failed: {}", e)))?;

        // Wait for response with timeout
        let timeout_duration = std::time::Duration::from_secs(self.config.timeout_secs);
        match tokio::time::timeout(timeout_duration, response_rx).await {
            Ok(Ok(frame)) => frame,
            Ok(Err(_)) => Err(SdkError::correlation_error("response channel closed")),
            Err(_) => Err(SdkError::timeout("response not received within timeout")),
        }
    }

    /// Background task for handling WebSocket communication.
    #[allow(unused_mut)]
    async fn background_task(
        ws: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        mut ws_rx: mpsc::UnboundedReceiver<Message>,
        pending: Arc<RwLock<HashMap<String, PendingRequest>>>,
    ) {
        let (mut ws_sink, mut ws_stream) = ws.split();

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                msg = ws_stream.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            // Parse frame
                            match Frame::from_json(&text) {
                                Ok(frame) => {
                                    // Handle frame type
                                    match frame.frame_type() {
                                        Ok(FrameType::BrokerHello) => {
                                            // Hello frame received, continue
                                        }
                                        Ok(FrameType::BrokerResponse) => {
                                            // Response frame, correlate with request
                                            if let Some(id) = frame.id() {
                                                if let Some(PendingRequest { response_tx }) =
                                                    pending.write().await.remove(&id)
                                                {
                                                    let _ = response_tx.send(Ok(frame));
                                                }
                                            }
                                        }
                                        _ => {
                                            // Other frame types
                                            tracing::debug!("Unhandled frame type: {:?}", frame.frame_type());
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to parse frame: {}", e);
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            break;
                        }
                        Some(Ok(_)) => {
                            // Other message types (binary, etc.), ignore
                        }
                        Some(Err(e)) => {
                            tracing::warn!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            break;
                        }
                    }
                }
                // Handle outgoing messages
                Some(msg) = ws_rx.recv() => {
                    if let Err(e) = ws_sink.send(msg).await {
                        tracing::warn!("Failed to send message: {}", e);
                        break;
                    }
                }
                else => break,
            }
        }

        // Fail all pending requests
        let mut pending = pending.write().await;
        for (_, PendingRequest { response_tx }) in pending.drain() {
            let _ = response_tx.send(Err(SdkError::connection_error("connection closed")));
        }
    }

    /// Close the transport.
    pub async fn close(self) -> SdkResult<()> {
        // Send close message
        let _ = self.tx.send(Message::Close(None));

        // Wait for background task to finish
        if let Some(handle) = self.task_handle.lock().await.take() {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_values() {
        let config = SdkTransportConfig::default();
        assert_eq!(config.url, "ws://127.0.0.1:8080");
        assert_eq!(config.max_payload_bytes, MAX_FRAME_BYTES);
        assert_eq!(config.timeout_secs, 30);
    }

    #[tokio::test]
    async fn transport_can_be_created() {
        // This test doesn't actually connect, just checks that the struct can be created
        let config = SdkTransportConfig::default();
        assert_eq!(config.url, "ws://127.0.0.1:8080");
    }

    #[test]
    fn request_frame_has_correlation_id() {
        let frame = Frame::broker_request(
            "test-id".to_string(),
            "test.operation".to_string(),
            Value::Object(serde_json::Map::new()),
        );
        assert_eq!(frame.id(), Some("test-id".to_string()));
    }
}
