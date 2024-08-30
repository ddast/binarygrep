use std::io;

#[derive(PartialEq, Eq)]
enum BufferState {
    Uninitialised,
    InitialisationPending,
    Initialised,
    EndOfFile,
}

/// Buffer that stores some bytes before and after the current byte view
pub struct Buffer {
    pub active_size: usize,
    pub min_index: isize,
    pub max_index: isize,
    buffer: Vec<u8>,
    root_index: usize,
    size: usize,
    state: BufferState,
    is_eof: bool,
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
            is_eof: false,
        }
    }

    /// Read bytes from `f` till the internal buffer is filled or EOF is reached
    pub fn read(&mut self, f: &mut impl std::io::Read) -> io::Result<usize> {
        let next_buffer = self.mut_buffer();
        let mut read_bytes = 0;
        loop {
            let n = f.read(&mut next_buffer[read_bytes..])?;
            if n == 0 {
                self.is_eof = true;
                self.eof_reached(read_bytes);
                break;
            }
            read_bytes += n;
            if read_bytes == next_buffer.len() {
                break;
            }
        }
        Ok(read_bytes)
    }

    /// Is the buffer in EOF state
    pub fn is_eof(&self) -> bool {
        self.is_eof
    }

    /// Returns the value at offset `i` if this is a valid index
    pub fn at(&self, i: isize) -> Option<u8> {
        if i < self.min_index || i >= self.max_index {
            return None;
        }
        let actual_index = self.get_absolute_index(i);
        Some(self.buffer[actual_index])
    }

    /// Return a view of the buffer from index `first` to `last`
    pub fn view(&self, first: isize, last: isize) -> Option<Vec<u8>> {
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
        Some(self.buffer[actual_index_first..actual_index_last].to_vec())
    }

    /// Returns a writeable buffer
    ///
    /// The intended usage is to repeatedly call this function to get a mutable buffer and fill
    /// the complete returned buffer with data.
    ///
    /// If the buffer cannot be filled comletely this should be signalled by calling `eof_reached()`
    /// with the bytes written in the last operation.
    ///
    /// Internally this keeps track of three buffer regions: PREVIOUS, CURRENT and NEXT.
    ///
    /// Initially all buffers are empty.  The first call will fill CURRENT and (possibly) NEXT.
    /// All subsequent calls will drop PREVIOUS, move CURRENT TO PREVIOUS, move NEXT to CURRENT and
    /// return the NEXT buffer for filling.
    fn mut_buffer(&mut self) -> &mut [u8] {
        let begin;
        let end;
        match self.state {
            BufferState::Uninitialised => {
                self.root_index = 0;
                begin = self.size;
                end = begin + 2 * self.size;
                self.min_index = 0;
                self.max_index = (end - begin) as isize;
                self.state = BufferState::InitialisationPending;
            }
            BufferState::InitialisationPending | BufferState::Initialised => {
                self.root_index = (self.root_index + self.size) % (3 * self.size);
                begin = (self.root_index + 2 * self.size) % (3 * self.size);
                end = begin + self.size;
                self.min_index = -(self.size as isize);
                self.max_index = (self.size + end - begin) as isize;
                self.state = BufferState::Initialised;
            }
            BufferState::EndOfFile => {
                self.root_index = 0;
                begin = 0;
                end = 0;
                self.min_index = 0;
                self.max_index = 0;
            }
        }
        self.active_size = self.size;
        &mut self.buffer[begin..end]
    }

    /// Signal that EOF has been reached and the last chunk has `remaining`  bytes.  No further
    /// data can be written to the buffer afterwards.
    fn eof_reached(&mut self, remaining: usize) {
        match self.state {
            BufferState::Uninitialised => self.active_size = 0, // this should not happen
            BufferState::InitialisationPending => self.active_size = remaining,
            BufferState::Initialised => self.active_size = self.size + remaining,
            BufferState::EndOfFile => self.active_size = 0,
        }
        self.state = BufferState::EndOfFile;
        self.max_index = self.active_size as isize;
    }

    /// Transforms an index relative to the current buffer to the real index of the buffer
    fn get_absolute_index(&self, i: isize) -> usize {
        let absolute_index;
        if i >= 0 {
            absolute_index = (self.root_index + self.size + (i as usize)) % (3 * self.size);
        } else {
            absolute_index = (self.root_index + self.size - (-i as usize)) % (3 * self.size);
        }
        absolute_index
    }
}
