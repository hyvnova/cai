use lazy_static::lazy_static;
use regex::Regex;
use std::{ io::{ self, Write }, process::{ Command, Stdio } };
use tempfile::NamedTempFile;

use crate::ui_trait::{ MsgRole, MsgType, UIBase };

// ─────────────────────────────────────────────────────────────────────────────
// pre-compiled regex: “dot = newline”, optional CR, tolerant closing fence
lazy_static! {
    static ref PYTHON_BLOCK_RE: Regex = Regex::new(r"(?s)```python\s*\r?\n(.*?)```").unwrap();
}

pub fn parse_python_block(ui: &dyn UIBase, response: &str, sys_message: &mut String) {
    // 1. collect code snippets
    let snippets: Vec<&str> = PYTHON_BLOCK_RE.captures_iter(response)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
        .collect();

    if snippets.is_empty() {
        return; // nothing to do
    }

    // 2. build full script (add our helper built-ins, then snippets)
    let mut script = String::new();
    script.push_str(include_str!("./_python_builtins.py"));
    script.push_str("\n\n");
    script.extend(snippets.join("\n").chars());

    // 3. write to a unique temp file
    let mut tmp = NamedTempFile::new().expect("tmp file");
    tmp.write_all(script.as_bytes()).expect("write tmp");

    let (python_cmd, extra_args) = get_python_name().expect("Python 3.x not found");

    // -X utf8   → PEP 597 “UTF-8 Mode”
    // PYTHONIOENCODING=utf-8  → same effect for pipes & files
    let mut cmd = Command::new(&python_cmd);
    cmd.args(&extra_args)
        .arg("-u")
        .arg("-X")
        .arg("utf8") // <── ★ turn on UTF-8 mode
        .arg(tmp.path())
        .env("PYTHONIOENCODING", "utf-8")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output();

    // 5. pack results for the higher-level prompt
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            if !stdout.trim().is_empty() {
                // Print the output to the UI
                ui.print_message(MsgRole::System, MsgType::TitleChildren {
                    title: "[Python Output]".into(),
                    content: vec![stdout.clone().into()],
                });

                sys_message.push_str("[Python Output]\n```\n");
                sys_message.push_str(&stdout);
                sys_message.push_str("\n```\n");
            }
            if !stderr.trim().is_empty() {
                // Print the error to the UI
                ui.print_message(MsgRole::System, MsgType::TitleChildren {
                    title: "[Python Error]".into(),
                    content: vec![stderr.clone().into()],
                });

                sys_message.push_str("[Python Error]\n```\n");
                sys_message.push_str(&stderr);
                sys_message.push_str("\n```\n");
            }
        }
        Err(e) => {
            sys_message.push_str(&format!("[Error while executing python script]\n\t{e}\n"));
        }
    }

    // No need to fs::remove_file; NamedTempFile deletes on drop.
    ui.print_message(MsgRole::System, MsgType::Plain("Executed Python code blocks.".into()));
}

/// Detects the first working Python executable on PATH.
///
/// Search order: `python3`, `python`, `py`.
///
/// Returns the command name that worked (`"python3"`, `"python"`, or `"py"`),
/// or `None` if none of them responded.
///
/// # Example
/// ```no_run
/// let python = get_python_name().expect("No Python interpreter found!");
/// let out = std::process::Command::new(python)
///     .arg("-c")
///     .arg("print('hello')")
///     .output()
///     .expect("failed to run python");
/// ```
pub fn get_python_name() -> Option<(String, Vec<String>)> {
    // (cmd, extra_args)  ← `extra_args` lets us do  ["py", "-3"]
    const CANDIDATES: [(&str, &[&str]); 4] = [
        ("python3", &[]),
        ("python", &[]),
        ("py", &["-3"]), // Windows launcher, force 3.x
        ("py.exe", &["-3"]), // just in case
    ];

    for (cmd, extra) in CANDIDATES {
        let mut check = Command::new(cmd);
        check.args(extra).arg("--version");

        if let Ok(out) = check.output() {
            if
                out.status.success() &&
                (String::from_utf8_lossy(&out.stdout).contains("Python") ||
                    String::from_utf8_lossy(&out.stderr).contains("Python"))
            {
                return Some((
                    cmd.to_string(),
                    extra
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ));
            }
        }
    }
    None
}
