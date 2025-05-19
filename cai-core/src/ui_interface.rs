// ==========================================================
// UI Interface Implementation
// Interface for UI's providers must follow to be used as a UI
// ==========================================================

pub enum MessageType {
    User,

    // Since assistant response can contain code blocks, etc, they would be handled in the UI side.
    Assistant,
    System,
    Error,
}


pub trait UIBase {
    /// Will be called when user input is needed
    fn get_user_input(&self) -> String;


    /// Will be called when a message is to be printed
    fn print_message(&self, message: &str, message_type: MessageType);
}
