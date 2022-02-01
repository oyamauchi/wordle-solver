use std::mem::MaybeUninit;
use std::sync::Once;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LetterScore {
    CORRECT,
    PRESENT,
    ABSENT,
}

pub type DetailScore = [LetterScore; 5];

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
                            vec.push([letters[a], letters[b], letters[c], letters[d], letters[e]]);
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

    result
}

/// Turn a 5-letter string of "a", "c", and "p" into a DetailScore.
pub fn parse_score_string(score_str: &str) -> Option<DetailScore> {
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

    Some(result)
}

pub fn render_score(score: &DetailScore) -> String {
    String::from_iter(score.iter().map(|letter| match letter {
        LetterScore::ABSENT => 'a',
        LetterScore::CORRECT => 'c',
        LetterScore::PRESENT => 'p',
    }))
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
