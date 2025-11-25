//! MQTT Bridge for IoT Device Integration
//!
//! Bridges MQTT messages from IoT devices (ESP32, etc.) to the WebSocket chat gateway.
//! Provides E2EE encryption/decryption for device messages.

#[cfg(feature = "mqtt-bridge")]
use {
    anyhow::{Context, Result},
    dashmap::DashMap,
    e2ee::{DoubleRatchet, EncryptedMessage, SessionStore},
    rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS, Transport},
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    std::time::Duration,
    tokio::sync::mpsc,
    tracing::{debug, error, info, warn},
};

/// MQTT message payload format
#[cfg(feature = "mqtt-bridge")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MqttMessage {
    /// Device ID (unique identifier)
    pub device_id: String,
    /// Message type: "text", "telemetry", "command"
    pub message_type: String,
    /// E2EE encrypted payload (if enabled)
    pub encrypted_payload: Option<Vec<u8>>,
    /// Plain payload (only for non-E2EE messages)
    pub plain_payload: Option<String>,
    /// Timestamp (Unix milliseconds)
    pub timestamp: u64,
}

/// MQTT bridge configuration
#[cfg(feature = "mqtt-bridge")]
#[derive(Debug, Clone)]
pub struct MqttBridgeConfig {
    /// MQTT broker host
    pub broker_host: String,
    /// MQTT broker port (1883 for plain, 8883 for TLS)
    pub broker_port: u16,
    /// Client ID prefix
    pub client_id_prefix: String,
    /// Enable TLS
    pub tls_enabled: bool,
    /// CA certificate path (for TLS)
    pub ca_cert_path: Option<String>,
    /// Client certificate path (for mutual TLS)
    pub client_cert_path: Option<String>,
    /// Client key path (for mutual TLS)
    pub client_key_path: Option<String>,
    /// Keep-alive interval (seconds)
    pub keep_alive: u64,
    /// Topic prefix for device messages
    pub topic_prefix: String,
}

impl Default for MqttBridgeConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id_prefix: "unhidra-bridge".to_string(),
            tls_enabled: false,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            keep_alive: 60,
            topic_prefix: "unhidra/devices".to_string(),
        }
    }
}

/// MQTT bridge for IoT device integration
#[cfg(feature = "mqtt-bridge")]
pub struct MqttBridge {
    /// MQTT client
    client: AsyncClient,
    /// Event loop handle
    event_loop: EventLoop,
    /// Device E2EE sessions (device_id -> SessionStore)
    device_sessions: Arc<DashMap<String, SessionStore>>,
    /// Message sender channel
    message_tx: mpsc::UnboundedSender<(String, Vec<u8>)>,
    /// Message receiver channel
    message_rx: mpsc::UnboundedReceiver<(String, Vec<u8>)>,
    /// Configuration
    config: MqttBridgeConfig,
}

#[cfg(feature = "mqtt-bridge")]
impl MqttBridge {
    /// Create a new MQTT bridge
    pub fn new(config: MqttBridgeConfig) -> Result<Self> {
        let client_id = format!("{}-{}", config.client_id_prefix, uuid::Uuid::new_v4());

        let mut mqttoptions = MqttOptions::new(
            &client_id,
            &config.broker_host,
            config.broker_port,
        );

        mqttoptions.set_keep_alive(Duration::from_secs(config.keep_alive));

        // Configure TLS if enabled
        if config.tls_enabled {
            if let Some(ca_path) = &config.ca_cert_path {
                let ca_cert = std::fs::read(ca_path)
                    .context("Failed to read CA certificate")?;

                let transport = if let (Some(cert_path), Some(key_path)) =
                    (&config.client_cert_path, &config.client_key_path) {
                    // Mutual TLS
                    let client_cert = std::fs::read(cert_path)
                        .context("Failed to read client certificate")?;
                    let client_key = std::fs::read(key_path)
                        .context("Failed to read client key")?;

                    Transport::tls_with_config(
                        rumqttc::TlsConfiguration::Simple {
                            ca: ca_cert,
                            alpn: None,
                            client_auth: Some((client_cert, client_key)),
                        }
                    )
                } else {
                    // Server-only TLS
                    Transport::tls_with_config(
                        rumqttc::TlsConfiguration::Simple {
                            ca: ca_cert,
                            alpn: None,
                            client_auth: None,
                        }
                    )
                };

                mqttoptions.set_transport(transport);
            }
        }

        let (client, event_loop) = AsyncClient::new(mqttoptions, 100);
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Ok(Self {
            client,
            event_loop,
            device_sessions: Arc::new(DashMap::new()),
            message_tx,
            message_rx,
            config,
        })
    }

