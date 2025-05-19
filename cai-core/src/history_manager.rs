//! ===============================================================
//! Conversation History Module
//!
//! Manages persistent conversation history for the AI client.
//! Handles saving, loading, summarizing, and pruning of messages.
//! Ensures context is preserved and efficiently managed.
//! ===============================================================

use std::fs;

use crate::{prompt::SUMMARY_HISTORY_PROMPT, types::{ChatMessage, MessageRole}};

/// Stores and manages the conversation history.
pub struct History {
    messages: Vec<ChatMessage>,
    pub file_path: String,
    max_history: usize,
    summary_size: usize,
}

impl History {
    /// Creates a new history manager.
    ///
    /// # Arguments
    /// * `file` - Path to the history file.
    /// * `max_history` - Maximum number of messages to keep.
    /// * `summary_size` - Number of messages to summarize at a time.
    pub fn new(file: &str, max_history: usize, summary_size: usize) -> Self {
        let messages: Vec<ChatMessage> = if let Ok(file) = fs::File::open(file) {
            serde_json::from_reader(file).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };
        History {
            messages,
            file_path: file.to_string(),
            max_history,
            summary_size,
        }
    }

    /// Adds a message to the history.
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(
            ChatMessage {
                role,
                content
            }
        )
    }

    /// Checks if the history needs summarization.
    pub fn needs_summarize(&self) -> bool {
        self.messages.len() > self.max_history
    }

    /// Generates a prompt for summarizing the conversation.
    pub fn get_summarize_prompt(&mut self) -> String {
        if self.messages.len() <= self.summary_size { return "".to_string(); }

        let messages: Vec<ChatMessage> = self.messages.drain(1..self.summary_size + 1).collect();
        let messages_str = messages.iter().map(|msg| format!("{:?}", msg.content)).collect::<Vec<String>>().join("\n");

        format!("{}\n---\n{}", SUMMARY_HISTORY_PROMPT, messages_str)
    }

    /// Inserts a summary into the history.
    pub fn insert_summary(&mut self, summary: String) {
        self.messages.insert(1, ChatMessage {
            role: MessageRole::System,
            content: summary,
        });
    }

    /// Returns the current history as a vector of messages.
    pub fn get(&self) -> Vec<ChatMessage> {
        self.messages.clone()
    }

    /// Saves the history to disk.
    pub fn save(&self) {
        if let Ok(file) = fs::File::create(&self.file_path) {
            serde_json::to_writer_pretty(file, &self.messages).unwrap();
        }
    }

    /// Loads the history from disk. -- not used since we load it in the constructor
    // pub fn load(&mut self) {
    //     let messages: Vec<ChatMessage> = if let Ok(file) = fs::File::open(&self.file_path) {
    //         serde_json::from_reader(file).unwrap_or_else(|_| Vec::new())
    //     } else {
    //         Vec::new()
    //     };
    //     self.messages = messages;
    // }

    /// Clears the history.
    pub fn clear(&mut self) {
        if let Some(init) = self.messages.first().cloned() {
            self.messages = vec![init];
        } else {
            // If there's no first message, initialize with a safe system prompt
            self.messages = vec![ChatMessage {
                role: MessageRole::System,
                content: SUMMARY_HISTORY_PROMPT.to_string(),
            }];
        }

        if let Ok(file) = fs::File::create(&self.file_path) {
            serde_json::to_writer_pretty(file,  &self.messages).unwrap();
        }
    }

    /// Checks if the history is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}