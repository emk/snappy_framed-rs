use snappy;
use std::io::{self, Write};

use consts::*;

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
            header_and_crc[0] = 0;
            header_and_crc[1] = (chunk_len & 0x0000FF) as u8;
            header_and_crc[2] = (chunk_len & 0x00FF00 >> 16) as u8;
            header_and_crc[3] = (chunk_len & 0xFF0000 >> 24) as u8;
            // TODO: Generate CRC.
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
    use self::*;

    use read::SnappyFramedDecoder;
    use dribble::DribbleWriter;
    use std::fs::File;
    use std::io::{Cursor, Read, Write};

    let mut original = File::open("data/arbres.txt").unwrap();
    let mut expected = vec!();
    original.read_to_end(&mut expected).unwrap();

    let mut compressed = vec!();
    {
        let mut compressor = SnappyFramedEncoder::new(&mut compressed).unwrap();
        let mut dribble = DribbleWriter::new(&mut compressor);
        let written = dribble.write(&expected).unwrap();
        dribble.flush().unwrap();
        assert_eq!(expected.len(), written);
    }

    let mut decompressed = vec!();
    let mut cursor = Cursor::new(&compressed as &[u8]);
    let mut decompressor = SnappyFramedDecoder::new(&mut cursor);
    decompressor.read_to_end(&mut decompressed).unwrap();

    // Did we survive the round-trip intact?
    assert_eq!(expected, decompressed);
}
