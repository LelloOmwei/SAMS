#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! # SAMS - Semantic Atom Management Stack
//!
//! A memory-safe, 32-byte Semantic Atom Management Stack for industrial IPC.
//! Provides zero-copy serialization, PQC security, and unified APIs across embedded and Linux systems.
//!
//! ## Overview
//!
//! SAMS is designed for high-performance industrial environments where deterministic processing
//! and memory safety are critical. The library enables seamless communication between
//! heterogeneous systems, from bare-metal microcontrollers to enterprise servers.
//!
//! ## Architecture
//!
//! The SAMS ecosystem consists of three main processing layers:
//!
//! ```text, ignore
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │   i.MX6 Linux    │    │   Cortex-M85     │    │   Cortex-M4      │
//! │   (Enterprise)   │◄──►│   (Edge AI)      │◄──►│   (Sensors)      │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!         │                       │                       │
//!         │                       │                       │
//!         ▼                       ▼                       ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    SAMS Library (Unified API)                     │
//! │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │
//! │  │   Types     │ │   Shield    │ │   Codec     │ │  Transport  │  │
//! │  │  (32-byte  │ │ (PQC/Anon)  │ │ (Zero-copy) │ │ (Zenoh)     │  │
//! │  │   Atoms)    │ │             │ │             │ │             │  │
//! │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Features
//!
//! - **🔧 no_std Compatible**: Works on bare-metal microcontrollers (Cortex-M85/M4)
//! - **📦 32-Byte Atoms**: Unified atom structure for all platforms with guaranteed memory layout
//! - **⚡ Zero-Copy Operations**: Sub-100ns serialization with no memory allocation
//! - **🛡️ PQC Ready**: Post-quantum cryptography support (HMAC-SHA256 → ML-DSA)
//! - **🌐 Transport Agnostic**: Zenoh-pico for embedded, full Zenoh for Linux
//! - **🔒 Data Protection**: 5-level anonymization with semantic obfuscation
//!
//! ## Quick Start
//!
//! ### Embedded (no_std)
//!
//! ```rust,ignore, ignore
//! #![no_std]
//! use sams::{AtomBuilder, Shield, AnonymizationLevel, transmit_atom};
//!
//! // Create and protect an atom
//! let mut atom = AtomBuilder::new()
//!     .entity_id(1)
//!     .value(450.0) // CO2 in ppm
//!     .telemetry_type(0x0006)
//!     .build();
//!
//! let mut shield = Shield::new();
//! shield.anonymize_atom(&mut atom)?;
//!
//! // Transmit atom (zero-copy)
//! transmit_atom(atom.as_bytes());
//! ```
//!
//! ### Linux (std)
//!
//! ```rust,ignore, ignore
//! use sams::{AtomBuilder, telemetry};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize transport
//!     let transport: /* ZenohTransport */ = ZenohTransport::new().await?;
//!
//!     // Create and publish atom
//!     let atom = AtomBuilder::new()
//!         .entity_id(123)
//!         .telemetry_type(telemetry::TEMPERATURE_C)
//!         .value(25.3)
//!         .build_now();
//!
//!     transport.publish_atom("sensors/temperature", &atom).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## 32-Byte Semantic Atom
//!
//! The core data structure is a deterministic 32-byte packet designed for high-performance
//! industrial communication:
//!
//! ```text, ignore
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
//! ## Feature Flags
//!
//! - **embedded**: no_std support for bare-metal systems
//! - **std**: Full standard library support (Linux/macOS/Windows)
//! - **pqc**: Post-quantum cryptography features
//! - **transport**: Zenoh transport integration
//!
//! ```toml, ignore
//! # For embedded systems
//! sams = { version = "0.1.0", default-features = false, features = ["embedded"] }
//!
//! # For Linux with full features
//! sams = { version = "0.1.0", features = ["std", "pqc", "transport"] }
//! ```
//!
//! ## Performance
//!
//! | Operation | Latency | Throughput | Memory |
//! |-----------|---------|------------|---------|
//! | Serialization | < 100ns | N/A | Zero-copy |
//! | PQC Signing | ~1ms | 1000 ops/s | 32B key |
//! | Zenoh Publish | ~1ms | 10K ops/s | Network dependent |
//! ```ignore
//!
//! ## Safety and Security
//!
//! - **Memory Safety**: Rust's ownership model prevents buffer overflows and data races
//! - **Type Safety**: Strong typing prevents data corruption at compile time
//! - **PQC Security**: Quantum-resistant cryptography for long-term security
//! - **Data Protection**: Built-in anonymization for privacy compliance
//!
//! ## Industrial Use Cases
//!
//! - **Manufacturing**: Real-time sensor data from production lines
//! - **Energy**: Smart grid monitoring and control systems
//! - **Transportation**: Vehicle-to-vehicle communication systems
//! - **Agriculture**: Precision farming sensor networks
//! - **Infrastructure**: Building automation and smart cities
//!
//! ## Examples
//!
//! See the [examples](https://github.com/LelloOmwei/SAMS/tree/main/examples) directory for:
//! - [embedded_demo.rs](https://github.com/LelloOmwei/SAMS/tree/main/examples/embedded_demo.rs) - Cortex-M usage
//! - [linux_cli.rs](https://github.com/LelloOmwei/SAMS/tree/main/examples/linux_cli.rs) - Linux CLI application
//!
//! ## License
//!
//! Licensed under either [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT) at your option.

pub mod types;
pub mod shield;
pub mod codec;

#[cfg(feature = "transport")]
pub mod transport;

pub mod utils;

// Re-export core types for convenience
pub use types::{SemanticAtom, AtomBuilder, EntityId, TelemetryType, TrustLevel};

/// Result type for SAMS operations
#[cfg(feature = "std")]
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Result type for SAMS operations (no_std)
#[cfg(not(feature = "std"))]
pub type Result<T> = core::result::Result<T, &'static str>;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Atom size in bytes (fixed for all platforms)
pub const ATOM_SIZE: usize = 32;

/// Fixed-point precision (hundredths)
pub const FIXED_POINT_PRECISION: u32 = 100;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_atom_size() {
        assert_eq!(ATOM_SIZE, 32);
        assert_eq!(core::mem::size_of::<SemanticAtom>(), 32);
    }

    #[test]
    fn test_basic_creation() {
        let atom = AtomBuilder::new()
            .entity_id(123)
            .value_fixed(5000)
            .build();

        assert_eq!(atom.entity_id(), 123);
        assert_eq!(atom.value_fixed(), 5000);
        assert_eq!(atom.get_value(), 50.0);
    }
}
