use std::cmp;
use std::fs;
use std::io;
use std::path::Path;
use std::u8;

use clap::Parser;
use colored::Colorize;

mod buffer;
use crate::buffer::Buffer;

mod search;
use search::Search;

mod bmsearch;
use bmsearch::BoyerMooreSearch;

mod simplesearch;
use simplesearch::SimpleSearch;

mod extendedsearch;
use extendedsearch::ExtendedSearch;

mod bgreperror;
pub use bgreperror::BgrepError;

const BUFFER_SIZE: usize = 4 * 1024 * 1024;

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Pattern as hexadecimal string
    pattern: String,
    /// Search for PATTERN in each file. "-" is standard input.
    #[arg(default_values_t = ["-".to_string()])]
    file: Vec<String>,
    /// Search in all files recursively, symbolic links are followed
    #[arg(short = 'r', long)]
    recursive: bool,
    /// Allow extended search patterns
    #[arg(short = 'x', long)]
    extended: bool,
    /// Use simple search algorithm
    #[arg(long)]
    simple_search: bool,
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

struct Bgrep<T: Search> {
    recursive: bool,
    after: usize,
    before: usize,
    with_filename: bool,
    no_ascii: bool,
    no_offset: bool,
    search: T,
}

impl<T: Search> Bgrep<T> {
    fn new(cli: &Cli) -> Result<Bgrep<T>, BgrepError> {
        let multiple_files = cli.file.len() > 1 || cli.recursive;
        Ok(Bgrep {
            recursive: cli.recursive,
            after: cmp::max(cli.after, cli.context),
            before: cmp::max(cli.before, cli.context),
            with_filename: (multiple_files && !cli.no_filename)
                || (!multiple_files && cli.with_filename),
            no_ascii: cli.no_ascii,
            no_offset: cli.no_offset,
            search: T::new(&cli.pattern)?,
        })
    }

    fn grep(&self, file: &str) -> Result<(), BgrepError> {
        if file == "-" {
            let mut f = io::stdin();
            self.grep_fd(&file, &mut f)?;
        } else {
            let path = Path::new(&file);
            self.grep_path(&path)?;
        }
        Ok(())
    }

    fn grep_path(&self, path: &Path) -> Result<(), BgrepError> {
        if path.is_dir() {
            if !self.recursive {
                return Err(BgrepError(format!(
                    "Error: '{}' is a directory",
                    &path.to_str().unwrap()
                )));
            }
            for entry in fs::read_dir(path)
                .map_err(|err| BgrepError(format!("Error while reading directory: {}", err)))?
            {
                let entry_path = entry
                    .map_err(|err| BgrepError(format!("Error accessing directory entry: {}", err)))?
                    .path();
                self.grep_path(&entry_path)?;
            }
        } else {
            let mut f = std::fs::File::open(path).map_err(|err| {
                BgrepError(format!(
                    "Cannot open file '{}': {}",
                    &path.to_str().unwrap(),
                    err
                ))
            })?;
            self.grep_fd(path.to_str().unwrap(), &mut f)?;
        }
        Ok(())
    }

    fn grep_fd(&self, filename: &str, f: &mut impl std::io::Read) -> Result<(), BgrepError> {
        let buffer_size = cmp::max(
            BUFFER_SIZE,
            self.search.max_pattern_len() + cmp::max(self.after, self.before),
        );
        let mut buffer = Buffer::new(buffer_size);
        let mut grep_ctr = 0;
        loop {
            buffer
                .read(f)
                .map_err(|err| BgrepError(format!("Error while reading: {}", err)))?;
            self.grep_buffer(&buffer, grep_ctr, &filename);
            grep_ctr += buffer.active_size;
            if buffer.is_eof() {
                break;
            }
        }
        Ok(())
    }

    fn grep_buffer(&self, buf: &Buffer, offset: usize, filename: &str) {
        let matches = self.search.search(buf, 0);
        for (i, match_len) in matches {
            let res_start = i as isize;
            let res_end = (i + match_len) as isize;
            let before_start = cmp::max((i - self.before) as isize, buf.min_index);
            let after_end = cmp::min((i + match_len + self.after) as isize, buf.max_index);
            if let (Some(before), Some(result), Some(after)) = (
                buf.view(before_start, res_start),
                buf.view(res_start, res_end),
                buf.view(res_end, after_end),
            ) {
                self.print_result(&filename, offset + i, &before, &result, &after);
            }
        }
    }

    fn print_result(
        &self,
        file: &str,
        address: usize,
        before: &Vec<u8>,
        result: &Vec<u8>,
        after: &Vec<u8>,
    ) {
        let filename = if self.with_filename { &file } else { "" };
        let offset = if self.no_offset {
            String::new()
        } else {
            format!("{:08x}", address)
        };
        let hex_before = &encode_hex(before);
        let hex_result = &encode_hex(result);
        let hex_after = &encode_hex(after);
        let ascii_before = if self.no_ascii {
            String::new()
        } else {
            ascii_interpretation(before)
        };
        let ascii_result = if self.no_ascii {
            String::new()
        } else {
            ascii_interpretation(result)
        };
        let ascii_after = if self.no_ascii {
            String::new()
        } else {
            ascii_interpretation(after)
        };
        println!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            filename.cyan(),
            if filename.is_empty() { "" } else { " " },
            offset.bold(),
            if offset.is_empty() { "" } else { ": " },
            hex_before,
            hex_result.magenta(),
            hex_after,
            if self.no_ascii { "" } else { "  " },
            ascii_before,
            ascii_result.magenta(),
            ascii_after
        );
    }
}

pub fn run() -> Result<(), BgrepError> {
    let cli = Cli::parse();
    if cli.extended {
        return run2::<ExtendedSearch>(&cli);
    } else if cli.simple_search {
        return run2::<SimpleSearch>(&cli);
    } else {
        return run2::<BoyerMooreSearch>(&cli);
    }
}

fn run2<T: Search>(cli: &Cli) -> Result<(), BgrepError> {
    let bgrep: Bgrep<T> = Bgrep::new(&cli)?;
    for file in &cli.file {
        bgrep.grep(&file)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
