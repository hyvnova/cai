use async_openai::{ config::OpenAIConfig, Client as APIClient };
use serde_json::{json, Value};

use crate::{model, models::Model};

const MAX_RETRIES: usize = 5; // Max retries for API requests

/// Keep retrying with exponential back-off until the call succeeds.
/// If the payload includes "tools", we return the RAW JSON string (so the caller can parse tool_calls).
/// Otherwise we extract the assistant text like before, falling back to raw JSON if needed.
pub async fn call_with_backoff(
    ai: &APIClient<OpenAIConfig>,
    payload: serde_json::Value
) -> Result<String, ()> {
    use rand::{rng, Rng};
    use std::time::Duration;
    use tokio::time::sleep;

    const BASE_DELAY: Duration = Duration::from_secs(2);
    const MAX_DELAY: Duration = Duration::from_secs(60);

    // Detect whether caller wants tool-calling raw JSON handling
    let wants_tools = payload.get("tools").is_some();

    let mut delay = BASE_DELAY;
    for _ in 0..MAX_RETRIES {
        match ai.chat().create_byot::<Value, Value>(payload.clone()).await {
            Ok(response_val) => {
                // When tools are declared, ALWAYS return raw JSON for the tool loop.
                if wants_tools {
                    return Ok(response_val.to_string());
                }

                // Legacy behavior: try to extract plain assistant text.
                // Chat Completions shape: choices[0].message.content
                if let Some(content) = response_val
                    .get("choices").and_then(|c| c.get(0))
                    .and_then(|c0| c0.get("message"))
                    .and_then(|m| m.get("content"))
                    .and_then(|s| s.as_str())
                {
                    return Ok(content.to_string());
                }

                // Fallbacks: Responses-style or raw JSON if shape is unexpected.
                // (Not expected here since we're calling chat(), but safe to keep.)
                if let Some(output_text) = response_val.get("output_text").and_then(|s| s.as_str()) {
                    return Ok(output_text.to_string());
                }

                // Give the caller raw JSON so they can decide how to handle it.
                return Ok(response_val.to_string());
            }

            Err(err) => {
                eprintln!("[Retry] Request failed ({}). Sleeping {:?}…", err, delay);

                // Exponential backoff with jitter (±20%)
                let jitter = {
                    let mut rng = rng();
                    let factor: f64 = rng.random_range(0.8..=1.2);
                    (delay.as_secs_f64() * factor) as u64
                };
                sleep(Duration::from_secs(jitter)).await;
                delay = (delay * 2).min(MAX_DELAY);
                continue;
            }
        }
    }

    Err(())
}

/// Returns the tools schema to send to Chat Completions (tool calling).
pub fn tools_json() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "run_terminal",
                "description": "Execute a command in the user's native shell",
                "parameters": {
                    "type": "object",
                    "properties": { "command": { "type": "string" } },
                    "required": ["command"],
                    "additionalProperties": false
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "run_python",
                "description": "Run a Python snippet in the project workspace",
                "parameters": {
                    "type": "object",
                    "properties": { "code": { "type": "string" } },
                    "required": ["code"],
                    "additionalProperties": false
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write content to a file path, creating parent directories as needed",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"],
                    "additionalProperties": false
                }
            }
        }
    ])
}

/// Merge b into a (shallow/deep for objects/arrays minimal)
pub fn merge_json(a: &mut Value, b: Value) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (k, vb) in b_map {
                if let Some(va) = a_map.get_mut(&k) {
                    merge_json(va, vb);
                } else {
                    a_map.insert(k, vb);
                }
            }
        }
        (Value::Array(a_arr), Value::Array(b_arr)) => {
            for v in b_arr { a_arr.push(v); }
        }
        (a_ref, b_val) => { *a_ref = b_val; }
    }
}

/// Infer our Model variant from a model id string.
pub fn model_variant_of(model_id: &str) -> Option<Model> {
    if let Some(s) = model!(Nano) { if s == model_id { return Some(Model::Nano); } }
    if let Some(s) = model!(Mini) { if s == model_id { return Some(Model::Mini); } }
    if let Some(s) = model!(Full) { if s == model_id { return Some(Model::Full); } }
    if let Some(s) = model!(Max)  { if s == model_id { return Some(Model::Max); } }
    None
}

/// Execute a shell command and capture stdout/stderr.
pub fn run_terminal(command: &str) -> String {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd").args(&["/C", command]).output();
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("bash").arg("-lc").arg(command).output();

    match output {
        Ok(o) => {
            let mut s = String::new();
            s.push_str(&String::from_utf8_lossy(&o.stdout));
            if !o.stderr.is_empty() {
                s.push_str("\n[stderr]\n");
                s.push_str(&String::from_utf8_lossy(&o.stderr));
            }
            s.trim().to_string()
        }
        Err(e) => format!("[terminal error] {}", e),
    }
}

/// Run Python code by writing to a temp file and executing.
pub fn run_python(code: &str) -> String {
    use std::{fs, process::Command};
    let mut tmp = std::env::temp_dir();
    tmp.push("agent_snippet.py");
    if let Err(e) = fs::write(&tmp, code) {
        return format!("[python error] cannot write temp file: {}", e);
    }
    #[cfg(target_os = "windows")]
    let output = Command::new("python").arg(tmp.to_string_lossy().to_string()).output();
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("python3").arg(tmp.to_string_lossy().to_string()).output();

    match output {
        Ok(o) => {
            let mut s = String::new();
            s.push_str(&String::from_utf8_lossy(&o.stdout));
            if !o.stderr.is_empty() {
                s.push_str("\n[stderr]\n");
                s.push_str(&String::from_utf8_lossy(&o.stderr));
            }
            s.trim().to_string()
        }
        Err(e) => format!("[python error] {}", e),
    }
}

/// Write file helper.
pub fn write_file(path: &str, content: &str) -> Result<(), String> {
    use std::{fs, path::Path};
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("cannot create dirs: {}", e));
        }
    }
    fs::write(p, content).map_err(|e| e.to_string())
}



pub fn request_profile_of(m: &Model) -> serde_json::Value {
    match m {
        Model::Nano => json!({ "max_completion_tokens": 2048 }),
        Model::Mini => json!({ "max_completion_tokens": 4096 }),
        Model::Full => json!({
            "max_completion_tokens": 8192,
            "tool_choice": "auto",
            "parallel_tool_calls": true
        }),
        Model::Max  => json!({
            "max_completion_tokens": 16384,
            "tool_choice": "auto",
            "parallel_tool_calls": true
        }),
    }
}
