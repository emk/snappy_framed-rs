#![cfg_attr(feature = "unstable", feature(test))]

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
