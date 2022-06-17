use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

use crate::loader::load_list_from_file;
use crate::score::compute_score;
use crate::solver::{Solver, Strategy};

fn thread_func(
    sender: Sender<[usize; 10]>,
    guessable: Arc<Vec<String>>,
    solutions: Arc<Vec<String>>,
    hard_mode: bool,
    strategy: Strategy,
    start_index: usize,
    end_index: usize,
) {
    let mut guess_counts = [0; 10];

    for answer in solutions[start_index..end_index].iter() {
        let mut state = Solver::new(
            guessable.as_ref(),
            solutions.as_ref(),
            hard_mode,
            false,
            strategy,
        );
        let mut guess_count = 0;

        loop {
            let guess = state.next_guess();
            let score = compute_score(guess, answer);
            guess_count += 1;

            if score.is_win() {
                guess_counts[guess_count] += 1;
                println!("{} {}", guess_count, answer);
                break;
            }

            state.respond_to_score(guess, &score);
        }
    }

    sender.send(guess_counts).unwrap();
}

/// Run the solver with each allowable solution, collecting a count of how many guesses were
/// required to solve each one. Splits the work out into threads for speed.
pub fn histogram(
    thread_count: usize,
    guessable_path: &Path,
    solution_path: &Path,
    hard_mode: bool,
    strategy: Strategy,
) {
    let guessable_list = Arc::new(load_list_from_file(guessable_path).unwrap());
    let solution_list = Arc::new(load_list_from_file(solution_path).unwrap());

    let mut start_index = 0;
    let count_per_thread = solution_list.len() / thread_count;
    let (sender, receiver) = channel();

    for i in 0..thread_count {
        let end_index = if i == thread_count - 1 {
            solution_list.len()
        } else {
            start_index + count_per_thread
        };
        let this_sender = sender.clone();
        let this_guessable = Arc::clone(&guessable_list);
        let this_solutions = Arc::clone(&solution_list);
        std::thread::spawn(move || {
            thread_func(
                this_sender,
                this_guessable,
                this_solutions,
                hard_mode,
                strategy,
                start_index,
                end_index,
            )
        });
        start_index += count_per_thread;
    }

    let mut totals = [0; 10];

    for _ in 0..thread_count {
        let one_result = receiver.recv().unwrap();
        for i in 0..one_result.len() {
            totals[i] += one_result[i];
        }
    }

    println!("{:?}", totals);
}
