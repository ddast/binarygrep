use crate::bgreperror::BgrepError;
use crate::buffer::Buffer;
use crate::search::Search;
use std::str::FromStr;

pub struct ExtendedSearch {
    pattern: Vec<PatternEntry>,
}

#[derive(Debug, PartialEq)]
enum PatternChar {
    Value(Vec<u8>),
    Wildcard,
}

#[derive(Debug, PartialEq)]
struct PatternEntry {
    patternchar: PatternChar,
    min_cnt: usize,
    max_cnt: usize,
}

fn parse_extended(pattern_input: &str) -> Result<Vec<PatternEntry>, BgrepError> {
    if !pattern_input.is_ascii() {
        return Err(BgrepError(format!(
            "Pattern contains non-ascii characters: {}",
            pattern_input
        )));
    }
    let pattern_str = pattern_input.replace(" ", "");
    let pattern_char: Vec<char> = pattern_str.chars().collect();
    let mut i = 0;
    let mut result = Vec::new();
    while i < pattern_char.len() {
        let mut patternentry = PatternEntry {
            patternchar: PatternChar::Value(vec![]),
            min_cnt: 1,
            max_cnt: 1,
        };
        if pattern_char[i].is_ascii_hexdigit() {
            let consumed = parse_hex_byte(&pattern_str[i..], &mut patternentry)?;
            i += consumed;
        } else if pattern_char[i] == '.' {
            patternentry.patternchar = PatternChar::Wildcard;
            i += 1;
        } else if pattern_char[i] == '[' {
            let consumed = parse_character_set(&pattern_str[i..], &mut patternentry)?;
            i += consumed;
        } else {
            return Err(BgrepError(format!(
                "Unexpected charater at index {}: {}",
                i, pattern_char[i]
            )));
        }
        if i < pattern_char.len() && pattern_char[i] == '{' {
            let consumed = parse_quantifier(&pattern_str[i..], &mut patternentry)?;
            i += consumed;
        }
        result.push(patternentry);
    }
    Ok(result)
}

fn parse_quantifier(pattern: &str, entry: &mut PatternEntry) -> Result<usize, BgrepError> {
    if let Some(end) = pattern.find('}') {
        for (i, val) in pattern[1..end].split(',').enumerate() {
            let cnt = usize::from_str(val).map_err(|err| {
                BgrepError(format!(
                    "Invalid decimal value in quantifier '{}': {}",
                    pattern, err
                ))
            })?;
            if i == 0 {
                entry.min_cnt = cnt;
                entry.max_cnt = cnt;
            } else if i == 1 {
                entry.max_cnt = cnt;
            } else {
                return Err(BgrepError(String::from(
                    "Quantifiers must contain only one or two values",
                )));
            }
        }
        if entry.min_cnt > entry.max_cnt {
            return Err(BgrepError(format!(
                "Min quantifier {} larger than max quantifier {}.",
                entry.min_cnt, entry.max_cnt
            )));
        }

        Ok(end + 1)
    } else {
        Err(BgrepError(String::from(
            "Incomplete quantifier.  Missing }",
        )))
    }
}

fn parse_character_set(pattern: &str, entry: &mut PatternEntry) -> Result<usize, BgrepError> {
    if let Some(end) = pattern.find(']') {
        for val in pattern[1..end].split(',') {
            if val.len() != 2 {
                return Err(BgrepError(format!(
                    "Invalid entry in character set: {}",
                    val
                )));
            }
            _ = parse_hex_byte(val, entry)?;
        }
        Ok(end + 1)
    } else {
        Err(BgrepError(String::from("Incomplete set.  Missing ]")))
    }
}

fn parse_hex_byte(pattern: &str, entry: &mut PatternEntry) -> Result<usize, BgrepError> {
    if pattern.len() < 2 {
        return Err(BgrepError(String::from(
            "Format error: Digit character not followed by another digit character",
        )));
    }
    let val = u8::from_str_radix(&pattern[0..2], 16)
        .map_err(|err| BgrepError(format!("Invalid hex pattern '{}': {}", pattern, err)))?;
    if let PatternChar::Value(charset) = &mut entry.patternchar {
        charset.push(val);
    } else {
        return Err(BgrepError(String::from("Could not push to charset")));
    }
    Ok(2)
}

fn search_single_pattern(
    data: &Buffer,
    offset: usize,
    pattern: &[PatternEntry],
    cnt: &[usize],
) -> Vec<(usize, usize)> {
    let mut result = vec![];
    for i in offset..data.active_size {
        let mut matched = true;
        let mut processed = 0;
        'pattern_loop: for (j, patternentry) in pattern.iter().enumerate() {
            for _ in 0..cnt[j] {
                if let Some(c_buf) = data.at((i + processed) as isize) {
                    processed += 1;
                    if let PatternChar::Value(charset) = &patternentry.patternchar
                        && !charset.contains(&c_buf)
                    {
                        matched = false;
                        break 'pattern_loop;
                    }
                    // else patternentry.patternchar must be PatternChar::Wildcard, and therefore,
                    // we do nothing and continue since this is equal to having a match
                } else {
                    return result;
                }
            }
        }
        if matched {
            result.push((i, cnt.iter().sum()));
        }
    }
    result
}

impl Search for ExtendedSearch {
    fn new(pattern: &str) -> Result<ExtendedSearch, BgrepError> {
        Ok(ExtendedSearch {
            pattern: parse_extended(pattern)?,
        })
    }

