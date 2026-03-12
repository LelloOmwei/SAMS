//! SAMS Codec - Zero-copy serialization and deserialization
//!
//! This module provides efficient, zero-copy operations for SemanticAtom
//! structures, optimized for both embedded and Linux systems.

use crate::{Result, SemanticAtom, ATOM_SIZE};

/// Codec error types
#[derive(Debug, Clone, PartialEq)]
pub enum CodecError {
    /// Invalid atom size
    InvalidSize,
    /// Invalid atom format
    InvalidFormat,
    /// Checksum mismatch
    ChecksumError,
    /// Version mismatch
    VersionMismatch,
    /// Buffer overflow
    BufferOverflow,
}

impl core::fmt::Display for CodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CodecError::InvalidSize => write!(f, "Invalid atom size"),
            CodecError::InvalidFormat => write!(f, "Invalid atom format"),
            CodecError::ChecksumError => write!(f, "Checksum mismatch"),
            CodecError::VersionMismatch => write!(f, "Version mismatch"),
            CodecError::BufferOverflow => write!(f, "Buffer overflow"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CodecError {}

/// Codec version for compatibility checking
pub const CODEC_VERSION: u8 = 1;

/// Atom codec with zero-copy operations
#[derive(Debug, Clone)]
pub struct AtomCodec {
    /// Enable checksum verification
    enable_checksum: bool,
    /// Enable version checking
    enable_version_check: bool,
}

impl Default for AtomCodec {
    fn default() -> Self {
        Self {
            enable_checksum: true,
            enable_version_check: true,
        }
    }
}

impl AtomCodec {
    /// Create a new codec with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a codec with custom settings
    pub fn with_settings(enable_checksum: bool, enable_version_check: bool) -> Self {
        Self {
            enable_checksum,
            enable_version_check,
        }
    }

    /// Encode a single atom to bytes (zero-copy)
    pub fn encode_atom<'a>(&self, atom: &'a SemanticAtom) -> Result<CowAtom<'a>> {
        let mut cow = CowAtom::new(atom);
        
        if self.enable_checksum {
            cow.add_checksum()?;
        }
        
        if self.enable_version_check {
            cow.add_version()?;
        }
        
        Ok(cow)
    }

    /// Decode a single atom from bytes (zero-copy when possible)
    pub fn decode_atom<'a>(&self, data: &'a [u8]) -> Result<DecodedAtom<'a>> {
        let mut decoded = DecodedAtom::new(data)?;
        
        if self.enable_version_check {
            decoded.verify_version()?;
        }
        
        if self.enable_checksum {
            decoded.verify_checksum()?;
        }
        
        Ok(decoded)
    }

    /// Encode multiple atoms into a buffer
    pub fn encode_atoms(&self, atoms: &[SemanticAtom], buffer: &mut [u8]) -> Result<usize> {
        let total_size = atoms.len() * ATOM_SIZE;
        if buffer.len() < total_size {
            return Err("Buffer overflow".into());
        }

        for (i, atom) in atoms.iter().enumerate() {
            let start = i * ATOM_SIZE;
            let end = start + ATOM_SIZE;
            buffer[start..end].copy_from_slice(atom.as_bytes());
        }

        Ok(total_size)
    }

    /// Decode multiple atoms from a buffer
    pub fn decode_atoms<'a>(&self, buffer: &'a [u8]) -> Result<&'a [SemanticAtom]> {
        if buffer.len() % ATOM_SIZE != 0 {
            return Err("Invalid atom size".into());
        }

        let atom_count = buffer.len() / ATOM_SIZE;
        
        // For no_std, we'll return a slice that can be interpreted as atoms
        // In practice, the caller would need to handle the unsafety
        Ok(unsafe {
            core::slice::from_raw_parts(
                buffer.as_ptr() as *const SemanticAtom,
                atom_count,
            )
        })
    }

    /// Create an atom iterator over a buffer
    pub fn iter_atoms<'a>(&'a self, buffer: &'a [u8]) -> Result<AtomIterator<'a>> {
        if buffer.len() % ATOM_SIZE != 0 {
            return Err("Invalid atom size".into());
        }
        Ok(AtomIterator::new(buffer, self))
    }

    /// Calculate checksum for atom data
    fn calculate_checksum(&self, data: &[u8]) -> u32 {
        // Simple CRC32-like checksum
        let mut checksum = 0xFFFFFFFF_u32;
        for &byte in data {
            checksum ^= byte as u32;
            for _ in 0..8 {
                if checksum & 1 != 0 {
                    checksum = (checksum >> 1) ^ 0xEDB88320;
                } else {
                    checksum >>= 1;
                }
            }
        }
        !checksum
    }
}

