//! SAMS Codec - Zero-copy serialization and deserialization
//!
//! This module provides efficient, zero-copy operations for SemanticAtom
//! structures, optimized for both embedded and Linux systems.

use crate::SemanticAtom;

/// SAMS Codec trait for serialization operations
pub trait SamsCodec {
    /// Error type for codec operations
    type Error;

    /// Encode a single atom
    fn encode_atom(&self, atom: &SemanticAtom) -> std::result::Result<Vec<u8>, Self::Error>;

    /// Decode a single atom
    fn decode_atom(&self, data: &[u8]) -> std::result::Result<SemanticAtom, Self::Error>;
}

// Milestone T3: Hardened Wasm Runtime Integration & Codec Development
// Proprietary zero-copy serialization algorithms will be implemented here
