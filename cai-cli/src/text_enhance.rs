//! ===============================================================
//! Client Utility Functions
//!
//!  • Markdown tinting (headings, bold/italic, lists, quotes)
//!  • Syntax‑highlighted ```lang``` blocks (via `syntect`)
//!  • ✍ `write[path]` blocks (parsed **before** code so later
//!    commands can rely on the files already being written)
//!
//! No public function signatures were changed.
//! ===============================================================

use colored::*;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

// ── Caches ──────────────────────────────────────────────────────
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME: Lazy<&'static syntect::highlighting::Theme> = Lazy::new(|| {
    static TS: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
    &TS.themes["base16-ocean.dark"]
});

// Code & write‑block regexes
static CODE_BLOCK_RE: Lazy<Regex> = Lazy::new(|| {
    // capture‑1 = language, capture‑2 = body
    Regex::new(r"```([A-Za-z0-9_+.\-]*)\n([\s\S]*?)```")
        .expect("CODE_BLOCK_RE")
});
static WRITE_BLOCK_RE: Lazy<Regex> = Lazy::new(|| {
    // (?s)  → dot matches newlines, keeps pattern compact & safe
    // capture‑1 = path, capture‑2 = body
    Regex::new(r"(?s)```write\[(.+?)\]\s*(.*?)\s*```")
        .expect("WRITE_BLOCK_RE")
});

// Lightweight Markdown
static MD_HEADER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^(#{1,6})\s+(.*)").unwrap());
static MD_BOLD_RE:   Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*\*(.*?)\*\*").unwrap());
static MD_ITALIC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*(.*?)\*").unwrap());
static MD_LIST_RE:   Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^[-*]\s+(.*)").unwrap());
static MD_QUOTE_RE:  Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^>\s+(.*)").unwrap());

// ── Public API ─────────────────────────────────────────────────
/// Stream‑print a mixed response containing Markdown prose,
/// syntax‑highlighted code blocks, and ✍ write‑blocks.
pub fn enhanced_print(resp: &str) {
    let mut cursor = 0;

    loop {
        let remainder = &resp[cursor..];

        // whichever block appears *next* in the remainder
        let write_m = WRITE_BLOCK_RE.find(remainder);
        let code_m  = CODE_BLOCK_RE.find(remainder);

        let (kind, start, end) = match (write_m, code_m) {
            (Some(w), Some(c)) if w.start() < c.start() => ("write", w.start(), w.end()),
            (Some(w), _)                                => ("write", w.start(), w.end()),
            (_, Some(c))                                => ("code",  c.start(), c.end()),
            (None, None) => {
                // print any trailing Markdown, then we’re done
                if !remainder.trim().is_empty() {
                    println!("{}", color_markdown(remainder));
                }
                break;
            }
        };

        // preceding Markdown → colourise & print
        if start > 0 {
            let md = &remainder[..start];
            if !md.trim().is_empty() {
                println!("{}", color_markdown(md));
            }
        }

        // actual block rendering
        let block = &resp[cursor + start .. cursor + end];
        match kind {
            "code" => {
                let cap  = CODE_BLOCK_RE.captures(block).expect("code captures");
                let lang = normalize_lang(cap.get(1).map_or("", |m| m.as_str()));
                let body = cap.get(2).unwrap().as_str();

                println!("{}", format!("┌─ {} ─┐", lang).dimmed());
                highlight_code(&lang, body);
                println!("{}", "└────────┘".dimmed());
                println!();
            }
            "write" => {
                let cap  = WRITE_BLOCK_RE.captures(block).expect("write captures");
                let path = cap.get(1).unwrap().as_str().trim();
                let body = cap.get(2).unwrap().as_str();

                println!("{}", format!("✍ write → {}", path).bold().yellow());
                println!("{}", body);
                println!();
            }
            _ => unreachable!(),
        }

        cursor += end;
    }
}

// ── Helpers ─────────────────────────────────────────────────────
fn color_markdown(md: &str) -> String {
    let mut out = MD_HEADER_RE
        .replace_all(md, |c: &Captures| {
            let level = c[1].len();
            let text  = &c[2];
            let styled = match level {
                1 => text.bold(),
                2 => text.bold().underline(),
                _ => text.normal(),
            };
            styled.bright_cyan().to_string()
        })
        .to_string();

    // Lists → • bullet
    out = MD_LIST_RE
        .replace_all(&out, |c: &Captures| format!("• {}", c[1].bright_green()))
        .to_string();

    // Blockquotes
    out = MD_QUOTE_RE
        .replace_all(&out, |c: &Captures| format!("┃ {}", c[1].bright_magenta()))
        .to_string();

    // Bold then italic (order avoids overlap)
    out = MD_BOLD_RE
        .replace_all(&out, |c: &Captures| c[1].bold().to_string())
        .to_string();
    out = MD_ITALIC_RE
        .replace_all(&out, |c: &Captures| c[1].italic().to_string())
        .to_string();

    out
}

/// Highlight code or fall back to plain text.
fn highlight_code(lang: &str, code: &str) {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(lang)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
    let mut hl = HighlightLines::new(syntax, &THEME);

    for line in LinesWithEndings::from(code) {
        let ranges = hl.highlight_line(line, &SYNTAX_SET).expect("highlight line");
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], true));
    }
}

/// Map loose language hints to syntect extensions.
fn normalize_lang(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "python"                       => "py",
        "javascript" | "node"          => "js",
        "typescript"                   => "ts",
        "csharp"                       => "cs",
        "c++"                          => "cpp",
        "shell" | "sh" | "bash"
        | "terminal"                   => "bash",
        "yaml"                         => "yml",
        "rust"                         => "rs",
        other                          => other,
    }
    .to_string()
}
