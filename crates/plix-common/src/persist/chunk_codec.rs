//! Chunk serialization codec.
//!
//! Provides encoding/decoding of chunks to/from binary format.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::chunk::{Chunk, ChunkCoord, CHUNK_BLOCK_COUNT};
use crate::types::BlockType;

use super::error::{PersistError, VersionCheck};

/// Current chunk data format version.
pub const CHUNK_FORMAT_VERSION: u8 = 1;

/// Expected size of encoded chunk data (approximate).
/// Used for buffer pre-allocation and validation.
pub const CHUNK_DATA_SIZE_APPROX: usize = 4109;

/// Serializable chunk data structure.
///
/// This is the on-disk format, separate from runtime Chunk struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    /// Format version for this chunk file.
    pub version: u8,

    /// Chunk coordinates.
    pub coord: ChunkCoord,

    /// Block data (4096 blocks = 16x16x16).
    #[serde(with = "BigArray")]
    pub blocks: [BlockType; CHUNK_BLOCK_COUNT],
}

impl ChunkData {
    /// Create ChunkData from a runtime Chunk.
    ///
    /// # Arguments
    /// * `chunk` - Runtime chunk to serialize.
    ///
    /// # Returns
    /// ChunkData ready for encoding.
    pub fn from_chunk(chunk: &Chunk) -> Self {
        Self {
            version: CHUNK_FORMAT_VERSION,
            coord: chunk.coord(),
            blocks: *chunk.blocks(),
        }
    }

    /// Convert ChunkData to a runtime Chunk.
    ///
    /// # Returns
    /// Runtime Chunk with data from this ChunkData.
    ///
    /// # Note
    /// The returned chunk has dirty=true (needs mesh rebuild on load).
    pub fn into_chunk(self) -> Chunk {
        Chunk::from_blocks(self.coord, self.blocks)
    }
}

/// Codec for chunk serialization/deserialization.
///
/// Stateless - all methods are associated functions.
pub struct ChunkCodec;

impl ChunkCodec {
    /// Encode a chunk to bytes.
    ///
    /// # Arguments
    /// * `chunk` - Runtime chunk to encode.
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Encoded bytes.
    /// * `Err(PersistError::Codec)` - If serialization fails (shouldn't happen).
    ///
    /// # Format
    /// bincode-encoded ChunkData struct.
    ///
    /// # Size
    /// Approximately 4109 bytes per chunk.
    pub fn encode(chunk: &Chunk) -> Result<Vec<u8>, PersistError> {
        let data = ChunkData::from_chunk(chunk);
        Self::encode_data(&data)
    }

    /// Encode chunk data directly.
    ///
    /// # Arguments
    /// * `data` - ChunkData struct to encode.
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Encoded bytes.
    /// * `Err(PersistError::Codec)` - If serialization fails.
    pub fn encode_data(data: &ChunkData) -> Result<Vec<u8>, PersistError> {
        bincode::serialize(data).map_err(|e| PersistError::Codec(e.to_string()))
    }

    /// Decode bytes to a chunk.
    ///
    /// # Arguments
    /// * `bytes` - Encoded chunk data.
    ///
    /// # Returns
    /// * `Ok(Chunk)` - Decoded runtime chunk.
    /// * `Err(PersistError::Codec)` - If deserialization fails.
    /// * `Err(PersistError::VersionMismatch)` - If version unsupported.
    ///
    /// # Validation
    /// * Checks version is supported.
    pub fn decode(bytes: &[u8]) -> Result<Chunk, PersistError> {
        let data = Self::decode_data(bytes)?;

        if data.version > CHUNK_FORMAT_VERSION {
            return Err(PersistError::VersionMismatch {
                world_version: data.version as u32,
                reason: VersionCheck::TooNew(data.version as u32),
            });
        }

        // Apply migration if needed
        let data = Self::migrate(data)?;

        Ok(data.into_chunk())
    }

    /// Decode bytes to ChunkData (for inspection/migration).
    ///
    /// # Arguments
    /// * `bytes` - Encoded chunk data.
    ///
    /// # Returns
    /// * `Ok(ChunkData)` - Decoded data struct.
    /// * `Err(PersistError::Codec)` - If deserialization fails.
    pub fn decode_data(bytes: &[u8]) -> Result<ChunkData, PersistError> {
        bincode::deserialize(bytes).map_err(|e| PersistError::Codec(e.to_string()))
    }

