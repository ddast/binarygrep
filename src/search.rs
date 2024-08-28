/// Adapted from Python example code at
/// https://en.wikipedia.org/wiki/Boyer%E2%80%93Moore_string-search_algorithm

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
    if pattern.is_empty() {
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
pub fn bad_character_table(pattern: &Vec<u8>) -> Vec<Option<usize>> {
    if pattern.is_empty() {
        return vec![];
    }
    let mut r = vec![None; (pattern.len() + 1) * ALPHABET_SIZE];
    let mut alpha = vec![None; ALPHABET_SIZE];
    for (i, c) in pattern.iter().enumerate() {
        alpha[usize::from(*c)] = Some(i);
        for (j, a) in alpha.iter().enumerate() {
            r[i + 1 + j * (pattern.len() + 1)] = *a;
        }
    }
    r
}

fn good_suffix_table(pattern: &Vec<u8>) -> Vec<Option<usize>> {
    let mut l = vec![None; pattern.len()];
    let mut rev_pattern = pattern.clone();
    rev_pattern.reverse();
    let mut n = fundamental_preprocess(&rev_pattern);
    n.reverse();
    for j in 0..(pattern.len() - 1) {
        let i = pattern.len() - n[j];
        if i != pattern.len() {
            l[i] = Some(j);
        }
    }
    l
}

fn full_shift_table(pattern: &Vec<u8>) -> Vec<usize> {
    let mut f = vec![0; pattern.len()];
    let z = fundamental_preprocess(&pattern);
    let mut longest = 0;
    for (i, &zv) in z.iter().rev().enumerate() {
        longest = if zv == i + 1 { std::cmp::max(zv, longest) } else {longest};
        f[pattern.len()-i-1] = longest;
    }
    f
}

fn string_search(pattern: &Vec<u8>, data: &Vec<u8>) -> Vec<Option<usize>> {
    if pattern.is_empty() || data.is_empty() || (data.len() < pattern.len()) {
        return vec![];
    }

    let mut matches = vec![];

    let r = bad_character_table(pattern);
    let l = good_suffix_table(pattern);
    let f = full_shift_table(pattern);

    let k = pattern.len() - 1;
    let previous_k = None;
    while k < data.len() {
        let mut i = Some(pattern.len() - 1);
        let mut h = Some(k);
        while i.is_some() && h.is_some() && (previous_k.is_none() || h > previous_k) && pattern[i.unwrap()] == data[h.unwrap()] {
            i = i.unwrap().checked_sub(1);
            h = h.unwrap().checked_sub(1);
        }
        if i.is_none() || h == previous_k {
            matches.push(Some(k - pattern.len() + 1));
            k += if pattern.len() > 1 {pattern.len() - f[1]} else {1};
        } else {
            let char_shift = i.unwrap() - r[data[h.unwrap()] + (pattern.len()+1)*i.unwrap()];
            if (i + 1 == pattern.len()) {
                let suffix_shift = 1;
            } else if (L[i + 1] == -1) {
                let suffix_shift = pattern.len() - f[i + 1];
            } else {
                let suffix_shift = pattern.len() - 1 - l[i + 1];
            }
            let shift = std::cmp::max(char_shift, suffix_shift);
            previous_k = if (shift >= i + 1) {k} else {previous_k};
            k += shift;
        }
    }
    matches
}
