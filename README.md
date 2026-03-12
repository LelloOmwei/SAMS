# SAMS - Semantic Atom Management Stack

[![Crates.io](https://img.shields.io/crates/v/sams.svg)](https://crates.io/crates/sams)
[![Documentation](https://docs.rs/sams/badge.svg)](https://docs.rs/sams)
[![License: Apache-2.0 OR MIT](https://img.shields.io/badge/License-Apache--2.0%20OR%20MIT-yellow.svg)](https://opensource.org/licenses/Apache-2.0)
[![Build Status](https://github.com/LelloOmwei/SAMS/workflows/Rust%20CI/badge.svg)](https://github.com/LelloOmwei/SAMS/actions)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

A memory-safe, 32-byte Semantic Atom Management Stack for industrial IPC. Provides zero-copy serialization, PQC security, and unified APIs across embedded and Linux systems.

## 🏗️ Architecture Overview

SAMS enables seamless communication between heterogeneous industrial systems, from bare-metal microcontrollers to enterprise servers:

```text
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   i.MX6 Linux    │    │   Cortex-M85     │    │   Cortex-M4      │
│   (Enterprise)   │◄──►│   (Edge AI)      │◄──►│   (Sensors)      │
│                 │    │                 │    │                 │
│ • Data Analytics│    │ • AI Processing │    │ • Sensor Nodes  │
│ • Storage       │    │ • Edge Computing│    │ • Real-time I/O │
│ • Networking    │    │ • Protocol Bridge│    │ • Low Power     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SAMS Library (Unified API)                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │
│  │   Types     │ │   Shield    │ │   Codec     │ │  Transport  │  │
│  │  (32-byte  │ │ (PQC/Anon)  │ │ (Zero-copy) │ │ (Zenoh)     │  │
│  │   Atoms)    │ │             │ │             │ │             │  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### System Interactions

**i.MX6 Linux (Enterprise Layer)**
- High-performance data processing and analytics
- Long-term storage and historical data management
- Network gateway to cloud services and enterprise systems
- Complex business logic and machine learning pipelines

**Cortex-M85 (Edge AI Layer)**
- Real-time AI inference using Arm Ethos-U NPU
- Protocol bridging between different communication standards
- Edge analytics and local decision making
- Data aggregation and preprocessing

**Cortex-M4 (Sensor Layer)**
- Direct sensor interfacing and data acquisition
- Low-power operation for battery-powered devices
- Real-time control loops and safety-critical functions
- Hardware-level data validation and filtering

## 🧬 Technical Specification: 32-Byte Semantic Atom

The core data structure is a deterministic 32-byte packet designed for high-performance industrial communication:

```text
┌─────────────────────────────────────────────────────────────┐
│                    Semantic Atom (32 bytes)                 │
├─────────────────────────────────────────────────────────────┤
│ Bytes 0-3   │ entity_id     │ Entity identifier              │
│ Bytes 4-7   │ sequence      │ Sequence number                │
│ Bytes 8-11  │ value_fixed   │ Fixed-point value (hundredths)  │
│ Bytes 12-15 │ status_flags  │ Status and metadata            │
│ Bytes 16-23 │ timestamp_us  │ Microsecond timestamp          │
│ Bytes 24-25 │ node_id       │ Node identifier                │
│ Bytes 26-27 │ telemetry_type│ Telemetry type                 │
│ Bytes 28-31 │ trust_pqc     │ Trust (8-bit) + PQC (24-bit)   │
└─────────────────────────────────────────────────────────────┘
```

### Field Specifications

| Field | Bytes | Type | Range | Description |
|-------|--------|------|-------|-------------|
| `entity_id` | 0-3 | u32 | 0-4,294,967,295 | Unique entity identifier |
| `sequence` | 4-7 | u32 | 0-4,294,967,295 | Monotonic sequence number |
| `value_fixed` | 8-11 | u32 | 0-4,294,967,295 | Fixed-point value (×100) |
| `status_flags` | 12-15 | u32 | Various | Status and control flags |
| `timestamp_us` | 16-23 | u64 | Unix timestamps | Microsecond precision |
| `node_id` | 24-25 | u16 | 0-65,535 | Source node identifier |
| `telemetry_type` | 26-27 | u16 | Standardized | Telemetry data type |
| `trust_pqc` | 28-31 | u32 | Composite | Trust level + PQC anchor |

### Memory Layout Guarantees

- **Deterministic Layout**: `#[repr(C)]` ensures identical layout across all platforms
- **Cache Alignment**: 32-byte size optimizes for modern processor caches
- **FFI Compatibility**: Safe to pass across language boundaries (C, C++, Python)
- **Zero-Copy Serialization**: Direct byte access without memory allocation

## 🚀 Getting Started

### For Embedded Systems (no_std)

Add to your `Cargo.toml`:

```toml
[dependencies]
sams = { version = "0.1.0", default-features = false, features = ["embedded"] }
```

Basic usage:

```rust
#![no_std]
use sams::{AtomBuilder, Shield, AnonymizationLevel};

// Create and protect an atom
let mut atom = AtomBuilder::new()
    .entity_id(1)
    .value(450.0) // CO2 in ppm
    .telemetry_type(sams::telemetry::CO2_PPM)
    .build();

// Apply anonymization
let shield = Shield::new();
shield.anonymize_atom(&mut atom)?;

// Transmit atom (zero-copy)
transmit_atom(atom.as_bytes());
```

### For Linux Systems (std)

Add to your `Cargo.toml`:

```toml
[dependencies]
sams = { version = "0.1.0", features = ["std", "pqc", "transport"] }
tokio = { version = "1.0", features = ["full"] }
```

Basic usage:

```rust
use sams::{ZenohTransport, AtomBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize transport
    let transport = ZenohTransport::new().await?;

    // Create and publish atom
    let atom = AtomBuilder::new()
        .entity_id(123)
        .value(22.5) // Temperature in Celsius
        .telemetry_type(sams::telemetry::TEMPERATURE_C)
        .build();

    transport.publish_atom("sensors/temperature", &atom).await?;
    
    // Subscribe to atoms
    let mut subscriber = transport.subscribe_atoms("sensors/*").await?;
    while let Some((atom, topic)) = subscriber.recv().await {
        println!("Received: {} = {:.2}°C", topic, atom.get_value());
    }
    
    Ok(())
}
```

## ⚡ Performance Characteristics

| Operation | Latency | Throughput | Memory Usage |
|-----------|---------|------------|--------------|
| Serialization | < 100ns | N/A | Zero-copy |
| Deserialization | < 100ns | N/A | Zero-copy |
| PQC Signing | ~1ms | 1,000 ops/s | 32B key |
| Zenoh Publish | ~1ms | 10K ops/s | Network dependent |
| Validation | < 50ns | 20M ops/s | In-place |

### Benchmarks

- **Zero-Copy Operations**: Sub-100ns serialization without allocation
- **Deterministic Processing**: Fixed latency regardless of data volume
- **Memory Efficiency**: < 10KB footprint for embedded deployment
- **Power Optimization**: < 100mW typical consumption on Cortex-M4

## 🛡️ Security Features

### Post-Quantum Cryptography (PQC)

```rust
use sams::Shield;

let mut shield = Shield::new();
shield.set_key(b"quantum_resistant_key_32_bytes")?;

// Sign atom with quantum-resistant cryptography
let packet = shield.create_protected_packet(&atom)?;

// Verify signature on receiver
let (received_atom, is_valid) = shield.extract_protected_packet(&packet)?;
assert!(is_valid);
```

### Data Anonymization

Five-level anonymization system for privacy compliance:

| Level | Protection | Use Case |
|-------|-------------|----------|
| None | No protection | Internal systems |
| Basic | Entity masking | Partner sharing |
| Medium | Entity + Node | Public datasets |
| High | Full obfuscation | Research |
| Maximum | Complete anonymization | Open data |

## 📊 Standard Telemetry Types

| Type | Hex | Unit | Range | Description |
|------|-----|------|-------|-------------|
| Temperature | 0x0002 | °C | -50 to 100 | Environmental monitoring |
| Humidity | 0x0003 | % | 0 to 100 | Climate control |
| Pressure | 0x0004 | Pa | 80000-120000 | Weather systems |
| CO2 | 0x0006 | ppm | 0-10000 | Air quality |
| Water Level | 0x0001 | mm | 0-10000 | Flood monitoring |
| Voltage | 0x0008 | V | 0-1000 | Power systems |
| Current | 0x0009 | A | 0-100 | Energy monitoring |

## 🏭 Industrial Use Cases

### Manufacturing
- **Real-time Production Monitoring**: Track sensor data from assembly lines
- **Quality Control**: Detect anomalies in manufacturing processes
- **Predictive Maintenance**: Monitor equipment health and schedule maintenance

### Energy Management
- **Smart Grid Monitoring**: Real-time power grid status and load balancing
- **Renewable Energy**: Solar and wind farm monitoring and optimization
- **Building Automation**: HVAC and lighting control systems

### Transportation
- **Vehicle-to-Vehicle Communication**: Real-time traffic and safety data
- **Fleet Management**: Track and optimize vehicle fleets
- **Infrastructure Monitoring**: Bridge and tunnel structural health

### Agriculture
- **Precision Farming**: Soil moisture, nutrient levels, and crop health
- **Livestock Monitoring**: Animal health and environmental conditions
- **Irrigation Control**: Automated watering systems based on sensor data

## 🔧 Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `std` | Standard library support | ✅ |
| `embedded` | no_std support for bare-metal | ❌ |
| `pqc` | Post-quantum cryptography | ❌ |
| `transport` | Zenoh transport integration | ❌ |

### Usage Examples

```toml
# Minimal embedded footprint
sams = { version = "0.1.0", default-features = false, features = ["embedded"] }

# Full-featured Linux deployment
sams = { version = "0.1.0", features = ["std", "pqc", "transport"] }

# Embedded with PQC security
sams = { version = "0.1.0", default-features = false, features = ["embedded", "pqc"] }
```

## 🧪 Testing and Validation

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run embedded tests
cargo test --target thumbv7m-none-eabi --no-default-features --features embedded

# Run benchmarks
cargo bench --all-features
```

### Cross-Compilation

```bash
# Add embedded targets
rustup target add thumbv7m-none-eabi
rustup target add thumbv8m.main-none-eabihf

# Verify embedded builds
cargo check --target thumbv7m-none-eabi --no-default-features --features embedded,pqc
```

## 📚 Documentation

- **[API Documentation](https://docs.rs/sams)**: Complete API reference
- **[Examples](https://github.com/LelloOmwei/SAMS/tree/main/examples)**: Usage examples
- **[Technical Whitepaper](https://github.com/LelloOmwei/SAMS/docs/whitepaper.pdf)**: Deep technical analysis
- **[Integration Guide](https://github.com/LelloOmwei/SAMS/docs/integration.md)**: System integration

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/LelloOmwei/SAMS.git
cd SAMS

# Install Rust toolchain
rustup toolchain install stable
rustup target add thumbv7m-none-eabi

# Run tests
cargo test --all-features

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

## 📄 License

This project is dual-licensed under either:

- **[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)**
- **[MIT License](https://opensource.org/licenses/MIT)**

at your option.

## 🏆 Acknowledgments

- **Sovereign Grid Network**: Original concept and architecture
- **Rust Embedded Working Group**: no_std ecosystem and best practices
- **Zenoh Project**: High-performance data distribution protocol
- **Arm Limited**: Cortex-M processor architecture and Ethos-U NPU

## 📞 Support

- **[Issues](https://github.com/LelloOmwei/SAMS/issues)**: Bug reports and feature requests
- **[Discussions](https://github.com/LelloOmwei/SAMS/discussions)**: Community discussions
- **[Discord](https://discord.gg/sams)**: Real-time chat (coming soon)

---

**SAMS** - Building the future of industrial semantic data infrastructure.