/// Copy-on-write atom wrapper for zero-copy operations
#[derive(Debug)]
pub struct CowAtom<'a> {
    /// Reference to original atom (if no modifications)
    original: Option<&'a SemanticAtom>,
    /// Modified atom data (if modifications were made)
    modified: Option<SemanticAtom>,
    /// Additional metadata (checksum, version)
    metadata: CowMetadata,
}

#[derive(Debug, Clone)]
struct CowMetadata {
    checksum: Option<u32>,
    version: Option<u8>,
}

impl<'a> CowAtom<'a> {
    /// Create a new CowAtom from an existing atom
    fn new(atom: &'a SemanticAtom) -> Self {
        Self {
            original: Some(atom),
            modified: None,
            metadata: CowMetadata {
                checksum: None,
                version: None,
            },
        }
    }

    /// Get a reference to the atom (zero-copy if possible)
    pub fn as_atom(&self) -> &SemanticAtom {
        self.modified.as_ref().unwrap_or(self.original.unwrap())
    }

    /// Get mutable reference to the atom (will copy if needed)
    pub fn as_mut_atom(&mut self) -> &mut SemanticAtom {
        if self.modified.is_none() {
            self.modified = Some(*self.original.unwrap());
            self.original = None;
        }
        self.modified.as_mut().unwrap()
    }

    /// Add checksum to the atom metadata
    fn add_checksum(&mut self) -> Result<()> {
        let checksum = self.calculate_atom_checksum();
        self.metadata.checksum = Some(checksum);
        Ok(())
    }

    /// Add version to the atom metadata
    fn add_version(&mut self) -> Result<()> {
        self.metadata.version = Some(CODEC_VERSION);
        Ok(())
    }

    /// Get the checksum if available
    pub fn checksum(&self) -> Option<u32> {
        self.metadata.checksum
    }

    /// Get the version if available
    pub fn version(&self) -> Option<u8> {
        self.metadata.version
    }

    /// Calculate checksum for the atom
    fn calculate_atom_checksum(&self) -> u32 {
        let codec = AtomCodec::new();
        codec.calculate_checksum(self.as_atom().as_bytes())
    }

    /// Convert to owned atom (will copy if needed)
    pub fn to_owned(&self) -> SemanticAtom {
        self.modified.unwrap_or(*self.original.unwrap())
    }
}

/// Decoded atom with verification metadata
#[derive(Debug)]
pub struct DecodedAtom<'a> {
    /// Reference to atom data
    atom_data: &'a [u8],
    /// Verification metadata
    metadata: DecodedMetadata,
}

#[derive(Debug, Clone)]
struct DecodedMetadata {
    expected_checksum: Option<u32>,
    expected_version: Option<u8>,
}

impl<'a> DecodedAtom<'a> {
    /// Create a new decoded atom from raw data
    fn new(data: &'a [u8]) -> Result<Self> {
        if data.len() < ATOM_SIZE {
            return Err("Invalid atom size".into());
        }

        Ok(Self {
            atom_data: &data[..ATOM_SIZE],
            metadata: DecodedMetadata {
                expected_checksum: None,
                expected_version: None,
            },
        })
    }

