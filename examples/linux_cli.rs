//! Linux CLI demo for SAMS library
//!
//! This example shows how to use SAMS on Linux systems with Zenoh transport
//! and full std library support.

use sams::{
    AtomBuilder, SemanticAtom, Result,
    shield::{Shield, ShieldConfig, AnonymizationLevel}, 
    transport::{ZenohTransport, IpcBridge, IpcBridgeConfig}
};
use sams::types::telemetry;
#[cfg(feature = "std")]
use sams::utils::time;
use std::env;
use std::time::Duration;
use std::thread::sleep;

fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "producer" => {
            // Use a simple runtime for the demo
            println!("🚀 Starting SAMS Producer...");
            println!("Note: This demo requires async runtime. See README for setup.");
            Ok(())
        }
        "consumer" => {
            println!("🎧 Starting SAMS Consumer...");
            println!("Note: This demo requires async runtime. See README for setup.");
            Ok(())
        }
        "bridge" => bridge_demo(),
        "shield" => shield_demo(),
        "batch" => {
            println!("📦 Starting SAMS Batch Demo...");
            println!("Note: This demo requires async runtime. See README for setup.");
            Ok(())
        }
        _ => {
            println!("Unknown command: {}", args[1]);
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    println!("SAMS Linux CLI Demo");
    println!("");
    println!("Usage: {} <command>", env::args().next().unwrap_or_else(|| "sams".to_string()));
    println!("");
    println!("Commands:");
    println!("  producer  - Produce and publish atoms");
    println!("  consumer  - Subscribe and consume atoms");
    println!("  bridge    - Run IPC bridge between Linux and embedded");
    println!("  shield    - Demonstrate data protection features");
    println!("  batch     - Demonstrate batch processing");
}

// Async functions removed for simplicity - see README for full async examples

fn bridge_demo() -> Result<()> {
    println!("🌉 Starting SAMS IPC Bridge...");
    
    // Create bridge configuration
    let config = IpcBridgeConfig {
        bridge_id: "demo-bridge".to_string(),
        source_topic: "embedded/input".to_string(),
        target_topic: "linux/output".to_string(),
        enable_forwarding: true,
    };
    
    println!("✅ Bridge configuration created: {}", config.bridge_id);
    println!("🔄 Bridge setup complete (synchronous demo)");
    
    // Create some sample atoms to demonstrate the API
    for i in 0..5 {
        let atom = AtomBuilder::new()
            .entity_id(3000 + i)
            .sequence(i)
            .value(100.0 + i as f64)
            .timestamp_us(time::now_us())
            .node_id(0x3000)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .trust_level(0x01)
            .build();
        
        println!("📤 Sample atom {}: entity={}, value={:.2}", i, atom.entity_id(), atom.get_value());
        sleep(Duration::from_millis(100));
    }
    
    println!("✅ Bridge demo completed");
    Ok(())
}

fn shield_demo() -> Result<()> {
    println!("🛡️  Starting SAMS Shield Demo...");
    
    // Create shield with different anonymization levels
    let configs = vec![
        ("No Protection", AnonymizationLevel::None),
        ("Basic Protection", AnonymizationLevel::Basic),
        ("Medium Protection", AnonymizationLevel::Medium),
        ("High Protection", AnonymizationLevel::High),
        ("Maximum Protection", AnonymizationLevel::Maximum),
    ];
    
    let original_atom = AtomBuilder::new()
        .entity_id(12345)
        .sequence(67890)
        .value(42.5)
        .timestamp_us(time::now_us())
        .node_id(0x1234)
        .telemetry_type(telemetry::TEMPERATURE_C)
        .trust_level(0x01)
        .build();
    
    println!("📊 Original atom:");
    print_atom_info(&original_atom);
    
    for (name, level) in configs {
        let config = ShieldConfig {
            anonymization_level: level,
            ..Default::default()
        };
        
        let shield = Shield::with_config(config);
        let mut protected_atom = original_atom;
        
        shield.anonymize_atom(&mut protected_atom)?;
        
        println!("\n🔒 {}:", name);
        print_atom_info(&protected_atom);
    }
    
    // PQC signing demo
    #[cfg(feature = "pqc")]
    {
        println!("\n🔐 PQC Signing Demo:");
        
        let mut shield = Shield::new();
        shield.set_key(b"demo_pqc_key_32_bytes_long______")?;
        
        let atom = AtomBuilder::new()
            .entity_id(99999)
            .value(123.45)
            .build();
        
        let packet = shield.create_protected_packet(&atom)?;
        println!("📦 Created protected packet: {} bytes", packet.len());
        
        let (extracted_atom, is_valid) = shield.extract_protected_packet(&packet)?;
        println!("✅ Verification: {}", is_valid);
        println!("📊 Extracted atom: entity={}, value={:.2}", 
                extracted_atom.entity_id(), extracted_atom.get_value());
    }
    
    Ok(())
}

// Batch demo removed - requires async runtime

fn print_atom_info(atom: &SemanticAtom) {
    println!("  Entity ID: {}", atom.entity_id());
    println!("  Sequence: {}", atom.sequence());
    println!("  Value: {:.2}", atom.get_value());
    println!("  Timestamp: {}", atom.timestamp_us());
    println!("  Node ID: {}", atom.node_id());
    println!("  Telemetry Type: {:#04x}", atom.telemetry_type());
    println!("  Trust Level: {:#02x}", atom.trust_level());
    println!("  Predicate: {:#08x}", atom.predicate_id());
}
