pub mod eval;
pub mod loader;
pub mod score;
pub mod solver;

pub fn read_guess_interactively<'a>(
    input: &mut dyn std::io::BufRead,
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