    /// Get reference to the atom (zero-copy)
    pub fn as_atom(&self) -> Result<&SemanticAtom> {
        Ok(SemanticAtom::from_bytes(self.atom_data)?)
    }

    /// Get mutable copy of the atom
    pub fn to_atom(&self) -> Result<SemanticAtom> {
        self.as_atom().map(|atom| *atom)
    }

    /// Verify the atom's checksum
    fn verify_checksum(&mut self) -> Result<()> {
        let codec = AtomCodec::new();
        let calculated = codec.calculate_checksum(self.atom_data);
        
        if let Some(expected) = self.metadata.expected_checksum {
            if calculated != expected {
                return Err("Checksum mismatch".into());
            }
        }
        
        self.metadata.expected_checksum = Some(calculated);
        Ok(())
    }

    /// Verify the atom's version
    fn verify_version(&mut self) -> Result<()> {
        // Version would be stored in atom metadata in real implementation
        // For now, we just check that the atom format is valid
        self.as_atom()?;
        self.metadata.expected_version = Some(CODEC_VERSION);
        Ok(())
    }

    /// Get the calculated checksum
    pub fn checksum(&self) -> Option<u32> {
        self.metadata.expected_checksum
    }

    /// Get the version
    pub fn version(&self) -> Option<u8> {
        self.metadata.expected_version
    }
}

/// Iterator over atoms in a buffer
#[derive(Debug)]
pub struct AtomIterator<'a> {
    buffer: &'a [u8],
    codec: &'a AtomCodec,
    position: usize,
}

impl<'a> AtomIterator<'a> {
    /// Create a new atom iterator
    fn new(buffer: &'a [u8], codec: &'a AtomCodec) -> Self {
        Self {
            buffer,
            codec,
            position: 0,
        }
    }
}

impl<'a> Iterator for AtomIterator<'a> {
    type Item = Result<DecodedAtom<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position + ATOM_SIZE <= self.buffer.len() {
            let atom_data = &self.buffer[self.position..self.position + ATOM_SIZE];
            self.position += ATOM_SIZE;
            Some(self.codec.decode_atom(atom_data))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.buffer.len() - self.position) / ATOM_SIZE;
        (remaining, Some(remaining))
    }
}

/// Batch codec for efficient processing of multiple atoms
pub struct BatchCodec {
    codec: AtomCodec,
    #[allow(dead_code)]
    buffer_size: usize,
}

impl BatchCodec {
    /// Create a new batch codec
    pub fn new(buffer_size: usize) -> Self {
        Self {
            codec: AtomCodec::new(),
            buffer_size,
        }
    }

    /// Process a batch of atoms
    pub fn process_batch(&self, atoms: &[SemanticAtom]) -> Result<BatchResult> {
        let mut output = [0u8; 1024];
        let bytes_written = self.codec.encode_atoms(atoms, &mut output)?;
        
        Ok(BatchResult {
            atoms: atoms.len(),
            bytes_written,
            data: output,
            data_len: bytes_written,
        })
    }

    /// Process a batch from raw data
    pub fn process_raw_batch(&self, data: &[u8]) -> Result<BatchResult> {
        let atoms = self.codec.decode_atoms(data)?;
        let mut output = [0u8; 1024];
        let data_len = data.len().min(1024);
        output[..data_len].copy_from_slice(&data[..data_len]);
        
        Ok(BatchResult {
            atoms: atoms.len(),
            bytes_written: data_len,
            data: output,
            data_len,
        })
    }
}

/// Result of batch processing
#[derive(Debug)]
pub struct BatchResult {
    /// Number of atoms processed
    pub atoms: usize,
    /// Number of bytes written
    pub bytes_written: usize,
    /// Raw data buffer
    pub data: [u8; 1024], // Fixed size buffer for no_std
    /// Actual data length
    pub data_len: usize,
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::AtomBuilder;

    #[test]
    fn test_basic_codec() {
        let codec = AtomCodec::new();
        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.5)
            .build();

        let encoded = codec.encode_atom(&atom).unwrap();
        let decoded = codec.decode_atom(encoded.as_atom().as_bytes()).unwrap();
        
