use std::cmp;
use std::error::Error;
use std::fs::File;
use std::io;
use std::u8;

use clap::Parser;

mod buffer;
use crate::buffer::Buffer;

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

struct Bgrep {
    cli: Cli,
}

impl Bgrep {
    fn new(cli: Cli) -> Bgrep {
        Bgrep { cli }
    }

    fn grep(&self) -> Result<(), Box<dyn Error>> {
        let filename = self.cli.file.to_str().unwrap_or("-");
        if filename == "-" {
            let mut f = io::stdin();
            self.grep_fd(&mut f)?;
        } else {
            let mut f = File::open(&self.cli.file)?;
            self.grep_fd(&mut f)?;
        }
        Ok(())
    }

    fn grep_fd(&self, f: &mut impl std::io::Read) -> Result<(), Box<dyn Error>> {
        let mut buffer = Buffer::new(BUFFER_SIZE);
        let pattern_bytes = decode_hex(&self.cli.pattern)?;
        let mut grep_ctr = 0;
        let mut eof = false;
        loop {
            let next_buffer = buffer.mut_buffer();
            let mut read_bytes = 0;
            loop {
                let n = f.read(&mut next_buffer[read_bytes..])?;
                if n == 0 {
                    eof = true;
                    buffer.eof_reached(read_bytes);
                    break;
                }
                read_bytes += n;
                if read_bytes == next_buffer.len() {
                    break;
                }
            }
            self.grep_buffer(&pattern_bytes, &buffer, grep_ctr);
            grep_ctr += buffer.active_size;
            if eof {
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
    bgrep.grep()?;
    Ok(())
}
