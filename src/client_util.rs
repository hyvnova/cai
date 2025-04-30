//! ===============================================================
//! Client Utility Functions (Refactored)
//!
//! * Syntax‑highlighted code blocks (using `syntect`)
//! * Lightweight Markdown tinting (headings, bold/italic, lists, blockquotes)
//! * Zero redundant regex recompiles – everything cached with `once_cell`
//! * One‑shot colour/style helpers for system / user messages
//! ===============================================================

use colored::*;
use once_cell::sync::Lazy;
use regex::Regex;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

// ────────────────────────────────────────────────────────────────
// Static caches (avoid loading theme / regex every call)
// ────────────────────────────────────────────────────────────────
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME: Lazy<&'static syntect::highlighting::Theme> = Lazy::new(|| {
    static TS: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
    &TS.themes["base16-ocean.dark"]
});

static CODE_BLOCK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"```([A-Za-z0-9_+.-]*)?\n([\s\S]*?)```")
        .expect("code‑block regex")
});

static MD_HEADER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^(#{1,6})\s*(.*)").unwrap());
static MD_BOLD_RE:   Lazy<Regex> = Lazy::new(|| Regex::new(r"\*\*(.*?)\*\*").unwrap());
static MD_ITALIC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\*(.*?)\*").unwrap());
static MD_LIST_RE:   Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^[-*]\s+(.*)").unwrap());
static MD_QUOTE_RE:  Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^>\s+(.*)").unwrap());

// ────────────────────────────────────────────────────────────────
// Public helpers
// ────────────────────────────────────────────────────────────────
/// Print the AI response with syntax highlighting & markdown tint.
/// Heavy lifting is delegated to helper fns; this fn is just the orchestrator.
pub fn enhanced_print(resp: &str) {
    let mut cursor = 0;
    for cap in CODE_BLOCK_RE.captures_iter(resp) {
        let m  = cap.get(0).unwrap();
        let lang_raw = cap.get(1).map_or("", |m| m.as_str());
        let lang = normalize_lang(lang_raw);
        let code = cap.get(2).unwrap().as_str();

        // ── print markdown that precedes this code block ──
        let markdown_chunk = &resp[cursor..m.start()];
        if !markdown_chunk.trim().is_empty() {
            println!("{}", color_markdown(markdown_chunk));
        }

        // ── print the code block, framed ──
        println!("{}", format!("▼ {} ▼", lang).on_black().bold().cyan());
        highlight_code(&lang, code);
        println!("{}", "▲".repeat(20).on_black().dimmed());

        cursor = m.end();
    }

    // Trailing markdown after last block
    if cursor < resp.len() {
        let tail = &resp[cursor..];
        if !tail.trim().is_empty() {
            println!("{}", color_markdown(tail));
        }
    }
}

/// Lightweight ANSI styling for markdown subsets.
pub fn color_markdown(text: &str) -> String {
    let mut out = text.to_string();

    // Headers
    out = MD_HEADER_RE.replace_all(&out, |caps: &regex::Captures| {
        match caps[1].len() {
            1 => caps[2].bold().bright_white(),
            2 => caps[2].bold().bright_blue(),
            3 => caps[2].bold().cyan(),
            _ => caps[2].italic().dimmed(),
        }.to_string()
    }).to_string();

    // Bold, then italic (order avoids overlap issues)
    out = MD_BOLD_RE.replace_all(&out, |c: &regex::Captures| c[1].bold().to_string()).to_string();
    out = MD_ITALIC_RE.replace_all(&out, |c: &regex::Captures| c[1].italic().to_string()).to_string();

    // Lists
    out = MD_LIST_RE.replace_all(&out, |c: &regex::Captures| {
        format!("• {}", c[1].bright_green())
    }).to_string();

    // Blockquotes
    out = MD_QUOTE_RE.replace_all(&out, |c: &regex::Captures| {
        format!("┃ {}", c[1].bright_magenta())
    }).to_string();

    out
}

/// Print code with syntect highlighting; fall back to plain text.
fn highlight_code(lang: &str, code: &str) {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(lang)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, &THEME);
    for line in LinesWithEndings::from(code) {
        let ranges = highlighter.highlight_line(line, &SYNTAX_SET).expect("highlight");
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], true));
    }
}

/// Map user/markdown language tags ➜ syntect extensions.
fn normalize_lang(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "python"       => "py",
        "javascript"   => "js",
        "typescript"   => "ts",
        "csharp"       => "cs",
        "c++"          => "cpp",
        "shell" | "terminal" | "sh" => "bash",
        "yaml"         => "yml",
        "rust"         => "rs",
        other           => other,
    }.to_string()
}

