use crate::buffer::Buffer;
use crate::search::Search;

pub struct SimpleSearch {
    pattern: Vec<u8>,
}

impl Search for SimpleSearch {
    fn new(pattern: Vec<u8>) -> SimpleSearch {
        SimpleSearch { pattern }
    }

    fn search(&self, data: &Buffer, offset: usize) -> Option<usize> {
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
                return Some(i);
            }
        }
        return None;
    }
}
