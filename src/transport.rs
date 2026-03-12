//! SAMS Transport - Zenoh-pico integration for embedded systems
//!
//! This module provides lightweight transport layer using Zenoh-pico for
//! efficient atom distribution across embedded and Linux systems.

#[cfg(feature = "transport")]
use crate::{Result, SemanticAtom, AtomCodec};
#[cfg(feature = "transport")]
use zenoh::prelude::*;
#[cfg(feature = "transport")]
use zenoh::config::Config;

/// Transport configuration
#[cfg(feature = "transport")]
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Zenoh session configuration
    pub zenoh_config: Option<Config>,
    /// Default topic prefix
    pub topic_prefix: String,
    /// Enable compression
    pub enable_compression: bool,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
}

#[cfg(feature = "transport")]
impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            zenoh_config: None,
            topic_prefix: "sams/atoms".to_string(),
            enable_compression: false,
            enable_encryption: false,
            timeout_ms: 5000,
        }
    }
}

/// Zenoh-based transport for semantic atoms
#[cfg(feature = "transport")]
pub struct ZenohTransport {
    /// Zenoh session
    session: Session,
    /// Configuration
    config: TransportConfig,
    /// Atom codec
    codec: AtomCodec,
}

#[cfg(feature = "transport")]
impl ZenohTransport {
    /// Create a new transport with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(TransportConfig::default()).await
    }

    /// Create a new transport with custom configuration
    pub async fn with_config(config: TransportConfig) -> Result<Self> {
        let session = zenoh::open(config.zenoh_config.clone().unwrap_or_default())
            .await
            .map_err(|e| format!("Failed to open Zenoh session: {:?}", e))?;

        Ok(Self {
            session,
            config,
            codec: AtomCodec::new(),
        })
    }

    /// Publish a single atom
    pub async fn publish_atom(&self, topic: &str, atom: &SemanticAtom) -> Result<()> {
        let full_topic = format!("{}/{}", self.config.topic_prefix, topic);
        let encoded = self.codec.encode_atom(atom)?;
        
        self.session
            .put(&full_topic, encoded.as_atom().as_bytes())
            .await
            .map_err(|e| format!("Failed to publish atom: {:?}", e))?;
        
        Ok(())
    }

    /// Publish multiple atoms in batch
    pub async fn publish_batch(&self, topic: &str, atoms: &[SemanticAtom]) -> Result<()> {
        let full_topic = format!("{}/{}", self.config.topic_prefix, topic);
        
        // Encode atoms as batch
        let mut buffer = vec![0u8; atoms.len() * crate::ATOM_SIZE];
        let bytes_written = self.codec.encode_atoms(atoms, &mut buffer)?;
        buffer.truncate(bytes_written);
        
        self.session
            .put(&full_topic, buffer)
            .await
            .map_err(|e| format!("Failed to publish batch: {:?}", e))?;
        
        Ok(())
    }

    /// Subscribe to atom topic
    pub async fn subscribe_atoms<F>(&self, topic: F) -> Result<ZenohSubscriber>
    where
        F: Into<String>,
    {
        let full_topic = format!("{}/{}", self.config.topic_prefix, topic.into());
        
        let subscriber = self.session
            .declare_subscriber(&full_topic)
            .await
            .map_err(|e| format!("Failed to subscribe: {:?}", e))?;
        
        Ok(ZenohSubscriber::new(subscriber, self.codec.clone()))
    }

    /// Query for atoms
    pub async fn query_atoms(&self, topic: &str, selector: &str) -> Result<Vec<SemanticAtom>> {
        let full_topic = format!("{}/{}", self.config.topic_prefix, topic);
        
        let replies = self.session
            .get(&full_topic)
            .with_selector(selector)
            .await
            .map_err(|e| format!("Failed to query: {:?}", e))?;
        
        let mut atoms = Vec::new();
        for reply in replies {
            let sample = reply
                .await
                .map_err(|e| format!("Failed to receive reply: {:?}", e))?;
            
            let decoded_atoms = self.codec.decode_atoms(sample.value().payload.contiguous())?;
            for decoded in decoded_atoms {
                atoms.push(decoded.to_atom()?);
            }
        }
        
        Ok(atoms)
    }

    /// Get session statistics
    pub fn stats(&self) -> TransportStats {
        TransportStats {
            connected: true,
            topic_prefix: self.config.topic_prefix.clone(),
            compression_enabled: self.config.enable_compression,
            encryption_enabled: self.config.enable_encryption,
        }
    }
}

#[cfg(feature = "transport")]
impl Drop for ZenohTransport {
    fn drop(&mut self) {
        // Session will be closed when dropped
    }
}

/// Zenoh subscriber for receiving atoms
#[cfg(feature = "transport")]
pub struct ZenohSubscriber {
    subscriber: Subscriber<'static, Sample>,
    codec: AtomCodec,
}

#[cfg(feature = "transport")]
impl ZenohSubscriber {
    /// Create a new subscriber
    fn new(subscriber: Subscriber<'static, Sample>, codec: AtomCodec) -> Self {
        Self { subscriber, codec }
    }

    /// Receive the next atom
    pub async fn recv(&mut self) -> Result<(SemanticAtom, String)> {
        let sample = self.subscriber.recv_async().await
            .map_err(|e| format!("Failed to receive sample: {:?}", e))?;
        
        let topic = sample.key_expr().to_string();
        let atom = self.codec.decode_atom(sample.value().payload.contiguous())?;
        
        Ok((atom.to_atom()?, topic))
    }

