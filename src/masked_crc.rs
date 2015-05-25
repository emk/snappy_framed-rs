use crc::crc32::checksum_castagnoli;

#[test]
fn unmasked_checksum() {
    // Test values from: https://www.ietf.org/rfc/rfc3720.txt , CRC
    // examples interpreted as little endian bytes.
    assert_eq!(checksum_castagnoli(b"123456789"), 0xe3069283);
    assert_eq!(0x8A9136AA, checksum_castagnoli(&[0; 32]));
    assert_eq!(0x62A8AB43, checksum_castagnoli(&[0xFF; 32]));

    let mut incrementing = vec!();
    for i in 0..32 { incrementing.push(i); }
    assert_eq!(32, incrementing.len());
    assert_eq!(0x46DD794E, checksum_castagnoli(&incrementing));
}

/// Apply masking to a CRC value.
fn mask(crc: u32) -> u32 {
    crc.rotate_right(15).wrapping_add(0xA282EAD8)
}

/// Compute a Snappy-format masked CRC.  According to the Snappy docs,
/// "Checksums are not stored directly, but masked, as checksumming data
/// and then its own checksum can be problematic."
pub fn masked_crc(bytes: &[u8]) -> u32 {
    // Also consider porting:
    // https://github.com/Voxer/sse4_crc32/blob/master/src/sse4_crc32.cpp
    mask(checksum_castagnoli(bytes))
}

#[test]
fn masked_checksum() {
    // Test value from two Java libraries, including:
    // https://github.com/dain/snappy/blob/master/src/test/java/org/iq80/snappy/SnappyFramedStreamTest.java
    // This checksum format appears to be compatible with the Java
    // implementations and the 'snappy' tool.  The SmallTalk implementation
    // claims to have been checked against 'snappy' as well.
    assert_eq!(0x9274CDA8, masked_crc(b"aaaaaaaaaaaabbbbbbbaaaaaa"));

    // Test values from:
    // https://github.com/andrix/python-snappy/blob/master/test_snappy.py 
    // These are endian-reversed!  The Python and Node libraries get this
    // backward, relative to the other libraries.
    //assert_eq!(0x8F2948BD, masked_crc(&[0; 50]));
    //assert_eq!(0xB214298A, masked_crc(&[1; 50]));
}

#[cfg(all(test, feature = "unstable"))]
mod benches {
    use test::Bencher;
    use super::*;

    #[bench]
    fn crc_speed(b: &mut Bencher) {
        let input = [0; 1024];
        b.bytes = input.len() as u64;
        b.iter(|| masked_crc(&input));
    }
}

