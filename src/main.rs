use std::io::{stdin, stdout, BufRead};
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

use argparse::{ArgumentParser, Parse, Store, StoreOption, StoreTrue};

mod loader;
mod score;
mod solver;

use loader::load_list_from_file;
use score::{compute_score, read_score_interactively};
use solver::{Solver, Strategy};

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
fn histogram(
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

fn read_guess_interactively(
    input: &mut dyn BufRead,
    output: &mut dyn std::io::Write,
    quiet: bool,
) -> String {
    let mut buf = String::new();

    loop {
        if !quiet {
            output.write(b"Guess: ").unwrap();
            output.flush().unwrap();
        }

        buf.clear();
        input.read_line(&mut buf).unwrap();
        buf.truncate(buf.len() - 1);

        if buf.len() == 5 && buf.as_bytes().iter().all(|b| *b >= b'a' && *b <= b'z') {
            return buf;
        }
        println!("Guess must be 5 lowercase letters");
    }
}

fn main() {
    let mut input = stdin().lock();
    let mut output = stdout();

    let mut do_histogram = false;
    let mut quiet = false;
    let mut thread_count = 8;
    let mut predetermined_solution: Option<String> = None;
    let mut first_guess: Option<String> = None;
    let mut enter_guesses = false;
    let mut hard_mode = false;
    let mut strategy = Strategy::GROUPSIZE;

    let mut guessable_path = "".to_string();
    let mut solutions_path = "".to_string();

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Solve wordle");
        parser.refer(&mut quiet).add_option(
            &["-q", "--quiet"],
            StoreTrue,
            concat!(
                "Only print guesses (and scores if --self-score is passed); ",
                "do not print prompts or info."
            ),
        );
        parser.refer(&mut first_guess).add_option(
            &["--first-guess"],
            StoreOption,
            "Use this as the first guess",
        );
        parser.refer(&mut enter_guesses).add_option(
            &["--enter-guesses"],
            StoreTrue,
            "Manually enter guesses instead of automatically using generated ones",
        );
        parser.refer(&mut hard_mode).add_option(
            &["--hard-mode"],
            StoreTrue,
            "Only guess words that are possible solutions",
        );
        parser.refer(&mut predetermined_solution).add_option(
            &["--self-score"],
            StoreOption,
            "Use this as the answer; output guesses and scores",
        );
        parser.refer(&mut do_histogram).add_option(
            &["--solve-all"],
            StoreTrue,
            concat!(
                "Run solver on every possible solution; report number of guesses required for ",
                "each. Ignores --self-score and --quiet."
            ),
        );
        parser.refer(&mut strategy).add_option(
            &["--strategy"],
            Parse,
            "Which solving strategy to use: groupcount or groupsize (default)",
        );
        parser.refer(&mut thread_count).add_option(
            &["--thread-count"],
            Parse,
            "Thread count for --solve-all runs",
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
        parser.parse_args_or_exit();
    }

    if do_histogram {
        histogram(
            thread_count,
            guessable_path.as_ref(),
            solutions_path.as_ref(),
            hard_mode,
            strategy,
        );
        return;
    }

    let guessable_list = load_list_from_file(guessable_path.as_ref()).unwrap();
    let solution_list = load_list_from_file(solutions_path.as_ref()).unwrap();

    if let Some(ref solution) = predetermined_solution {
        if !solution_list.contains(solution) {
            println!("'{}' is not in the solution list!", solution);
            std::process::exit(1);
        }
    }

    let mut state = Solver::new(&guessable_list, &solution_list, hard_mode, !quiet, strategy);

    loop {
        let guess = first_guess.unwrap_or_else(|| {
            if enter_guesses {
                read_guess_interactively(&mut input, &mut output, quiet)
            } else {
                String::from(state.next_guess())
            }
        });

        if !enter_guesses {
            if quiet {
                println!("{}", guess);
            } else {
                println!("Guess: {}", guess);
            }
        }

        let score = match predetermined_solution {
            Some(ref solution) => {
                let s = compute_score(&guess, solution);
                if quiet {
                    println!("{}", s);
                } else {
                    println!("Score: {}", s);
                }
                s
            }
            None => read_score_interactively(&mut input, &mut output, quiet),
        };

        if score.is_win() {
            if !quiet {
                println!("Win!");
            }
            break;
        }

        state.respond_to_score(&guess, &score);
        first_guess = None;
    }
}
