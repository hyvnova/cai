//! ===============================================================
//! Console AI Framework - Main Entry Point
//!
//! This file orchestrates the CLI loop, manages user/AI interaction,
//! and coordinates memory, history, and command parsing modules.
//! ===============================================================

use std::io::{ self, BufRead, Write };
use std::{env, path::PathBuf};

use colored::Colorize;
use parsers::{
    commands::parse_commands_block,
    memory::parse_memory_block,
    voice::parse_say_block,
    write::parse_write_block,
};

// ===================== Local Modules =====================
mod prompt;
use prompt::{ format_sys_prompt, MEMORY_PROMPT, RESUME_PROMPT };

mod memory_manager;
use memory_manager::MemoryManager;

mod history_manager;
use history_manager::History;

mod client;
use client::Client;

mod shell;
use shell::Shell;
use types::MessageRole;

mod parsers;
mod client_util;
mod text_macro;
mod types;

// ===================== Configuration Constants =====================

/// Maximum number of messages to keep in conversation history.
const MAX_HISTORY: usize = 28;

/// Number of messages to summarize (should be < MAX_HISTORY).
const SUMMARY_SIZE: usize = MAX_HISTORY / 3;

/// Maximum allowed consecutive continue tokens before requiring user input.
const MAX_CONTINUE: usize = 20;

/// Special tokens for control flow in AI responses.
const RESTART_TOKEN: &str = "$$RESTART$$";
const CONTINUE_TOKEN: &str = "$$CONTINUE$$";

/// Default model to use if none is specified.
/// o4-mini | gpt-4.1 | gpt-3.5-turbo
const DEFAULT_MODEL: &str = "o4-mini-2025-04-16";

// ===============================================================
/// ## Main Async Entry Point
///
/// Initializes all core components and runs the main CLI loop.
/// Handles user input, AI responses, and special command parsing.
// ===============================================================
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // --- Determine Current Working Directory (absolute path) ---
    let current_path: PathBuf = env
        ::current_dir()
        .expect("Couldn't optain current dir path.")
        .canonicalize()
        .expect("Failed to create absolute path of current directory");

    // --- Select Model (default: gpt-4.1) ---
    let model: String = std::env
        ::args()
        .collect::<Vec<String>>()
        .get(1)
        .unwrap_or(&DEFAULT_MODEL.to_string())
        .to_string();

    // --- Initialize Core Components ---
    let mut ai: Client = Client::new(
        model,
        History::new("history.json", MAX_HISTORY, SUMMARY_SIZE),
        MemoryManager::new("memory.txt")
    );

    let mut shell = Shell::new(current_path.to_str().unwrap());

    /*
    ---------------------------------------------------------------
    | continues: Number of consecutive continues
    | If it reaches MAX_CONTINUE, it will reset to 0
    | When 0, user input is required; otherwise, AI continues
    ---------------------------------------------------------------
    */
    let mut continues: usize = 0;

    // Buffer for the latest AI response
    let mut response: String = String::new();

    // --- System Prompt or Resume ---
    if ai.history.is_empty() {
        response = ai.send_message(
            MessageRole::System,
            &format_sys_prompt(
                CONTINUE_TOKEN,
                RESTART_TOKEN,
                ai.memory.read(None).as_str(), // Pass memory content
                MEMORY_PROMPT, // Pass memory instructions
                current_path.to_str().unwrap()
            )
        ).await;
    } else {
        response = ai.send_message(MessageRole::System, RESUME_PROMPT).await;
    }

    // ===================== CLI Commands =====================
    // Short user-sided commands to fix issues or change behavior
    // Collect all command-line arguments into a vector
    let args: Vec<String> = env
        ::args()
        .collect::<Vec<String>>()
        .get(2..)
        .unwrap_or(&[])
        .to_vec();

    for arg in args.iter() {
        match arg.to_lowercase().as_str() {
            // Delete History
            "clear_history" | "cls_h" => {
                ai.history.clear();
            }
            // Delete memory
            "clear_memory" | "cls_m" => {
                ai.memory.clear();
            }
            // Unknown command
            other => {
                eprintln!("Unknown command: {}", other);
            }
        }
    }

    // ===============================================================
    // Main Interaction Loop
    //
    // Keeps the interaction between the user and the AI.
    // Also processes the AI response for special tokens and blocks.
    // ===============================================================
    'mainloop: loop {
        // --- User Input Phase ---
        if continues == 0 {
            let header = "[You]: ".green().bold();
            print!("{}", header);
            io::stdout().flush().unwrap();

            let mut input: String = String::new();
            io::stdin().lock().read_line(&mut input).unwrap();
            let mut input: String = input.trim().to_string();

            // Provide a default message if input is empty.
            if input.is_empty() {
                input = "No Message".to_string();
            }

            // Exit condition.
            if input.eq_ignore_ascii_case("q") {
                ai.history.save();
                println!("[SYSTEM] Chat history saved. Exiting.");
                break;
            }

            println!();

            response = ai.send_message(MessageRole::User, &input).await;
        }

        // --- AI Response Processing Phase ---
        'processing_loop: loop {
            /*
            ---------------------------------------------------------------
            | sys_message: Message that will be sent to AI after response processing
            | When empty, no further processing is needed
            | When not empty, triggers another AI processing round
            ---------------------------------------------------------------
            */
            let mut sys_message: String = String::new();

            // --- Parse and Execute Special Blocks ---
            parse_write_block(&response); // Handles file write instructions
            parse_say_block(&response, &mut shell); // Handles voice/say instructions
            parse_commands_block(&response, &mut shell, &mut sys_message); // Handles shell commands
            parse_memory_block(&response, &mut ai, &mut sys_message); // Handles memory updates

            // * Token processing ----------------------------
            if response.contains(RESTART_TOKEN) {
                ai.history.save();
                println!("[SYSTEM] Chat history saved. Restarting.");
                break 'mainloop;
            }

            // --- Continue token logic ---
            if response.contains(CONTINUE_TOKEN) {
                println!("[SYSTEM] Continuing.");

                if sys_message.is_empty() {
                    response = ai.send_message(MessageRole::System, "[Continue]").await;
                } else {
                    response = ai.send_message(MessageRole::System, &sys_message).await;
                }
                continues += 1;

                // Enforce continue limit.
                if continues >= MAX_CONTINUE {
                    continues = 0;
                    response = ai.send_message(
                        MessageRole::System,
                        "[You've reached the maximum number of continues.]"
                    ).await;
                    break 'processing_loop;
                }

                continue 'processing_loop;
            } else {
                // If there was a system message, send it and continue processing.
                if !sys_message.is_empty() {
                    response = ai.send_message(MessageRole::System, &sys_message).await;
                }
                // Reset continue counter if needed.
                if continues != 0 {
                    continues = 0;
                }
                // If no further processing is needed, break.
                if sys_message.is_empty() {
                    break 'processing_loop;
                } else {
                    continue 'processing_loop;
                }
            }
        }
    }

    Ok(())
}
// ===================== End of main.rs =====================
