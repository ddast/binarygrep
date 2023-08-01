use std::cmp;

#[derive(PartialEq, Eq)]
enum BufferState {
    Uninitialised,
    Initialised,
}

/// Buffer that stores some bytes before and after the current byte view
pub struct Buffer {
    pub active_size: usize,
    buffer: Vec<u8>,
    root_index: usize,
    size: usize,
    pub min_index: i32,
    pub max_index: i32,
    state: BufferState,
}

impl Buffer {
    /// Create a new buffer with the given size
    ///
    /// The buffer will keep `size` previous bytes, `size` current bytes and `size` next bytes.
    pub fn new(size: usize) -> Buffer {
        Buffer {
            buffer: vec![0; 3 * size],
            root_index: 0,
            size,
            active_size: size,
            min_index: 0,
            max_index: 0,
            state: BufferState::Uninitialised,
        }
    }

    /// Returns a writeable buffer with at most `max_size` bytes
    ///
    /// The intended usage is to repeatedly call this function to get a mutable buffer and fill
    /// the complete returned buffer with data.
    ///
    /// Internally this keeps track of three buffer regions: PREVIOUS, CURRENT and NEXT.
    ///
    /// Initially all buffers are empty.  The first call will fill CURRENT and (possibly) NEXT.
    /// All subsequent calls will drop PREVIOUS, move CURRENT TO PREVIOUS, move NEXT to CURRENT and
    /// return the NEXT buffer for filling.
    ///
    /// If this function returns a buffer smaller than `max_size`, it can be called again to get
    /// the NEXT buffer.  If this function returns a buffer of size `max_size`, then further calls
    /// will only return a zero-size buffer.
    pub fn mut_buffer(&mut self, max_size: usize) -> &mut [u8] {
        let begin;
        let end;
        if self.state == BufferState::Uninitialised {
            self.root_index = 0;
            begin = self.size;
            end = begin + cmp::min(2 * self.size, max_size);
            self.min_index = 0;
            self.max_index = (end - begin) as i32;
            self.state = BufferState::Initialised;
        } else if self.state == BufferState::Initialised {
            self.root_index = (self.root_index + self.size) % (3 * self.size);
            begin = (self.root_index + 2 * self.size) % (3 * self.size);
            end = begin + cmp::min(self.size, max_size);
            self.min_index = -(self.size as i32);
            self.max_index = (self.size + end - begin) as i32;
        } else {
            self.root_index = 0;
            begin = 0;
            end = 0;
            self.min_index = 0;
            self.max_index = 0;
        }
        if end - begin == max_size {
            self.active_size = self.max_index as usize;
        } else {
            self.active_size = self.size;
        }
        //println!("self.size {}; self.root_index {}; begin {}; end {}; self.min_index {}; self.max_index {}",
        //         self.size, self.root_index, begin, end, self.min_index, self.max_index);
        &mut self.buffer[begin..end]
    }

    pub fn eof_reached(&mut self, remaining: usize) {
        if self.state == BufferState::Uninitialised {
            self.active_size = remaining;
        } else if self.state == BufferState::Initialised {
            self.active_size = self.size + remaining;
        } else {
            self.active_size = 0;
        }
        self.max_index = self.active_size as i32;
    }

    fn get_absolute_index(&self, i: i32) -> usize {
        let absolute_index;
        if i >= 0 {
            absolute_index = (self.root_index + self.size + (i as usize)) % (3 * self.size);
        } else {
            absolute_index = (self.root_index + self.size - (-i as usize)) % (3 * self.size);
        }
        absolute_index
    }

    /// Returns the value at offset `i` if this is a valid index
    pub fn at(&self, i: i32) -> Option<u8> {
        if i < self.min_index || i >= self.max_index {
            return None;
        }
        let actual_index = self.get_absolute_index(i);
        //println!("Accessing offset {i} via actual index {actual_index}");
        Some(self.buffer[actual_index])
    }

    pub fn view(&self, first: i32, last: i32) -> Option<Vec<u8>> {
        if first > last
            || first < self.min_index
            || first > self.max_index
            || last < self.min_index
            || last > self.max_index
        {
            return None;
        }
        let actual_index_first = self.get_absolute_index(first);
        let actual_index_last = self.get_absolute_index(last);
        //println!("first {actual_index_first}; last {actual_index_last}");
        Some(self.buffer[actual_index_first..actual_index_last].to_vec())
    }
}
