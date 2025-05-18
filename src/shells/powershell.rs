use std::{io::{BufReader, Write}, process::{Child, Command, Stdio}};


use super::util::{ensure_dir, run_command_loop, ShellBackend};


// ────────────────────────────────────────────────────────────────
// PowerShell  (Windows)
// ────────────────────────────────────────────────────────────────
#[cfg(windows)]
pub struct PowerShell {
    pub child: Child,
    pub stdin: std::process::ChildStdin,
    pub stdout: BufReader<std::process::ChildStdout>,
}

#[cfg(windows)]
impl PowerShell {
    pub fn spawn(cwd: &str) -> anyhow::Result<Self> {
        let cwd = ensure_dir(cwd)?;

        let mut child = Command::new("pwsh.exe")
            .args(["-NoLogo", "-NoProfile", "-NonInteractive"])
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("pwsh stdin");
        let stdout = BufReader::new(child.stdout.take().expect("pwsh stdout"));

        // Suppress prompt noise
        writeln!(stdin, "function prompt {{ '' }}")?;
        stdin.flush()?;

        Ok(Self { child, stdin, stdout })
    }
}

#[cfg(windows)]
impl ShellBackend for PowerShell {
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
