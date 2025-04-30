//! ===============================================================
//! Shell Abstraction Module
//!
//! Provides an interface for executing shell commands and managing the working directory.
//! Enables the AI to interact with the system shell in a controlled manner.
//! ===============================================================

use std::io::{BufReader, BufRead, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio, Child};
use std::time::{Duration, Instant};
use std::thread;

use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
}

fn strip_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

/// OS shell picker
fn pick_shell_binary() -> &'static str {
    if cfg!(windows) {
        "pwsh.exe"
    } else {
        "sh"
    }
}

/// Represents a shell session for command execution.
pub struct Shell {
    child: Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

const DEFAULT_SHELL_ARGS: [&str; 3] = ["-NoLogo", "-NoProfile", "-NonInteractive"];

impl Shell {
    /// Creates a new shell instance with the given working directory.
    pub fn new(cwd: &str) -> Self {
        let init_path = PathBuf::from(cwd);
        if !init_path.exists() {
            // create the directory if it doesn't exist
            std::fs::create_dir_all(&init_path).expect("failed to create shell's initial directory");
        }

        let mut child = Command::new(pick_shell_binary())
            .args(&DEFAULT_SHELL_ARGS)
            .current_dir(PathBuf::from(init_path))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn shell");      

        let mut stdin = child.stdin.take().expect("failed to open stdin");
        let stdout = child.stdout.take().expect("failed to open stdout");
        let stdout = BufReader::new(stdout);

        // Disable the prompt to prevent clutter.
        writeln!(stdin, "function prompt {{ '' }}").expect("failed to write command");
        stdin.flush().expect("failed to flush command");

        Shell { child, stdin, stdout }
    }

    /// Executes a shell command and returns the output.
    pub fn execute(&mut self, command: &str, timeout_seconds: Option<u64>) -> String {
        let marker = "[END_OF_COMMAND_OUTPUT]";
        writeln!(self.stdin, "{}", command).expect("failed to write command");
        self.stdin.flush().expect("failed to flush command");
        writeln!(self.stdin, "echo {}", marker).expect("failed to write marker");
        self.stdin.flush().expect("failed to flush marker");

        let mut output = String::new();
        let mut line = String::new();

        let start_time = Instant::now();
        loop {
            line.clear();

            // If timeout is set, check before reading line
            if let Some(limit) = timeout_seconds {
                if start_time.elapsed() > Duration::from_secs(limit) {
                    // Kill the shell process
                    let _ = self.child.kill();
                    let msg = format!("Stopped execution. Command `{}` timeout.", command);
                    output.push_str(&msg);
                    break;
                }
            }

            // set a short read timeout by making the stream non-blocking in a hacky way
            // but .read_line() on BufReader doesn't support timeouts, so use a short thread::sleep
            match self.stdout.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if line.trim() == marker {
                        break;
                    }
                    output.push_str(&line);
                }
                Err(e) => {
                    output.push_str(&format!("\n[Shell error: {}]", e));
                    break;
                }
            }
            // If timeout is set, let this loop react quickly
            if timeout_seconds.is_some() {
                thread::sleep(Duration::from_millis(25));
            }
        }

        strip_ansi_codes(&output)
    }
}