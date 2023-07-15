use std::cmp;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::u8;

use clap::Parser;

const BUFFER_SIZE: usize = 4 * 1024 * 1024;

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Hex pattern
    pattern: String,
    /// Path to file
    file: std::path::PathBuf,
    /// Print <N> bytes after the found pattern
    #[arg(short = 'A', long, default_value_t = 0, value_name = "N")]
    after: usize,
    /// Print <N> bytes before the found pattern
    #[arg(short = 'B', long, default_value_t = 0, value_name = "N")]
    before: usize,
    /// Print <N> bytes before and after the found pattern
    #[arg(short = 'C', long, default_value_t = 0, value_name = "N")]
    context: usize,
    /// Print filename along matches
    #[arg(short = 'H', long, default_value_t = false)]
    with_filename: bool,
    /// Suppress ASCII interpretation
    #[arg(long, default_value_t = false)]
    no_ascii: bool,
    /// Suppress 0-based offset of matched bytes
    #[arg(long, default_value_t = false)]
    no_offset: bool,
}

#[derive(Debug)]
struct BgrepError {
    desc: String,
}

impl BgrepError {
    fn new(desc: &str) -> BgrepError {
        BgrepError {
            desc: String::from(desc),
        }
    }
}

impl std::fmt::Display for BgrepError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl Error for BgrepError {}

fn decode_hex(pattern_input: &String) -> Result<Vec<u8>, Box<dyn Error>> {
    if !pattern_input.is_ascii() {
        println!("Pattern contains non-ascii characters: {}", pattern_input);
        return Err(Box::new(BgrepError::new(
            "Pattern contains non-ascii characters",
        )));
    }
    let pattern = pattern_input.replace(" ", "");
    if pattern.len() % 2 != 0 {
        println!(
            "Pattern does not have even amount of characters: {}",
            pattern_input
        );
        return Err(Box::new(BgrepError::new(
            "Pattern does not have even amount of characters",
        )));
    }
    (0..(pattern.len() / 2))
        .map(|i| Ok(u8::from_str_radix(&pattern[(2 * i)..(2 * i + 2)], 16)?))
        .collect()
}

fn hex_string(buffer: &Vec<u8>) -> String {
    let mut result = String::new();
    for x in buffer {
        result.push_str(format!("{x:02x}").as_str());
    }
    result
}

fn ascii_interpretation(buffer: &Vec<u8>) -> String {
    let mut ascii = String::new();
    for x in buffer {
        if *x >= 0x20 && *x <= 0x7e {
            ascii.push(*x as char);
        } else {
            ascii.push('.');
        }
    }
    ascii
}

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

struct Bgrep {
    cli: Cli,
}

impl Bgrep {
    fn new(cli: Cli) -> Bgrep {
        Bgrep {
            cli,
        }
    }

    fn grep_file(&self) -> Result<(), Box<dyn Error>> {
        let mut f = File::open(&self.cli.file)?;
        let mut buffer = Buffer::new(BUFFER_SIZE);
        let pattern_bytes = decode_hex(&self.cli.pattern)?;

        let mut read_ctr = 0;
        let mut grep_ctr = 0;
        let filesize = f.metadata()?.len();

        loop {
            let remaining = (filesize - read_ctr) as usize;
            let mut next_buffer = buffer.mut_buffer(remaining);
            let read_size = next_buffer.len();
            f.read_exact(&mut next_buffer)?;
            self.grep_buffer(&pattern_bytes, &buffer, grep_ctr);
            read_ctr += read_size as u64;
            grep_ctr += buffer.active_size;
            //println!("read_ctr {read_ctr}; grep_ctr {grep_ctr}, filesize {filesize}; read_size {read_size}");
            if read_ctr >= filesize {
                break;
            }
        }
        Ok(())
    }

    fn grep_buffer(&self, pattern: &Vec<u8>, buf: &Buffer, offset: usize) {
        //println!("active size: {}", buf.active_size);
        for i in 0..buf.active_size {
            let mut matched = true;
            for (j, c_pattern) in pattern.iter().enumerate() {
                if let Some(c_buf) = buf.at((i + j) as i32) {
                    //println!("Comparing {:02x} with {:02x}", c_buf, c_pattern);
                    if c_buf != *c_pattern {
                        matched = false;
                        break;
                    }
                } else {
                    return;
                }
            }
            if matched {
                let first = cmp::max((i - self.cli.before) as i32, buf.min_index);
                let last = cmp::min((i + pattern.len() + self.cli.after) as i32, buf.max_index);
                if let Some(result) = buf.view(first, last) {
                    self.print_result(offset + i, &result);
                }
            }
        }
    }

    fn print_result(&self, address: usize, buffer: &Vec<u8>) {
        let mut output = String::new();
        if self.cli.with_filename {
            output.push_str(self.cli.file.to_str().unwrap_or("(path not UTF8)"));
            output.push(' ');
        }
        if !self.cli.no_offset {
            output.push_str(format!("{:08x}: ", address).as_str());
        }
        output.push_str(&hex_string(buffer));
        if !self.cli.no_ascii {
            output.push_str("  ");
            output.push_str(&ascii_interpretation(buffer));
        }
        println!("{output}");
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut cli = Cli::parse();
    cli.before = cmp::max(cli.before, cli.context);
    cli.after = cmp::max(cli.after, cli.context);
    let bgrep = Bgrep::new(cli);
    bgrep.grep_file()?;
    Ok(())
}
