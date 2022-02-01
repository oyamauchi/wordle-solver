use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

use argparse::{ArgumentParser, Parse, Store, StoreOption, StoreTrue};

mod score;
mod solver;

use score::{compute_score, parse_score_string, render_score, DetailScore, LetterScore};
use solver::Solver;

/// Read a 5-letter a/c/p string from stdin via interactive prompts.
fn read_score_interactively(quiet: bool) -> DetailScore {
    let input = stdin();
    let mut output = stdout();
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

/// Read a word list from a file (one word per line).
fn load_list_from_file(path: &Path) -> Result<Vec<String>, std::io::Error> {
    let reader = File::open(path)?;
    let mut bufreader = BufReader::new(reader);

    let mut result = Vec::new();
    let mut buffer = String::new();
    while bufreader.read_line(&mut buffer)? > 0 {
        result.push(String::from(buffer.trim_end()));
        buffer.clear();
    }

    Ok(result)
}

fn thread_func(
    sender: Sender<[usize; 10]>,
    guessable: Arc<Vec<String>>,
    solutions: Arc<Vec<String>>,
    start_index: usize,
    end_index: usize,
) {
    let mut guess_counts = [0; 10];

    for answer in solutions[start_index..end_index].iter() {
        let mut state = Solver::new(guessable.as_ref(), solutions.as_ref(), false);
        let mut guess_count = 0;

        loop {
            let guess = state.next_guess();
            let score = compute_score(guess, answer);
            guess_count += 1;

            if score.iter().all(|letter| *letter == LetterScore::CORRECT) {
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
fn histogram(thread_count: usize, guessable_path: &Path, solution_path: &Path) {
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

fn main() {
    let mut do_histogram = false;
    let mut quiet = false;
    let mut thread_count = 8;
    let mut predetermined_solution: Option<String> = None;

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

    let mut state = Solver::new(&guessable_list, &solution_list, !quiet);

    loop {
        let guess = state.next_guess();
        if quiet {
            println!("{}", guess);
        } else {
            println!("Guess: {}", guess);
        }

        let score = match predetermined_solution {
            Some(ref solution) => {
                let s = compute_score(guess, solution);
                if quiet {
                    println!("{}", render_score(&s));
                } else {
                    println!("Score: {}", render_score(&s));
                }
                s
            }
            None => read_score_interactively(quiet),
        };

        if score.iter().all(|letter| *letter == LetterScore::CORRECT) {
            if !quiet {
                println!("Win!");
            }
            break;
        }

        state.respond_to_score(guess, &score);
    }
}
