/// The size of a chunk header.
pub const HEADER_SIZE: usize = 4;

/// The size of a chunk CRC.
pub const CRC_SIZE: usize = 4;

/// The maximum size of the uncompressed data stored in a chunk.
pub const MAX_UNCOMPRESSED_CHUNK: usize = 65_536;
