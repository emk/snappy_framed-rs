use snappy;
use std::cmp::min;
use std::io::{self, Read};

use buffer::Buffer;

/// The largest allowable chunk size (uncompressed).
const MAX_UNCOMPRESSED_CHUNK: usize = 65_536 + 4;

/// A framed chunk in a Snappy stream.
struct Chunk<'a> {
    chunk_type: u8,
    data: &'a [u8]
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
            match try!(self.ensure_buffered(4, source)) {
                None => return Ok(None),
                Some(chunk_header) => {
                    println!("shifted: {}", chunk_header[3] as usize);
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

/// Decode a stream containing `snappy`-compressed frames.
pub struct SnappyFramedDecoder<R: Read> {
    source: R,
    input: Buffer,
    output: Buffer
}

impl<R: Read> SnappyFramedDecoder<R> {
    pub fn new(source: R) -> Self {
        SnappyFramedDecoder{
            source: source,
            input: Buffer::new(1024*1024),
            output: Buffer::new(MAX_UNCOMPRESSED_CHUNK)
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
                        match chunk.chunk_type {
                            // Compressed data.
                            0x00 => {
                                // TODO: Output size check.
                                // TODO: Malformed data check.
                                let compressed = &chunk.data[4..];
                                let data = snappy::uncompress(compressed)
                                    .expect("Snappy decompression failure");
                                self.output.set_data(&data);
                                break;
                            }

                            // Uncompressed data.
                            0x01 => {
                                // TODO: Output size check.
                                // TODO: Malformed data check.
                                let data = &chunk.data[4..];
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

#[test]
fn decode_example_stream() {
    use std::fs::File;
    use dribble::DribbleReader;

    let mut compressed = File::open("data/arbres.txt.sz").unwrap();
    let mut dribble = DribbleReader::new(&mut compressed);
    let mut decompressor = SnappyFramedDecoder::new(&mut dribble);
    let mut decompressed = vec!();
    decompressor.read_to_end(&mut decompressed).unwrap();

    let mut original = File::open("data/arbres.txt").unwrap();
    let mut expected = vec!();
    original.read_to_end(&mut expected).unwrap();

    assert_eq!(expected, decompressed);
}
