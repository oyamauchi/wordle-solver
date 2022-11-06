/// Solves multiple boards at once; e.g. https://quordle.com , https://duotrigordle.com
use std::io::{stdin, stdout};

use argparse::{ArgumentParser, Parse, Store, StoreTrue};

use wordle_solver::eval::{eval_guess, reduce_eval};
use wordle_solver::loader::load_list_from_file;
use wordle_solver::read_guess_interactively;
use wordle_solver::score::{read_score_interactively, DetailScore};
use wordle_solver::solver::{Solver, Strategy};

pub struct MultiSolver<'a> {
    solvers: Vec<Solver<'a>>,
    responded: Vec<bool>,
    done: Vec<bool>,

    guessable_list: &'a [String],
    solution_list: &'a [String],
    strategy: Strategy,
}

impl<'a> MultiSolver<'a> {
    pub fn new(
        count: usize,
        guessable_list: &'a [String],
        solution_list: &'a [String],
        strategy: Strategy,
    ) -> MultiSolver {
        let mut solvers = Vec::new();
        for _ in 0..count {
            solvers.push(Solver::new(
                guessable_list,
                solution_list,
                false,
                true,
                strategy,
            ));
        }
        MultiSolver {
            solvers,
            responded: vec![false; count],
            done: vec![false; count],
            guessable_list,
            solution_list,
            strategy,
        }
    }

    pub fn index_needing_response(&self) -> Option<usize> {
        (0..self.responded.len()).find(|idx| !self.responded[*idx] && !self.done[*idx])
    }

    pub fn all_done(&self) -> bool {
        self.done.iter().all(|d| *d)
    }

    pub fn next_guess(&self) -> &'a str {
        for (index, solver) in self.solvers.iter().enumerate() {
            if !self.done[index] && solver.get_possibilities().len() == 1 {
                return solver.get_possibilities()[0];
            }
        }

        let mut best_eval = (i32::MIN, i32::MIN);
        let mut best_guesses = Vec::new();

        for guess in self.solution_list.iter().chain(self.guessable_list.iter()) {
            // Don't get evals from solvers that are already done.
            let reduced = self
                .solvers
                .iter()
                .filter(|solver| solver.get_possibilities().len() != 1)
                .map(|solver| eval_guess(guess, solver.get_possibilities()))
                .reduce(reduce_eval)
                .unwrap();

            let eval = if self.strategy == Strategy::GroupCount {
                (reduced.count, reduced.size)
            } else {
                (reduced.size, reduced.count)
            };

            if eval > best_eval {
                best_eval = eval;
                best_guesses.clear();
            }
            if eval == best_eval {
                best_guesses.push(guess);
            }
        }

        best_guesses[0]
    }

    pub fn respond_to_score(&mut self, index: usize, guess: &'a str, score: DetailScore) {
        assert!(!self.responded[index]);
        self.solvers[index].respond_to_score(guess, score);
        self.responded[index] = true;
        if score.is_win() {
            self.done[index] = true;
        }
    }

    pub fn next_round(&mut self) {
        self.responded = vec![false; self.responded.len()];
    }
}

fn main() {
    let mut input = stdin().lock();
    let mut output = stdout();

    let mut count = 4;
    let mut enter_guesses = false;
    let mut strategy = Strategy::GroupSize;
    let mut guessable_path = "".to_string();
    let mut solutions_path = "".to_string();

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Solve wordle");

        parser.refer(&mut strategy).add_option(
            &["--strategy"],
            Parse,
            "Which solving strategy to use: groupcount or groupsize (default)",
        );
        parser.refer(&mut enter_guesses).add_option(
            &["--enter-guesses"],
            StoreTrue,
            "Manually enter guesses instead of automatically using generated ones",
        );

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
        parser.refer(&mut count).required().add_argument(
            "count",
            Store,
            "How many boards to solve",
        );
        parser.parse_args_or_exit();
    }

    let guessable_list = load_list_from_file(guessable_path.as_ref()).unwrap();
    let solution_list = load_list_from_file(solutions_path.as_ref()).unwrap();

    let mut solver = MultiSolver::new(count, &guessable_list, &solution_list, strategy);

    loop {
        println!("==============================");

        let guess = if enter_guesses {
            println!("Recommended: {}", solver.next_guess());
            read_guess_interactively(&mut input, &mut output, &guessable_list, &solution_list)
        } else {
            let g = solver.next_guess();
            println!("Guess: {}", g);
            g
        };
        solver.next_round();

        while let Some(index) = solver.index_needing_response() {
            println!("Need score for index {}", index);
            let score = read_score_interactively(&mut input, &mut output);
            solver.respond_to_score(index, guess, score);
        }

        if solver.all_done() {
            println!("Win!");
            break;
        }
    }
}
