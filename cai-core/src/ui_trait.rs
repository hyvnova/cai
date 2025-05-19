// ==========================================================
// UI Interface Implementation
// Interface for UI's providers must follow to be used as a UI
// ==========================================================

pub enum MsgRole {
    // Since assistant response can contain code blocks, etc, they would be handled in the UI side.
    Assistant,
    System,
    Error,
}

pub enum MsgType {
    // Plain text
    Plain(String),

    // E.g
    // [Command Execution Results] <- title
    // Command A results <- child 0
    // Command B results <- child 1
    TitleChildren {
        title: String,
        content: Vec<String>,
    },
}


pub trait UIBase {
    /// Will be called when user input is needed
    /// Also should print the input
    fn get_user_input(&self) -> String;


    /// Will be called when a message is to be printed
    fn print_message(&self, message_type: MsgRole, message_format: MsgType);
}
