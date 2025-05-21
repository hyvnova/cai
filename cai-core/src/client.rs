//! ===============================================================
//! AI Client Module
//!
//! Encapsulates the logic for interacting with the AI model (e.g., OpenAI GPT).
//! Handles message sending, API requests, and response management.
//! Provides a clean interface for the CLI to communicate with the AI.
//! ===============================================================

use crate::types::MessageRole;
use crate::ui_trait::MsgType;
use crate::ui_trait::{MsgRole, UIBase};
use crate::{ history_manager::History, memory_manager::MemoryManager };

use async_openai::{ Client as APIClient, config::OpenAIConfig };
use async_openai::types::CreateCompletionRequestArgs;
use serde_json::{json, Value};

const INDEPENDENT_MAX_TOKENS: u32 = 4000; // Max tokens for independent requests
const MAX_TOKENS: u32 = 32000; // Max tokens for chat requests

/// Represents the main AI client for chat interaction.
pub struct Client {
    model: String,
    pub history: History,
    pub memory: MemoryManager,
    ai: APIClient<OpenAIConfig>,
    last_req_time: std::time::Instant,
}

impl Client {
    /// Creates a new AI client instance.
    ///
    /// # Arguments
    /// * `model` - The model name (e.g., "gpt-4.1").
    /// * `history` - Conversation history manager.
    /// * `memory` - Persistent memory manager.
    /// * `api_key` - API key for authentication.
    pub fn new(model: String, history: History, memory: MemoryManager) -> Self {
        println!("[DEBUG] Model={}", model);

        // Create a OpenAI client with api key from env var OPENAI_API_KEY and default base url.
        let ai: APIClient<OpenAIConfig> = APIClient::new();

        Self {
            model,
            history,
            memory,
            ai,
            last_req_time: std::time::Instant::now(),
        }
    }

    /// Just a normal system request to the AI, doesn't save the response or uses the history
    pub async fn make_independent_request(&mut self, content: &str) -> Result<String, String> {

        // Make the request to the AI
        let result: Value = self.ai
            .chat()
            .create_byot(
                json!({
                    "model": self.model.clone(),
                    "messages": [
                        {
                            "role": "system",
                            "content": format!("[SYSTEM] {}", content)
                        }
                    ],
                    "stream": false,
                    "max_completion_tokens": INDEPENDENT_MAX_TOKENS,
                })
            )
            .await
            .expect(
                "Failed to get response for independent request"
            );

        if let Some(content) = result["choices"][0]["message"]["content"].as_str() {
            return Ok(content.to_string());
        } else {
            return Err("[ERROR] No content in response for independent request".to_string());
        }
    }

    /// Sends a message to the AI and returns the response.
    ///
    /// # Arguments
    /// * `role` - The role of the message sender (user/system).
    /// * `content` - The message content.
    pub async fn send_message(&mut self, ui: &dyn UIBase, role: MessageRole, content: &str) -> String {
        self.history.add_message(role, content.to_string());

        // * Keep history small but informative
        if self.history.needs_summarize() {
            let prompt = self.history.get_summarize_prompt(); // Drains messages here
            match self.make_independent_request(&prompt).await {
                Ok(summary) => {
                    self.history.insert_summary(format!("[Conversation summary]\n{}", summary));
                }
                Err(e) => {

                    ui.print_message(
                        MsgRole::System,
                        MsgType::Plain(format!("[ERROR] Summarization failed: {}", e.to_string())),
                    );

                    // Save history to disk
                    self.history.save();
                    // Exit the program
                    std::process::exit(1);
                }
            }
        }


        // Wait for 1 second before sending the next request
        let elapsed = std::time::Instant::now().duration_since(self.last_req_time);
        if elapsed < std::time::Duration::from_secs(1) {
            std::thread::sleep(std::time::Duration::from_secs(1) - elapsed);
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

        let response: Value = match self.ai
                .chat()
                .create_byot(
                json!({
                        "model": self.model.clone(),
                        "messages": self.history.get(),
                        "stream": false,
                        "max_completion_tokens": MAX_TOKENS,
                    })
                )
                .await {
                    Ok(res) => res,
                    Err(e) => {
                        println!(
                            "[ERROR] Failed to get response: {}",
                            e.to_string()
                        );
                        self.history.add_message(
                            MessageRole::System,
                            format!("[ERROR] Failed to get response: {}", e.to_string())
                        );
                        return String::from("[No message]");
                    }
                };
                

        self.last_req_time = std::time::Instant::now();

       if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
            self.history.add_message(MessageRole::Assistant, content.to_string());

            ui.print_message(
                MsgRole::Assistant,
                MsgType::Plain(content.to_string()),
            );

            return content.to_string();
        } else {
            println!("[ERROR] No content in response.");
            self.history.add_message(
                MessageRole::System,
                format!("[ERROR] No message recieved from assistant")
            );
            return String::from("[No message]");
        }
        
    }
}
