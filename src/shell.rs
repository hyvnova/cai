//! ===============================================================
//! Cross-Platform Shell Abstraction
//! ===============================================================

use crate::shells::{self, util::ShellBackend};


// ────────────────────────────────────────────────────────────────
// Shell struct -- public API used or run AI's commands
// ────────────────────────────────────────────────────────────────
pub struct Shell {
    inner: Box<dyn ShellBackend>,
}

impl Shell {
    /// Create a shell rooted at `cwd`.  Picks the right backend for the OS.
    pub fn new(cwd: &str) -> anyhow::Result<Self> {
        let inner: Box<dyn ShellBackend> = {
            #[cfg(windows)]
            {
                Box::new(shells::PowerShell::spawn(cwd)?)
            }
            #[cfg(unix)]
            {
                Box::new(shells::ShShell::spawn(cwd)?)
            }
        };

        Ok(Self { inner })
    }

    /// Delegate to the concrete backend.
    pub fn execute(&mut self, command: &str, timeout_secs: Option<u64>) -> anyhow::Result<String> {
        self.inner.execute(command, timeout_secs)
    }
}




