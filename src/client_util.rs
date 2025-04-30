//! ===============================================================
//! Client Utility Functions
//!
//! Provides enhanced output formatting and printing utilities for the CLI.
//! Includes syntax highlighting, colored output, and user-friendly display helpers.
//! ===============================================================

use colored::Colorize;
use regex::Regex;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

/// Adds syntax highlight
/// Adds coloring to markdown
pub fn enhanced_print(response: &str) {
    let code_block_re = Regex::new(r"```([A-Za-z0-9_+-]+)?\n([\s\S]*?)```").unwrap();
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"]; // 👀 aesthetic

    let mut last_end = 0;
    for cap in code_block_re.captures_iter(response) {
        let m = cap.get(0).unwrap();
        let lang = normalize_lang(cap.get(1).map_or("", |m| m.as_str()));
        let code = cap.get(2).unwrap().as_str();

        // 🔹 Print markdown before this block, but styled
        let markdown = &response[last_end..m.start()];
        println!("{}", color_markdown(markdown));

        // 🔸 Frame start
        println!("{}", format!("▼ {} ▼", lang).on_black().bold().cyan());

        // 🔸 Syntax-highlighted code block
        let syntax = ps.find_syntax_by_extension(&lang)
                       .or_else(|| Some(ps.find_syntax_plain_text()))
                       .unwrap();

        let mut h = HighlightLines::new(syntax, theme);
        for line in LinesWithEndings::from(code) {
            let ranges = h.highlight_line(line, &ps).expect("Highlighting failed");
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            print!("{}", escaped);
        }

        // 🔹 Frame end
        println!("{}", "▲".repeat(20).on_black().dimmed());

        last_end = m.end();
    }

    // Final trailing text after last block
    if last_end < response.len() {
        let remaining = &response[last_end..];
        println!("{}", color_markdown(remaining));
    }
}

fn normalize_lang(lang: &str) -> String {
    let lower = lang.to_lowercase();
    match lower.as_str() {
        "python"       => "py".to_string(),
        "javascript"   => "js".to_string(),
        "typescript"   => "ts".to_string(),
        "csharp" => "cs".to_string(),
        "c++"  => "cpp".to_string(),
        "shell" | "terminal" | "sh" => "bash".to_string(),
        "yaml" => "yml".to_string(),
        "rust" => "rs".to_string(),
        other          => other.to_string(),
    }
}


/// Prints a string to the console with optional syntax highlighting.
///
/// # Arguments
/// * `s` - The string to print.
/// * `lang` - Optional language for syntax highlighting.
pub fn print_with_highlight(s: &str, lang: Option<&str>) {
    let syntax = match lang {
        Some(l) => {
            let ps = SyntaxSet::load_defaults_newlines();
            ps.find_syntax_by_extension(l)
        },
        None => None,
    };

    // Use pretty print or fallback to plain output
}

/// Prints a string to the console in a distinct color for system messages.
pub fn print_system_message(msg: &str) {
    println!("{}", msg.bright_red());
}

/// Prints a string to the console in a distinct color for user messages.
pub fn print_user_message(msg: &str) {
    println!("{}", msg.bright_green());
}

pub fn color_markdown(text: &str) -> String {
    let mut result = text.to_string();

    // Headers (## Header)
    let re_header = Regex::new(r"(?m)^(#{1,6})\s*(.*)").unwrap();
    result = re_header.replace_all(&result, |caps: &regex::Captures| {
        let level = caps[1].len();
        let content = &caps[2];
        match level {
            1 => content.bold().bright_white().to_string(),
            2 => content.bold().bright_blue().to_string(),
            3 => content.bold().cyan().to_string(),
            _ => content.italic().dimmed().to_string(),
        }
    }).to_string();

    // Bold **text**
    let re_bold = Regex::new(r"\*\*(.*?)\*\*").unwrap();
    result = re_bold.replace_all(&result, |caps: &regex::Captures| {
        caps[1].bold().to_string()
    }).to_string();

    // Italic *text* (do this AFTER bold, now we don’t care about overlap)
    let re_italic = Regex::new(r"\*(.*?)\*").unwrap();
    result = re_italic.replace_all(&result, |caps: &regex::Captures| {
        caps[1].italic().to_string()
    }).to_string();

    // Lists (- item)
    let re_list = Regex::new(r"(?m)^[-*]\s+(.*)").unwrap();
    result = re_list.replace_all(&result, |caps: &regex::Captures| {
        format!("• {}", &caps[1].bright_green())
    }).to_string();

    // Blockquotes
    let re_quote = Regex::new(r"(?m)^>\s+(.*)").unwrap();
    result = re_quote.replace_all(&result, |caps: &regex::Captures| {
        format!("┃ {}", &caps[1].bright_magenta())
    }).to_string();

    result
}
