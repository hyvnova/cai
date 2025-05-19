//! ===============================================================
//! Parsers Module
//!
//! Aggregates all block parsers for special AI response blocks.
//! Each submodule handles a specific block type (commands, memory, voice, write).
//! Provides a unified interface for block parsing in the CLI loop.
//! ===============================================================

mod commands;
pub use commands::parse_commands_block;

mod memory;
pub use memory::parse_memory_block;

mod write;
pub use write::parse_write_block;