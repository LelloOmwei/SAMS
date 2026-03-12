//! SAMS Shield - Data anonymization and PQC protection
//!
//! This module provides semantic obfuscation, anonymization, and post-quantum
//! cryptography features extracted and enhanced from the OMWEI-MVP project.

use crate::{Result, SemanticAtom};
use crate::types::trust;

/// Error types for shield operations
#[derive(Debug, Clone, PartialEq)]
pub enum ShieldError {
    /// Invalid key size
    InvalidKeySize,
    /// Invalid signature
    InvalidSignature,
    /// Cryptographic operation failed
    CryptoError,
    /// Buffer size mismatch
    BufferSizeError,
}

impl core::fmt::Display for ShieldError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ShieldError::InvalidKeySize => write!(f, "Invalid key size"),
            ShieldError::InvalidSignature => write!(f, "Invalid signature"),
            ShieldError::CryptoError => write!(f, "Cryptographic operation failed"),
            ShieldError::BufferSizeError => write!(f, "Buffer size mismatch"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ShieldError {}

/// Anonymization levels for semantic atoms
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnonymizationLevel {
    /// No anonymization - raw data
    None = 0,
    /// Basic anonymization - entity ID masking
    Basic = 1,
    /// Medium anonymization - entity + node ID masking
    Medium = 2,
    /// High anonymization - all identifying information masked
    High = 3,
    /// Maximum anonymization - complete data obfuscation
    Maximum = 4,
}

/// Semantic obfuscation strategies
#[derive(Debug, Clone, Copy)]
pub enum ObfuscationStrategy {
    /// Replace with zeros
    Zeroize,
    /// Add random noise
    Noise,
    /// Use hash-based masking
    HashMask,
    /// Apply differential privacy
    DifferentialPrivacy,
    /// Round to nearest bucket
    Bucketize,
}

/// Shield configuration for atom protection
#[derive(Debug, Clone)]
pub struct ShieldConfig {
    /// Anonymization level
    pub anonymization_level: AnonymizationLevel,
    /// Obfuscation strategy
    pub obfuscation_strategy: ObfuscationStrategy,
    /// Enable PQC signing
    pub enable_pqc: bool,
    /// Privacy budget for differential privacy
    pub privacy_budget: f64,
}

impl Default for ShieldConfig {
    fn default() -> Self {
        Self {
            anonymization_level: AnonymizationLevel::Medium,
            obfuscation_strategy: ObfuscationStrategy::HashMask,
            enable_pqc: true,
            privacy_budget: 1.0,
        }
    }
}

/// SAMS Shield - Main protection interface
pub struct Shield {
    config: ShieldConfig,
    #[cfg(feature = "pqc")]
    hmac_key: [u8; 32],
}

impl Shield {
    /// Create a new shield with default configuration
    pub fn new() -> Self {
        Self::with_config(ShieldConfig::default())
    }

    /// Create a new shield with custom configuration
    pub fn with_config(config: ShieldConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "pqc")]
            hmac_key: [0u8; 32], // Will be initialized in set_key
        }
    }

    /// Set the HMAC key for PQC operations
    #[cfg(feature = "pqc")]
    pub fn set_key(&mut self, key: &[u8]) -> Result<()> {
        if key.len() != 32 {
            return Err("Invalid key size".into());
        }
        self.hmac_key.copy_from_slice(key);
        Ok(())
    }

    /// Anonymize a semantic atom
    pub fn anonymize_atom(&self, atom: &mut SemanticAtom) -> Result<()> {
        match self.config.anonymization_level {
            AnonymizationLevel::None => {
                // No anonymization
            }
            AnonymizationLevel::Basic => {
                self.anonymize_basic(atom)?;
            }
            AnonymizationLevel::Medium => {
                self.anonymize_basic(atom)?;
                self.anonymize_medium(atom)?;
            }
            AnonymizationLevel::High => {
                self.anonymize_basic(atom)?;
                self.anonymize_medium(atom)?;
                self.anonymize_high(atom)?;
            }
            AnonymizationLevel::Maximum => {
                self.anonymize_maximum(atom)?;
            }
        }
        Ok(())
    }

    /// Apply semantic obfuscation to atom values
    pub fn obfuscate_values(&self, atom: &mut SemanticAtom) -> Result<()> {
        match self.config.obfuscation_strategy {
            ObfuscationStrategy::Zeroize => {
                atom.value_fixed = 0;
            }
            ObfuscationStrategy::Noise => {
                self.add_noise(atom)?;
            }
            ObfuscationStrategy::HashMask => {
                self.hash_mask_value(atom)?;
            }
            ObfuscationStrategy::DifferentialPrivacy => {
                self.apply_differential_privacy(atom)?;
            }
            ObfuscationStrategy::Bucketize => {
                self.bucketize_value(atom)?;
            }
        }
        Ok(())
    }

    /// Sign an atom with PQC (HMAC-SHA256 for now, will be ML-DSA)
    #[cfg(feature = "pqc")]
    pub fn sign_atom(&self, atom: &SemanticAtom) -> Result<[u8; 32]> {
        use sha2::Sha256;
        use hmac::{Hmac, Mac};

        let mut mac = Hmac::<Sha256>::new_from_slice(&self.hmac_key)
            .map_err(|_| "Crypto error")?;
        mac.update(atom.as_bytes());
        let result = mac.finalize();
        let bytes = result.into_bytes();
        
        let mut signature = [0u8; 32];
        signature.copy_from_slice(&bytes);
        Ok(signature)
    }

    /// Verify an atom's PQC signature
    #[cfg(feature = "pqc")]
    pub fn verify_atom(&self, atom: &SemanticAtom, signature: &[u8]) -> Result<bool> {
        if signature.len() != 32 {
            return Err("Invalid signature".into());
        }

        let expected_signature = self.sign_atom(atom)?;
        Ok(expected_signature.as_slice() == signature)
    }

    /// Create a protected atom packet (atom + signature)
    #[cfg(feature = "pqc")]
    pub fn create_protected_packet(&self, atom: &SemanticAtom) -> Result<[u8; crate::ATOM_SIZE + 32]> {
        let signature = self.sign_atom(atom)?;
        let mut packet = [0u8; crate::ATOM_SIZE + 32];
        
        // Copy atom data
        packet[..crate::ATOM_SIZE].copy_from_slice(atom.as_bytes());
        // Copy signature
        packet[crate::ATOM_SIZE..].copy_from_slice(&signature);
        
        Ok(packet)
    }

    /// Extract and verify atom from protected packet
    #[cfg(feature = "pqc")]
    pub fn extract_protected_packet(&self, packet: &[u8]) -> Result<(SemanticAtom, bool)> {
        if packet.len() != crate::ATOM_SIZE + 32 {
            return Err("Buffer size error".into());
        }

        let atom_data = &packet[..crate::ATOM_SIZE];
        let signature = &packet[crate::ATOM_SIZE..];

        let atom = SemanticAtom::from_bytes(atom_data)?;
        let is_valid = self.verify_atom(&atom, signature)?;

        Ok((atom.clone(), is_valid))
    }

    // Private anonymization methods

    fn anonymize_basic(&self, atom: &mut SemanticAtom) -> Result<()> {
        // Mask entity ID with hash
        atom.entity_id = self.hash_u32(atom.entity_id);
        Ok(())
    }

    fn anonymize_medium(&self, atom: &mut SemanticAtom) -> Result<()> {
        // Mask node ID
        atom.node_id = self.hash_u16(atom.node_id);
        Ok(())
    }

    fn anonymize_high(&self, atom: &mut SemanticAtom) -> Result<()> {
        // Mask sequence number
        atom.sequence = self.hash_u32(atom.sequence);
        // Coarsen timestamp to nearest hour
        const HOUR_US: u64 = 3_600_000_000; // 1 hour in microseconds
        atom.timestamp_us = (atom.timestamp_us / HOUR_US) * HOUR_US;
        Ok(())
    }

    fn anonymize_maximum(&self, atom: &mut SemanticAtom) -> Result<()> {
        // Complete obfuscation - keep only telemetry type and basic structure
        atom.entity_id = 0;
        atom.sequence = 0;
        atom.value_fixed = 0;
        atom.status_flags = 0;
        atom.timestamp_us = 0;
        atom.node_id = 0;
        atom.trust_pqc = trust::RAW as u32;
        Ok(())
    }

    fn add_noise(&self, atom: &mut SemanticAtom) -> Result<()> {
        // Add controlled noise to value
        let noise = (self.hash_u32(atom.value_fixed) % 100) as i32 - 50; // ±50 units
        let new_value = (atom.value_fixed as i32 + noise).max(0) as u32;
        atom.value_fixed = new_value;
        Ok(())
    }

    fn hash_mask_value(&self, atom: &mut SemanticAtom) -> Result<()> {
        // Hash-based value masking
        atom.value_fixed = self.hash_u32(atom.value_fixed);
        Ok(())
    }

    fn apply_differential_privacy(&self, atom: &mut SemanticAtom) -> Result<()> {
        // Simple differential privacy - Laplace noise
        // In production, use proper DP library
        let scale = 1.0 / self.config.privacy_budget;
        let noise = self.laplace_noise(scale);
        let noisy_value = atom.get_value() + noise;
        atom.value_fixed = (noisy_value * crate::FIXED_POINT_PRECISION as f64) as u32;
        Ok(())
    }

    fn bucketize_value(&self, atom: &mut SemanticAtom) -> Result<()> {
        // Round to nearest bucket of 10 units
        const BUCKET_SIZE: u32 = 1000; // 10.00 in hundredths
        atom.value_fixed = (atom.value_fixed / BUCKET_SIZE) * BUCKET_SIZE;
        Ok(())
    }

    // Utility methods

    fn hash_u32(&self, value: u32) -> u32 {
        // Simple hash function - in production use proper cryptographic hash
        let mut hash = value.wrapping_mul(0x9e3779b9);
        hash ^= hash >> 16;
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= hash >> 13;
        hash = hash.wrapping_mul(0xc2b2ae35);
        hash
    }

    fn hash_u16(&self, value: u16) -> u16 {
        let hash = self.hash_u32(value as u32);
        (hash >> 16) as u16
    }

    fn laplace_noise(&self, scale: f64) -> f64 {
        // Simple Laplace noise generator
        // In production, use proper statistical library
        let uniform = (self.hash_u32(scale as u32) % 1000) as f64 / 1000.0;
        let sign = if uniform < 0.5 { -1.0 } else { 1.0 };
        sign * scale * self.log_approximation(-uniform + 1.0)
    }

    /// Simple natural logarithm approximation for no_std
    fn log_approximation(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        // Simple approximation: ln(x) ≈ 2 * (x - 1) / (x + 1) for x > 0
        // This is a rough approximation, good enough for noise generation
        let y = (x - 1.0) / (x + 1.0);
        2.0 * y
    }
}

