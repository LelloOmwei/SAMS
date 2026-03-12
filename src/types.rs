//! SAMS Types - Core data structures and constants
//!
//! This module defines the fundamental 32-byte SemanticAtom structure and associated
//! constants for telemetry types, trust levels, and entity identifiers.
//!
//! ## 32-Byte Semantic Atom Structure
//!
//! The core data structure is a deterministic 32-byte packet designed for high-performance
//! industrial communication:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Semantic Atom (32 bytes)                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Bytes 0-3   │ entity_id     │ Entity identifier              │
//! │ Bytes 4-7   │ sequence      │ Sequence number                │
//! │ Bytes 8-11  │ value_fixed   │ Fixed-point value (hundredths)  │
//! │ Bytes 12-15 │ status_flags  │ Status and metadata            │
//! │ Bytes 16-23 │ timestamp_us  │ Microsecond timestamp          │
//! │ Bytes 24-25 │ node_id       │ Node identifier                │
//! │ Bytes 26-27 │ telemetry_type│ Telemetry type                 │
//! │ Bytes 28-31 │ trust_pqc     │ Trust (8-bit) + PQC (24-bit)   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Memory Layout Guarantees
//!
//! The SemanticAtom uses `#[repr(C)]` to ensure:
//! - **Deterministic layout**: Same memory layout across all platforms
//! - **FFI compatibility**: Safe to pass across language boundaries
//! - **Zero-copy serialization**: Direct byte access without copying
//! - **Cache alignment**: Optimized for modern processor caches
//!
//! ## Fixed-Point Arithmetic
//!
//! Values are stored as fixed-point numbers with hundredths precision:
//! - **Storage**: u32 representing value × 100
//! - **Range**: 0.00 to 42,949,672.95
//! - **Precision**: ±0.01 units
//! - **Performance**: No floating-point operations required
//!
//! ## Telemetry Types
//!
//! Standardized telemetry type identifiers ensure interoperability:
//!
//! | Type | Hex | Unit | Range | Example |
//! |------|-----|------|-------|---------|
//! | Temperature | 0x0002 | °C | -50 to 100 | 22.5°C |
//! | Humidity | 0x0003 | % | 0 to 100 | 45.0% |
//! | Pressure | 0x0004 | Pa | 80000-120000 | 101325Pa |
//! | CO2 | 0x0006 | ppm | 0-10000 | 450ppm |
//!
//! ## Trust Levels
//!
//! Trust levels indicate data verification status:
//! - **RAW (0x00)**: Unverified sensor data
//! - **VERIFIED (0x01)**: Basic validation performed
//! - **ANOMALY (0x02)**: Flagged as anomalous
//! - **ENTERPRISE (0xFF)**: Enterprise-grade verified data
//!
//! ## Usage Examples
//!
//! ```rust
//! use sams::{AtomBuilder, telemetry};
//!
//! // Create a temperature atom
//! let atom = AtomBuilder::new()
//!     .entity_id(0x00010002) // Temperature sensor
//!     .telemetry_type(telemetry::TEMPERATURE_C)
//!     .value(22.5) // 22.5°C
//!     .build();
//!
//! // Access atom fields
//! assert_eq!(atom.entity_id(), 0x00010002);
//! assert_eq!(atom.telemetry_type(), telemetry::TEMPERATURE_C);
//! assert_eq!(atom.get_value(), 22.5);
//! ```

use crate::{ATOM_SIZE, FIXED_POINT_PRECISION};
use serde::{Deserialize, Serialize};

/// Entity identifier type
pub type EntityId = u32;

/// Telemetry type identifier
pub type TelemetryType = u16;

/// Trust level (0-255)
pub type TrustLevel = u8;

/// Trust level constants
pub mod trust {
    use super::TrustLevel;
    
    /// Raw/untrusted data
    pub const RAW: TrustLevel = 0x00;
    /// Verified data
    pub const VERIFIED: TrustLevel = 0x01;
    /// Anomalous data
    pub const ANOMALY: TrustLevel = 0x02;
    /// Enterprise-grade data
    pub const ENTERPRISE: TrustLevel = 0xFF;
}

