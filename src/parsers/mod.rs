//! ===============================================================
//! Parsers Module
//!
//! Aggregates all block parsers for special AI response blocks.
//! Each submodule handles a specific block type (commands, memory, voice, write).
//! Provides a unified interface for block parsing in the CLI loop.
//! ===============================================================

pub mod commands;
pub mod memory;
pub mod voice;
pub mod write;