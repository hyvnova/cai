use lazy_static::lazy_static;
use regex::Regex;

// Write block parser
// Writing block look like this:
// ```write[some/file/path.txt]
// Some content here
// ```
// 
// ! This must be the first or among the firsts parsers to run, because
// !usually commands depend on files being already written. Ex. executing a script


lazy_static!(
    static ref WRITE_BLOCK_RE: Regex = Regex::new(
        r"\`\`\`write\[([?: a-zA-Z0-9._\-\\\/()]+)\]\s?([\w\W]*?)\s?\`\`\`"
    ).unwrap();
);

pub fn parse_write_block(response: &str) {
    for capture in WRITE_BLOCK_RE.captures_iter(&response) {
        let file_path = capture.get(1).unwrap().as_str();
        let content = capture.get(2).unwrap().as_str();

        println!("[SYSTEM] Writing to file: {}", file_path);
        std::fs::write(file_path, content).unwrap();
    }
}