// Boyer-Moore string search
// Translated C implementation from
// https://en.wikipedia.org/wiki/Boyer%E2%80%93Moore_string-search_algorithm

use crate::buffer::Buffer;

const ALPHABET_LEN: usize = 256;

pub struct BoyerMooreSearch {
    delta1: Vec<isize>,
    delta2: Vec<isize>,
    pat: Vec<u8>
}

impl BoyerMooreSearch {
    pub fn new(pat: Vec<u8>) -> BoyerMooreSearch {
        let mut delta1 = vec![pat.len() as isize; ALPHABET_LEN];
        let mut delta2 = vec![0; pat.len()];
        make_delta1(&mut delta1, &pat);
        make_delta2(&mut delta2, &pat);
        BoyerMooreSearch {
            delta1,
            delta2,
            pat
        }
    }

    pub fn search(&self, data: &Buffer, offset: usize) -> Option<usize> {
        let patlen = self.pat.len();

        if patlen == 0 {
            return None;
        }

        if offset >= data.active_size {
            return None;
        }


        let mut i = offset as isize + patlen as isize - 1;
        //println!("offset: {:08x}; active_size: {}; i: {}", offset, data.active_size, i);
        while i < (data.active_size + patlen) as isize {
            let mut j = patlen as isize - 1;
            if data.at(i).is_none() {
                break;
            }
            while j >= 0 {
                if data.at(i as isize).unwrap() == self.pat[j as usize] {
                    i -= 1;
                    j -= 1;
                } else {
                    break;
                }
            }
            if j < 0 {
                return Some((i + 1) as usize);
            }
            let shift = std::cmp::max(self.delta1[data.at(i as isize).unwrap() as usize], self.delta2[j as usize]);
            i += shift;
        }

        return None;
    }
}

fn make_delta1(delta1: &mut Vec<isize>, pat: &Vec<u8>) {
    for i in 0..pat.len() {
        delta1[pat[i] as usize] = pat.len() as isize - 1 - i as isize;
    }
}

fn is_prefix(word: &Vec<u8>, pos: isize) -> bool {
    let suffixlen = word.len() - pos as usize;
    for i in 0..suffixlen {
        if word[i] != word[pos as usize + i] {
            return false;
        }
    }
    return true;
}

fn suffix_length(word: &Vec<u8>, pos: isize) -> usize {
    for i in 0..pos + 1 {
        if word[(pos - i) as usize] != word[word.len() - 1 - i as usize] {
            return i as usize;
        }
    }
    return pos as usize;
}

fn make_delta2(delta2: &mut Vec<isize>, pat: &Vec<u8>) {
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
