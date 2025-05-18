/// This is commands parser
/// If finds and executes commands in the response
/// Commands are written in a code block like this:
/// ```terminal
/// echo "Hello World"
/// ```

use lazy_static::lazy_static;
use regex::Regex;
use colored::*; 
use crate::shell::Shell;

lazy_static!(
    static ref COMMANDS_BLOCK_RE: Regex = Regex::new(
        r"\`\`\`terminal\s?([\w\W]*?)\s?\`\`\`"
    ).unwrap();
);


pub fn parse_commands_block(response: &str, shell: &mut Shell, sys_message: &mut String) {
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

        println!("[SYSTEM] Executing {} commands.", commands.len());

        if !commands.is_empty() {
            let header = "[Command Execution Results]".bold().underline().bright_blue();
            sys_message.push_str(&format!("{}\n", header));

            for command in commands {
                let command_title = format!(">> {}", command).bold().green();
                let command_output = shell.execute(&command, Some(10));

                // Format command output as a gray, indented block
                let formatted_output = command_output
                    .unwrap_or_else(|_| "Command execution failed.".to_string())
                    .lines()
                    .map(|line| format!("    {}", line.bright_black()))
                    .collect::<Vec<String>>()
                    .join("\n");

                // Print nicely formatted output
                println!("{}", command_title);
                println!("{}", formatted_output);

                // Push to sys_message
                sys_message.push_str(&format!("{}\n{}\n\n", command_title, formatted_output));
            }

            sys_message.push_str("\n");
        }
    }
}
