
// ! Python block parser
// 
// Example:
// ```python
// import os
// current_directory = os.getcwd()
// print(current_directory) # Prints the current working directory
// ```


use lazy_static::lazy_static;
use regex::Regex;

use crate::ui_trait::UIBase;


lazy_static!(
    static ref PYTHON_BLOCK_RE: Regex = Regex::new(
       r"```python\s*\n([\w\W]*?)\n?```"
    ).unwrap();
);


pub fn parse_python_block(ui: &dyn UIBase, response: &str, sys_message: &mut String) {
    let mut codes: Vec<String> = Vec::new();

    for capture in PYTHON_BLOCK_RE.captures_iter(&response) {
        let code = capture.get(1).unwrap().as_str().to_string();
        codes.push(code);
    }



    // * Assemble all codes into a single script
    if codes.is_empty() {
        return;
    }

    let script = codes.join("\n");
    let script_path = "./__temp_cai_script.py";
    std::fs::write(script_path, script).unwrap();

    // * Execute the script
    let output = std::process::Command::new("python3")
        .arg(script_path)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            sys_message.push_str(&format!("[Python Output]\n```\n{}\n```", stdout));
            sys_message.push_str(&format!("[Python Error]\n```\n{}\n```", stderr));
        }

        Err(error) => {
            sys_message.push_str(&format!("[Error while executing python script]\n\t {}\n", error));
        }
    }

    // * Clean up the temporary script file
    std::fs::remove_file(script_path).unwrap();

    ui.print_message(
        crate::ui_trait::MsgRole::System,
        crate::ui_trait::MsgType::Plain(format!("Executed Python code blocks."))
    );
}