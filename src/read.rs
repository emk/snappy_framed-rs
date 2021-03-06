//! Implementations of `Read` using Snappy compression.

use snappy;
use std::cmp::min;
use std::io::{self, Read};

use buffer::Buffer;
use consts::*;
use masked_crc::*;

/// Should we verify or ignore the CRC when reading?
#[derive(Debug, PartialEq, Eq)]
pub enum CrcMode {
    /// Verify that the CRC values in the stream are correct.
    Verify,
    /// Ignore the CRC values.
    Ignore
}

/// A framed chunk in a Snappy stream.
#[derive(Debug)]
struct Chunk<'a> {
    chunk_type: u8,
    data: &'a [u8]
}

impl<'a> Chunk<'a> {
    fn crc(&self) -> io::Result<u32> {
        if self.data.len() < CRC_SIZE {
            Err(io::Error::new(io::ErrorKind::Other, "Snappy CRC truncated"))
        } else {
            Ok((self.data[0] as u32) |
               (self.data[1] as u32) << 8 |
               (self.data[2] as u32) << 16 |
               (self.data[3] as u32) << 24)
        }
    }
}

fn check_crc(expected: u32, data: &[u8]) -> io::Result<()> {
    let actual = masked_crc(data);
    if expected == actual {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other,
                           format!("Invalid Snappy CRC (expected {:x}, got {:x})",
                                   expected, actual)))
    }
}

// Add some input-related convenience functions to Buffer.  We can't put
// these in the `SnappyFramedDecoder` itself because they return references
// to our internal buffer, and thereby render it unavailable until we're
// done.  But if we render `SnappyFramedDecoder` unavailable, we can't
// write to our output buffer.  So it's better to keep this separate.
impl Buffer {
    /// Make sure we have at least the specified number of bytes buffered.
    fn ensure_buffered<R: Read>(&mut self, bytes: usize, source: &mut R) ->
        io::Result<Option<&[u8]>>
    {
        // If we don't have enough data buffered, go get more.
        if self.buffered() < bytes {
            // If our input buffer is too small to hold a chunk, resize it.
            // This should never fire for reasonable input files, so we're not
            // concerned about speed.
            let capacity = self.capacity();
            if bytes > capacity {
                warn!("Snappy chunk of {} bytes required growing buffer", bytes);
                self.add_capacity(bytes - capacity);
            }

            // Move any partial data to the start of the buffer.
            self.move_data_to_start();

            // Try to fill up our buffer.
            loop {
                let bytes_read = {
                    let space = self.space_to_fill();
                    if space.len() == 0 { break; /* Full. */ }
                    try!(source.read(space))
                };
                self.added(bytes_read);
                if bytes_read == 0 { break; /* No more, at least for now. */ }
            }

            // Decide what to do if we still don't have enough data.
            if self.buffered() == 0 {
                // No data, so we're presumably at the end of the file.
                // Return nothing.
                return Ok(None);
            } else if self.buffered() < bytes {
                // Partial data, so fail with an error.
                return Err(io::Error::new(io::ErrorKind::Other,
                                          "Incomplete Snappy chunk"));
            }
        }

        // Actually return our bytes.
        Ok(Some(self.consume(bytes)))
    }

    /// Read in the next input chunk.
    fn next_chunk<R: Read>(&mut self, source: &mut R) ->
        io::Result<Option<Chunk>>
    {
        let (chunk_type, chunk_len) = {
            match try!(self.ensure_buffered(HEADER_SIZE, source)) {
                None => return Ok(None),
                Some(chunk_header) => {
                    (chunk_header[0],
                     ((chunk_header[3] as usize) << 16 |
                      (chunk_header[2] as usize) << 8 |
                      (chunk_header[1] as usize)))
                }
            }
        };
        let data = try!(self.ensure_buffered(chunk_len, source))
            .expect("Snappy chunk header with missing data");
        Ok(Some(Chunk{chunk_type: chunk_type, data: data}))
    }    
}

/// Decode a stream containing Snappy-compressed frames.
///
/// ```
/// use std::io::{Cursor, Read};
/// use snappy_framed::read::{CrcMode, SnappyFramedDecoder};
///
/// let compressed: &[u8] =
///     &[0xff, 0x06, 0x00, 0x00, 0x73, 0x4e, 0x61, 0x50, 0x70, 0x59, 0x01,
///       0x0a, 0x00, 0x00, 0xd3, 0xfe, 0x2e, 0x7a, 0x48, 0x65, 0x6c, 0x6c,
///       0x6f, 0x21];
/// let mut cursor = Cursor::new(&compressed as &[u8]);
/// let mut decoder = SnappyFramedDecoder::new(&mut cursor, CrcMode::Verify);
/// let mut output = vec!();
/// decoder.read_to_end(&mut output).unwrap();
///
/// assert_eq!(b"Hello!", &output as &[u8]);
/// ```

pub struct SnappyFramedDecoder<R: Read> {
    source: R,
    input: Buffer,
    output: Buffer,
    mode: CrcMode
}

