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
/// ```text, ignore
/// ┌─────────────────────────────────────────────────────────────┐
/// │                    Semantic Atom (32 bytes)                 │
/// ├─────────────────────────────────────────────────────────────┤
/// │ Bytes 0-3   │ entity_id     │ Entity identifier              │
/// │ Bytes 4-7   │ sequence      │ Sequence number                │
/// │ Bytes 8-11  │ value_fixed   │ Fixed-point value (hundredths)  │
/// │ Bytes 12-15 │ status_flags  │ Status and metadata            │
/// │ Bytes 16-23 │ timestamp_us  │ Microsecond timestamp          │
/// │ Bytes 24-25 │ node_id       │ Node identifier                │
/// │ Bytes 26-27 │ telemetry_type│ Telemetry type                 │
/// │ Bytes 28-31 │ trust_pqc     │ Trust (8-bit) + PQC (24-bit)   │
/// └─────────────────────────────────────────────────────────────┘
/// ```

use serde::{Serialize, Deserialize};

/// Entity identifier type (32 bits)
pub type EntityId = u32;

/// Telemetry type identifier (16 bits)
pub type TelemetryType = u16;

/// Trust level identifier (8 bits)
pub type TrustLevel = u8;

/// Predicate identifier (32 bits)
pub type PredicateId = u32;

/// Fixed-point value type (32 bits, hundredths precision)
pub type FixedPointValue = u32;

/// Core Semantic Atom structure (32 bytes)
///
/// This is the fundamental data structure for the SAMS framework, providing
/// a deterministic, memory-safe representation of industrial telemetry data.
///
/// ## Memory Layout (32 bytes total)
///
/// ```text, ignore
/// Bytes 0-3:   entity_id     - Unique entity identifier
/// Bytes 4-7:   sequence      - Sequence number
/// Bytes 8-11:  value_fixed   - Fixed-point value (hundredths)
/// Bytes 12-15: status_flags  - Status flags and metadata
/// Bytes 16-23: timestamp_us  - Microsecond timestamp
/// Bytes 24-25: node_id       - Node identifier
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
    pub value_fixed: FixedPointValue,
    
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
            trust_pqc: 0,
        }
    }
}

impl SemanticAtom {
    /// Create a new SemanticAtom with basic values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get the atom as raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const SemanticAtom as *const u8,
                std::mem::size_of::<SemanticAtom>(),
            )
        }
    }
    
    /// Get the fixed-point value as f64
    pub fn get_value(&self) -> f64 {
        self.value_fixed as f64 / 100.0
    }
    
    /// Set value from f64 (converted to fixed-point)
    pub fn set_value(&mut self, value: f64) {
        self.value_fixed = (value * 100.0) as u32;
    }
}

/// Telemetry type constants
pub mod telemetry {
    use super::TelemetryType;
    
    /// Temperature in Celsius
    pub const TEMPERATURE_C: TelemetryType = 0x0002;
    /// Humidity in percent
    pub const HUMIDITY: TelemetryType = 0x0003;
    /// Pressure in hPa
    pub const PRESSURE: TelemetryType = 0x0004;
    /// CO2 in ppm
    pub const CO2_PPM: TelemetryType = 0x0006;
    /// Water level in millimeters
    pub const WATER_LEVEL_MM: TelemetryType = 0x0007;
    /// Flow rate in liters per minute
    pub const FLOW_LPM: TelemetryType = 0x0008;
    /// Power in watts
    pub const POWER_W: TelemetryType = 0x0009;
    /// Voltage in volts
    pub const VOLTAGE_V: TelemetryType = 0x000A;
    /// Current in amperes
    pub const CURRENT_A: TelemetryType = 0x000B;
    /// Energy in watt-hours
    pub const ENERGY_WH: TelemetryType = 0x000C;
    /// Frequency in Hz
    pub const FREQUENCY_HZ: TelemetryType = 0x000D;
    /// Position (generic)
    pub const POSITION: TelemetryType = 0x000E;
    /// Velocity (generic)
    pub const VELOCITY: TelemetryType = 0x000F;
    /// Acceleration (generic)
    pub const ACCELERATION: TelemetryType = 0x0010;
    /// Angular position
    pub const ANGLE: TelemetryType = 0x0011;
    /// Angular velocity
    pub const ANGULAR_VELOCITY: TelemetryType = 0x0012;
    /// Angular acceleration
    pub const ANGULAR_ACCELERATION: TelemetryType = 0x0013;
    /// Distance in meters
    pub const DISTANCE: TelemetryType = 0x0014;
    /// Speed in meters per second
    pub const SPEED: TelemetryType = 0x0015;
    /// Mass in kilograms
    pub const MASS: TelemetryType = 0x0016;
    /// Force in newtons
    pub const FORCE: TelemetryType = 0x0017;
    /// Torque in newton-meters
    pub const TORQUE: TelemetryType = 0x0018;
    /// Energy in joules
    pub const ENERGY: TelemetryType = 0x0019;
    /// Power density
    pub const POWER_DENSITY: TelemetryType = 0x001A;
    /// Generic count
    pub const COUNT: TelemetryType = 0x001B;
    /// Binary state (0/1)
    pub const BINARY: TelemetryType = 0x001C;
    /// Analog signal (generic)
    pub const ANALOG: TelemetryType = 0x001D;
    /// Digital signal (generic)
    pub const DIGITAL: TelemetryType = 0x001E;
    /// String data
    pub const STRING: TelemetryType = 0x001F;
    /// Custom telemetry type
    pub const CUSTOM: TelemetryType = 0xFF00;
}

