use crate::{client::Client, model, prompt::LEVEL_PROMPTS, types::MessageRole, ui_trait::UIBase};

/// Injects the 9 levels of thinking into assisant response
/// This helps improve reasoning and humanity of responses
/// Or that i'd to like to say but this is slop
pub async fn inject_levels(response: &mut String, user_prompt: &String, assistant: &mut Client, ui: &dyn UIBase) {


    let levels_prompt = LEVEL_PROMPTS.join("\n");
    let prompt = format!("[User Prompt]\n{}\n[System Prompt]\n{}", user_prompt, levels_prompt);
    *response = assistant.send_message(ui, MessageRole::System, &prompt, model!(Smart)).await;

    // 2. Apply every level prompt in order..
    // for &prompt in LEVEL_PROMPTS.iter() {
    //     *response = assistant.send_message(ui, MessageRole::System, prompt, None).await;
    // }

}