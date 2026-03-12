//! SAMS Codec - Zero-copy serialization and deserialization
//!
//! This module provides efficient, zero-copy operations for SemanticAtom
//! structures, optimized for both embedded and Linux systems.

use crate::SemanticAtom;

/// SAMS Codec trait for serialization operations
pub trait SamsCodec {
    /// Error type for codec operations
    type Error;

    /// Encode a single atom into a fixed 32-byte buffer
    fn encode_atom(&self, atom: &SemanticAtom, output: &mut [u8; 32]) -> core::result::Result<(), Self::Error>;

    /// Decode a single atom from a byte buffer
    fn decode_atom(&self, data: &[u8]) -> core::result::Result<SemanticAtom, Self::Error>;
}

// Milestone T3: Hardened Wasm Runtime Integration & Codec Development
// Proprietary zero-copy serialization algorithms will be implemented here
