use std::iter::repeat;

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
            // TODO: Replace with a fast memmove routine.
            for i in 0..self.buffered() {
                self.buffer[i] = self.buffer[self.begin+i];
            }
            self.end = self.buffered();
            self.begin = 0;
        }
    }

    pub fn space_to_fill(&mut self) -> &mut [u8] {
        &mut self.buffer[self.end..]
    }

    pub fn set_data(&mut self, data: &[u8]) {
        // TODO: Replace with a memory copy routine.
        for i in 0..data.len() {
            self.buffer[i] = data[i];
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
        // TODO: Replace with a fast memcopy routine.
        for i in 0..bytes {
            dest[i] = self.buffer[self.begin+i];
        }
        self.begin += bytes;
    }

    pub fn add_capacity(&mut self, bytes: usize) {
        self.buffer.extend(repeat(0).take(bytes));
    }
}
