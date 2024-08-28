// Boyer-Moore string search
// Translated C implementation from
// https://en.wikipedia.org/wiki/Boyer%E2%80%93Moore_string-search_algorithm
const ALPHABET_LEN: usize = 256;

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

fn boyer_moore(data: &Vec<u8>, pat: &Vec<u8>) -> usize {
    if pat.len() == 0 {
        return 0;
    }

    let mut delta1 = vec![pat.len() as isize; ALPHABET_LEN];
    let mut delta2 = vec![0; pat.len()];
    make_delta1(&mut delta1, pat);
    make_delta2(&mut delta2, pat);

    let mut i = pat.len() as isize - 1;
    while i < data.len() as isize {
        let mut j = pat.len() as isize - 1;
        while j >= 0 && data[i as usize] == pat[j as usize] {
            i -= 1;
            j -= 1;
        }
        if j < 0 {
            return i as usize + 1;
        }
        let shift = std::cmp::max(delta1[data[i as usize] as usize], delta2[j as usize]);
        i += shift;
    }

    return 0;
}
