//! Linux CLI demo for SAMS library
//!
//! This example shows how to use SAMS on Linux systems with Zenoh transport
//! and full std library support.

use sams::{
    AtomBuilder, SemanticAtom, Shield, ShieldConfig, AnonymizationLevel, 
    ZenohTransport, IpcBridge, IpcBridgeConfig
};
use sams::telemetry;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "producer" => producer_demo().await,
        "consumer" => consumer_demo().await,
        "bridge" => bridge_demo().await,
        "shield" => shield_demo().await,
        "batch" => batch_demo().await,
        _ => {
            println!("Unknown command: {}", args[1]);
            print_usage();
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

async fn producer_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting SAMS Producer...");
    
    // Initialize Zenoh transport
    let transport = ZenohTransport::new().await?;
    println!("✅ Zenoh transport initialized");
    
    // Create atoms with different telemetry types
    let atoms = vec![
        // Water level atom
        AtomBuilder::new()
            .entity_id(1001)
            .sequence(1)
            .value(1250.5) // 12.505m
            .timestamp_us(sams::utils::time::now_us()?)
            .node_id(0x1001)
            .telemetry_type(telemetry::WATER_LEVEL_MM)
            .trust_level(0x01)
            .build(),
        
        // Temperature atom
        AtomBuilder::new()
            .entity_id(1002)
            .sequence(2)
            .value(22.5) // 22.5°C
            .timestamp_us(sams::utils::time::now_us()?)
            .node_id(0x1001)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .trust_level(0x01)
            .build(),
        
        // CO2 atom
        AtomBuilder::new()
            .entity_id(1003)
            .sequence(3)
            .value(450.0) // 450 ppm
            .timestamp_us(sams::utils::time::now_us()?)
            .node_id(0x1001)
            .telemetry_type(telemetry::CO2_PPM)
            .trust_level(0x01)
            .build(),
    ];
    
    println!("📊 Created {} atoms", atoms.len());
    
    // Publish individual atoms
    for (i, atom) in atoms.iter().enumerate() {
        let topic = format!("demo/sensor{}", i + 1);
        transport.publish_atom(&topic, atom).await?;
        println!("📡 Published atom to {}: entity={}, value={:.2}", 
                topic, atom.entity_id(), atom.get_value());
    }
    
    // Publish batch
    let batch_topic = "demo/batch";
    transport.publish_batch(batch_topic, &atoms).await?;
    println!("📦 Published batch of {} atoms to {}", atoms.len(), batch_topic);
    
    // Continuous production
    println!("🔄 Starting continuous production (Ctrl+C to stop)...");
    let mut sequence = 100;
    
    loop {
        let atom = AtomBuilder::new()
            .entity_id(2001)
            .sequence(sequence)
            .value(42.0 + (sequence % 10) as f64)
            .timestamp_us(sams::utils::time::now_us()?)
            .node_id(0x2001)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .trust_level(0x01)
            .build();
        
        transport.publish_atom("demo/continuous", &atom).await?;
        println!("📡 Continuous: sequence={}, value={:.2}", sequence, atom.get_value());
        
        sequence += 1;
        sleep(Duration::from_secs(1)).await;
    }
}

