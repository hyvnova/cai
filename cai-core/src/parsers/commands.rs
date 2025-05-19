/// This is commands parser
/// If finds and executes commands in the response
/// Commands are written in a code block like this:
/// ```terminal
/// echo "Hello World"
/// ```

use lazy_static::lazy_static;
use regex::Regex;
use crate::{shell::Shell, ui_trait::{MsgType, MsgRole, UIBase}};

lazy_static!(
    static ref COMMANDS_BLOCK_RE: Regex = Regex::new(
        r"\`\`\`terminal\s?([\w\W]*?)\s?\`\`\`"
    ).unwrap();
);


pub fn parse_commands_block(ui: &dyn UIBase, response: &str, shell: &mut Shell, sys_message: &mut String) {
    if COMMANDS_BLOCK_RE.is_match(response) {
        let blocks = COMMANDS_BLOCK_RE.find_iter(response);

        let commands = blocks
            .flat_map(|block| {
                block
                    .as_str()
                    .replace("```terminal", "")
                    .replace("```", "")
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| line.trim().to_string())
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<String>>();

        ui.print_message(
            MsgRole::System,
            MsgType::Plain(format!("[SYSTEM] Executing {} commands.", commands.len())),
        );
        
        if !commands.is_empty() {
            let title = "[Command Execution Results]";
            sys_message.push_str(&format!("{}\n", title));
            
            let mut ui_content_children: Vec<String> = Vec::with_capacity(commands.len());

            for command in commands {
                let command_output = shell.execute(&command, Some(10))
                    .unwrap_or_else(|_| "Command execution failed.".to_string());

                // Push to sys_message
                let content = format!("{} -> {}\n", command, command_output);
                sys_message.push_str(&content);

                ui_content_children.push(content);
            }

            // Print the command output
            ui.print_message(
                MsgRole::System,
                MsgType::TitleChildren {
                    title: title.to_string(),
                    content: ui_content_children,
                },
            );
        }
    }
}
