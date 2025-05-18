use std::{io::{BufReader, Write}, process::{Child, Command, Stdio}};


use super::util::{ensure_dir, run_command_loop, ShellBackend};

// ────────────────────────────────────────────────────────────────
//  POSIX-sh  (Linux / macOS / *BSD)
// ────────────────────────────────────────────────────────────────
#[cfg(unix)]
pub struct ShShell {
    pub child: Child,
    pub stdin: std::process::ChildStdin,
    pub stdout: BufReader<std::process::ChildStdout>,
}

#[cfg(unix)]
impl ShShell {
    pub fn spawn(cwd: &str) -> anyhow::Result<Self> {
        let cwd = ensure_dir(cwd)?;

        let mut child = Command::new("sh")
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("sh stdin");
        let stdout = BufReader::new(child.stdout.take().expect("sh stdout"));

        // Suppress prompt noise (empty PS1)
        writeln!(stdin, "PS1=''")?;
        stdin.flush()?;

        Ok(Self { child, stdin, stdout })
    }
}

#[cfg(unix)]
impl ShellBackend for ShShell {
    fn execute(&mut self, command: &str, timeout_secs: Option<u64>) -> anyhow::Result<String> {
        run_command_loop(
            &mut self.child,
            &mut self.stdin,
            &mut self.stdout,
            command,
            timeout_secs,
        )
    }
}