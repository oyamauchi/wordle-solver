//! A solver for Absurdle's challenge mode.

use argparse::{ArgumentParser, Store, StoreTrue};

use wordle_solver::loader::load_list_from_file;
use wordle_solver::score::{compute_score, DetailScore};

struct Solver<'a> {
    target_word: &'a str,
    possibilities: Vec<&'a str>,
    history: Vec<(&'a str, DetailScore)>,
    guessable_list: &'a [String],
    solutions_list: &'a [String],
    hard_mode: bool,
}

impl<'a> Solver<'a> {
    fn new(
        target_word: &'a str,
        guessable_list: &'a [String],
        solutions_list: &'a [String],
        hard_mode: bool,
    ) -> Self {
        Solver {
            target_word,
            guessable_list,
            solutions_list,
            history: Vec::new(),
            hard_mode,
            possibilities: Vec::from_iter(solutions_list.iter().map(|s| s.as_str())),
        }
    }

    /// Compute a vector of possible next guesses. They are sorted in order from worst to best.
    pub fn next_guess(&self) -> Vec<&'a str> {
        if self.possibilities.len() == 1 {
            return self.possibilities.clone();
        }

        // When you enter a guess in Absurdle challenge mode, it iterates through all possible
        // scores, and returns the one that eliminates the fewest remaining possibilities. Ties
        // are broken by Absurdle's "entropyLost" metric, calculated in score.rs.
        //
        // To pick a guess, therefore, we iterate through all possible guesses, and:
        //   1. Compute the score that Absurdle would give for that guess
        //   2. Compute how many possibilities are eliminated by that score
        //   3. Make sure that score wouldn't eliminate the target word as a possibility
        //
        // Return the guess that had the highest number in step 2 and passed step 3.
        //
        // In other words: pick the guess for which the least-informative score is as informative
        // as possible.

        let all_scores = DetailScore::all_possible();
        let mut guesses: Vec<&str> = Vec::new();
        let mut eliminated_by_best_guess = 0;

        'next_guess: for guess in self.guessable_list.iter().chain(self.solutions_list.iter()) {
            // Don't guess the target word. (The winning move is covered by the len == 1 case).
            if guess == self.target_word {
                continue;
            }

            // Follow the hard mode rule: can't make a guess that is ruled out by previous scores.
            if self.hard_mode {
                for (prev_guess, prev_score) in self.history.iter() {
                    if compute_score(prev_guess, guess) != *prev_score {
                        continue 'next_guess;
                    }
                }
            }

            let mut min_eliminated_by_this_guess = usize::MAX;
            let mut score_that_eliminates_min: Option<&DetailScore> = None;

            'next_score: for possible_score in all_scores.iter() {
                let mut eliminated_by_this_score = 0;

                for possibility in self.possibilities.iter() {
                    if compute_score(guess, possibility) != *possible_score {
                        eliminated_by_this_score += 1;
                    }

                    // eliminated_by_this_score only goes up, so if it's already greater than the
                    // minimum seen so far, no need to keep evaluating this score
                    if eliminated_by_this_score > min_eliminated_by_this_guess {
                        continue 'next_score;
                    }
                }

                if eliminated_by_this_score < min_eliminated_by_this_guess {
                    score_that_eliminates_min = Some(possible_score);
                    min_eliminated_by_this_guess = eliminated_by_this_score;
                } else if eliminated_by_this_score == min_eliminated_by_this_guess {
                    // If multiple scores result in the same number of eliminated possibilities,
                    // Absurdle will return the one with the lowest "entropyLost".
                    if score_that_eliminates_min.unwrap().absurdle_entropy_lost()
                        > possible_score.absurdle_entropy_lost()
                    {
                        score_that_eliminates_min = Some(possible_score);
                    }
                }