/// Telemetry type constants
pub mod telemetry {
    use super::TelemetryType;
    
    /// Water level in millimeters
    pub const WATER_LEVEL_MM: TelemetryType = 0x0001;
    /// Temperature in Celsius
    pub const TEMPERATURE_C: TelemetryType = 0x0002;
    /// Humidity percentage
    pub const HUMIDITY_PERCENT: TelemetryType = 0x0003;
    /// Pressure in Pascals
    pub const PRESSURE_PA: TelemetryType = 0x0004;
    /// Precipitation in millimeters
    pub const PRECIPITATION_MM: TelemetryType = 0x0005;
    /// CO2 concentration in ppm
    pub const CO2_PPM: TelemetryType = 0x0006;
}

/// Predicate constants (semantic validation results)
pub mod predicate {
    /// Normal operation
    pub const NORMAL: u32 = 0x00000001;
    /// Warning condition
    pub const WARNING: u32 = 0x00000002;
    /// Critical condition
    pub const CRITICAL: u32 = 0x00000003;
    /// Data consistency check
    pub const CONSISTENCY: u32 = 0x00000004;
    /// Triangulated data
    pub const TRIANGULATED: u32 = 0x00000005;
    /// Anomaly detected
    pub const ANOMALY: u32 = 0x00000006;
    /// Stable trend
    pub const TREND_STABLE: u32 = 0x00000007;
    /// Rapidly rising trend
    pub const TREND_RISING_FAST: u32 = 0x00000008;
    /// Sensor stuck warning
    pub const SENSOR_STUCK: u32 = 0x00000009;
}

/// The unified 32-byte Semantic Atom structure
///
/// This structure combines the best features from both SGN and OMWEI approaches:
/// - Fixed memory layout for FFI compatibility
/// - Zero-copy serialization support
/// - Embedded-friendly design
/// - Rich metadata for enterprise use
///
/// ## Memory Layout (32 bytes total)
///
/// ```text
/// Bytes 0-3:   entity_id     - Unique entity identifier
/// Bytes 4-7:   sequence      - Sequence number (from OMWEI)
/// Bytes 8-11:  value_fixed   - Fixed-point value (hundredths)
/// Bytes 12-15: status_flags  - Status flags and metadata
/// Bytes 16-23: timestamp_us  - Microsecond timestamp
/// Bytes 24-25: node_id       - Node identifier (from OMWEI)
/// Bytes 26-27: telemetry_type - Telemetry type
/// Bytes 28-31: trust_pqc     - Trust (8-bit) + PQC anchor (24-bit)
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticAtom {
    /// Entity identifier (32 bits)
    pub entity_id: EntityId,
    
    /// Sequence number for ordering (32 bits)
    pub sequence: u32,
    
    /// Fixed-point measurement value (32 bits, hundredths of unit)
    pub value_fixed: u32,
    
    /// Status flags and metadata (32 bits)
    pub status_flags: u32,
    
    /// Timestamp in microseconds since Unix epoch (64 bits)
    pub timestamp_us: u64,
    
    /// Node identifier (16 bits)
    pub node_id: u16,
    
    /// Telemetry type (16 bits)
    pub telemetry_type: TelemetryType,
    
    /// Trust level (8 bits) + PQC anchor (24 bits)
    pub trust_pqc: u32,
}

impl Default for SemanticAtom {
    fn default() -> Self {
        Self {
            entity_id: 0,
            sequence: 0,
            value_fixed: 0,
            status_flags: 0,
            timestamp_us: 0,
            node_id: 0,
            telemetry_type: 0,
            trust_pqc: trust::RAW as u32,
        }
    }
}

impl SemanticAtom {
    /// Create a new semantic atom with basic fields
    pub fn new(
        entity_id: EntityId,
        sequence: u32,
        value_fixed: u32,
        timestamp_us: u64,
        node_id: u16,
        telemetry_type: TelemetryType,
        trust_level: TrustLevel,
    ) -> Self {
        Self {
            entity_id,
            sequence,
            value_fixed,
            status_flags: predicate::NORMAL,
            timestamp_us,
            node_id,
            telemetry_type,
            trust_pqc: (trust_level as u32) & 0xFF,
        }
    }

