Usage: binarygrep [OPTIONS] <PATTERN> [FILE]...

Arguments:
  <PATTERN>  Pattern as hexadecimal string
  [FILE]...  Search for PATTERN in each file. "-" is standard input [default: -]

Options:
  -r, --recursive      Search in all files recursively, symbolic links are followed
  -x, --extended       Enable extended search patterns (see below for syntax)
  -A, --after <N>      Print <N> bytes after the found pattern [default: 0]
  -B, --before <N>     Print <N> bytes before the found pattern [default: 0]
  -C, --context <N>    Print <N> bytes before and after the found pattern [default: 0]
  -H, --with-filename  Print filename along matches (default for multiple files)
      --no-filename    Do not print filename along matches (default for single file)
      --no-ascii       Suppress ASCII interpretation in output
      --no-offset      Suppress 0-based offset of matched bytes in output
  -h, --help           Print help
  -V, --version        Print version


Extended patterns consist of:
- Bytes in hexadecimal notation
- The wildcard character matching an arbitrary single byte: .
- Character sets: [02,ac,77] (either 0x02, 0xac or 0x77)
- Quantifiers: 03{5} (five times 0x03), 03{2,5} (two till five times 0x03)
- Spaces since they are always ignored
Example: 00{10} .{1,3} [00,FF]{2,3} AA BB
