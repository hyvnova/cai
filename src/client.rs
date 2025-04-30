//! ===============================================================
//! AI Client Module
//!
//! Encapsulates the logic for interacting with the AI model (e.g., OpenAI GPT).
//! Handles message sending, API requests, and response management.
//! Provides a clean interface for the CLI to communicate with the AI.
//! ===============================================================

use crate::client_util::enhanced_print;
use crate::{ history::History, memory::MemoryManager };

use colored::*;


use openai_api_rs::v1::error::APIError;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionRequest,
    ChatCompletionMessage,
    Content,
    MessageRole,
};

/// Represents the main AI client for chat interaction.
pub struct Client {
    model: String,
    pub history: History,
    pub memory: MemoryManager,
    ai: OpenAIClient,
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
    pub fn new(model: String, history: History, memory: MemoryManager, api_key: &str) -> Self {

        println!("[DEBUG] Model={}", model);

        let ai = OpenAIClient::builder()
            .with_api_key(api_key)
            .build()
            .expect("Failed to create AI client, likely due to invalid API key.");

        Self {
            model,
            history,
            memory,
            ai,
            last_req_time: std::time::Instant::now(),
        }
    }

    /// Just a normal system request to the AI, doesn't save the response or uses the history
    pub async fn make_independent_request(&mut self, content: &str) -> Result<String, APIError> {
        let req = ChatCompletionRequest::new(
            self.model.clone(),
            vec![ChatCompletionMessage {
                role: MessageRole::system,
                content: Content::Text(content.to_string()),
                name: None,
                tool_call_id: None,
                tool_calls: None
            }]
        );

        let result = self.ai.chat_completion(req).await?;

        Ok(result.choices[0].message.content.clone().unwrap_or("[No message]".to_string()))
    }

    /// Sends a message to the AI and returns the response.
    ///
    /// # Arguments
    /// * `role` - The role of the message sender (user/system).
    /// * `content` - The message content.
    pub async fn send_message(&mut self, role: MessageRole, content: &str) -> String {
        self.history.add_message(role, Content::Text(content.to_string()));

        // Keep history small but informative
        if self.history.needs_summarize() {
            let prompt = self.history.get_summarize_prompt();
            let summary = self.make_independent_request(&prompt).await.unwrap_or_else(|e| {
                eprintln!("\x1b[1;31m[ERROR]\x1b[0m \x1b[3;31m{}\x1b[0m", e);
                "[No summary could be generated]".to_string()
            });

            self.history.insert_summary(format!("[Conversation summary]\n{}", summary));
        }

        let req = ChatCompletionRequest::new(self.model.to_string(), self.history.get());

        // Wait for 1 second before sending the next request
        let elapsed = std::time::Instant::now().duration_since(self.last_req_time);
        if elapsed < std::time::Duration::from_secs(1) {
            std::thread::sleep(std::time::Duration::from_secs(1) - elapsed);
        }

        let result = match self.ai.chat_completion(req).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("\x1b[1;31m[ERROR]\x1b[0m \x1b[3;31m{}\x1b[0m", e);
                self.history.save();
                std::process::exit(1);
            }
        };

        self.last_req_time = std::time::Instant::now();

        let response_content: String = result.choices[0].message.content
            .clone()
            .unwrap_or("[No message]".to_string());

        self.history.add_message(MessageRole::assistant, Content::Text(response_content.clone()));

        let header = "[A. ]".bold().blue();
        println!("{}", &header);
        enhanced_print(&response_content);
        println!("\x1b[0m");
        println!();

        return response_content;
    }
}