    /// Validate encoded chunk data without full decode.
    ///
    /// # Arguments
    /// * `bytes` - Encoded chunk data.
    ///
    /// # Returns
    /// * `Ok(ChunkCoord)` - Chunk coordinates if valid.
    /// * `Err(PersistError)` - If invalid.
    ///
    /// # Performance
    /// Faster than full decode - only reads header.
    ///
    /// # Checks
    /// * Minimum size for header.
    /// * Version field is supported.
    /// * Can extract coordinates.
    pub fn validate(bytes: &[u8]) -> Result<ChunkCoord, PersistError> {
        // Minimum size: version (1) + coord (3*4) + blocks (4096)
        const MIN_SIZE: usize = 1 + 12 + CHUNK_BLOCK_COUNT;
        if bytes.len() < MIN_SIZE {
            return Err(PersistError::Codec(format!(
                "chunk data too small: {} < {}",
                bytes.len(),
                MIN_SIZE
            )));
        }

        // Decode to get version and coord
        let data = Self::decode_data(bytes)?;

        if !Self::is_version_supported(data.version) {
            if data.version > CHUNK_FORMAT_VERSION {
                return Err(PersistError::VersionMismatch {
                    world_version: data.version as u32,
                    reason: VersionCheck::TooNew(data.version as u32),
                });
            } else {
                return Err(PersistError::VersionMismatch {
                    world_version: data.version as u32,
                    reason: VersionCheck::TooOld(data.version as u32),
                });
            }
        }

        Ok(data.coord)
    }

    /// Check if a version number is supported.
    ///
    /// # Arguments
    /// * `version` - Chunk format version.
    ///
    /// # Returns
    /// * `true` if version can be loaded (possibly with migration).
    /// * `false` if version is unsupported.
    pub fn is_version_supported(version: u8) -> bool {
        version >= 1 && version <= CHUNK_FORMAT_VERSION
    }

    /// Migrate chunk data from old version to current.
    ///
    /// # Arguments
    /// * `data` - ChunkData with old version.
    ///
    /// # Returns
    /// * `Ok(ChunkData)` - Migrated data with current version.
    /// * `Err(PersistError)` - If migration not possible.
    ///
    /// # Note
    /// Currently no migrations defined (v1 only).
    /// Will be implemented when v2 is introduced.
    pub fn migrate(mut data: ChunkData) -> Result<ChunkData, PersistError> {
        // Currently only version 1 exists, no migration needed
        if data.version == CHUNK_FORMAT_VERSION {
            return Ok(data);
        }

        // Future: add migration logic here
        // match data.version {
        //     1 => { /* migrate v1 to v2 */ }
        //     _ => {}
        // }

        // Update version after migration
        data.version = CHUNK_FORMAT_VERSION;
        Ok(data)
    }
}

/// Generate filename for a chunk coordinate.
///
/// # Arguments
/// * `coord` - Chunk coordinates.
///
/// # Returns
/// Filename in format `<x>_<y>_<z>.bin`
///
/// # Examples
/// * (0, 0, 0) -> "0_0_0.bin"
/// * (-5, 10, 3) -> "-5_10_3.bin"
pub fn chunk_filename(coord: ChunkCoord) -> String {
    format!("{}_{}_{}.bin", coord.x, coord.y, coord.z)
}

