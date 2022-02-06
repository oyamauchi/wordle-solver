use crate::score::{all_possible_scores, compute_score, DetailScore};

pub struct Solver<'a> {
    /// Possible solutions that haven't been eliminated yet.
    possibilities: Vec<&'a str>,

    /// Words that we're allowed to guess, but aren't possible solutions.
    guessable_list: &'a [String],

    /// All words that can be solutions.
    solution_list: &'a [String],

    /// Whether to print log messages.
    verbose: bool,
}

impl<'a> Solver<'a> {
    pub fn new(guessable_list: &'a [String], solution_list: &'a [String], verbose: bool) -> Self {
        Solver {
            possibilities: Vec::from_iter(solution_list.iter().map(|s| s.as_str())),
            guessable_list,
            solution_list,
            verbose,
        }
    }

    /// Return the next word to guess. This is where the min-max implementation is.
    pub fn next_guess(&self) -> &'a str {
        if self.possibilities.len() == 1 {
            return self.possibilities[0];
        }

        let mut max_min_eliminated = 0;
        let mut max_min_guesses: Vec<&str> = Vec::new();

        // For each allowable guess, we compute how many possible solutions will be eliminated by
        // each possible score. We take the minimum of all those, and find the guess that maximizes
        // that minimum. In other words, find the guess that eliminates the most possible solutions
        // even in the worst-case scenario.
        for guess in self.solution_list.iter().chain(self.guessable_list.iter()) {
            let mut min_possibilities_eliminated = usize::MAX;

            for possible_score in all_possible_scores().iter() {
                let mut eliminated_by_this_score = 0;

                for possible_solution in self.possibilities.iter() {
                    if compute_score(guess, possible_solution) != *possible_score {
                        eliminated_by_this_score += 1;
                    }

                    // eliminated_by_this_score only goes up, so if it's already greater than the
                    // minimum seen so far, the if-condition immediately below will never be true
                    // and there's no need to proceed further with this score.
                    if eliminated_by_this_score > min_possibilities_eliminated {
                        break;
                    }
                }

                if eliminated_by_this_score < min_possibilities_eliminated {
                    min_possibilities_eliminated = eliminated_by_this_score;
                }

                // min_possibilities_eliminated only goes down, so if it's already less than the
                // maximum seen so far, the if-condition immediately below will never be true and
                // there's no need to proceed further with this guess.
                if min_possibilities_eliminated < max_min_eliminated {
                    break;
                }
            }

            if min_possibilities_eliminated > max_min_eliminated {
                max_min_eliminated = min_possibilities_eliminated;
                max_min_guesses.clear();
            }
            if min_possibilities_eliminated == max_min_eliminated {
                max_min_guesses.push(guess);
            }
        }

        // Return a guess that is a possible solution, if any.
        for guess in max_min_guesses.iter() {
            if self.possibilities.contains(guess) {
                return guess;
            }
        }

        // This is OK -- we won't win with this guess, but it will maximize the new info we get.
        if self.verbose {
            println!("Guessing a word that is not a possible solution");
        }

        // It would be correct to use any word in max_min_guesses, but using the last one happens
        // to allow the program to solve all words in 5 tries or fewer. It's not a considered
        // strategy; it's really just a function of the ordering of the words in the lists.
        return max_min_guesses.last().unwrap();
    }

    /// Whittle down the possibilities set given the actual score for a guess. Note that this
    /// doesn't assume the guess is one that `next_guess` actually returned; it can be anything.
    pub fn respond_to_score(&mut self, guess: &str, score: &DetailScore) {
        let mut read_index = 0;
        let mut write_index = 0;

        // Collect the still-possible solutions at the beginning, then chop off the end.
        loop {
            while read_index < self.possibilities.len() {
                if compute_score(guess, self.possibilities[read_index]) == *score {
                    break;
                }
                read_index += 1;
            }

            if read_index < self.possibilities.len() {
                self.possibilities[write_index] = self.possibilities[read_index];
                read_index += 1;
                write_index += 1;
            } else {
                break;
            }
        }

        self.possibilities.truncate(write_index);
        self.possibilities.shrink_to_fit();

        if self.possibilities.len() == 0 {
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
