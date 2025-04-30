//! ===============================================================
//! Persistent Memory Module
//!
//! Manages long-term memory for the AI client.
//! Allows reading, writing, and clearing of persistent memory.
//! Used to provide context and continuity across sessions.
//! ===============================================================

use std::fs::{ self, OpenOptions };
use std::io::Write;

use regex::Regex;

/// Handles persistent memory storage and retrieval.
pub struct MemoryManager {
    pub file_path: String,
}

impl MemoryManager {
    /// Creates a new memory manager.
    ///
    /// # Arguments
    /// * `file` - Path to the memory file.
    pub fn new(file: &str) -> Self {
        MemoryManager {
            file_path: file.to_string(),
        }
    }

    /// Reads memory from disk or returns the current memory.
    ///
    /// # Arguments
    /// * `default` - Optional default value if memory is empty.
    pub fn read(&self, default: Option<&str>) -> String {
        println!("[SYSTEM] Viewing memory: {}", default.unwrap_or("all"));

        let content = fs::read_to_string(&self.file_path).unwrap_or_else(|_| "No memory found.".to_string());

        if let Some(pat) = default {
            content
                .lines()
                .filter(|line| Regex::new(pat).unwrap().is_match(line))
                .collect::<Vec<&str>>()
                .join("\n")
        } else {
            content
        }
    }

    /// Writes new memory to disk.
    pub fn write(&mut self, content: &str) {
        let mut file = OpenOptions::new().append(true).create(true).open(&self.file_path).unwrap();
        println!("[SYSTEM] Adding to memory: {}", content);
        writeln!(file, "{}", content).unwrap();

    }

    /// Clears the memory file.
    pub fn clear(&mut self) {
        let _ = fs::write(&self.file_path, "");
    }
}