    /// Get the entity ID
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Get the sequence number
    pub fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Get the fixed-point value
    pub fn value_fixed(&self) -> u32 {
        self.value_fixed
    }

    /// Get the value as f64 (converts from fixed-point)
    pub fn get_value(&self) -> f64 {
        self.value_fixed as f64 / FIXED_POINT_PRECISION as f64
    }

    /// Get the timestamp in microseconds
    pub fn timestamp_us(&self) -> u64 {
        self.timestamp_us
    }

    /// Get the node ID
    pub fn node_id(&self) -> u16 {
        self.node_id
    }

    /// Get the telemetry type
    pub fn telemetry_type(&self) -> TelemetryType {
        self.telemetry_type
    }

    /// Get the trust level (extracted from trust_pqc)
    pub fn trust_level(&self) -> TrustLevel {
        (self.trust_pqc & 0xFF) as TrustLevel
    }

    /// Get the PQC anchor (extracted from trust_pqc)
    pub fn pqc_anchor(&self) -> u32 {
        self.trust_pqc >> 8
    }

    /// Get the predicate ID (extracted from status_flags)
    pub fn predicate_id(&self) -> u32 {
        self.status_flags
    }

    /// Set the predicate ID
    pub fn set_predicate(&mut self, predicate: u32) {
        self.status_flags = predicate;
    }

    /// Set the PQC anchor
    pub fn set_pqc_anchor(&mut self, anchor: u32) {
        self.trust_pqc = ((anchor & 0x00FFFFFF) << 8) | (self.trust_level() as u32);
    }

    /// Get a slice to the atom's bytes (zero-copy)
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const SemanticAtom as *const u8,
                ATOM_SIZE,
            )
        }
    }

    /// Create an atom from bytes (zero-copy)
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The slice is exactly 32 bytes long
    /// - The slice contains valid atom data
    /// - The slice's memory is properly aligned
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        debug_assert_eq!(bytes.len(), ATOM_SIZE);
        &*(bytes.as_ptr() as *const SemanticAtom)
    }

    /// Create an atom from bytes with validation
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, &'static str> {
        if bytes.len() != ATOM_SIZE {
            return Err("Invalid atom size");
        }
        
        // Check alignment
        if (bytes.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err("Misaligned atom data");
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Convert atom to byte array
    pub fn to_bytes(&self) -> [u8; ATOM_SIZE] {
        let mut bytes = [0u8; ATOM_SIZE];
        bytes.copy_from_slice(self.as_bytes());
        bytes
    }

    /// Increment the sequence number
    pub fn increment_sequence(&mut self) {
        self.sequence = self.sequence.wrapping_add(1);
    }

    /// Update the timestamp to current time (if std is available)
    #[cfg(feature = "std")]
    pub fn update_timestamp_now(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            self.timestamp_us = duration.as_micros() as u64;
        }
    }

    /// Check if the atom has a valid timestamp
    pub fn is_valid_timestamp(&self) -> bool {
        // Simple validation: timestamp should be after year 2000
        const YEAR_2000_US: u64 = 946684800_000_000; // Jan 1, 2000 in microseconds
        self.timestamp_us > YEAR_2000_US
    }

    /// Check if the atom is trusted
    pub fn is_trusted(&self) -> bool {
        self.trust_level() > trust::RAW
    }
}

/// Builder for creating SemanticAtoms with a fluent interface
pub struct AtomBuilder {
    entity_id: EntityId,
    sequence: u32,
    value_fixed: u32,
    timestamp_us: u64,
    node_id: u16,
    telemetry_type: TelemetryType,
    trust_level: TrustLevel,
    predicate: u32,
    pqc_anchor: u32,
}

impl Default for AtomBuilder {
    fn default() -> Self {
        Self {
            entity_id: 0,
            sequence: 0,
            value_fixed: 0,
            timestamp_us: 0,
            node_id: 0,
            telemetry_type: 0,
            trust_level: trust::RAW,
            predicate: predicate::NORMAL,
            pqc_anchor: 0,
        }
    }
}

