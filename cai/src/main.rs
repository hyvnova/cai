//! ===============================================================
//! Console AI Framework - Main Entry Point
//!
//! This file orchestrates the CLI loop, manages user/AI interaction,
//! and coordinates memory, history, and command parsing modules.
//! ===============================================================

use std::{ env, path::PathBuf };

mod auto_git_pull;
use auto_git_pull::{check_and_pull, AutoGitStatus};



// ===================== Local Modules =====================
use cai_core::{
    // Client module -- handles AI interactions
    client::Client, constants::*, levels::inject_levels, parsers::*, prompt::*, shell::Shell, types::MessageRole, ui_trait::{MsgRole, MsgType, UIBase}
};



// ===================== Selecting UI =====================
#[cfg(feature = "cli")]
use cai_cli::UI;



// ===============================================================
/// ## Main Async Entry Point
///
/// Initializes all core components and runs the main CLI loop.
/// Handles user input, AI responses, and special command parsing.
// ===============================================================
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let auto_pull_result: AutoGitStatus = check_and_pull();
    match auto_pull_result {
        AutoGitStatus::AlreadyUpToDate | AutoGitStatus::LocalChanges  => {}

        // Since there were changes, end the program and let the user restart it.
        AutoGitStatus::Pulled => {
            println!("[SYSTEM] Reloading... New commit detected.");
            println!("[SYSTEM] Restart the program to apply the changes.");
            println!("[SYSTEM] Exiting...");
            return Ok(());
        }

        _ => {
            #[cfg(debug_assertions)]
            println!("[DEBUG] AutoGitPull: {:?}", auto_pull_result);
        }
    }


    let ui: &dyn UIBase = &UI;
    ui.init(); // Initialize the UI -- setup configuration, etc.
    

    // ===== Initialize AI Client and it's utilities =====
     // --- Ensure a OPENAI_API_KEY is set in the environment ---
     if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("[ERROR] OPENAI_API_KEY not set in the environment.");
        eprintln!("Please set it before running the program.");
        return Ok(());
    }


    // --- Determine Current Working Directory (absolute path) ---
    let mut current_path: PathBuf = env
        ::current_dir()
        .expect("Couldn't optain current dir path.")
        .canonicalize()
        .expect("Failed to create absolute path of current directory");

    // If in debug mode, create a test directory for the AI to work in.
    if cfg!(debug_assertions) {
        // Check if the directory exists, if not, create it.
        if !current_path.join("ai_test_dir").exists() {
            std::fs
                ::create_dir_all(current_path.join("ai_test_dir"))
                .expect("Failed to create test directory.");
        }
        current_path = current_path.join("ai_test_dir");
    }

    // --- Select Model (default: gpt-4.1) ---
    let model: String = std::env
        ::args()
        .collect::<Vec<String>>()
        .get(1)
        .unwrap_or(&DEFAULT_MODEL.to_string())
        .to_string();

    // --- Initialize Core Components ---
    let mut assistant: Client = Client::new();
    assistant.model = model;

    let mut shell: Shell = Shell::new(current_path.to_str().unwrap()).expect("Failed to create shell. *cries*");


    /*
    ---------------------------------------------------------------
    | continues: Number of consecutive continues
    | If it reaches MAX_CONTINUE, it will reset to 0
    | When 0, user input is required; otherwise, AI continues
    ---------------------------------------------------------------
    */
    let mut continues: usize = 1; // 1 so first response get's passed into processing loop - nasty fix but whatever

    // Buffer for the latest AI response
    let mut response: String = String::new();

    // --- System Prompt or Resume ---
    if assistant.history.is_empty() {
        response = assistant.send_message(
            ui,
            MessageRole::System,
            &get_sys_prompt(
                assistant.memory.read(None).as_str(), // Pass memory content
                current_path.to_str().unwrap(),
            ),
            None
        ).await;
    } else {
        response = assistant.send_message(ui, MessageRole::System, RESUME_PROMPT, None).await;
    }

    // ===================== CLI Commands =====================
    // Short user-sided commands to fix issues or change behavior
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
                assistant.history.clear();
            }
            // Delete memory
            "clear_memory" | "cls_m" => {
                assistant.memory.clear();
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

            let mut input: String = ui.get_user_input();

            // Provide a default message if input is empty.
            if input.is_empty() {
                input = "No Message".to_string();
            }

            // Exit condition.
            if input.eq_ignore_ascii_case("q") {
                assistant.history.save();
                println!("[SYSTEM] Chat history saved. Exiting.");
                break;
            }

            println!();

            // response = assistant.send_message(ui, MessageRole::User, &input, None).await;

            // Apply levels of reasoning
            inject_levels(&mut response, &input, &mut assistant, ui).await;
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

            // Handles file write instructions
            if let Err(e) = parse_write_block(ui, &response) {
                // Add to response the failure message
                sys_message.push_str(&format!("[Write Block Failed]\n{}\n", e));
            }

            parse_python_block(ui, &response, &mut sys_message); // Handles python code execution
            parse_commands_block(ui, &response, &mut shell, &mut sys_message); // Handles shell commands
            // parse_memory_block(ui, &response, &mut assistant, &mut sys_message); // Handles memory updates

            // * Token processing ----------------------------
            if response.contains(RESTART_TOKEN) {
                assistant.history.save();

                ui.print_message(
                    MsgRole::System,
                    MsgType::Plain("[SYSTEM] Chat history saved. Restarting....".to_string()),
                );

                break 'mainloop;
            }

            // --- Continue token logic ---
            if response.contains(CONTINUE_TOKEN) {
                ui.print_message(
                    MsgRole::System,
                    MsgType::Plain("[SYSTEM] Continuing".to_string()),
                );

                if sys_message.is_empty() {
                    response = assistant.send_message(ui, MessageRole::System, "[Continue]", None).await;
                } else {
                    response = assistant.send_message(ui, MessageRole::System, &sys_message, None).await;
                }
                continues += 1;

                // Enforce continue limit.
                if continues >= MAX_CONTINUE {
                    continues = 0;
                    response = assistant.send_message(
                        ui,
                        MessageRole::System,
                        "[You've reached the maximum number of continues.]",
                        None
                    ).await;
                    break 'processing_loop;
                }

                continue 'processing_loop;
            } else {
                // If there was a system message, send it and continue processing.
                if !sys_message.is_empty() {
                    response = assistant.send_message(ui, MessageRole::System, &sys_message, None).await;
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