    /// Start the MQTT bridge
    pub async fn start(mut self) -> Result<()> {
        info!(
            broker = %self.config.broker_host,
            port = self.config.broker_port,
            tls = self.config.tls_enabled,
            "Starting MQTT bridge"
        );

        // Subscribe to device topics
        let topic = format!("{}/#", self.config.topic_prefix);
        self.client
            .subscribe(&topic, QoS::AtLeastOnce)
            .await
            .context("Failed to subscribe to device topics")?;

        info!(topic = %topic, "Subscribed to device messages");

        // Spawn event loop task
        let event_loop = self.event_loop;
        let device_sessions = Arc::clone(&self.device_sessions);
        let message_tx = self.message_tx.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            Self::event_loop_task(event_loop, device_sessions, message_tx, config).await;
        });

        // Process messages from queue
        self.process_messages().await;

        Ok(())
    }

    /// Event loop task (handles MQTT events)
    async fn event_loop_task(
        mut event_loop: EventLoop,
        device_sessions: Arc<DashMap<String, SessionStore>>,
        message_tx: mpsc::UnboundedSender<(String, Vec<u8>)>,
        config: MqttBridgeConfig,
    ) {
        loop {
            match event_loop.poll().await {
                Ok(event) => {
                    if let Err(e) = Self::handle_event(
                        event,
                        &device_sessions,
                        &message_tx,
                        &config,
                    ).await {
                        error!(error = %e, "Failed to handle MQTT event");
                    }
                }
                Err(e) => {
                    error!(error = %e, "MQTT connection error");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Handle MQTT event
    async fn handle_event(
        event: Event,
        device_sessions: &DashMap<String, SessionStore>,
        message_tx: &mpsc::UnboundedSender<(String, Vec<u8>)>,
        config: &MqttBridgeConfig,
    ) -> Result<()> {
        match event {
            Event::Incoming(Packet::Publish(publish)) => {
                debug!(
                    topic = %publish.topic,
                    payload_len = publish.payload.len(),
                    "Received MQTT message"
                );

                // Extract device ID from topic: unhidra/devices/{device_id}/messages
                let parts: Vec<&str> = publish.topic.split('/').collect();
                if parts.len() < 3 {
                    warn!(topic = %publish.topic, "Invalid topic format");
                    return Ok(());
                }

                let device_id = parts[2];

                // Parse message
                let mqtt_msg: MqttMessage = serde_json::from_slice(&publish.payload)
                    .context("Failed to parse MQTT message")?;

                // Decrypt if encrypted
                let payload = if let Some(encrypted) = mqtt_msg.encrypted_payload {
                    if let Some(session) = device_sessions.get(device_id) {
                        let encrypted_msg: EncryptedMessage = serde_json::from_slice(&encrypted)?;
                        session.decrypt(device_id, &encrypted_msg)?
                    } else {
                        warn!(device_id = %device_id, "No E2EE session for device");
                        return Ok(());
                    }
                } else if let Some(plain) = mqtt_msg.plain_payload {
                    plain.into_bytes()
                } else {
                    warn!("Message has no payload");
                    return Ok(());
                };

                // Forward to message queue
                let _ = message_tx.send((device_id.to_string(), payload));

                info!(
                    device_id = %device_id,
                    message_type = %mqtt_msg.message_type,
                    "Processed device message"
                );
            }
            Event::Incoming(Packet::ConnAck(_)) => {
                info!("Connected to MQTT broker");
            }
            Event::Incoming(Packet::SubAck(_)) => {
                debug!("Subscription acknowledged");
            }
            Event::Incoming(Packet::PingResp) => {
                debug!("Ping response received");
            }
            _ => {
                debug!(event = ?event, "MQTT event");
            }
        }

        Ok(())
    }

    /// Process messages from the queue
    async fn process_messages(&mut self) {
        while let Some((device_id, payload)) = self.message_rx.recv().await {
            info!(
                device_id = %device_id,
                payload_len = payload.len(),
                "Forwarding device message to chat system"
            );
            // TODO: Integrate with ConnectionManager or chat-service gRPC
        }
    }

    /// Publish message to device
    pub async fn publish_to_device(
        &self,
        device_id: &str,
        message_type: &str,
        payload: &[u8],
    ) -> Result<()> {
        // Encrypt if E2EE session exists
        let (encrypted_payload, plain_payload) = if let Some(session) = self.device_sessions.get(device_id) {
            let encrypted = session.encrypt(device_id, payload)?;
            let encrypted_bytes = serde_json::to_vec(&encrypted)?;
            (Some(encrypted_bytes), None)
        } else {
            (None, Some(String::from_utf8_lossy(payload).to_string()))
        };

        let mqtt_msg = MqttMessage {
            device_id: device_id.to_string(),
            message_type: message_type.to_string(),
            encrypted_payload,
            plain_payload,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };

        let topic = format!("{}/{}/commands", self.config.topic_prefix, device_id);
        let payload = serde_json::to_vec(&mqtt_msg)?;

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
            .context("Failed to publish to device")?;

        info!(device_id = %device_id, topic = %topic, "Published message to device");

        Ok(())
    }

    /// Register E2EE session for device
    pub fn register_device_session(&self, device_id: String, session: SessionStore) {
        self.device_sessions.insert(device_id, session);
    }
}

#[cfg(all(test, feature = "mqtt-bridge"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mqtt_bridge_creation() {
        let config = MqttBridgeConfig::default();
        let _bridge = MqttBridge::new(config);
    }

    #[test]
    fn test_mqtt_message_serialization() {
        let msg = MqttMessage {
            device_id: "esp32-001".to_string(),
            message_type: "telemetry".to_string(),
            encrypted_payload: None,
            plain_payload: Some("temperature:25.5".to_string()),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: MqttMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.device_id, "esp32-001");
        assert_eq!(parsed.message_type, "telemetry");
    }
}
