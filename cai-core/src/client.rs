//! ===============================================================
//! AI Client Module
//!
//! Encapsulates the logic for interacting with the AI model (e.g., OpenAI GPT).
//! Handles message sending, API requests, and response management.
//! Provides a clean interface for the CLI to communicate with the AI.
//! ===============================================================

use crate::client_util::*;
use crate::model;
use crate::prompt::get_model_choosing_prompt;
use crate::types::{ChatMessage, MessageRole};
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
    pub model: String,
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
            Err(e) => { Err(e) }
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
        content: &str,
        model: Option<String>
    ) -> String {
        // 1) Record user/system content
        self.history.add_message(role, content.to_string());

        // 2) Summarize if needed
        if self.history.needs_summarize() {
            if let Err(e) = self.perform_history_summary().await {
                ui.print_message(
                    MsgRole::System,
                    MsgType::Plain(format!("[ERROR] Summarization failed: {}", e))
                );
                self.history.save();
                std::process::exit(1);
            }
        }

        // 3) Choose model via LLM (fallback to provided or default)
        let chosen_model: String = self
            .choose_model(content, self.history.get())
            .await
            .unwrap_or(model.unwrap_or(self.model.clone()));
        println!("[DEBUG] Chosen model: {}", chosen_model);

        // 4) Working copy of messages (don't pollute persistent history with tool traffic)
        let mut rt_messages: Value = json!(self.history.get()); // Vec<messages> as Value
        let mut assistant_final_text: Option<String> = None;

        // 5) Base request JSON
        let mut req = json!({
            "model": chosen_model,
            "messages": rt_messages,
        });

        // 6) Tools + profile
        req["tools"] = tools_json();
        if let Some(variant) = model_variant_of(&chosen_model) {
            merge_json(&mut req, request_profile_of(&variant));
        }
        
        // 7) Tool-calling loop (limited retries)
        for _ in 0..5 {
            // NOTE: call_with_backoff should return the RAW JSON string for tool calls.
            let resp = call_with_backoff(&self.ai, req.clone()).await;

            match resp {
                Ok(s) => {
                    // Try parse as JSON (Responses/ChatCompletions raw body)
                    if let Ok(v) = serde_json::from_str::<Value>(&s) {
                        if let Some(choice) = v.get("choices").and_then(|c| c.get(0)) {
                            let msg = &choice["message"];
                            // Tools?
                            if let Some(tc) = msg.get("tool_calls").and_then(|x| x.as_array()) {
                                // Append the assistant msg (with tool_calls) to runtime messages
                                rt_messages.as_array_mut().unwrap().push(msg.clone());

                                // Execute each tool
                                for t in tc {
                                    let call_id = t.get("id").and_then(|x| x.as_str()).unwrap_or("");
                                    let fname = t.get("function").and_then(|f| f.get("name")).and_then(|x| x.as_str()).unwrap_or("");
                                    let fargs = t.get("function").and_then(|f| f.get("arguments")).and_then(|x| x.as_str()).unwrap_or("{}");
                                    let args_val: Value = serde_json::from_str(fargs).unwrap_or(json!({}));

                                    let result = match fname {
                                        "run_terminal" => {
                                            let cmd = args_val.get("command").and_then(|x| x.as_str()).unwrap_or("");
                                            run_terminal(cmd)
                                        },
                                        "run_python" => {
                                            let code = args_val.get("code").and_then(|x| x.as_str()).unwrap_or("");
                                            run_python(code)
                                        },
                                        "write_file" => {
                                            let path = args_val.get("path").and_then(|x| x.as_str()).unwrap_or("");
                                            let content = args_val.get("content").and_then(|x| x.as_str()).unwrap_or("");
                                            match write_file(path, content) {
                                                Ok(_) => String::from("[write_file] OK"),
                                                Err(e) => format!("[write_file] ERROR: {}", e),
                                            }
                                        },
                                        _ => format!("[tool error] Unknown tool: {}", fname),
                                    };

                                    // Push tool result
                                    rt_messages.as_array_mut().unwrap().push(json!({
                                        "role": "tool",
                                        "tool_call_id": call_id,
                                        "content": result
                                    }));
                                }

                                // Re-ask with augmented messages
                                req["messages"] = rt_messages.clone();
                                continue;
                            }

                            // No tool calls → take assistant content
                            if let Some(text) = msg.get("content").and_then(|x| x.as_str()) {
                                assistant_final_text = Some(text.to_string());
                                break;
                            }
                        }
                    }

                    // If not JSON or missing fields → treat as final assistant text
                    assistant_final_text = Some(s);
                    break;
                }
                Err(_) => {
                    assistant_final_text = Some(String::from("[ERROR] Request failed."));
                    break;
                }
            }
        }

        let content = assistant_final_text.unwrap_or_else(|| String::from("[No message]"));

        // Only the final assistant text goes into persistent history/UI
        self.history.add_message(MessageRole::Assistant, content.clone());
        ui.print_message(MsgRole::Assistant, MsgType::Plain(content.clone()));

        if self.history.needs_summarize() {
            let _ = self.perform_history_summary().await;
        }
        self.history.save();

        content
    }



    /// Chooses the appropriate model based on the content complexity.
    /// Chooses the appropriate model based on the content complexity.
    pub async fn choose_model(&mut self, content: &str, history: Vec<ChatMessage>) -> Option<String> {
        let p: String = get_model_choosing_prompt(content, &history);
        // Use a cheap decider; fallback to default on failure
        match self.make_independent_request(&p, model!(Mini)).await {
            Ok(choice) => {
                match choice.trim() {
                    "nano" => model!(Nano),
                    "mini" => model!(Mini),
                    "full" => model!(Full),
                    "max"  => model!(Max),
                    other => {
                        eprintln!("[ERROR] Unknown model choice: {}", other);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Model choosing failed: {}", e);
                None
            }
        }
    }

}