impl<R: Read> SnappyFramedDecoder<R> {
    /// Create a new decoder wrapping the specified `source`, and using the
    /// CRC verification options indicated by `mode`.
    pub fn new(source: R, mode: CrcMode) -> Self {
        SnappyFramedDecoder{
            source: source,
            input: Buffer::new(1024*1024),
            output: Buffer::new(MAX_UNCOMPRESSED_CHUNK),
            mode: mode
        }
    }
}

impl<R: Read> Read for SnappyFramedDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.output.empty() {
            loop {
                match try!(self.input.next_chunk(&mut self.source)) {
                    None => return Ok(0),
                    Some(chunk) => {
                        //println!("chunk: {:?}", chunk);
                        match chunk.chunk_type {
                            // Compressed data.
                            0x00 => {
                                // TODO: Output size check.
                                // TODO: Malformed data check.
                                let crc = try!(chunk.crc());
                                let compressed = &chunk.data[CRC_SIZE..];
                                let data = snappy::uncompress(compressed)
                                    .expect("Snappy decompression failure");
                                if self.mode == CrcMode::Verify {
                                    try!(check_crc(crc, &data));
                                }
                                self.output.set_data(&data);
                                break;
                            }

                            // Uncompressed data.
                            0x01 => {
                                // TODO: Output size check.
                                // TODO: Malformed data check.
                                let crc = try!(chunk.crc());
                                let data = &chunk.data[CRC_SIZE..];
                                if self.mode == CrcMode::Verify {
                                    try!(check_crc(crc, &data));
                                }
                                self.output.set_data(&data);
                                break;
                            }

                            // Reserved unskippable chunks.
                            0x02...0x7F => {}
                            // Reserved skippable chunks.
                            0x80...0xFD => {}
                            // Padding.
                            0xFE => {}
                            // Stream identifier.  
                            0xFF => {}
                            _ => unreachable!()
                        }
                    }
                }
            }
        }

        let to_copy = min(self.output.buffered(), buf.len());
        self.output.copy_out_and_consume(to_copy, buf);
        Ok(to_copy)
    }
}

#[cfg(test)]
fn large_compressed_data(repeats: usize) -> io::Result<Vec<u8>> {
    use std::io::Write;

    use write::SnappyFramedEncoder;
    use test_helpers::*;

    // Build a large vector containing repeated text data.
    let hunk = try!(read_file("data/arbres.txt"));
    let data = repeat_data(&hunk, repeats);

    // Compress it.
    let mut compressed = vec!();
    {
        let mut compressor = try!(SnappyFramedEncoder::new(&mut compressed));
        let written = try!(compressor.write(&data));
        assert_eq!(data.len(), written);
        try!(compressor.flush());
    }
    Ok(compressed)
}

#[test]
fn decode_example_stream() {
    use std::fs::File;
    use dribble::DribbleReader;

    use test_helpers::*;

    let mut compressed = File::open("data/arbres.txt.sz").unwrap();
    let mut dribble = DribbleReader::new(&mut compressed);
    let mut decompressor = SnappyFramedDecoder::new(&mut dribble,
                                                    CrcMode::Verify);
    let mut decompressed = vec!();
    decompressor.read_to_end(&mut decompressed).unwrap();

    let expected = read_file("data/arbres.txt").unwrap();
    assert_eq!(expected, decompressed);
}

#[test]
fn encode_and_decode_large_data() {
    use std::io::Cursor;

    use test_helpers::*;

    // Build a multi-MB vector containing text data.
    let hunk = read_file("data/arbres.txt").unwrap();
    let input = repeat_data(&hunk, 1000);
    let compressed = large_compressed_data(1000).unwrap();

    // Decode it.
    let mut cursor = Cursor::new(&compressed as &[u8]);
    let mut decompressor = SnappyFramedDecoder::new(&mut cursor,
                                                    CrcMode::Verify);
    let mut decompressed = vec!();
    decompressor.read_to_end(&mut decompressed).unwrap();

    // Compare it.
    assert_eq!(input, decompressed);
}

// Test for invalid inputs:
//   - No identifier chunk.
//   - Incomplete chunks: All positions return errors.
//   - Reserved chunk types.
//   - Bad CRC.
//   - Overlong chunks (both compressed--two variants--and uncompressed).

#[cfg(all(test, feature = "unstable"))]
mod benches {
    use std::io::{Cursor, Read};
    use test::Bencher;

    use super::{CrcMode, SnappyFramedDecoder, large_compressed_data};

    #[bench]
    fn decompress_speed(b: &mut Bencher) {
        let data = large_compressed_data(250).unwrap();

        let mut output_bytes = 0;
        let mut buffer = vec![0; 128*1024];
        b.iter(|| {
            output_bytes = 0;
            {
                let mut cursor = Cursor::new(&data as &[u8]);
                let mut decoder = SnappyFramedDecoder::new(&mut cursor,
                                                           CrcMode::Ignore);
                // Avoid read_to_end here; it's too slow for large blocks
                // of data.
                loop {
                    let bytes_read = decoder.read(&mut buffer).unwrap();
                    output_bytes += bytes_read;
                    if bytes_read == 0 { break; }
                }
            }
            buffer[0]
        });
        b.bytes = output_bytes as u64;
    }        
}
