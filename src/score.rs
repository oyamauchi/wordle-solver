use std::fmt::{Display, Write};
use std::hash::Hash;
use std::io::BufRead;
use std::num::Wrapping;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
enum LetterScore {
    Absent = 0,
    Present,
    Correct,
}

const LETTERS: [char; 3] = ['a', 'p', 'c'];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DetailScore(u8);

pub const NUM_POSSIBLE_SCORES: u8 = 3u8.pow(5);

impl DetailScore {
    pub fn is_win(&self) -> bool {
        self.0 == NUM_POSSIBLE_SCORES - 1
    }
    pub fn as_num(&self) -> u8 {
        self.0
    }

    #[allow(dead_code)]
    pub fn all_possible() -> Vec<Self> {
        Vec::from_iter((0..NUM_POSSIBLE_SCORES).map(DetailScore))
    }

    /// Returns an equivalent of a score used by Absurdle. It's called "entropyLost" in Absurdle,
    /// and is supposed to be a measure of how much information the score holds. This is not the
    /// exact formula that Absurdle uses (which involves exponentiating 10, which is inefficient),
    /// but it is equivalent for comparison purposes. This is how Absurdle breaks ties when
    /// deciding what score to return, if there are multiple scores that would leave possibility
    /// sets of the same size.
    ///
    /// In principle, this is lexicographic comparison of a tuple consisting of:
    /// - Count of CORRECT letters
    /// - Count of PRESENT letters
    /// - Count of ABSENT letters
    /// - Each letter from left to right, with CORRECT > PRESENT > ABSENT
    /// For efficiency, pack these into an int instead of using a real tuple.
    #[allow(dead_code)]
    pub fn absurdle_entropy_lost(&self) -> u32 {
        let mut counts = [0_u32, 0, 0];
        let mut result = 0u16;
        let mut num = self.0;

        // The letters are stored in DetailScore with the leftmost letter as the highest-order.
        // Iterate backwards to make the math slightly easier.
        for i in (0..5).rev() {
            let letter_num = (num % 3) as u16;
            num /= 3;

            // 2 bits for each letter score
            result += letter_num << ((4 - i) * 2);
            counts[letter_num as usize] += 1;
        }

        // 3 bits for each count (they can be up to 5)
        (result as u32) + (counts[2] << 16) + (counts[1] << 13) + (counts[0] << 10)
    }
}

impl Display for DetailScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut num = self.0;
        let mut divisor = 81;
        for _ in 0..5 {
            let quotient = num / divisor;
            f.write_char(LETTERS[quotient as usize])?;
            num -= quotient * divisor;
            divisor /= 3;
        }
        Ok(())
    }
}

fn pack_score(score: &[LetterScore; 5]) -> DetailScore {
    let mut num = 0;
    for letter in score.iter() {
        num *= 3;
        num += *letter as u8;
    }
    DetailScore(num)
}

pub fn compute_score(guess: &str, solution: &str) -> DetailScore {
    let mut result = [LetterScore::Absent; 5];
    let a = 'a' as usize;

    let guess_bytes = guess.as_bytes();
    let sol_bytes = solution.as_bytes();

    // Count how many of each letter there is in the solution.
    let mut solution_counts = [Wrapping(0u8); 26];

    // Go unsafe to avoid bounds checks.
    unsafe {
        for i in 0..5 {
            let c = *sol_bytes.get_unchecked(i);
            *solution_counts.get_unchecked_mut(c as usize - a) += 1;
        }

        // Identify correct letters.
        for i in 0..5 {
            let c_guess = *guess_bytes.get_unchecked(i);
            if c_guess == *sol_bytes.get_unchecked(i) {
                // Subtract this letter from solution_counts so that other copies of the same letter
                // elsewhere in the guess don't use this letter in the solution to count a PRESENT.
                *solution_counts.get_unchecked_mut(c_guess as usize - a) -= 1;
                *result.get_unchecked_mut(i) = LetterScore::Correct;
            }
        }

        for i in 0..5 {
            let c_guess = *guess_bytes.get_unchecked(i);
            let res_i = result.get_unchecked_mut(i);
            let solcount = solution_counts.get_unchecked_mut(c_guess as usize - a);
            if *res_i != LetterScore::Correct && solcount.0 > 0 {
                *solcount -= 1;
                *res_i = LetterScore::Present;
            }
        }
    }

    pack_score(&result)
}

/// Turn a 5-letter string of "a", "c", and "p" into a DetailScore.
fn parse_score_string(score_str: &str) -> Option<DetailScore> {
    if score_str.len() != 5 {
        return None;
    }

    let mut result = [LetterScore::Absent; 5];

    for (i, c) in score_str.chars().enumerate() {
        result[i] = match c {
            'a' => LetterScore::Absent,
            'c' => LetterScore::Correct,
            'p' => LetterScore::Present,
            _ => return None,
        }
    }

    Some(pack_score(&result))
}

/// Read a 5-letter a/c/p string from stdin via interactive prompts.
pub fn read_score_interactively(
    input: &mut dyn BufRead,
    output: &mut dyn std::io::Write,
) -> DetailScore {
    let mut buf = String::new();

    loop {
        output.write_all(b"Score: ").unwrap();
        output.flush().unwrap();

        buf.clear();
        input.read_line(&mut buf).unwrap();

        match parse_score_string(buf.trim_end()) {
            Some(score) => return score,
            None => println!(
        "Score must be 5 characters, all either 'a' (absent), 'c' (correct), or 'p' (present)."
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_score() {
        macro_rules! assert_score {
            ($expected:literal, $guess:literal, $solution:literal) => {
                assert_eq!(
                    parse_score_string($expected).unwrap(),
                    compute_score($guess, $solution)
                );
            };
        }

        assert_score!("aaaaa", "squid", "maker");
        assert_score!("cccca", "squid", "squib");

        // Doubled letters in guess
        assert_score!("aappa", "espoo", "glorp");
        assert_score!("aaapp", "espoo", "footy");

        // Same letter correct and present
        assert_score!("caaaa", "aabbb", "acccc");
        assert_score!("acaca", "motto", "lofty");
        assert_score!("apaac", "arise", "verge");
        assert_score!("pacca", "repeg", "paper");
    }
}
