//! Normally, [Snappy compression][snappy] works entirely in memory, but
//! there is also a ["Snappy framed"][framed] format which can be read and
//! written in streaming mode.  We provide `Read` and `Write`
//! implementations for framed snappy data.
//!
//! The API to this library is designed to be similar to that of
//! [`flate2`][flate2], though we have not yet implemented
//! `read::SnappyFramedEncoder` or `write::SnappyFramedDecoder`.
//!
//! ### A note about checksums
//!
//! The "Snappy framed" format includes CRC-32C checksums, which can be
//! computed relatively efficiently on Intel processors.  However,
//! implementations disagree on the byte order of the checksums!  The Java
//! implementations, the [`snzip`][snzip] command-line tool and the
//! SmallTalk implementation use one byte order, and the Node.js and Python
//! implementations use another.
//!
//! For now, we have chosen to generate checksums in the format used by the
//! Java and `snzip` libraries. When reading, we can either verify
//! Java-format checksums, or we can ignore the checksums entirely.
//!
//! ### Limitations
//!
//! This library is still a work in progress:
//!
//! - Invalid streams will probably result in a panic.
//! - Decompression performance has been tuned a fair bit, except for CRCs,
//!   but there's probably an extra 25% or so to be gained by further
//!   tweaking.
//! - We currently assume that you will `write` data in large blocks when
//!   compressing, and we will generate poorly-compressed data if you make
//!   lots of small writes.  This could be fixed by using an internal write
//!   buffer.
//!
//! [snappy]: http://code.google.com/p/snappy/
//! [framed]: http://code.google.com/p/snappy/source/browse/trunk/framing_format.txt
//! [snzip]: https://github.com/kubo/snzip
//! [flate2]: http://alexcrichton.com/flate2-rs/flate2/index.html

#![cfg_attr(feature = "unstable", feature(test))]
#![warn(missing_docs)]

extern crate crc;
#[cfg(test)] extern crate dribble;
#[macro_use] extern crate log;
extern crate snappy;
#[cfg(all(test, feature = "unstable"))] extern crate test;

mod consts;
#[cfg(test)] mod test_helpers;
mod buffer;
mod masked_crc;
pub mod read;
pub mod write;