impl Default for Shield {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::{AtomBuilder, types::telemetry};

    #[test]
    fn test_basic_anonymization() {
        let shield = Shield::new();
        let mut atom = AtomBuilder::new()
            .entity_id(12345)
            .node_id(678)
            .value(42.5)
            .build();

        let original_entity_id = atom.entity_id;
        shield.anonymize_atom(&mut atom).unwrap();

        assert_ne!(atom.entity_id, original_entity_id);
        assert_eq!(atom.node_id, 678); // Should not be changed at basic level
        assert_eq!(atom.get_value(), 42.5); // Value should not change
    }

    #[test]
    fn test_medium_anonymization() {
        let config = ShieldConfig {
            anonymization_level: AnonymizationLevel::Medium,
            ..Default::default()
        };
        let shield = Shield::with_config(config);
        
        let mut atom = AtomBuilder::new()
            .entity_id(12345)
            .node_id(678)
            .value(42.5)
            .build();

        let original_entity_id = atom.entity_id;
        let original_node_id = atom.node_id;
        
        shield.anonymize_atom(&mut atom).unwrap();

        assert_ne!(atom.entity_id, original_entity_id);
        assert_ne!(atom.node_id, original_node_id);
        assert_eq!(atom.get_value(), 42.5); // Value should not change
    }

