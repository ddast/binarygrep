use crate::bgreperror::BgrepError;
use crate::buffer::Buffer;
use crate::search::decode_hex;
use crate::search::Search;

pub struct ExtendedSearch {
    pattern: Vec<u8>,
}

impl Search for ExtendedSearch {
    fn new(pattern: &str) -> Result<ExtendedSearch, BgrepError> {
        Ok(ExtendedSearch {
            pattern: decode_hex(pattern)?,
        })
    }

    fn search(&self, data: &Buffer, offset: usize) -> Option<(usize, usize)> {
        if self.pattern.len() == 0 {
            return None;
        }

        if offset >= data.active_size {
            return None;
        }

        for i in offset..data.active_size {
            let mut matched = true;
            for (j, c_pattern) in self.pattern.iter().enumerate() {
                if let Some(c_buf) = data.at((i + j) as isize) {
                    if c_buf != *c_pattern {
                        matched = false;
                        break;
                    }
                } else {
                    return None;
                }
            }
            if matched {
                return Some((i, self.pattern.len()));
            }
        }
        return None;
    }

    fn max_pattern_len(&self) -> usize {
        self.pattern.len()
    }
}
