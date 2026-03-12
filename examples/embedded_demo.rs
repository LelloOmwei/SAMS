//! Embedded demo for SAMS library
//!
//! This example shows how to use SAMS on a bare-metal microcontroller
//! (Cortex-M85/M4 compatible) with no_std support.

#![no_std]
#![no_main]

use sams::{AtomBuilder, shield::{Shield, ShieldConfig, AnonymizationLevel}};

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

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize SAMS components
    let _shield = Shield::new();
    
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
        let _ = protected_shield.anonymize_atom(&mut co2_atom);
        let _ = protected_shield.anonymize_atom(&mut temp_atom);
        
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
        // NOP instruction for delay
        core::hint::spin_loop();
    }
}