/// Trust level constants
pub mod trust {
    use super::TrustLevel;
    
    /// Untrusted data
    pub const UNTRUSTED: TrustLevel = 0x00;
    /// Low trust
    pub const LOW: TrustLevel = 0x01;
    /// Medium trust
    pub const MEDIUM: TrustLevel = 0x02;
    /// High trust
    pub const HIGH: TrustLevel = 0x03;
    /// Critical trust
    pub const CRITICAL: TrustLevel = 0x04;
    /// Maximum trust
    pub const MAXIMUM: TrustLevel = 0x05;
}

/// Predicate type constants
pub mod predicate {
    use super::PredicateId;
    
    /// No predicate
    pub const NONE: PredicateId = 0x00000000;
    /// Value range predicate
    pub const RANGE: PredicateId = 0x00000001;
    /// Threshold predicate
    pub const THRESHOLD: PredicateId = 0x00000002;
    /// Change detection predicate
    pub const CHANGE: PredicateId = 0x00000003;
    /// Anomaly detection predicate
    pub const ANOMALY: PredicateId = 0x00000004;
    /// Trend analysis predicate
    pub const TREND: PredicateId = 0x00000005;
    /// Correlation predicate
    pub const CORRELATION: PredicateId = 0x00000006;
    /// Statistical predicate
    pub const STATISTICAL: PredicateId = 0x00000007;
    /// Temporal predicate
    pub const TEMPORAL: PredicateId = 0x00000008;
    /// Spatial predicate
    pub const SPATIAL: PredicateId = 0x00000009;
    /// Custom predicate
    pub const CUSTOM: PredicateId = 0xFFFFFFFF;
}

/// Status flag constants
pub mod status {
    /// Valid data flag
    pub const VALID: u32 = 0x00000001;
    /// Error flag
    pub const ERROR: u32 = 0x00000002;
    /// Warning flag
    pub const WARNING: u32 = 0x00000004;
    /// Alert flag
    pub const ALERT: u32 = 0x00000008;
    /// Diagnostic flag
    pub const DIAGNOSTIC: u32 = 0x00000010;
    /// Maintenance flag
    pub const MAINTENANCE: u32 = 0x00000020;
    /// Calibration flag
    pub const CALIBRATION: u32 = 0x00000040;
    /// Test flag
    pub const TEST: u32 = 0x00000080;
    /// Simulation flag
    pub const SIMULATION: u32 = 0x00000100;
    /// Manual override flag
    pub const MANUAL_OVERRIDE: u32 = 0x00000200;
    /// Communication flag
    pub const COMMUNICATION: u32 = 0x00000400;
    /// Power flag
    pub const POWER: u32 = 0x00000800;
    /// Security flag
    pub const SECURITY: u32 = 0x00001000;
    /// Performance flag
    pub const PERFORMANCE: u32 = 0x00002000;
    /// Quality flag
    pub const QUALITY: u32 = 0x00004000;
    /// Compliance flag
    pub const COMPLIANCE: u32 = 0x00008000;
}
