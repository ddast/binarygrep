// Boyer-Moore string search
// Translated C implementation from
// https://en.wikipedia.org/wiki/Boyer%E2%80%93Moore_string-search_algorithm

use crate::buffer::Buffer;

const ALPHABET_LEN: usize = 256;

struct BoyerMooreSearch {
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

    pub fn search(&self, data: &Buffer) -> usize {
        let patlen = self.pat.len();

        if patlen == 0 {
            return 0;
        }

        let mut delta1 = vec![patlen as isize; ALPHABET_LEN];
        let mut delta2 = vec![0; patlen];
        make_delta1(&mut delta1, &self.pat);
        make_delta2(&mut delta2, &self.pat);

        let mut i = patlen as isize - 1;
        while i < data.active_size as isize {
            let mut j = patlen as isize - 1;
            while j >= 0 {
                if data.at(i as i32).unwrap() == self.pat[j as usize] { // TODO change i32 to isize
                    i -= 1;
                    j -= 1;
                } else {
                    break;
                }
            }
            if j < 0 {
                return i as usize + 1;
            }
            let shift = std::cmp::max(delta1[data.at(i as i32).unwrap() as usize], delta2[j as usize]); // TODO change i32 to isize
            i += shift;
        }

        return 0;
    }
}

fn make_delta1(delta1: &mut Vec<isize>, pat: &Vec<u8>) {
    for i in 0..pat.len() {
        delta1[pat[i] as usize] = pat.len() as isize - 1 - i as isize;
    }
}

fn is_prefix(word: &Vec<u8>, pos: isize) -> bool {
    let suffixlen = word.len() as isize - pos;
    for i in 0..suffixlen {
        if word[i as usize] != word[pos as usize + 1] {
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