    fn search(&self, data: &Buffer, offset: usize) -> Vec<(usize, usize)> {
        let mut result = vec![];
        if self.pattern.is_empty() {
            return result;
        }

        if offset >= data.active_size {
            return result;
        }

        // To iterate over all combinations of repeated characters, the currently selected
        // combination is tracked in `cnt`.  This vector is increased in the following loop to
        // contain all possible allowed combinations.
        let mut cnt: Vec<usize> = self.pattern.iter().map(|p| p.min_cnt).collect();
        'cnt_loop: loop {
            result.append(&mut search_single_pattern(
                data,
                offset,
                &self.pattern,
                &cnt,
            ));
            for i in 0..self.pattern.len() {
                if cnt[i] < self.pattern[i].max_cnt {
                    cnt[i] += 1;
                    break;
                } else {
                    if i == self.pattern.len() - 1 {
                        break 'cnt_loop;
                    }
                    cnt[i] = self.pattern[i].min_cnt;
                }
            }
        }
        result
    }

    fn max_pattern_len(&self) -> usize {
        let mut count = 0;
        for p in &self.pattern {
            count += p.max_cnt;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let input = String::from("630f29.5bde0881b7");
        let expected = vec![
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x63]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x0f]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x29]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Wildcard,
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x5b]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0xde]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x08]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x81]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0xb7]),
                min_cnt: 1,
                max_cnt: 1,
            },
        ];
        match parse_extended(&input) {
            Err(_) => assert!(false),
            Ok(result) => assert_eq!(result, expected),
        }
    }

    #[test]
    fn test_parse_valid_quantifier() {
        let input = String::from("63{2}0f{1,1}29{4}.{5,10}5bde{3,4}0881b7{7,12}");
        let expected = vec![
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x63]),
                min_cnt: 2,
                max_cnt: 2,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x0f]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x29]),
                min_cnt: 4,
                max_cnt: 4,
            },
            PatternEntry {
                patternchar: PatternChar::Wildcard,
                min_cnt: 5,
                max_cnt: 10,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x5b]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0xde]),
                min_cnt: 3,
                max_cnt: 4,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x08]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x81]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0xb7]),
                min_cnt: 7,
                max_cnt: 12,
            },
        ];
        match parse_extended(&input) {
            Err(_) => assert!(false),
            Ok(result) => assert_eq!(result, expected),
        }
    }

    #[test]
    fn test_parse_valid_character_set() {
        let input = String::from("[63,0f,29].5b[de]08[81,b7]");
        let expected = vec![
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x63, 0x0f, 0x29]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Wildcard,
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x5b]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0xde]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x08]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x81, 0xb7]),
                min_cnt: 1,
                max_cnt: 1,
            },
        ];
        match parse_extended(&input) {
            Err(_) => assert!(false),
            Ok(result) => assert_eq!(result, expected),
        }
    }

    #[test]
    fn test_parse_valid_combined() {
        let input = String::from("[63,0f,29]{3,10}.{2}5b[de]{7,20}08{2}[81,b7]{3,9}");
        let expected = vec![
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x63, 0x0f, 0x29]),
                min_cnt: 3,
                max_cnt: 10,
            },
            PatternEntry {
                patternchar: PatternChar::Wildcard,
                min_cnt: 2,
                max_cnt: 2,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x5b]),
                min_cnt: 1,
                max_cnt: 1,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0xde]),
                min_cnt: 7,
                max_cnt: 20,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x08]),
                min_cnt: 2,
                max_cnt: 2,
            },
            PatternEntry {
                patternchar: PatternChar::Value(vec![0x81, 0xb7]),
                min_cnt: 3,
                max_cnt: 9,
            },
        ];
        match parse_extended(&input) {
            Err(_) => assert!(false),
            Ok(result) => assert_eq!(result, expected),
        }
    }

    #[test]
    fn test_parse_invalid_length() {
        let input = String::from("[63,0f,29]{3,10}.{2}5bf[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_hex_char() {
        let input = String::from("[63,0f,29]{3,10}.{2}5z[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_quantifier_empty() {
        let input = String::from("[63,0f,29]{3,10}.{}5f[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_quantifier_minempty() {
        let input = String::from("[63,0f,29]{3,10}.{,2}5f[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_quantifier_maxempty() {
        let input = String::from("[63,0f,29]{3,10}.{2,}5f[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_quantifier_too_many() {
        let input = String::from("[63,0f,29]{3,10}.{2,3,4}5f[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_quantifier_min_greater_max() {
        let input = String::from("[63,0f,29]{10,3}.{2}5f[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_quantifier_missing_close_bracket() {
        let input = String::from("[63,0f,29]{10,3.{2}5f[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_quantifier_missing_open_bracket() {
        let input = String::from("[63,0f,29]10,3}.{2}5f[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_set_missing_comma() {
        let input = String::from("[63,0f29]{3,10}.{2}5b[de]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_set_empty_entry() {
        let input = String::from("[63,0f,29]{3,10}.{2}5b[de,]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_set_non_hex() {
        let input = String::from("[63,0f,29]{3,10}.{2}5b[dz,]{7,20}08{2}[81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_set_missing_close_bracket() {
        let input = String::from("[63,0f,29]{3,10}.{2}5b[dz,]{7,20}08{2}[81,b7{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_invalid_set_missing_open_bracket() {
        let input = String::from("[63,0f,29]{3,10}.{2}5b[dz,]{7,20}08{2}81,b7]{3,9}");
        match parse_extended(&input) {
            Err(_) => (),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn test_max_length() {
        let input = String::from("[63,0f,29]{3,10}.{2}5b[de]{7,20}08{2}[81,b7]{3,9}");
        if let Ok(extendedsearch) = ExtendedSearch::new(&input) {
            assert_eq!(extendedsearch.max_pattern_len(), 44);
        } else {
            assert!(false);
        }
    }
}
