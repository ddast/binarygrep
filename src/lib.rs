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
    /// Pattern as hexadecimal string
    pattern: String,
    /// Search for PATTERN in each file. "-" is standard input.
    #[arg(default_values_t = ["-".to_string()])]
    file: Vec<String>,
    /// Print <N> bytes after the found pattern
    #[arg(short = 'A', long, default_value_t = 0, value_name = "N")]
    after: usize,
    /// Print <N> bytes before the found pattern
    #[arg(short = 'B', long, default_value_t = 0, value_name = "N")]
    before: usize,
    /// Print <N> bytes before and after the found pattern
    #[arg(short = 'C', long, default_value_t = 0, value_name = "N")]
    context: usize,
    /// Print filename along matches (default for multiple files)
    #[arg(short = 'H', long)]
    with_filename: bool,
    /// Do not print filename along matches (default for single file)
    #[arg(long)]
    no_filename: bool,
    /// Suppress ASCII interpretation in output
    #[arg(long, default_value_t = false)]
    no_ascii: bool,
    /// Suppress 0-based offset of matched bytes in output
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

fn encode_hex(buffer: &Vec<u8>) -> String {
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
    pattern_bytes: Vec<u8>,
    file: String,
    after: usize,
    before: usize,
    with_filename: bool,
    no_ascii: bool,
    no_offset: bool,
}

impl Bgrep {
    fn grep(&self) -> Result<(), Box<dyn Error>> {
        if self.file == "-" {
            let mut f = io::stdin();
            self.grep_fd(&mut f)?;
        } else {
            let mut f = File::open(&self.file)?;
            self.grep_fd(&mut f)?;
        }
        Ok(())
    }

    fn grep_fd(&self, f: &mut impl std::io::Read) -> Result<(), Box<dyn Error>> {
        let mut buffer = Buffer::new(BUFFER_SIZE);
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
            self.grep_buffer(&buffer, grep_ctr);
            grep_ctr += buffer.active_size;
            if eof {
                break;
            }
        }
        Ok(())
    }

    fn grep_buffer(&self, buf: &Buffer, offset: usize) {
        //println!("active size: {}", buf.active_size);
        for i in 0..buf.active_size {
            let mut matched = true;
            for (j, c_pattern) in self.pattern_bytes.iter().enumerate() {
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
                let first = cmp::max((i - self.before) as i32, buf.min_index);
                let last = cmp::min(
                    (i + self.pattern_bytes.len() + self.after) as i32,
                    buf.max_index,
                );
                if let Some(result) = buf.view(first, last) {
                    self.print_result(offset + i, &result);
                }
            }
        }
    }

    fn print_result(&self, address: usize, buffer: &Vec<u8>) {
        let mut output = String::new();
        if self.with_filename {
            output.push_str(&self.file);
            output.push(' ');
        }
        if !self.no_offset {
            output.push_str(format!("{:08x}: ", address).as_str());
        }
        output.push_str(&encode_hex(buffer));
        if !self.no_ascii {
            output.push_str("  ");
            output.push_str(&ascii_interpretation(buffer));
        }
        println!("{output}");
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    println!("{}, {}", cli.with_filename, cli.no_filename);
    let filecount = cli.file.len();
    for file in cli.file {
        let bgrep = Bgrep {
            file,
            pattern_bytes: decode_hex(&cli.pattern)?,
            after: cmp::max(cli.after, cli.context),
            before: cmp::max(cli.before, cli.context),
            with_filename: (filecount > 1 && !cli.no_filename)
                || (filecount == 1 && cli.with_filename),
            no_ascii: cli.no_ascii,
            no_offset: cli.no_offset,
        };
        bgrep.grep()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_hex_valid() {
        let hex_string = String::from("b081133bbf0cb70a288734");
        let expected_hexbytes: Vec<u8> = vec![
            0xb0, 0x81, 0x13, 0x3b, 0xbf, 0x0c, 0xb7, 0x0a, 0x28, 0x87, 0x34,
        ];
        match decode_hex(&hex_string) {
            Err(_) => assert!(false),
            Ok(hexbytes) => assert_eq!(hexbytes, expected_hexbytes),
        }
    }

    #[test]
    fn test_decode_hex_spaces() {
        let hex_string = String::from("b 08 11 33b bf0 cb7 0a28 8734");
        let expected_hexbytes: Vec<u8> = vec![
            0xb0, 0x81, 0x13, 0x3b, 0xbf, 0x0c, 0xb7, 0x0a, 0x28, 0x87, 0x34,
        ];
        match decode_hex(&hex_string) {
            Err(_) => assert!(false),
            Ok(hexbytes) => assert_eq!(hexbytes, expected_hexbytes),
        }
    }

    #[test]
    fn test_decode_hex_invalid_length() {
        let hex_string = String::from("b081133bbf0cb70a28873");
        match decode_hex(&hex_string) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_decode_hex_invalid_characters() {
        let hex_string = String::from("b081133zbf0cb70a288734");
        match decode_hex(&hex_string) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_encode_hex_valid() {
        let hexbytes: Vec<u8> = vec![
            0xb0, 0x81, 0x13, 0x3b, 0xbf, 0x0c, 0xb7, 0x0a, 0x28, 0x87, 0x34,
        ];
        let expected_hexstring = String::from("b081133bbf0cb70a288734");
        assert_eq!(encode_hex(&hexbytes), expected_hexstring);
    }

    #[test]
    fn test_ascii_interpretation_valid() {
        let hexbytes: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67,
            68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
            90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108,
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142,
            143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159,
            160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176,
            177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193,
            194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210,
            211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227,
            228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244,
            245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
        ];
        let expected_ascii = "................................ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~.................................................................................................................................";
        assert_eq!(ascii_interpretation(&hexbytes), expected_ascii);
    }
}
