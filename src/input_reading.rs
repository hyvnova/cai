use std::io::{self, BufRead};

/// Read stdin based on the following rules:
/// - If the input starts with `;;`, continue reading until another `;;` is encountered.
///   Remove the `;;` markers from the final input.
/// - Otherwise, read a single line of input.
pub fn read_user_input() -> io::Result<String> {
    let stdin = io::stdin();
    let mut buf = String::new();
    let mut lines = stdin.lock().lines();

    if let Some(first_line_result) = lines.next() {
        let first_line = first_line_result?;
        if first_line.trim_start() == ";;" {
            // Read until another `;;` is encountered
            for line_result in lines {
                let line = line_result?;
                if line.trim_end() == ";;" {
                    break;
                }
                buf.push_str(&line);
                buf.push('\n'); // Keep \n for normalization later
            }
        } else {
            // Single-line input
            buf = first_line;
        }
    }

    // Normalize CRLF first, collapse newlines to spaces, trim ends
    let cleaned = buf
        .replace("\r\n", "\n")
        .replace('\n', " ")
        .trim()
        .to_string();

    Ok(cleaned)
}
