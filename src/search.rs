use crate::bgreperror::BgrepError;
use crate::buffer::Buffer;

pub trait Search {
    fn new(pat: &str) -> Result<Self, BgrepError>
    where
        Self: Sized;
    fn search(&self, data: &Buffer, offset: usize) -> Option<(usize, usize)>;
    fn max_pattern_len(&self) -> usize;
}

pub fn decode_hex(pattern_input: &str) -> Result<Vec<u8>, BgrepError> {
    if !pattern_input.is_ascii() {
        return Err(BgrepError(format!(
            "Hex pattern contains non-ascii characters: {}",
            pattern_input
        )));
    }
    let pattern = pattern_input.replace(" ", "");
    if pattern.len() % 2 != 0 {
        return Err(BgrepError(format!(
            "Hex pattern does not have even amount of characters: {}",
            pattern_input
        )));
    }
    (0..(pattern.len() / 2))
        .map(|i| {
            Ok(
                u8::from_str_radix(&pattern[(2 * i)..(2 * i + 2)], 16).map_err(|err| {
                    BgrepError(format!("Invalid hex pattern '{}': {}", pattern_input, err))
                })?,
            )
        })
        .collect()
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
}
