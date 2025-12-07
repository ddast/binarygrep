use crate::bgreperror::BgrepError;
use crate::buffer::Buffer;
use crate::search::decode_hex;
use crate::search::Search;

pub struct SimpleSearch {
    pattern: Vec<u8>,
}

impl Search for SimpleSearch {
    fn new(pattern: &str) -> Result<SimpleSearch, BgrepError> {
        Ok(SimpleSearch {
            pattern: decode_hex(pattern)?,
        })
    }

    fn search(&self, data: &Buffer, offset: usize) -> Vec<(usize, usize)> {
        let mut result = vec![];
        if self.pattern.len() == 0 {
            return result;
        }

        if offset >= data.active_size {
            return result;
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
                    return result;
                }
            }
            if matched {
                result.push((i, self.pattern.len()));
            }
        }
        return result;
    }

    fn max_pattern_len(&self) -> usize {
        self.pattern.len()
    }
}
