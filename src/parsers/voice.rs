/// Say or Voice block parser
/// Say block look like this:
/// ```say
/// Some content here
/// ```
/// This parser will extract the content and synthesize it to speech and play it 

/// To Synthesize speech example
/// ```
/// echo {Content} | \ 
///     piper --model "C:\bin\tts_piper\voices\kris.onnx" \ 
///     -c "C:\bin\tts_piper\voices\kris.json" \ 
///     --output_file {PATH}
/// ```


use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};


use crate::shell::Shell;

lazy_static!(
    static ref VOICE_BLOCK_RE: Regex = Regex::new(
        r"\`\`\`say\s?([\w\W]*?)\s?\`\`\`"
    ).unwrap();
);

const PIPER_VOICE_PATH: &str = "C:\\bin\\tts_piper\\voices\\";
const VOICE: &str = "kris";
const OUTPUT_PATH: &str = r"C:\Users\Hyvnt\T\Rust\console-ai-framework\voice";

pub fn parse_say_block(response: &str, shell: &mut Shell) {
    if VOICE_BLOCK_RE.is_match(&response) {

        // Get an output stream handle to the default physical sound device.
        // Note that no sound will be played if _stream is dropped
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(0.25);
        sink.set_speed(1.05);

        let blocks = VOICE_BLOCK_RE.find_iter(&response);

        for (i, block) in blocks.enumerate() {
            let text = block
                .as_str()
                .to_string()
                .replace("```say", "")
                .replace("```", "")
                .trim()
                .to_string();
            

            let output_file = format!("{0}\\{1}.wav", OUTPUT_PATH, i);
            shell.execute(&format!("echo \"{0}\" | piper --model \"{1}{2}.onnx\" --config \"{1}{2}.json\" --output_file \"{3}\" -q", text, PIPER_VOICE_PATH, VOICE, output_file), Some(15));


            // Load a sound from a file, using a path relative to Cargo.toml
            let file = BufReader::new(File::open(output_file).unwrap());
            // Decode that sound file into a source
            let source = Decoder::new(file).unwrap();
            
            sink.append(source);
        }
        
        sink.sleep_until_end();

    }
}