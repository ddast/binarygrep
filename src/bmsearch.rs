// Boyer-Moore string search
// Translated C implementation from
// https://en.wikipedia.org/wiki/Boyer%E2%80%93Moore_string-search_algorithm

use crate::bgreperror::BgrepError;
use crate::buffer::Buffer;
use crate::search::Search;
use crate::search::decode_hex;

const ALPHABET_LEN: usize = 256;

pub struct BoyerMooreSearch {
    delta1: Vec<isize>,
    delta2: Vec<isize>,
    pat: Vec<u8>,
}

impl BoyerMooreSearch {
    fn search_next(&self, data: &Buffer, offset: usize) -> Option<(usize, usize)> {
        let patlen = self.pat.len();

        if patlen == 0 {
            return None;
        }

        if offset >= data.active_size {
            return None;
        }

        let mut i = offset as isize + patlen as isize - 1;
        while i < (data.active_size + patlen - 1) as isize {
            let mut j = patlen as isize - 1;
            if data.at(i).is_none() {
                break;
            }
            while j >= 0 {
                if data.at(i).unwrap() == self.pat[j as usize] {
                    i -= 1;
                    j -= 1;
                } else {
                    break;
                }
            }
            if j < 0 {
                return Some(((i + 1) as usize, patlen));
            }
            let shift = std::cmp::max(
                self.delta1[data.at(i).unwrap() as usize],
                self.delta2[j as usize],
            );
            i += shift;
        }

        None
    }
}

impl Search for BoyerMooreSearch {
    fn new(pattern_hex: &str) -> Result<BoyerMooreSearch, BgrepError> {
        let pat = decode_hex(pattern_hex)?;
        let mut delta1 = vec![pat.len() as isize; ALPHABET_LEN];
        let mut delta2 = vec![0; pat.len()];
        make_delta1(&mut delta1, &pat);
        make_delta2(&mut delta2, &pat);
        Ok(BoyerMooreSearch {
            delta1,
            delta2,
            pat,
        })
    }

    fn search(&self, data: &Buffer, offset: usize) -> Vec<(usize, usize)> {
        let mut start_at = offset;
        let mut result = vec![];
        loop {
            if let Some((i, match_len)) = self.search_next(data, start_at) {
                result.push((i, match_len));
                start_at = i + 1;
            } else {
                return result;
            }
        }
    }

    fn max_pattern_len(&self) -> usize {
        self.pat.len()
    }
}

fn make_delta1(delta1: &mut [isize], pat: &[u8]) {
    for i in 0..pat.len() {
        delta1[pat[i] as usize] = pat.len() as isize - 1 - i as isize;
    }
}

fn is_prefix(word: &[u8], pos: isize) -> bool {
    let suffixlen = word.len() - pos as usize;
    for i in 0..suffixlen {
        if word[i] != word[pos as usize + i] {
            return false;
        }
    }
    true
}

fn suffix_length(word: &[u8], pos: isize) -> usize {
    for i in 0..pos + 1 {
        if word[(pos - i) as usize] != word[word.len() - 1 - i as usize] {
            return i as usize;
        }
    }
    pos as usize
}

fn make_delta2(delta2: &mut [isize], pat: &[u8]) {
    let mut last_prefix_index = 1;

    for p in (0..pat.len() as isize).rev() {
        if is_prefix(pat, p + 1) {
            last_prefix_index = p + 1;
        }
        delta2[p as usize] = last_prefix_index + (pat.len() as isize - 1 - p);
    }

    for p in 0..pat.len() as isize - 1 {
        let slen = suffix_length(pat, p);
        if pat[p as usize - slen] != pat[pat.len() - 1 - slen] {
            delta2[pat.len() - 1 - slen] = pat.len() as isize - 1 - p + slen as isize;
        }
    }
}
