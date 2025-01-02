Usage: binarygrep [OPTIONS] <PATTERN> [FILE]...

Arguments:
  <PATTERN>  Pattern as hexadecimal string
  [FILE]...  Search for PATTERN in each file. "-" is standard input [default: -]

Options:
  -r, --recursive      Search in all files recursively, symbolic links are followed
  -x, --extended       Allow extended search patterns
      --simple-search  Use simple search algorithm
  -A, --after <N>      Print <N> bytes after the found pattern [default: 0]
  -B, --before <N>     Print <N> bytes before the found pattern [default: 0]
  -C, --context <N>    Print <N> bytes before and after the found pattern [default: 0]
  -H, --with-filename  Print filename along matches (default for multiple files)
      --no-filename    Do not print filename along matches (default for single file)
      --no-ascii       Suppress ASCII interpretation in output
      --no-offset      Suppress 0-based offset of matched bytes in output
  -h, --help           Print help
  -V, --version        Print version
