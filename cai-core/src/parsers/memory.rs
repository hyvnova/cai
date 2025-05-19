use regex::Regex;

use crate::{client::Client, prompt::MEMORY_PROMPT, ui_trait::{MsgRole, MsgType, UIBase}};

lazy_static::lazy_static! {
    static ref MEMORY_BLOCK_RE: Regex = Regex::new(
        r"```memory\[([a-zA-Z0-9._]+)\]\s?([\w\W]*?)\s?```"
    ).unwrap();
}

pub fn parse_memory_block(ui: &dyn UIBase, response: &str, ai: &mut Client, sys_message: &mut String) {
    if MEMORY_BLOCK_RE.is_match(&response) {
        // Per each memory block, extract the action and content
        // Then, perform the action on the memory
        // If the action requires a response, update the response

        for capture in MEMORY_BLOCK_RE.captures_iter(&response) {
            let action = capture.get(1).unwrap().as_str();
            let content = capture.get(2).unwrap().as_str();

            // Optional response to send back
            match action {
                "add" => {
                    ai.memory.add(content);
                }
                "update" => {
                    let mut lines = content.lines();
                    let pat = lines.next().unwrap();
                    let rep = lines.next().unwrap();

                    ai.memory.update(pat, rep);
                }
                "delete" => {
                    ai.memory.delete(content);
                }
                "view" => {
                    let pat = if content.trim().is_empty() { None } else { Some(content) };

                    let content = format!("[Memory View]\n{}\n", ai.memory.read(pat));
                    sys_message.push_str(content.as_str());
                    ui.print_message(
                        MsgRole::System,
                        MsgType::Plain(content),
                    );
                }
                _ => {
                    let content =  format!("Invalid memory action {}.\n{}", action, MEMORY_PROMPT);

                    sys_message.push_str(
                        &content,
                    );

                    ui.print_message(
                        MsgRole::System,
                        MsgType::Plain(content),
                    );
                }
            }
        }
    }
}