impl AtomBuilder {
    /// Create a new atom builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the entity ID
    pub fn entity_id(mut self, entity_id: EntityId) -> Self {
        self.entity_id = entity_id;
        self
    }

    /// Set the sequence number
    pub fn sequence(mut self, sequence: u32) -> Self {
        self.sequence = sequence;
        self
    }

    /// Set the fixed-point value
    pub fn value_fixed(mut self, value_fixed: u32) -> Self {
        self.value_fixed = value_fixed;
        self
    }

    /// Set the value from f64 (converts to fixed-point)
    pub fn value(mut self, value: f64) -> Self {
        self.value_fixed = (value * FIXED_POINT_PRECISION as f64) as u32;
        self
    }

    /// Set the timestamp
    pub fn timestamp_us(mut self, timestamp_us: u64) -> Self {
        self.timestamp_us = timestamp_us;
        self
    }

    /// Set the node ID
    pub fn node_id(mut self, node_id: u16) -> Self {
        self.node_id = node_id;
        self
    }

    /// Set the telemetry type
    pub fn telemetry_type(mut self, telemetry_type: TelemetryType) -> Self {
        self.telemetry_type = telemetry_type;
        self
    }

    /// Set the trust level
    pub fn trust_level(mut self, trust_level: TrustLevel) -> Self {
        self.trust_level = trust_level;
        self
    }

    /// Set the predicate
    pub fn predicate(mut self, predicate: u32) -> Self {
        self.predicate = predicate;
        self
    }

    /// Set the PQC anchor
    pub fn pqc_anchor(mut self, pqc_anchor: u32) -> Self {
        self.pqc_anchor = pqc_anchor;
        self
    }

    /// Build the atom
    pub fn build(self) -> SemanticAtom {
        let mut atom = SemanticAtom::new(
            self.entity_id,
            self.sequence,
            self.value_fixed,
            self.timestamp_us,
            self.node_id,
            self.telemetry_type,
            self.trust_level,
        );
        atom.set_predicate(self.predicate);
        atom.set_pqc_anchor(self.pqc_anchor);
        atom
    }

    /// Build the atom with current timestamp (if std is available)
    #[cfg(feature = "std")]
    pub fn build_now(self) -> SemanticAtom {
        let mut atom = self.build();
        atom.update_timestamp_now();
        atom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_structure() {
        assert_eq!(core::mem::size_of::<SemanticAtom>(), 32);
        assert_eq!(core::mem::align_of::<SemanticAtom>(), 4);
    }

    #[test]
    fn test_atom_builder() {
        let atom = AtomBuilder::new()
            .entity_id(123)
            .sequence(456)
            .value(42.5)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .trust_level(trust::VERIFIED)
            .build();

        assert_eq!(atom.entity_id(), 123);
        assert_eq!(atom.sequence(), 456);
        assert_eq!(atom.get_value(), 42.5);
        assert_eq!(atom.telemetry_type(), telemetry::TEMPERATURE_C);
        assert_eq!(atom.trust_level(), trust::VERIFIED);
    }

    #[test]
    fn test_fixed_point_conversion() {
        let atom = AtomBuilder::new()
            .value(123.45)
            .build();

        assert_eq!(atom.value_fixed(), 12345);
        assert_eq!(atom.get_value(), 123.45);
    }

    #[test]
    fn test_serialization() {
        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.0)
            .build();

        let bytes = atom.to_bytes();
        assert_eq!(bytes.len(), 32);

        let parsed = SemanticAtom::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.entity_id(), 123);
        assert_eq!(parsed.get_value(), 42.0);
    }

    #[test]
    fn test_pqc_anchor_extraction() {
        let atom = AtomBuilder::new()
            .pqc_anchor(0x123456)
            .trust_level(0xAB)
            .build();

        assert_eq!(atom.pqc_anchor(), 0x123456);
        assert_eq!(atom.trust_level(), 0xAB);
    }
}
