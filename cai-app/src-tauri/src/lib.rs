use cai_core::ui_trait::{MsgRole, MsgType, UIBase};
use tauri::Emitter;



pub struct UI;

impl UIBase for UI {
    fn init(&self) -> bool {
        // Initialize the UI
        true
    }

    /// This function acquires user input from frontend
    /// And appends it to the chat 
    fn get_user_input(&self) -> String {

        tauri::emi


        String::new()
    }

    fn print_message(&self, message_type: MsgRole, message_format: MsgType) {
        
    }
}
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
