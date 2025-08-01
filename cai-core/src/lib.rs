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

// Contains the passive context for the AI -- used to make the AI "think" better
pub mod passive_context;

// Contains the configuration constants
pub mod constants;

pub mod models;

// Levels of reasoning -- provides an injection function that, after initial AI solution/response will inject the 9 levels
// To further fuck up the response
pub mod levels;