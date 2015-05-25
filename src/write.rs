use snappy;
use std::io::{self, Write};

use consts::*;
use masked_crc::*;

/// Appears at the front of all Snappy framed streams.
const STREAM_IDENTIFIER: [u8; 10] =
    [0xFF, 0x06, 0x00, 0x00, 0x73, 0x4E, 0x61, 0x50, 0x70, 0x59];

/// Encode a stream containing Snappy-compressed frames.
pub struct SnappyFramedEncoder<W: Write> {
    dest: W
}

impl<W: Write> SnappyFramedEncoder<W> {
    pub fn new(dest: W) -> io::Result<Self> {
        let mut encoder = SnappyFramedEncoder{dest: dest};
        try!(encoder.write_header());
        Ok(encoder)
    }

    fn write_header(&mut self) -> io::Result<()> {
        try!(self.dest.write_all(&STREAM_IDENTIFIER));
        Ok(())
    }
}

impl<W: Write> Write for SnappyFramedEncoder<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for data in buf.chunks(MAX_UNCOMPRESSED_CHUNK) {
            let compressed = snappy::compress(data);

            let mut header_and_crc = [0; HEADER_SIZE+CRC_SIZE];
            let chunk_len = CRC_SIZE + compressed.len();
            let crc = masked_crc(&data);
            header_and_crc[0] = 0;
            header_and_crc[1] = ((chunk_len & 0x0000FF)      ) as u8;
            header_and_crc[2] = ((chunk_len & 0x00FF00) >>  8) as u8;
            header_and_crc[3] = ((chunk_len & 0xFF0000) >> 16) as u8;
            header_and_crc[4] = ((crc & 0x000000FF)      ) as u8;
            header_and_crc[5] = ((crc & 0x0000FF00) >>  8) as u8;
            header_and_crc[6] = ((crc & 0x00FF0000) >> 16) as u8;
            header_and_crc[7] = ((crc & 0xFF000000) >> 24) as u8;
            try!(self.dest.write_all(&header_and_crc));

            try!(self.dest.write_all(&compressed));
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.dest.flush()
    }    
}

#[test]
fn encode_example_stream() {
    use dribble::DribbleWriter;
    use std::io::{Cursor, Read, Write};

    use read::{CrcMode, SnappyFramedDecoder};
    use test_helpers::*;

    let expected = read_file("data/arbres.txt").unwrap();

    let mut compressed = vec!();
    {
        let mut compressor = SnappyFramedEncoder::new(&mut compressed).unwrap();
        let mut dribble = DribbleWriter::new(&mut compressor);
        let written = dribble.write(&expected).unwrap();
        assert_eq!(expected.len(), written);
        dribble.flush().unwrap();
    }

    let mut decompressed = vec!();
    let mut cursor = Cursor::new(&compressed as &[u8]);
    let mut decompressor = SnappyFramedDecoder::new(&mut cursor,
                                                    CrcMode::Verify);
    decompressor.read_to_end(&mut decompressed).unwrap();

    // Did we survive the round-trip intact?
    assert_eq!(expected, decompressed);
}