/// Parse chunk coordinate from filename.
///
/// # Arguments
/// * `filename` - Filename to parse.
///
/// # Returns
/// * `Some(ChunkCoord)` if valid chunk filename.
/// * `None` if filename doesn't match pattern.
///
/// # Pattern
/// `<x>_<y>_<z>.bin` where x, y, z are signed integers.
pub fn parse_chunk_filename(filename: &str) -> Option<ChunkCoord> {
    let filename = filename.strip_suffix(".bin")?;
    let parts: Vec<&str> = filename.split('_').collect();
    if parts.len() != 3 {
        return None;
    }

    let x: i32 = parts[0].parse().ok()?;
    let y: i32 = parts[1].parse().ok()?;
    let z: i32 = parts[2].parse().ok()?;

    Some(ChunkCoord::new(x, y, z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_data_from_chunk() {
        let mut chunk = Chunk::new(ChunkCoord::new(1, 2, 3));
        chunk.set_block(0, 0, 0, BlockType::STONE);
        chunk.set_block(15, 15, 15, BlockType::BRICK);

        let data = ChunkData::from_chunk(&chunk);
        assert_eq!(data.version, CHUNK_FORMAT_VERSION);
        assert_eq!(data.coord, ChunkCoord::new(1, 2, 3));
        assert_eq!(data.blocks[0], BlockType::STONE);
    }

    #[test]
    fn test_chunk_data_into_chunk() {
        let data = ChunkData {
            version: CHUNK_FORMAT_VERSION,
            coord: ChunkCoord::new(5, 6, 7),
            blocks: [BlockType::AIR; CHUNK_BLOCK_COUNT],
        };

        let chunk = data.into_chunk();
        assert_eq!(chunk.coord(), ChunkCoord::new(5, 6, 7));
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let mut chunk = Chunk::new(ChunkCoord::new(1, 2, 3));
        chunk.set_block(0, 0, 0, BlockType::STONE);
        chunk.set_block(15, 15, 15, BlockType::BRICK);
        chunk.set_block(7, 8, 9, BlockType::METAL);

        let bytes = ChunkCodec::encode(&chunk).unwrap();
        let decoded = ChunkCodec::decode(&bytes).unwrap();

        assert_eq!(decoded.coord(), chunk.coord());
        assert_eq!(decoded.get_block(0, 0, 0), BlockType::STONE);
        assert_eq!(decoded.get_block(15, 15, 15), BlockType::BRICK);
        assert_eq!(decoded.get_block(7, 8, 9), BlockType::METAL);
    }

    #[test]
    fn test_encode_size() {
        let chunk = Chunk::new(ChunkCoord::new(0, 0, 0));
        let bytes = ChunkCodec::encode(&chunk).unwrap();
        // Should be close to CHUNK_DATA_SIZE_APPROX
        assert!(bytes.len() > 4000 && bytes.len() < 5000);
    }

    #[test]
    fn test_validate_valid() {
        let chunk = Chunk::new(ChunkCoord::new(10, -5, 3));
        let bytes = ChunkCodec::encode(&chunk).unwrap();
        let coord = ChunkCodec::validate(&bytes).unwrap();
        assert_eq!(coord, ChunkCoord::new(10, -5, 3));
    }

    #[test]
    fn test_validate_too_small() {
        let bytes = vec![0u8; 10];
        assert!(ChunkCodec::validate(&bytes).is_err());
    }

    #[test]
    fn test_is_version_supported() {
        assert!(ChunkCodec::is_version_supported(1));
        assert!(ChunkCodec::is_version_supported(CHUNK_FORMAT_VERSION));
        assert!(!ChunkCodec::is_version_supported(0));
        assert!(!ChunkCodec::is_version_supported(CHUNK_FORMAT_VERSION + 1));
    }

    #[test]
    fn test_chunk_filename() {
        assert_eq!(chunk_filename(ChunkCoord::new(0, 0, 0)), "0_0_0.bin");
        assert_eq!(chunk_filename(ChunkCoord::new(-5, 10, 3)), "-5_10_3.bin");
        assert_eq!(
            chunk_filename(ChunkCoord::new(100, -20, 7)),
            "100_-20_7.bin"
        );
    }

    #[test]
    fn test_parse_chunk_filename_valid() {
        assert_eq!(
            parse_chunk_filename("0_0_0.bin"),
            Some(ChunkCoord::new(0, 0, 0))
        );
        assert_eq!(
            parse_chunk_filename("-5_10_3.bin"),
            Some(ChunkCoord::new(-5, 10, 3))
        );
        assert_eq!(
            parse_chunk_filename("100_-20_7.bin"),
            Some(ChunkCoord::new(100, -20, 7))
        );
    }

    #[test]
    fn test_parse_chunk_filename_invalid() {
        assert_eq!(parse_chunk_filename("invalid"), None);
        assert_eq!(parse_chunk_filename("0_0.bin"), None);
        assert_eq!(parse_chunk_filename("0_0_0_0.bin"), None);
        assert_eq!(parse_chunk_filename("a_b_c.bin"), None);
        assert_eq!(parse_chunk_filename("0_0_0.txt"), None);
    }

    #[test]
    fn test_filename_roundtrip() {
        let coords = [
            ChunkCoord::new(0, 0, 0),
            ChunkCoord::new(-10, 5, 100),
            ChunkCoord::new(1000, -500, 0),
            ChunkCoord::new(-1, -1, -1),
        ];

        for coord in coords {
            let filename = chunk_filename(coord);
            let parsed = parse_chunk_filename(&filename).unwrap();
            assert_eq!(parsed, coord, "Roundtrip failed for {:?}", coord);
        }
    }

    #[test]
    fn test_decode_invalid_bytes() {
        let invalid = vec![0xFF; 100];
        assert!(ChunkCodec::decode(&invalid).is_err());
    }
}
