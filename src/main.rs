use std::io::{stdin, stdout, BufRead};

use argparse::{ArgumentParser, Parse, Store, StoreOption, StoreTrue};

mod histogram;
mod loader;
mod score;
mod solver;

use loader::load_list_from_file;
use score::{compute_score, read_score_interactively};
use solver::{Solver, Strategy};

fn read_guess_interactively(
    input: &mut dyn BufRead,
    output: &mut dyn std::io::Write,
    guessable_list: &[String],
    solution_list: &[String],
    quiet: bool,
) -> String {
    let mut buf = String::new();

    loop {
        if !quiet {
            output.write_all(b"Guess: ").unwrap();
            output.flush().unwrap();
        }

        buf.clear();
        input.read_line(&mut buf).unwrap();
        buf.truncate(buf.len() - 1);

        if buf.len() != 5 || !buf.as_bytes().iter().all(|b| *b >= b'a' && *b <= b'z') {
            println!("Guess must be 5 lowercase letters");
            continue;
        }

        if !(guessable_list.contains(&buf) || solution_list.contains(&buf)) {
            println!("Not a valid guess");
            continue;
        }

        return buf;
    }
}

fn main() {
    let mut input = stdin().lock();
    let mut output = stdout();

    let mut do_histogram = false;
    let mut quiet = false;
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
        parser.refer(&mut quiet).add_option(
            &["-q", "--quiet"],
            StoreTrue,
            concat!(
                "Only print guesses (and scores if --self-score is passed); ",
                "do not print prompts or info."
            ),
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
                "Run both solver strategies on every possible solution; report number of guesses ",
                "required for each. Ignores --self-score, --strategy, and --quiet."
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

    let mut state = Solver::new(&guessable_list, &solution_list, hard_mode, !quiet, strategy);

    loop {
        let guess = if enter_guesses {
            if !quiet {
                println!("Recommended: {}", state.next_guess());
            }
            read_guess_interactively(
                &mut input,
                &mut output,
                &guessable_list,
                &solution_list,
                quiet,
            )
        } else {
            let g = String::from(state.next_guess());
            if quiet {
                println!("{}", g);
            } else {
                println!("Guess: {}", g);
            }
            g
        };

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
    }
}
