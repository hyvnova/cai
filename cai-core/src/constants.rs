// ===================== Configuration Constants =====================

// Default names
pub const DEFAULT_HISTORY_FILE_NAME: &str = "history.json";
pub const DEFAULT_MEMORY_FILE_NAME: &str = "memory.json";

/// Maximum number of messages to keep in conversation history.
pub const MAX_HISTORY: usize = 28;

/// Number of messages to summarize (should be < MAX_HISTORY).
pub const SUMMARY_SIZE: usize = MAX_HISTORY / 3;

/// Maximum allowed consecutive continue tokens before requiring user input.
pub const MAX_CONTINUE: usize = 20;

/// Special tokens for control flow in AI responses.
pub const RESTART_TOKEN: &str = "$$RESTART$$";
pub const CONTINUE_TOKEN: &str = "$$CONTINUE$$";

/// Default model to use if none is specified.
/// o4-mini | gpt-4.1 | gpt-3.5-turbo
pub const DEFAULT_MODEL: &str = "gpt-4.1";

/// Language and OS for the AI to use in its responses.
pub const LANGUAGE: &str = "Español (Incluyendo jerga y modismos contemporáneos propios de la juventud).";

#[cfg(unix)]
pub const OS: &str = "Linux (Ubuntu 22.04)";

#[cfg(windows)]
pub const OS: &str = "Windows 11";
