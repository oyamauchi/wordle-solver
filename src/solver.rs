use std::collections::HashMap;

use crate::score::{compute_score, DetailScore};

#[derive(Clone, Copy, PartialEq)]
pub enum Strategy {
    GroupSize,
    GroupCount,
}

impl argparse::FromCommandLine for Strategy {
    fn from_argument(s: &str) -> Result<Self, String> {
        match s {
            "groupsize" => Ok(Self::GroupSize),
            "groupcount" => Ok(Self::GroupCount),
            _ => Err("strategies are 'groupcount' and 'groupsize'".to_string()),
        }
    }
}

pub struct Solver<'a> {
    /// Possible solutions that haven't been eliminated yet.
    possibilities: Vec<&'a str>,

    /// Words that we're allowed to guess, but aren't possible solutions.
    guessable_list: &'a [String],

    /// All words that can be solutions.
    solution_list: &'a [String],

    /// Guesses made so far, and the scores they got. Only used in hard mode.
    history: Vec<(String, DetailScore)>,

    /// Only allowed to guess words that match scores seen so far.
    hard_mode: bool,

    /// Whether to print log messages.
    verbose: bool,

    /// Which solving strategy to use.
    strategy: Strategy,
}

impl<'a> Solver<'a> {
    pub fn new(
        guessable_list: &'a [String],
        solution_list: &'a [String],
        hard_mode: bool,
        verbose: bool,
        strategy: Strategy,
    ) -> Self {
        Solver {
            possibilities: Vec::from_iter(solution_list.iter().map(|s| s.as_str())),
            guessable_list,
            solution_list,
            history: Vec::new(),
            hard_mode,
            verbose,
            strategy,
        }
    }

    /// Return the next word to guess.
    pub fn next_guess(&self) -> &'a str {
        if self.possibilities.len() == 1 {
            return self.possibilities[0];
        }

        let mut best_eval = (i32::MIN, i32::MIN);
        let mut best_guesses: Vec<&str> = Vec::new();

        'next_guess: for guess in self.solution_list.iter().chain(self.guessable_list.iter()) {
            // For hard mode, filter out guesses that don't match the information we have so far.
            if self.hard_mode {
                for (prev_guess, score) in self.history.iter() {
                    if compute_score(prev_guess, guess) != *score {
                        continue 'next_guess;
                    }
                }
            }

            let eval = self.eval_guess(guess, &self.strategy);
            if eval > best_eval {
                best_eval = eval;
                best_guesses.clear();
            }
            if eval == best_eval {
                best_guesses.push(guess);
            }
        }

        // Of the best guesses, prefer one that is a possible solution given the scores we've
        // gotten so far. If there isn't one, that's OK; we won't win on this turn but it should
        // maximize the new info we get.
        best_guesses
            .iter()
            .find(|guess| self.possibilities.contains(guess))
            .unwrap_or_else(|| {
                if self.verbose {
                    println!("Guessing a word that is not a possible solution");
                }
                &best_guesses[0]
            })
    }

    /// Score the given guess according to the strategy. Higher score is better.
    fn eval_guess(&self, guess: &str, strategy: &Strategy) -> (i32, i32) {
        let mut groups = HashMap::new();

        // For each possible solution, compute what score this guess would get if that were the
        // actual solution. All strategies make use of this information.
        //
        // Count how many possible solutions would result in each possible score.
        for possible_sol in self.possibilities.iter() {
            let score = compute_score(guess, possible_sol);

            match groups.get_mut(&score) {
                Some(count) => *count += 1,
                None => {
                    groups.insert(score, 1);
                }
            }
        }

        let count_eval = groups.len() as i32;
        let size_eval = -*groups.values().max().unwrap();
        if *strategy == Strategy::GroupCount {
            (count_eval, size_eval)
        } else {
            (size_eval, count_eval)
        }
    }

    /// Whittle down the possibilities set given the actual score for a guess. Note that this
    /// doesn't assume the guess is one that `next_guess` actually returned; it can be anything.
    pub fn respond_to_score(&mut self, guess: &str, score: &DetailScore) {
        if self.hard_mode {
            self.history.push((String::from(guess), score.clone()));
        }

        self.possibilities
            .retain(|possibility| compute_score(guess, possibility) == *score);

        if self.possibilities.is_empty() {
            // This should not happen absent human error in playing the game.
            panic!("No possibilities left");
        }

        if self.verbose {
            if self.possibilities.len() <= 10 {
                println!("Possibilities left: {}", self.possibilities.join(", "));
            } else {
                println!("{} possibilities left", self.possibilities.len());
            }
        }
    }
}
