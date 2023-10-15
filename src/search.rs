const ALPHABET_SIZE:usize = 256;

pub fn bad_character_table(pattern: &Vec<u8>) -> Vec<i32>
{
    let mut bad_char_table = vec![-1; (pattern.len()+1)*ALPHABET_SIZE];
    let mut alpha = vec![-1; ALPHABET_SIZE];
    for (i, c) in pattern.iter().enumerate() {
        alpha[*c as usize] = i as i32;
        for (j, a) in alpha.iter().enumerate() {
            bad_char_table[i + 1 + j * (pattern.len() + 1)] = *a;
        }
    }
    bad_char_table
}