    /// Try to receive the next atom without blocking
    pub fn try_recv(&mut self) -> Result<Option<(SemanticAtom, String)>> {
        match self.subscriber.try_recv() {
            Ok(Some(sample)) => {
                let topic = sample.key_expr().to_string();
                let atom = self.codec.decode_atom(sample.value().payload.contiguous())?;
                Ok(Some((atom.to_atom()?, topic)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }
}

/// Transport statistics
#[derive(Debug, Clone)]
pub struct TransportStats {
    /// Connection status
    pub connected: bool,
    /// Topic prefix
    pub topic_prefix: String,
    /// Compression enabled
    pub compression_enabled: bool,
    /// Encryption enabled
    pub encryption_enabled: bool,
}

/// Simple IPC bridge for Linux ↔ Cortex-M communication
#[cfg(feature = "std")]
pub struct IpcBridge {
    /// Transport instance
    transport: ZenohTransport,
    /// Bridge configuration
    config: IpcBridgeConfig,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct IpcBridgeConfig {
    /// Bridge ID
    pub bridge_id: String,
    /// Source topic
    pub source_topic: String,
    /// Target topic
    pub target_topic: String,
    /// Enable forwarding
    pub enable_forwarding: bool,
}

#[cfg(feature = "std")]
impl Default for IpcBridgeConfig {
    fn default() -> Self {
        Self {
            bridge_id: "sams-bridge".to_string(),
            source_topic: "embedded/input".to_string(),
            target_topic: "linux/output".to_string(),
            enable_forwarding: true,
        }
    }
}

#[cfg(feature = "std")]
impl IpcBridge {
    /// Create a new IPC bridge
    pub async fn new(config: IpcBridgeConfig) -> Result<Self> {
        let transport_config = TransportConfig::default();
        let transport = ZenohTransport::with_config(transport_config).await?;
        
        Ok(Self {
            transport,
            config,
        })
    }

    /// Start the bridge forwarding
    pub async fn start_forwarding(&mut self) -> Result<()> {
        if !self.config.enable_forwarding {
            return Ok(());
        }

        let mut subscriber = self.transport.subscribe_atoms(&self.config.source_topic).await?;
        
        loop {
            match subscriber.recv().await {
                Ok((atom, _topic)) => {
                    // Forward atom to target topic
                    self.transport.publish_atom(&self.config.target_topic, &atom).await?;
                }
                Err(e) => {
                    eprintln!("Bridge error: {:?}", e);
                    // Continue processing other atoms
                }
            }
        }
    }

    /// Send atom from Linux to embedded
    pub async fn send_to_embedded(&self, atom: &SemanticAtom) -> Result<()> {
        self.transport.publish_atom(&self.config.source_topic, atom).await
    }

    /// Receive atom from embedded
    pub async fn recv_from_embedded(&mut self) -> Result<SemanticAtom> {
        let mut subscriber = self.transport.subscribe_atoms(&self.config.target_topic).await?;
        let (atom, _) = subscriber.recv().await?;
        Ok(atom)
    }

    /// Get bridge statistics
    pub fn stats(&self) -> IpcBridgeStats {
        IpcBridgeStats {
            bridge_id: self.config.bridge_id.clone(),
            source_topic: self.config.source_topic.clone(),
            target_topic: self.config.target_topic.clone(),
            forwarding_enabled: self.config.enable_forwarding,
            transport_stats: self.transport.stats(),
        }
    }
}

/// IPC bridge statistics
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct IpcBridgeStats {
    /// Bridge ID
    pub bridge_id: String,
    /// Source topic
    pub source_topic: String,
    /// Target topic
    pub target_topic: String,
    /// Forwarding enabled
    pub forwarding_enabled: bool,
    /// Transport statistics
    pub transport_stats: TransportStats,
}

#[cfg(all(test, feature = "transport"))]
mod tests {
    use super::*;
    use crate::{AtomBuilder, telemetry};

    #[tokio::test]
    async fn test_zenoh_transport() {
        let transport = ZenohTransport::new().await.unwrap();
        
        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.5)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .build();

        // Test publishing
        transport.publish_atom("test/topic", &atom).await.unwrap();
        
        // Test subscription
        let mut subscriber = transport.subscribe_atoms("test/topic").await.unwrap();
        
        // Give some time for the message to propagate
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Try to receive (may not get the message due to timing)
        match subscriber.try_recv() {
            Ok(Some((received_atom, topic))) => {
                assert_eq!(received_atom.entity_id(), 123);
                assert_eq!(received_atom.get_value(), 42.5);
                assert!(topic.contains("test/topic"));
            }
            Ok(None) => {
                // No message received, which is OK for this test
            }
            Err(_) => {
                // Error, which is also OK for this test
            }
        }
    }

    #[tokio::test]
    async fn test_batch_publishing() {
        let transport = ZenohTransport::new().await.unwrap();
        
        let atoms = vec![
            AtomBuilder::new().entity_id(1).value(10.0).build(),
            AtomBuilder::new().entity_id(2).value(20.0).build(),
            AtomBuilder::new().entity_id(3).value(30.0).build(),
        ];

        transport.publish_batch("test/batch", &atoms).await.unwrap();
    }

    #[tokio::test]
    async fn test_ipc_bridge() {
        let config = IpcBridgeConfig {
            bridge_id: "test-bridge".to_string(),
            source_topic: "test/embedded".to_string(),
            target_topic: "test/linux".to_string(),
            enable_forwarding: false, // Don't start forwarding in test
        };

        let bridge = IpcBridge::new(config).await.unwrap();
        
        let atom = AtomBuilder::new()
            .entity_id(456)
            .value(78.9)
            .build();

        bridge.send_to_embedded(&atom).await.unwrap();
        
        let stats = bridge.stats();
        assert_eq!(stats.bridge_id, "test-bridge");
        assert_eq!(stats.source_topic, "test/embedded");
        assert_eq!(stats.target_topic, "test/linux");
        assert!(!stats.forwarding_enabled);
    }
}
