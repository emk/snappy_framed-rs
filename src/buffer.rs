use std::iter::repeat;
use std::ptr::{copy, copy_nonoverlapping};

/// An I/O buffer with various convenience functions.  This is an internal
/// class, and the API is subject to change.
pub struct Buffer {
    /// Our data.
    buffer: Vec<u8>,
    /// The start of the unread data in the buffer.
    begin: usize,
    /// The end of the unread data in the buffer.
    end: usize
}

/// Regular Buffer interface.
impl Buffer {
    pub fn new(sz: usize) -> Buffer {
        Buffer{buffer: vec![0; sz], begin: 0, end: 0}
    }
    
    pub fn capacity(&self) -> usize { self.buffer.len() }
    pub fn buffered(&self) -> usize { self.end - self.begin }
    pub fn empty(&self) -> bool { self.buffered() == 0 }

    pub fn move_data_to_start(&mut self) {
        if self.begin > 0 {
            unsafe {
                copy(self.buffer.as_ptr().offset(self.begin as isize),
                     self.buffer.as_mut_ptr(),
                     self.buffered());
            }
            self.end = self.buffered();
            self.begin = 0;
        }
    }

    pub fn space_to_fill(&mut self) -> &mut [u8] {
        &mut self.buffer[self.end..]
    }

    pub fn set_data(&mut self, data: &[u8]) {
        assert!(data.len() <= self.buffer.len());
        unsafe {
            // HOTSPOT: Slow copies here have a drastic impact on performance.
            copy_nonoverlapping(data.as_ptr(),
                                self.buffer.as_mut_ptr(),
                                data.len());
        }
        self.begin = 0;
        self.end = data.len();
    }

    pub fn added(&mut self, bytes: usize) {
        self.end += bytes;
    }

    pub fn consume(&mut self, bytes: usize) -> &[u8] {
        let result = &self.buffer[self.begin..self.begin+bytes];
        self.begin += bytes;
        result
    }

    pub fn copy_out_and_consume(&mut self, bytes: usize, dest: &mut [u8]) {
        assert!(bytes <= dest.len());
        unsafe {
            // HOTSPOT: Slow copies here have a drastic impact on performance.
            copy_nonoverlapping(self.buffer.as_ptr().offset(self.begin as isize),
                                dest.as_mut_ptr(),
                                bytes);
        }
        self.begin += bytes;
    }

    pub fn add_capacity(&mut self, bytes: usize) {
        self.buffer.extend(repeat(0).take(bytes));
    }
}
