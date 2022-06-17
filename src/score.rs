use std::fmt::{Display, Write};
use std::hash::Hash;
use std::io::BufRead;
use std::mem::MaybeUninit;
use std::sync::Once;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LetterScore {
    CORRECT,
    PRESENT,
    ABSENT,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetailScore {
    inner: [LetterScore; 5],
}

impl DetailScore {
    pub fn is_win(&self) -> bool {
        self.inner
            .iter()
            .all(|letter| *letter == LetterScore::CORRECT)
    }
}

impl Display for DetailScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for letter in self.inner.iter() {
            f.write_char(match letter {
                LetterScore::ABSENT => 'a',
                LetterScore::CORRECT => 'c',
                LetterScore::PRESENT => 'p',
            })?;
        }
        Ok(())
    }
}

impl Hash for DetailScore {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.iter().for_each(|letter| letter.hash(state))
    }
}

/// All possible scores for a five-letter guess.
static mut POSSIBLE_SCORES: MaybeUninit<Vec<DetailScore>> = MaybeUninit::uninit();
static ONCE: Once = Once::new();

/// Generate all possible scores for a five-letter guess. This is actually more than all possible
/// score, because e.g. it is not possible to have a score of CCCCP. But that doesn't matter for
/// our purposes.
pub fn all_possible_scores() -> &'static Vec<DetailScore> {
    ONCE.call_once(|| {
        let mut vec = Vec::new();

        let letters = [
            LetterScore::ABSENT,
            LetterScore::PRESENT,
            LetterScore::CORRECT,
        ];

        for a in 0..3 {
            for b in 0..3 {
                for c in 0..3 {
                    for d in 0..3 {
                        for e in 0..3 {
                            let inner =
                                [letters[a], letters[b], letters[c], letters[d], letters[e]];
                            vec.push(DetailScore { inner });
                        }
                    }
                }
            }
        }

        unsafe { POSSIBLE_SCORES.write(vec) };
    });

    unsafe { POSSIBLE_SCORES.assume_init_ref() }
}

pub fn compute_score(guess: &str, solution: &str) -> DetailScore {
    let mut result = [LetterScore::ABSENT; 5];
    let a = 'a' as usize;

    // Count how many of each letter there is in the solution.
    let mut solution_counts = [0; 26];
    for c in solution.chars() {
        solution_counts[c as usize - a] += 1;
    }

    // Identify correct letters.
    for (i, (c_guess, c_sol)) in guess.chars().zip(solution.chars()).enumerate() {
        if c_guess == c_sol {
            // Subtract this letter from solution_counts so that other copies of the same letter
            // elsewhere in the guess don't use this letter in the solution to count a PRESENT.
            solution_counts[c_guess as usize - a] -= 1;
            result[i] = LetterScore::CORRECT;
        }
    }

    for (i, c_guess) in guess.chars().enumerate() {
        if result[i] != LetterScore::CORRECT && solution_counts[c_guess as usize - a] > 0 {
            solution_counts[c_guess as usize - a] -= 1;
            result[i] = LetterScore::PRESENT;
        }
    }

    DetailScore { inner: result }
}

/// Turn a 5-letter string of "a", "c", and "p" into a DetailScore.
fn parse_score_string(score_str: &str) -> Option<DetailScore> {
    if score_str.len() != 5 {
        return None;
    }

    let mut result = [LetterScore::ABSENT; 5];

    for (i, c) in score_str.chars().enumerate() {
        result[i] = match c {
            'a' => LetterScore::ABSENT,
            'c' => LetterScore::CORRECT,
            'p' => LetterScore::PRESENT,
            _ => return None,
        }
    }

    Some(DetailScore { inner: result })
}

/// Read a 5-letter a/c/p string from stdin via interactive prompts.
pub fn read_score_interactively(
    input: &mut dyn BufRead,
    output: &mut dyn std::io::Write,
    quiet: bool,
) -> DetailScore {
    let mut buf = String::new();

    loop {
        if !quiet {
            output.write(b"Score: ").unwrap();
            output.flush().unwrap();
        }

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
