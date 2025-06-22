//! ===============================================================
//! AI Client Module
//!
//! Encapsulates the logic for interacting with the AI model (e.g., OpenAI GPT).
//! Handles message sending, API requests, and response management.
//! Provides a clean interface for the CLI to communicate with the AI.
//! ===============================================================

use crate::client_util::call_with_backoff;
use crate::passive_context::passive_context;
use crate::types::MessageRole;
use crate::ui_trait::MsgType;
use crate::ui_trait::{ MsgRole, UIBase };
use crate::{ history_manager::History, memory_manager::MemoryManager };

use async_openai::{ Client as APIClient, config::OpenAIConfig };
use serde_json::{ json, Value };
use super::constants::*;

const INDEPENDENT_MAX_TOKENS: u32 = 4000; // Max tokens for independent requests
const MAX_TOKENS: u32 = 32000; // Max tokens for chat requests
const MAX_RETRIES: usize = 10; // Max retries for API requests

/// Represents the main AI client for chat interaction.
pub struct Client {
    model: String,
    pub history: History,
    pub memory: MemoryManager,
    ai: APIClient<OpenAIConfig>,
}

impl Client {
    /// Creates a new AI client instance using the default model and configuration.
    ///
    /// # Arguments
    /// * `model` - The model name (e.g., "gpt-4.1").
    /// * `history` - Conversation history manager.
    /// * `memory` - Persistent memory manager.
    /// * `api_key` - API key for authentication.
    pub fn new() -> Self {
        let model: String = DEFAULT_MODEL.to_string();

        println!("[DEBUG] Model={}", model);

        // Create a OpenAI client with api key from env var OPENAI_API_KEY and default base url.
        let ai: APIClient<OpenAIConfig> = APIClient::new();

        Self {
            model,
            history: History::new(DEFAULT_HISTORY_FILE_NAME, MAX_HISTORY, SUMMARY_SIZE),
            memory: MemoryManager::new(DEFAULT_MEMORY_FILE_NAME),
            ai,
        }
    }

    /// Just a normal system request to the AI, doesn't save the response or uses the history
    pub async fn make_independent_request(
        &mut self,
        content: &str,
        model: Option<String>
    ) -> Result<String, String> {
        let result = call_with_backoff(
            &self.ai,
            json!({
                "model": model.unwrap_or_else(|| self.model.clone()),
                "messages": [
                    {
                        "role": "system",
                        "content": content
                    }
                ],
                "stream": false,
                "max_completion_tokens": INDEPENDENT_MAX_TOKENS,
            })
        ).await;

        match result {
            Ok(content) => {
                // println!("[DEBUG] Independent request successful");
                Ok(content)
            }

            Err(()) => Err("Failed to make independent request".to_string()),
        }
    }

    /// Creates an inserts a chat history summary into the history. Should be called if `needs_summarize()` returns true.
    /// If ok -> returns the summary, otherwise returns an error message.
    pub async fn perform_history_summary(&mut self) -> Result<String, String> {
        let prompt = self.history.get_summarize_prompt(); // Drains messages here
        match self.make_independent_request(&prompt, None).await {
            Ok(summary) => {
                self.history.insert_summary(format!("[Conversation summary]\n{}", summary));
                Ok(summary) 
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    /// Sends a message to the AI and returns the response.
    ///
    /// # Arguments
    /// * `role` - The role of the message sender (user/system).
    /// * `content` - The message content.
    pub async fn send_message(
        &mut self,
        ui: &dyn UIBase,
        role: MessageRole,
        content: &str
    ) -> String {
        self.history.add_message(role, content.to_string());

        // Add passive context to the message --
        // ! REMOVE AFTER REQUEST
        self.history.add_message(MessageRole::System, passive_context());

        // * Keep history small but informative
        if self.history.needs_summarize() {
            match self.perform_history_summary().await {
                Ok(_) => {
                    // println!("[DEBUG] History summary performed successfully");
                }
                Err(e) => {
                    ui.print_message(
                        MsgRole::System,
                        MsgType::Plain(format!("[ERROR] Summarization failed: {}", e.to_string()))
                    );

                    // Save history to disk
                    self.history.save();
                    // Exit the program
                    std::process::exit(1);
                }
                
            }
        }

        // --- START DEBUG ---
        // Print the request structure just before sending
        // Use serde_json to attempt serialization and print the result or error
        // match serde_json::to_string_pretty(&req) {
        //     Ok(json_string) => {
        //         println!("[DEBUG] Request JSON Payload:\n{}", json_string);
        //     }
        //     Err(e) => {
        //         eprintln!("[DEBUG] FAILED TO SERIALIZE REQUEST TO JSON: {:?}", e);
        //         // Optionally print the raw request object too
        //         eprintln!("[DEBUG] Raw Request Object: {:#?}", req);
        //     }
        // }
        // --- END DEBUG ---

        let response = call_with_backoff(
            &self.ai,
            json!({
                "model": self.model.clone(),
                "messages": self.history.get(),
                "stream": false,
                "max_completion_tokens": MAX_TOKENS,
            })
        ).await;

        // ! REMOVING PASSIVE CONTEXT
        self.history.messages.pop();

        match response {
            Ok(content) => {
                // println!("[DEBUG] Request successful");
                self.history.add_message(MessageRole::Assistant, content.to_string());
                ui.print_message(MsgRole::Assistant, MsgType::Plain(content.to_string()));
                return content;
            }
            Err(()) => {
                eprintln!("[ERROR] No content in response.");
                self.history.add_message(
                    MessageRole::System,
                    format!("[ERROR] No content in response.")
                );
                return String::from("[No message]");
            }
        }
    }
}
