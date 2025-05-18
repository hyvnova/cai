use std::{
    io::{ BufRead, BufReader, Write },
    path::{ Path, PathBuf },
    process::Child,
    thread,
    time::{Duration, Instant},
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

// ────────────────────────────────────────────────────────────────
// Shared engine for executing a command and capturing output
// (keeps the two impls DRY).
// ────────────────────────────────────────────────────────────────
pub fn run_command_loop(
    child: &mut Child,
    stdin: &mut std::process::ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
    command: &str,
    timeout_secs: Option<u64>
) -> anyhow::Result<String> {
    const MARKER: &str = "[END_OF_COMMAND_OUTPUT]";

    // 1. send the command
    writeln!(stdin, "{}", command)?;
    stdin.flush()?;

    // 2. send the marker
    writeln!(stdin, "echo {}", MARKER)?;
    stdin.flush()?;

    // 3. read until we hit the marker (or timeout / EOF)
    let start = Instant::now();
    let mut buf = String::new();
    let mut line = String::new();

    loop {
        line.clear();

        // timeout?
        if let Some(limit) = timeout_secs {
            if start.elapsed() > Duration::from_secs(limit) {
                let _ = child.kill();
                buf.push_str(&format!("\n[Execution timed-out after {}s]", limit));
                break;
            }
        }

        match stdout.read_line(&mut line) {
            Ok(0) => {
                break;
            } // EOF
            Ok(_) => {
                if line.trim() == MARKER {
                    break;
                }
                buf.push_str(&line);
            }
            Err(e) => {
                buf.push_str(&format!("\n[Shell read error: {}]", e));
                break;
            }
        }

        if timeout_secs.is_some() {
            thread::sleep(Duration::from_millis(25));
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