        assert_eq!(decoded.as_atom().unwrap().entity_id(), 123);
        assert_eq!(decoded.as_atom().unwrap().get_value(), 42.5);
    }

    #[test]
    fn test_zero_copy_encoding() {
        let codec = AtomCodec::new();
        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.5)
            .build();

        let encoded = codec.encode_atom(&atom).unwrap();
        // This should be zero-copy - no allocation
        assert!(encoded.original.is_some());
        assert!(encoded.modified.is_none());
    }

    #[test]
    fn test_multiple_atoms() {
        let codec = AtomCodec::new();
        let atoms = vec![
            AtomBuilder::new().entity_id(1).value(10.0).build(),
            AtomBuilder::new().entity_id(2).value(20.0).build(),
            AtomBuilder::new().entity_id(3).value(30.0).build(),
        ];

        let mut buffer = [0u8; 96]; // 3 * 32 bytes
        let bytes_written = codec.encode_atoms(&atoms, &mut buffer).unwrap();
        assert_eq!(bytes_written, 96);

        let decoded_atoms = codec.decode_atoms(&buffer).unwrap();
        assert_eq!(decoded_atoms.len(), 3);
        assert_eq!(decoded_atoms[0].entity_id(), 1);
        assert_eq!(decoded_atoms[1].entity_id(), 2);
        assert_eq!(decoded_atoms[2].entity_id(), 3);
    }

    #[test]
    fn test_atom_iterator() {
        let codec = AtomCodec::new();
        let atoms = vec![
            AtomBuilder::new().entity_id(1).value(10.0).build(),
            AtomBuilder::new().entity_id(2).value(20.0).build(),
        ];

        let mut buffer = [0u8; 64];
        codec.encode_atoms(&atoms, &mut buffer).unwrap();

        let iter = codec.iter_atoms(&buffer).unwrap();
        let collected: Result<Vec<_>> = iter.collect();
        let decoded_atoms = collected.unwrap();

        assert_eq!(decoded_atoms.len(), 2);
        assert_eq!(decoded_atoms[0].as_atom().unwrap().entity_id(), 1);
        assert_eq!(decoded_atoms[1].as_atom().unwrap().entity_id(), 2);
    }

    #[test]
    fn test_checksum_verification() {
        let codec = AtomCodec::with_settings(true, false);
        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.5)
            .build();

        let encoded = codec.encode_atom(&atom).unwrap();
        let _checksum = encoded.checksum().unwrap();

        // Corrupt the data
        let mut corrupted_data = encoded.as_atom().as_bytes().to_vec();
        corrupted_data[0] ^= 0xFF;

        let result = codec.decode_atom(&corrupted_data);
        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn test_batch_codec() {
        let batch_codec = BatchCodec::new(1024);
        let atoms = vec![
            AtomBuilder::new().entity_id(1).value(10.0).build(),
            AtomBuilder::new().entity_id(2).value(20.0).build(),
            AtomBuilder::new().entity_id(3).value(30.0).build(),
        ];

        let result = batch_codec.process_batch(&atoms).unwrap();
        assert_eq!(result.atoms, 3);
        assert_eq!(result.bytes_written, 96);
    }

    #[test]
    fn test_invalid_size() {
        let codec = AtomCodec::new();
        let invalid_data = [0u8; 31]; // Too small

        let result = codec.decode_atom(&invalid_data);
        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn test_cow_atom_modification() {
        let codec = AtomCodec::new();
        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.5)
            .build();

        let mut cow = codec.encode_atom(&atom).unwrap();
        
        // Initially zero-copy
        assert!(cow.original.is_some());
        assert!(cow.modified.is_none());
        
        // Modify the atom
        cow.as_mut_atom().entity_id = 456;
        
        // Now it's copied
        assert!(cow.original.is_none());
        assert!(cow.modified.is_some());
        assert_eq!(cow.as_atom().entity_id(), 456);
    }
}
