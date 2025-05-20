mod input_reading;
mod text_enhance;

use std::io::{ self, Write };
use cai_core::ui_trait::{ UIBase, MsgRole, MsgType };
use colored::Colorize;
use text_enhance::enhanced_print;

pub struct UI;

impl UIBase for UI {
    fn init(&self) -> bool {
        true
    }

    fn get_user_input(&self) -> String {
        let header = "[You]: ".green().bold();
        print!("{}", header);
        io::stdout().flush().unwrap();

        input_reading::read_user_input().expect("Failed to read user input")
    }

    fn print_message(&self, message_type: MsgRole, message_format: MsgType) {
        match message_type {
            MsgRole::Assistant => {

                match message_format {
                    MsgType::Plain(message) => {
                        let header = "[A. ]".bold().blue();
                        println!("{}", &header);
                        enhanced_print(&message);
                        println!("\x1b[0m"); // Reset color
                        println!();
                    }
                    MsgType::TitleChildren { title, content } => {
                        let header = "[A. ]".bold().blue();
                        println!("{}", &header);
                        enhanced_print(&title);
                        println!();

                        for child in content {
                            enhanced_print(&child);
                            println!();
                        }
                    }
                }
            }

            MsgRole::System => {

                match message_format {
                    MsgType::Plain(message) => {
                        let header = "[SYS] ".magenta().bold();
                        println!("{}\n\t{}\n", &header, message);
                    }
                    MsgType::TitleChildren { title, content } => {
                        let header = "[SYS] ".magenta().bold();
                        println!("{}{}\n", &header, title);

                        for child in content {
                            let header = "> ".green().bold();
                            println!("{}{}", &header, child);
                        }
                    }
                    
                }
            }

            MsgRole::Error => {
                match message_format {
                    MsgType::Plain(message) => {
                        let header = "[ERROR] ".red().bold();
                        println!("{}{}", &header, message.red());
                    }
                    
                    MsgType::TitleChildren { title, content } => {
                        let header = "[ERROR] ".red().bold();
                        println!("{}{}", &header, title.red().bold());

                        for child in content {
                            println!("\t{}", child.red());
                        }
                    }
                    
                }
            }
        }
    }
}
