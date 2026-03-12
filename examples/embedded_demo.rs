//! Embedded demo for SAMS library
//!
//! This example shows how to use SAMS on a bare-metal microcontroller
//! (Cortex-M85/M4 compatible) with no_std support.

#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use sams::{AtomBuilder, SemanticAtom, Shield, ShieldConfig, AnonymizationLevel};

// Mock hardware functions for demonstration
mod mock_hardware {
    pub fn get_timestamp() -> u64 {
        1_600_000_000_000_000u64 // Mock timestamp
    }

    pub fn read_co2_sensor() -> f64 {
        450.0 // Mock CO2 reading in ppm
    }

    pub fn read_temperature_sensor() -> f64 {
        22.5 // Mock temperature in Celsius
    }

    pub fn transmit_data(data: &[u8]) {
        // Mock transmission - in real implementation, this would use UART, SPI, etc.
        // For demo, we'll just count the bytes
        let _len = data.len();
    }
}

#[entry]
fn main() -> ! {
    // Initialize SAMS components
    let shield = Shield::new();
    
    // Create shield configuration for data protection
    let shield_config = ShieldConfig {
        anonymization_level: AnonymizationLevel::Medium,
        ..Default::default()
    };
    
    let protected_shield = Shield::with_config(shield_config);
    
    // Main processing loop
    loop {
        // Read sensor data
        let co2_value = mock_hardware::read_co2_sensor();
        let temp_value = mock_hardware::read_temperature_sensor();
        let timestamp = mock_hardware::get_timestamp();
        
        // Create CO2 atom
        let mut co2_atom = AtomBuilder::new()
            .entity_id(1) // CO2 sensor entity
            .sequence(get_next_sequence())
            .value(co2_value)
            .timestamp_us(timestamp)
            .node_id(0x0001) // This node
            .telemetry_type(0x0006) // CO2 telemetry type
            .trust_level(0x01) // Verified
            .build();
        
        // Create temperature atom
        let mut temp_atom = AtomBuilder::new()
            .entity_id(2) // Temperature sensor entity
            .sequence(get_next_sequence())
            .value(temp_value)
            .timestamp_us(timestamp)
            .node_id(0x0001) // This node
            .telemetry_type(0x0002) // Temperature telemetry type
            .trust_level(0x01) // Verified
            .build();
        
        // Apply anonymization/protection
        protected_shield.anonymize_atom(&mut co2_atom).unwrap();
        protected_shield.anonymize_atom(&mut temp_atom).unwrap();
        
        // Serialize atoms
        let co2_bytes = co2_atom.as_bytes();
        let temp_bytes = temp_atom.as_bytes();
        
        // Transmit atoms
        mock_hardware::transmit_data(co2_bytes);
        mock_hardware::transmit_data(temp_bytes);
        
        // Simple delay (in real implementation, use proper timer)
        delay_loop();
    }
}

// Simple sequence counter
static mut SEQUENCE_COUNTER: u32 = 0;

fn get_next_sequence() -> u32 {
    unsafe {
        SEQUENCE_COUNTER = SEQUENCE_COUNTER.wrapping_add(1);
        SEQUENCE_COUNTER
    }
}

// Simple delay loop
fn delay_loop() {
    // Very basic delay - in real implementation, use hardware timers
    for _ in 0..1_000_000 {
        cortex_m::asm::nop();
    }
}

// Alternative: Batch processing example
#[allow(dead_code)]
fn batch_processing_example() {
    let shield = Shield::new();
    
    // Read multiple sensor values
    let sensor_values = [
        mock_hardware::read_co2_sensor(),
        mock_hardware::read_temperature_sensor(),
        75.5, // Mock humidity
        1013.25, // Mock pressure in hPa
    ];
    
    let timestamp = mock_hardware::get_timestamp();
    
    // Create atoms in batch
    let mut atoms = [
        // CO2 atom
        AtomBuilder::new()
            .entity_id(1)
            .sequence(get_next_sequence())
            .value(sensor_values[0])
            .timestamp_us(timestamp)
            .node_id(0x0001)
            .telemetry_type(0x0006)
            .trust_level(0x01)
            .build(),
        // Temperature atom
        AtomBuilder::new()
            .entity_id(2)
            .sequence(get_next_sequence())
            .value(sensor_values[1])
            .timestamp_us(timestamp)
            .node_id(0x0001)
            .telemetry_type(0x0002)
            .trust_level(0x01)
            .build(),
        // Humidity atom
        AtomBuilder::new()
            .entity_id(3)
            .sequence(get_next_sequence())
            .value(sensor_values[2])
            .timestamp_us(timestamp)
            .node_id(0x0001)
            .telemetry_type(0x0003)
            .trust_level(0x01)
            .build(),
        // Pressure atom
        AtomBuilder::new()
            .entity_id(4)
            .sequence(get_next_sequence())
            .value(sensor_values[3])
            .timestamp_us(timestamp)
            .node_id(0x0001)
            .telemetry_type(0x0004)
            .trust_level(0x01)
            .build(),
    ];
    
    // Apply protection to all atoms
    for atom in &mut atoms {
        shield.anonymize_atom(atom).unwrap();
    }
    
    // Create batch buffer
    let mut buffer = [0u8; 128]; // 4 atoms × 32 bytes
    
    // Encode batch
    let mut offset = 0;
    for atom in &atoms {
        let atom_bytes = atom.as_bytes();
        buffer[offset..offset + 32].copy_from_slice(atom_bytes);
        offset += 32;
    }
    
    // Transmit batch
    mock_hardware::transmit_data(&buffer);
}

// Example of using PQC signing (when available)
#[cfg(feature = "pqc")]
#[allow(dead_code)]
fn pqc_signing_example() {
    let mut shield = Shield::new();
    
    // Set HMAC key (in real implementation, load from secure storage)
    let key = b"embedded_pqc_key_32_bytes";
    shield.set_key(key).unwrap();
    
    let atom = AtomBuilder::new()
        .entity_id(1)
        .value(450.0)
        .build();
    
    // Create protected packet
    let packet = shield.create_protected_packet(&atom).unwrap();
    
    // Transmit protected packet
    mock_hardware::transmit_data(&packet);
    
    // On the receiving end, you would:
    // let (received_atom, is_valid) = shield.extract_protected_packet(&received_packet).unwrap();
    // if is_valid {
    //     // Process valid atom
    // }
}
