use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::path::Path;

/// Read a word list from a file (one word per line).
pub fn load_list_from_file(path: &Path) -> Result<Vec<String>, Error> {
    let reader = File::open(path)?;
    let mut bufreader = BufReader::new(reader);

    let mut result = Vec::new();
    let mut buffer = String::new();
    while bufreader.read_line(&mut buffer)? > 0 {
        let trimmed = buffer.trim_end();
        if !trimmed.as_bytes().iter().all(u8::is_ascii_lowercase) || trimmed.len() != 5 {
            let msg = format!("Invalid word: {} (must be 5 lowercase letters)", trimmed);
            return Err(Error::new(ErrorKind::InvalidData, msg));
        }
        result.push(String::from(trimmed));
        buffer.clear();
    }

    Ok(result)
}
