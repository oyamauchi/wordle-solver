use std::io::{stdin, stdout, BufRead};

use argparse::{ArgumentParser, Parse, Store, StoreOption, StoreTrue};

mod histogram;
mod loader;
mod score;
mod solver;

use loader::load_list_from_file;
use score::{compute_score, read_score_interactively};
use solver::{Solver, Strategy};

fn read_guess_interactively<'a>(
    input: &mut dyn BufRead,
    output: &mut dyn std::io::Write,
    guessable_list: &'a [String],
    solution_list: &'a [String],
) -> &'a str {
    let mut buf = String::new();

    loop {
        output.write_all(b"Guess: ").unwrap();
        output.flush().unwrap();

        buf.clear();
        input.read_line(&mut buf).unwrap();
        buf.truncate(buf.len() - 1);

        if buf.len() != 5 || !buf.as_bytes().iter().all(u8::is_ascii_lowercase) {
            println!("Guess must be 5 lowercase letters");
            continue;
        }

        for guess in guessable_list.iter().chain(solution_list.iter()) {
            if *guess == buf {
                return guess;
            }
        }

        println!("Not a valid guess");
    }
}

fn main() {
    let mut input = stdin().lock();
    let mut output = stdout();

    let mut do_histogram = false;
    let mut thread_count = 8;
    let mut predetermined_solution: Option<String> = None;
    let mut enter_guesses = false;
    let mut hard_mode = false;
    let mut strategy = Strategy::GroupSize;

    let mut guessable_path = "".to_string();
    let mut solutions_path = "".to_string();

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Solve wordle");
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
                "Run both solver strategies on every possible solution; report number of guesses ",
                "required for each. Ignores --self-score and --strategy."
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
        histogram::histogram(
            thread_count,
            guessable_path.as_ref(),
            solutions_path.as_ref(),
            hard_mode,
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

    let mut state = Solver::new(&guessable_list, &solution_list, hard_mode, true, strategy);

    loop {
        let guess = if enter_guesses {
            println!("Recommended: {}", state.next_guess());
            read_guess_interactively(&mut input, &mut output, &guessable_list, &solution_list)
        } else {
            let g = state.next_guess();
            println!("Guess: {}", g);
            g
        };

        let score = match predetermined_solution {
            Some(ref solution) => {
                let s = compute_score(guess, solution);
                println!("Score: {}", s);
                s
            }
            None => read_score_interactively(&mut input, &mut output),
        };

        if score.is_win() {
            println!("Win!");
            break;
        }

        state.respond_to_score(guess, score);
    }
}