                // min_eliminated_by_this_guess only goes down, so if it's already less than the
                // maximum seen so far, no need to keep evaluating this guess.
                if min_eliminated_by_this_guess < eliminated_by_best_guess {
                    continue 'next_guess;
                }
            }

            // Now we know the score that Absurdle would give for this guess. Make sure it doesn't
            // eliminate the target word.
            if compute_score(guess, self.target_word) != *score_that_eliminates_min.unwrap() {
                continue 'next_guess;
            }

            if min_eliminated_by_this_guess > eliminated_by_best_guess {
                // In the normal version of the solver, we clear the guesses vec here, because at
                // this point, the guesses in the vec are known to be non-optimal. But here, it's
                // possible that none of the optimal guesses will work out, and we'll need to fall
                // back to worse ones. The vec will naturally be sorted in increasing order of
                // possibilities eliminated, because any subsequently evaluated guesses that
                // eliminate fewer than this one will be pruned out, above.
                eliminated_by_best_guess = min_eliminated_by_this_guess;
            }
            if min_eliminated_by_this_guess == eliminated_by_best_guess {
                guesses.push(guess);
            }
        }

        guesses
    }

    pub fn respond_to_score(&mut self, guess: &'a str, score: &DetailScore) {
        // Keep only possibilities that fit the score we got.
        self.possibilities
            .retain(|poss| compute_score(guess, *poss) == *score);

        if self.possibilities.is_empty() {
            // This should not happen absent human error in playing the game.
            panic!("No possibilities left");
        }
    }

    pub fn solve(&mut self) {
        // The approach is to keep a stack of possible guesses at each step. We will repeatedly
        // test a sequence of guesses consisting of the last one from each level of the stack. If
        // this leads us to a dead end (no possible guesses left that don't eliminate the target
        // word), we backtrack by popping a guess from the last level and trying again. We will
        // maintain the invariant that every level of this stack is a nonempty vec.
        let mut stack: Vec<Vec<&str>> = Vec::new();

        loop {
            let next_guesses = self.next_guess();

            if next_guesses.is_empty() {
                // No way to proceed. Backtrack.
                println!("✗");

                // The last guess, at least, led us to a loss. Drop it.
                stack.last_mut().unwrap().pop();

                // Restore the invariant that every level of the stack is a nonempty vector. If
                // that pop made the last level empty, pop that whole level and drop the last guess
                // of the previous level.
                while stack.last().unwrap().is_empty() {
                    stack.pop();
                    if stack.is_empty() {
                        println!("Total failure!");
                        return;
                    }
                    stack.last_mut().unwrap().pop();
                }
            } else if next_guesses.len() == 1 && *next_guesses.first().unwrap() == self.target_word
            {
                println!("{} ✔", self.target_word);
                return;
            } else {
                // Neither a win nor a loss. Add this set of guesses to the stack and keep going.
                stack.push(next_guesses);
                println!();
            }

            // Reconstruct the state for the current stack of guesses.
            self.history.clear();
            self.possibilities = Vec::from_iter(self.solutions_list.iter().map(|s| s.as_str()));

            for guesses in stack.iter() {
                let best_guess = *guesses.last().unwrap();
                print!("{} ", best_guess);
                let score = compute_score(best_guess, self.target_word);
                self.respond_to_score(best_guess, &score);
                self.history.push((best_guess, score));
            }
        }
    }
}

fn main() {
    let mut guessable_path = "".to_string();
    let mut solutions_path = "".to_string();
    let mut target_word = "".to_string();
    let mut hard_mode = false;

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Solve Absurdle challenge mode");
        parser.refer(&mut guessable_path).required().add_argument(
            "guessable-path",
            Store,
            "The path to the file of guessable strings",
        );
        parser.refer(&mut solutions_path).required().add_argument(
            "solutions-path",
            Store,
            "The path to the file of possible solutions",
        );
        parser.refer(&mut target_word).required().add_argument(
            "target-word",
            Store,
            "The target word",
        );
        parser.refer(&mut hard_mode).add_option(
            &["--hard-mode"],
            StoreTrue,
            "Guesses must use all previously gained information",
        );
        parser.parse_args_or_exit();
    }

    let guessable = load_list_from_file(guessable_path.as_ref()).unwrap();
    let solutions = load_list_from_file(solutions_path.as_ref()).unwrap();

    if !solutions.contains(&target_word) {
        println!("'{}' is not in the solution list", target_word);
        std::process::exit(1);
    }

    let mut solver = Solver::new(target_word.as_str(), &guessable, &solutions, hard_mode);
    solver.solve();
}