    #[test]
    fn test_obfuscation_zeroize() {
        let config = ShieldConfig {
            obfuscation_strategy: ObfuscationStrategy::Zeroize,
            ..Default::default()
        };
        let shield = Shield::with_config(config);
        
        let mut atom = AtomBuilder::new()
            .value(42.5)
            .build();

        shield.obfuscate_values(&mut atom).unwrap();
        assert_eq!(atom.get_value(), 0.0);
    }

    #[test]
    fn test_obfuscation_bucketize() {
        let config = ShieldConfig {
            obfuscation_strategy: ObfuscationStrategy::Bucketize,
            ..Default::default()
        };
        let shield = Shield::with_config(config);
        
        let mut atom = AtomBuilder::new()
            .value(47.8) // Should be rounded to 40.0
            .build();

        shield.obfuscate_values(&mut atom).unwrap();
        assert_eq!(atom.get_value(), 40.0);
    }

    #[cfg(feature = "pqc")]
    #[test]
    fn test_pqc_signing() {
        let mut shield = Shield::new();
        shield.set_key(b"test_key_32_bytes_long______").unwrap();

        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.5)
            .build();

        let signature = shield.sign_atom(&atom).unwrap();
        assert_eq!(signature.len(), 32);

        let is_valid = shield.verify_atom(&atom, &signature).unwrap();
        assert!(is_valid);
    }

    #[cfg(feature = "pqc")]
    #[test]
    fn test_protected_packet() {
        let mut shield = Shield::new();
        shield.set_key(b"test_key_32_bytes_long______").unwrap();

        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.5)
            .build();

        let packet = shield.create_protected_packet(&atom).unwrap();
        assert_eq!(packet.len(), crate::ATOM_SIZE + 32);

        let (extracted_atom, is_valid) = shield.extract_protected_packet(&packet).unwrap();
        assert!(is_valid);
        assert_eq!(extracted_atom.entity_id(), 123);
        assert_eq!(extracted_atom.get_value(), 42.5);
    }

    #[test]
    fn test_maximum_anonymization() {
        let config = ShieldConfig {
            anonymization_level: AnonymizationLevel::Maximum,
            ..Default::default()
        };
        let shield = Shield::with_config(config);
        
        let mut atom = AtomBuilder::new()
            .entity_id(12345)
            .node_id(678)
            .sequence(999)
            .value(42.5)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .build();

        shield.anonymize_atom(&mut atom).unwrap();

        assert_eq!(atom.entity_id, 0);
        assert_eq!(atom.node_id, 0);
        assert_eq!(atom.sequence, 0);
        assert_eq!(atom.get_value(), 0.0);
        assert_eq!(atom.telemetry_type(), telemetry::TEMPERATURE_C); // Telemetry type preserved
    }
}
