use std::cmp::Ordering;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

use wordle_solver::loader::load_list_from_file;
use wordle_solver::score::compute_score;
use wordle_solver::solver::{Solver, Strategy};

struct ThreadResult {
    groupsize_counts: [usize; 10],
    groupcount_counts: [usize; 10],
    count_size_tie: [usize; 3],
}

fn run_solver<'a>(mut solver: Solver<'a>, first_guess: &'a str, answer: &str) -> u8 {
    let mut score = compute_score(first_guess, answer);
    solver.respond_to_score(first_guess, score);

    let mut guess_count = 1;

    loop {
        if score.is_win() {
            return guess_count;
        }

        let guess = solver.next_guess();
        score = compute_score(guess, answer);
        solver.respond_to_score(guess, score);
        guess_count += 1;
    }
}

fn thread_func(
    sender: Sender<ThreadResult>,
    guessable: Arc<Vec<String>>,
    solutions: Arc<Vec<String>>,
    hard_mode: bool,
    start_index: usize,
    end_index: usize,
) {
    let mut groupsize_counts = [0; 10];
    let mut groupcount_counts = [0; 10];
    let mut count_size_tie = [0; 3];

    let guessable = guessable.as_ref();
    let solutions = solutions.as_ref();

    let size_first_guess =
        Solver::new(guessable, solutions, false, false, Strategy::GroupSize).next_guess();
    let count_first_guess =
        Solver::new(guessable, solutions, false, false, Strategy::GroupCount).next_guess();

    for answer in solutions[start_index..end_index].iter() {
        let groupsize = Solver::new(guessable, solutions, hard_mode, false, Strategy::GroupSize);
        let size_result = run_solver(groupsize, size_first_guess, answer);
        groupsize_counts[size_result as usize] += 1;

        let groupcount = Solver::new(guessable, solutions, hard_mode, false, Strategy::GroupCount);
        let count_result = run_solver(groupcount, count_first_guess, answer);
        groupcount_counts[count_result as usize] += 1;

        println!("{} {} {}", count_result, size_result, answer);
        match size_result.cmp(&count_result) {
            Ordering::Less => count_size_tie[1] += 1,
            Ordering::Equal => count_size_tie[2] += 1,
            Ordering::Greater => count_size_tie[0] += 1,
        };
    }

    sender
        .send(ThreadResult {
            groupsize_counts,
            groupcount_counts,
            count_size_tie,
        })
        .unwrap();
}

/// Run the solver with each allowable solution, collecting a count of how many guesses were
/// required to solve each one. Splits the work out into threads for speed.
pub fn histogram(
    thread_count: usize,
    guessable_path: &Path,
    solution_path: &Path,
    hard_mode: bool,
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
                start_index,
                end_index,
            )
        });
        start_index += count_per_thread;
    }

    std::mem::drop(sender);

    let mut groupcount_totals = [0; 10];
    let mut groupsize_totals = [0; 10];
    let mut count_size_tie = [0; 3];

    for result in receiver.iter() {
        for i in 0..10 {
            groupcount_totals[i] += result.groupcount_counts[i];
            groupsize_totals[i] += result.groupsize_counts[i];
        }
        for (i, count) in count_size_tie.iter_mut().enumerate() {
            *count += result.count_size_tie[i];
        }
    }

    println!("GROUPCOUNT: {:?}", groupcount_totals);
    println!("GROUPSIZE:  {:?}", groupsize_totals);
    println!(
        "RECORD (count wins - size wins - tie): {:?}",
        count_size_tie
    );
}
