use cai_core::ui_trait::{MsgRole, MsgType, UIBase};


pub struct UI;

impl UIBase for UI {
    fn init(&self) -> bool {
        
        true
    }

    /// This function acquires user input from frontend
    /// And appends it to the chat 
    fn get_user_input(&self) -> String {
        String::new()
    }

    fn print_message(&self, message_type: MsgRole, message_format: MsgType) {
        
    }
}
