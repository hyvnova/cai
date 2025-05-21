pub mod ui_trait;


// Client module -- handles AI interactions
pub mod client;

// Contains the prompt templates and instructions
pub mod prompt;

// Contains the parsers for different blocks in AI responses
pub mod parsers;

// Contains the shell -- used to run commands
pub mod shell;

// Contains the shell implementations for different platforms
pub mod shells;
pub mod types;

// Handles the history of messages
pub mod history_manager;

// Memory module -- handles memory management
pub mod memory_manager;

// Essentially a wrapper around the OpenAI API to respect the rate limits
pub mod client_util;