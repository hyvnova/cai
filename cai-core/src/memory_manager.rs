//! ===============================================================
//! Persistent Memory Module
//!
//! Manages long-term memory for the AI client.
//! Allows reading, writing, and clearing of persistent memory.
//! Used to provide context and continuity across sessions.
//! ===============================================================


use regex::Regex;

/// Handles persistent memory storage and retrieval.
pub struct MemoryManager {
    pub file_path: String,
    memory: Vec<String>,
}

impl MemoryManager {
    /// Creates a new memory manager and loads memory from disk.
    pub fn new(file: &str) -> Self {
        let mut manager = MemoryManager {
            file_path: file.to_string(),
            memory: Vec::new(),
        };
        manager.load();
        manager
    }

    /// Adds a new memory fragment and saves to disk.
    pub fn add(&mut self, content: &str) {
        self.memory.push(content.trim().to_string());
        self.save();
        println!("[SYSTEM] Added to memory: {}", content);
    }

    /// Updates the first memory fragment matching `pattern` with `replacement` and saves.
    pub fn update(&mut self, pattern: &str, replacement: &str) {
        let re = Regex::new(pattern).unwrap();
        if let Some(pos) = self.memory.iter().position(|line| re.is_match(line)) {
            self.memory[pos] = replacement.trim().to_string();
            self.save();
            println!("[SYSTEM] Updated memory: '{}' -> '{}'", pattern, replacement);
        } else {
            println!("[SYSTEM] No memory matched pattern: {}", pattern);
        }
    }

    /// Deletes the first memory fragment matching `pattern` and saves.
    pub fn delete(&mut self, pattern: &str) {
        let re = Regex::new(pattern).unwrap();
        if let Some(pos) = self.memory.iter().position(|line| re.is_match(line)) {
            let removed = self.memory.remove(pos);
            self.save();
            println!("[SYSTEM] Deleted memory: {}", removed);
        } else {
            println!("[SYSTEM] No memory matched pattern: {}", pattern);
        }
    }

    /// Reads memory, optionally filtering by a pattern.
    pub fn read(&self, pattern: Option<&str>) -> String {
        if let Some(pat) = pattern {
            let re = Regex::new(pat).unwrap();
            self.memory
                .iter()
                .filter(|line| re.is_match(line))
                .cloned()
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            self.memory.join("\n")
        }
    }

    /// Loads memory from disk into memory vector.
    pub fn load(&mut self) {
        self.memory = std::fs::read_to_string(&self.file_path)
            .map(|content| content.lines().map(|l| l.to_string()).collect())
            .unwrap_or_else(|_| Vec::new());
    }

    /// Saves the current memory vector to disk.
    pub fn save(&self) {
        let _ = std::fs::write(&self.file_path, self.memory.join("\n"));
    }

    /// Clears the memory file and in-memory vector.
    pub fn clear(&mut self) {
        self.memory.clear();
        self.save();
    }
}