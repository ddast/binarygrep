/// Adapted from Python example code at https://en.wikipedia.org/wiki/Boyer%E2%80%93Moore_string-search_algorithm

const ALPHABET_SIZE: usize = 256;

fn match_length(pattern: &Vec<u8>, mut idx1: usize, mut idx2: usize) -> usize {
    if idx1 == idx2 {
        return pattern.len() - idx1;
    }
    let mut match_count = 0;
    while idx1 < pattern.len() && idx2 < pattern.len() && pattern[idx1] == pattern[idx2] {
        match_count += 1;
        idx1 += 1;
        idx2 += 1;
    }
    match_count
}

fn fundamental_preprocess(pattern: &Vec<u8>) -> Vec<usize> {
    if pattern.len() == 0 {
        return vec![];
    }
    if pattern.len() == 1 {
        return vec![1];
    }
    let mut z = vec![0, pattern.len()];
    z[0] = pattern.len();
    z[1] = match_length(pattern, 0, 1);
    for i in 2..(1 + z[1]) {
        z[i] = z[1] - 1 + 1;
    }
    let mut l = 0;
    let mut r = 0;
    for i in (2 + z[1])..pattern.len() {
        if i <= r {
            let k = i - l;
            let b = z[k];
            let a = r - i + 1;
            if b < a {
                z[i] = b;
            } else {
                z[i] = a + match_length(pattern, a, r + 1);
                l = i;
                r = i + z[i] - 1;
            }
        } else {
            z[i] = match_length(pattern, 0, i);
            if z[i] > 0 {
                l = i;
                r = i + z[i] - 1;
            }
        }
    }
    z
}

/// Generate the bad character table from `pattern` for constant time lookup.  The table has to be
/// interpreted as a two-dimensional table where [i + (pattern.len()+1) * j] means character i and
/// position j.
pub fn bad_character_table(pattern: &Vec<u8>) -> Vec<i32> {
    if pattern.len() == 0 {
        return vec![];
    }
    let mut r = vec![-1; (pattern.len() + 1) * ALPHABET_SIZE];
    let mut alpha = vec![-1; ALPHABET_SIZE];
    for (i, c) in pattern.iter().enumerate() {
        alpha[*c as usize] = i as i32;
        for (j, a) in alpha.iter().enumerate() {
            r[i + 1 + j * (pattern.len() + 1)] = *a;
        }
    }
    r
}

fn good_suffix_table(pattern: &Vec<u8>) -> Vec<i32> {
    let mut l = vec![-1; pattern.len()];
    let mut rev_pattern = pattern.clone();
    rev_pattern.reverse();
    let mut n = fundamental_preprocess(&rev_pattern);
    n.reverse();
    for j in 0..(pattern.len() - 1) {
        let i = pattern.len() - n[j];
        if i != pattern.len() {
            l[i] = j as i32;
        }
    }
    l
}

fn full_shift_table(pattern: &Vec<u8>) -> Vec<i32> {
    let f = vec![0, pattern.len()];
    let z = fundamental_preprocess(&pattern);
    let mut longest = 0;
    // TODO
    f
}