async fn consumer_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Starting SAMS Consumer...");
    
    // Initialize Zenoh transport
    let transport = ZenohTransport::new().await?;
    println!("✅ Zenoh transport initialized");
    
    // Subscribe to individual atom topics
    let mut subscriber1 = transport.subscribe_atoms("demo/sensor1").await?;
    let mut subscriber2 = transport.subscribe_atoms("demo/sensor2").await?;
    let mut subscriber3 = transport.subscribe_atoms("demo/sensor3").await?;
    
    // Subscribe to batch topic
    let mut batch_subscriber = transport.subscribe_atoms("demo/batch").await?;
    
    // Subscribe to continuous topic
    let mut continuous_subscriber = transport.subscribe_atoms("demo/continuous").await?;
    
    println!("🔗 Subscribed to all demo topics");
    println!("👂 Listening for atoms (Ctrl+C to stop)...");
    
    loop {
        tokio::select! {
            // Individual atoms
            Some((atom, topic)) = async { 
                match subscriber1.try_recv() {
                    Ok(Some(result)) => Some(result),
                    _ => None,
                }
            } => {
                println!("💧 Water level: entity={}, value={:.2}m, topic={}", 
                        atom.entity_id(), atom.get_value() / 1000.0, topic);
            }
            
            Some((atom, topic)) = async {
                match subscriber2.try_recv() {
                    Ok(Some(result)) => Some(result),
                    _ => None,
                }
            } => {
                println!("🌡️  Temperature: entity={}, value={:.1}°C, topic={}", 
                        atom.entity_id(), atom.get_value(), topic);
            }
            
            Some((atom, topic)) = async {
                match subscriber3.try_recv() {
                    Ok(Some(result)) => Some(result),
                    _ => None,
                }
            } => {
                println!("💨 CO2: entity={}, value={:.0}ppm, topic={}", 
                        atom.entity_id(), atom.get_value(), topic);
            }
            
            // Batch atoms
            Some((atom, topic)) = async {
                match batch_subscriber.try_recv() {
                    Ok(Some(result)) => Some(result),
                    _ => None,
                }
            } => {
                println!("📦 Batch atom: entity={}, value={:.2}, topic={}", 
                        atom.entity_id(), atom.get_value(), topic);
            }
            
            // Continuous atoms
            Some((atom, topic)) = async {
                match continuous_subscriber.try_recv() {
                    Ok(Some(result)) => Some(result),
                    _ => None,
                }
            } => {
                println!("🔄 Continuous: sequence={}, value={:.2}, topic={}", 
                        atom.sequence(), atom.get_value(), topic);
            }
            
            _ => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn bridge_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌉 Starting SAMS IPC Bridge...");
    
    // Create bridge configuration
    let config = IpcBridgeConfig {
        bridge_id: "demo-bridge".to_string(),
        source_topic: "embedded/input".to_string(),
        target_topic: "linux/output".to_string(),
        enable_forwarding: true,
    };
    
    // Create and start bridge
    let mut bridge = IpcBridge::new(config).await?;
    println!("✅ Bridge created: {}", bridge.stats().bridge_id);
    
    // Simulate receiving from embedded and forwarding to Linux
    println!("🔄 Starting bridge forwarding (Ctrl+C to stop)...");
    
    // In a real implementation, this would run indefinitely
    // For demo, we'll run for a short time
    tokio::spawn(async move {
        bridge.start_forwarding().await
    });
    
    // Simulate some traffic
    for i in 0..10 {
        let atom = AtomBuilder::new()
            .entity_id(3000 + i)
            .sequence(i)
            .value(100.0 + i as f64)
            .timestamp_us(sams::utils::time::now_us()?)
            .node_id(0x3000)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .trust_level(0x01)
            .build();
        
        bridge.send_to_embedded(&atom).await?;
        println!("📤 Sent to embedded: entity={}", atom.entity_id());
        
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("✅ Bridge demo completed");
    Ok(())
}

async fn shield_demo() -> Result<(), Box<dyn std::error::Error>> {
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
        .timestamp_us(sams::utils::time::now_us()?)
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
        
        shield.anonymize_atom(&mut protected_atom).unwrap();
        
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

async fn batch_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Starting SAMS Batch Demo...");
    
    // Create transport
    let transport = ZenohTransport::new().await?;
    
    // Generate large batch of atoms
    let mut atoms = Vec::new();
    let base_time = sams::utils::time::now_us()?;
    
    for i in 0..1000 {
        let atom = AtomBuilder::new()
            .entity_id(4000 + (i % 10))
            .sequence(i)
            .value(20.0 + (i % 50) as f64)
            .timestamp_us(base_time + (i as u64 * 1000)) // 1ms apart
            .node_id(0x4000)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .trust_level(0x01)
            .build();
        
        atoms.push(atom);
    }
    
    println!("📊 Generated {} atoms", atoms.len());
    
    // Publish in batches
    const BATCH_SIZE: usize = 100;
    for (batch_num, chunk) in atoms.chunks(BATCH_SIZE).enumerate() {
        let topic = format!("demo/batch/{}", batch_num);
        transport.publish_batch(&topic, chunk).await?;
        println!("📦 Published batch {}: {} atoms", batch_num, chunk.len());
        
        // Small delay to avoid overwhelming the system
        sleep(Duration::from_millis(10)).await;
    }
    
    // Query and process batches
    println!("🔍 Querying batch atoms...");
    let queried_atoms = transport.query_atoms("demo/batch/*", "*").await?;
    println!("📊 Queried {} atoms", queried_atoms.len());
    
    // Calculate statistics
    let values: Vec<f64> = queried_atoms.iter()
        .map(|atom| atom.get_value())
        .collect();
    
    let avg = values.iter().sum::<f64>() / values.len() as f64;
    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    println!("📈 Statistics:");
    println!("  Average: {:.2}", avg);
    println!("  Min: {:.2}", min);
    println!("  Max: {:.2}", max);
    println!("  Count: {}", values.len());
    
    Ok(())
}

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
