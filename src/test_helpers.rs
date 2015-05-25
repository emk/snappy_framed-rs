//! Helper functions for our unit tests.

use std::convert::AsRef;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Read a test file into memory.
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut f = try!(File::open(path));
    let mut data = vec!();
    try!(f.read_to_end(&mut data));
    Ok(data)
}

/// Repeat `data` `n` times.
pub fn repeat_data(data: &[u8], n: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() * n);
    for _ in 0..n { result.extend(data.iter().cloned()); }
    result
}
