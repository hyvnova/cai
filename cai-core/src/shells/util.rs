use std::{
    borrow::Cow, io::{ BufRead, BufReader, Write }, path::{ Path, PathBuf }, process::Child, time::{Duration, Instant}
};

use lazy_static::lazy_static;
use regex::Regex;

// ────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────
lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
}

fn strip_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

/// Run a single shell *command*, capture everything up to the sentinel,
/// tolerate non-UTF-8 bytes, and optionally time-out.
///
/// `timeout_secs == None`  → wait forever.
///
/// **Note**: we now redirect the command’s stderr (file-descriptor 2)
/// into stdout (file-descriptor 1) so both streams are captured.
pub fn run_command_loop(
    child:  &mut Child,
    stdin:  &mut std::process::ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
    command: &str,
    timeout_secs: Option<u64>,
) -> anyhow::Result<String> {
    const MARKER: &str = "[END_OF_COMMAND_OUTPUT]";

    //----------------------------------------------------------------------
    // 1. send the command itself, but push "2>&1" so stderr is folded
    //----------------------------------------------------------------------
    writeln!(stdin, "{} 2>&1", command)?;   // <-- change is right here
    stdin.flush()?;

    // 2. emit the sentinel
    writeln!(stdin, "echo {}", MARKER)?;
    stdin.flush()?;

    //----------------------------------------------------------------------
    // 3. read lines until we see the sentinel or we time-out
    //----------------------------------------------------------------------
    let start = Instant::now();
    let mut buf      = String::new();
    let mut raw_line = Vec::<u8>::new();

    loop {
        // … unchanged timeout / read-loop logic …
        if let Some(limit) = timeout_secs {
            if start.elapsed() > Duration::from_secs(limit) {
                let _ = child.kill();
                let _ = child.wait(); // reap zombie
                buf.push_str(&format!("\n[Execution timed-out after {limit}s]"));
                break;
            }
        }

        raw_line.clear();
        match stdout.read_until(b'\n', &mut raw_line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                // convert bytes → string, never error
                let line = match std::str::from_utf8(&raw_line) {
                    Ok(s)  => Cow::Borrowed(s),
                    Err(_) => Cow::Owned(String::from_utf8_lossy(&raw_line).into_owned()),
                };

                if line.trim() == MARKER {
                    break;
                }
                buf.push_str(&line);
            }
            Err(e) => {
                buf.push_str(&format!("\n[Shell read error: {e}]"));
                break;
            }
        }
    }

    Ok(strip_ansi_codes(&buf))
}

pub fn ensure_dir<P: AsRef<Path>>(p: P) -> anyhow::Result<PathBuf> {
    let pb = PathBuf::from(p.as_ref());
    if !pb.exists() {
        std::fs::create_dir_all(&pb)?;
    }
    Ok(pb)
}



// ────────────────────────────────────────────────────────────────
// Trait that every shell implementation must satisfy
// ────────────────────────────────────────────────────────────────
pub trait ShellBackend: Send {
    /// Execute a command and return its captured stdout (with ANSI stripped).
    /// `timeout_secs == None`  ➜ wait forever.
    fn execute(&mut self, command: &str, timeout_secs: Option<u64>) -> anyhow::Result<String>;
}
