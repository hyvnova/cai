use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::BufReader;

// ── only pull in rodio when the "voice" feature is requested ──
#[cfg(feature = "voice")]
use rodio::{Decoder, OutputStream, Sink};

use crate::shell::Shell;

lazy_static! {
    static ref VOICE_BLOCK_RE: Regex =
        Regex::new(r"\`\`\`say\s?([\w\W]*?)\s?\`\`\`").unwrap();
}

const PIPER_VOICE_PATH: &str = "C:\\bin\\tts_piper\\voices\\";
const VOICE: &str = "kris";
const OUTPUT_PATH: &str =
    r"C:\Users\Hyvnt\T\Rust\console-ai-framework\voice";


/// ─────────────────────────────────────────────────────────────
/// Real implementation when the `voice` feature *is* enabled
/// ─────────────────────────────────────────────────────────────
#[cfg(feature = "voice")]
pub fn parse_say_block(response: &str, shell: &mut Shell) {
    if VOICE_BLOCK_RE.is_match(response) {
        // audio output handle
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(0.25);
        sink.set_speed(1.05);

        for (i, block) in VOICE_BLOCK_RE.find_iter(response).enumerate() {
            let text = block
                .as_str()
                .replace("```say", "")
                .replace("```", "")
                .trim()
                .to_owned();

            let output_file = format!(r"{OUTPUT_PATH}\{i}.wav");
            shell
                .execute(
                    &format!(
                        r#"echo "{text}" | piper --model "{PIPER_VOICE_PATH}{VOICE}.onnx" \
                            --config "{PIPER_VOICE_PATH}{VOICE}.json" \
                            --output_file "{output_file}" -q"#
                    ),
                    Some(15),
                )
                .expect("failed to synthesize speech with piper");

            let file = BufReader::new(File::open(output_file).unwrap());
            let source = Decoder::new(file).unwrap();
            sink.append(source);
        }

        sink.sleep_until_end();
    }
}
