use cai_core::{ client::Client, prompt::{ get_sys_prompt }, types::MessageRole, ui_trait::UIBase };
use pipeline::{launch, PipelineResult};

// ===================== Selecting UI =====================
#[cfg(feature = "cli")]
use cai_cli::UI;

#[cfg(feature = "app")]
use cai_app::UI; // Here we use lib because in tauri project main.rs it's entry for tauri and lib is for being able to import from here
// ===============================================================

#[tokio::main]
async fn main() {
    const CURRENT_PATH: &str = r#"C:\Users\Hyvnt\T\Python\wuwa-mod-manager"#;

    UI.init();
    let ui: &dyn UIBase = &UI;

    let mut client: Client = Client::new();

    // System message
    client.history.add_message(MessageRole::System, get_sys_prompt(client.memory.read(None).as_str(), CURRENT_PATH));

    let tests_prompts = vec!["Lee el archivo principal de este proyecto"];

    for prompt in tests_prompts {

        let mut result: Option<String> = None;
        let mut user_input: String = prompt.to_string();

        while result.is_none() {

            let launch_result: PipelineResult = launch(ui, &user_input, &mut client).await;

            match launch_result {
                PipelineResult::Success(response) => {
                    result = Some(response);
                    break;
                },
            

                PipelineResult::Retry => {

                    user_input = ui.get_user_input();

                    // If on empty or "q" -> exit
                    if user_input.trim().is_empty() || user_input.trim().eq_ignore_ascii_case("q") {
                        std::process::exit(0);
                    }

                    continue;
                }

                PipelineResult::Error(error) => {
                    println!("Prompt: {}\nError: {}", prompt, error);
                    std::process::exit(1);
                }
            }
        }


        println!("Prompt: {}\nResponse: {}", prompt, result.unwrap_or_default());
        println!("----------------------------------------------------------------\n");
    }
}
