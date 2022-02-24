use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Read a word list from a file (one word per line).
pub fn load_list_from_file(path: &Path) -> Result<Vec<String>, std::io::Error> {
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
