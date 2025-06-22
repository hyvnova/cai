pub mod prompts;


use cai_core::{ client::Client, types::MessageRole, ui_trait::{MsgRole, MsgType, UIBase} };
use serde_json::Value;


// ! This enum definition is tied to the `eval_user_prompt` function and therefore `prompts::USER_PROMPT_EVAL`
// ! If either it's changed this enum should be updated accordingly
#[derive(Debug, Clone, PartialEq)]
enum EvalInputResult {
    Proceed(Vec<String>),
    Reject,
    Clarify(Vec<String>),
    Error(String),
}


/// Evaluates the user prompt to determine if it should be clarified, proceed, or rejected.
/// * Clarify -- Missing information CANNOT be obtained by the AI
/// * Proceed -- Missing information CAN be obtained by the AI
/// * Reject -- The prompt is invalid and cannot be processed
async fn eval_user_prompt(
    ui: &dyn UIBase,
    user_prompt: &str,
    client: &mut Client
) -> EvalInputResult {
    // println!("{}", format!("Evaluating input: {}", user_prompt));

    let sys_prompt: String = prompts::get_user_prompt_eval(
        user_prompt
    );

    // `res` should be of a JSON with the following structure:
    // {
    //   "step": "prompt_assessment",
    //   "result": "<clarify | proceed | reject>",
    //   "confidence": <number 0-1>,
    //   "missing": [ "<string>", … ]   # required ONLY if result == "clarify"
    // }
    //1
    let res: String = client.send_message(
            ui,
            MessageRole::System,
            &sys_prompt,
            model!(Mid)
        ).await;


    // Try to parse the response as JSON
    let res_obj: Value = match serde_json::from_str::<serde_json::Value>(&res) {
        Ok(json) => {
            // println!("{}", format!("Parsed JSON: {}", json));
            json
        }
        Err(e) => {
            return EvalInputResult::Error(format!("[ERROR] Failed to parse JSON: {}", e));
        }
    };

    // Check if the response is valid
    if !res_obj.is_object() {
        return EvalInputResult::Error(
            format!("[ERROR] Response is not in the expected JSON format.")
        );
    }

    let action = res_obj.get("result").and_then(Value::as_str).unwrap_or("reject");
    // println!("{}", format!("Action determined: {}", action));

    let default_vec: Vec<Value> = Vec::with_capacity(0); // Default value for missing information -- used in case no vector it's provided
    match action {
        "clarify"  | "proceed" => {

            let missing: Vec<String> = res_obj
                .get("missing")
                .and_then(Value::as_array)
                .unwrap_or(&default_vec)
                .iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect::<Vec<String>>();


            // Clarify -- Missing information CANNOT be obtained by the AI
            if action == "clarify" {
                // println!("{}", format!("Missing information: {:?}", missing));
                return EvalInputResult::Clarify(missing);

            // Proceed -- Missing information can be obtained by the AI
            } else {  
                println!("{}", format!("Can proceed, required information: {:?}", missing));
                return EvalInputResult::Proceed(missing);
            }
        }

        "reject" => {
            let confidence = res_obj.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
            // println!("{}", format!("Confidence level: {}", confidence));
            return EvalInputResult::Reject;
        }
        _ => EvalInputResult::Error(format!("[ERROR] Unknown action: {}", action)),
    }
}



/// Polishes the user prompt to make it more concise and clear.
/// Will try to infer aka make up missing information if possible in order to close the scope of broad prompts.
async fn polish_user_prompt(
    ui: &dyn UIBase,
    user_prompt: &str,
    client: &mut Client
) -> String {
    let polished_prompt: String = prompts::get_polish_user_prompt(user_prompt);
    client.send_message(ui, MessageRole::System, &polished_prompt, model!(Low)).await
}


pub enum PipelineResult {
    Retry, 
    Success, // Response
    Error(String), // Error message
}

/// Entry point for the pipeline, receives the prompt, chat history, and other parameters.
/// If everything goes as expected, after possible asking the user for more information, and doing some socratic reasoning,
/// It will return a polished response that can be used in the chat.
/// Notice that, as long as the AI finds it necessary, it will keep "doing things" and re-prompting itself until it reaches a point where it can return a response.
pub async fn launch(ui: &dyn UIBase, prompt: &str, client: &mut Client) -> PipelineResult {

    let polished_prompt: String = polish_user_prompt(ui, prompt, client).await;

    // println!("{}", format!("Polished prompt: {}", polished_prompt));

    let eval_result: EvalInputResult = eval_user_prompt(ui, &polished_prompt, client).await;
    // println!("{}", format!("Evaluation result: {:?}", eval_result));

    // * Handle evaluation result
    match eval_result {

        // If error, tf you want me to do? Just deal with it. 
        EvalInputResult::Error(e) => {
            ui.print_message(MsgRole::Error, MsgType::Plain(e.to_string()));
            // exit
            return PipelineResult::Error(e.to_string());
        }

        // If rejected, prompt sucks, we need to compose a message telling the user to do better frfr
        EvalInputResult::Reject => {
            const KINDLY_REJECT_PROMPT: &str = r#"The user send a prompt you rejected. You need to send a message that will help the user to improve their prompt. The message should be concise and to the point."#;
            client.send_message(ui, MessageRole::System, KINDLY_REJECT_PROMPT, model!(Low)).await;

            // Play from the beginning
            return PipelineResult::Retry;
        }

        
        // If clarify, we need to ask for more information
        EvalInputResult::Clarify(missing) => {
            // Compose a prompt asking for the missing information
            let clarify_prompt: String = format!("The user prompt is missing the following information: {:?}. Send a easily understandable message to let the user know what information is missing and how to provide it.", missing);
            
            client.send_message(ui, MessageRole::System, &clarify_prompt, model!(Low)).await;

            // Play from the beginning
            return PipelineResult::Retry;
        }


        // If proceed, we can continue
        EvalInputResult::Proceed(_) => {
            return PipelineResult::Success;
        }

        _ => {
            return PipelineResult::Error("[ERROR] Unknown evaluation result".to_string());
        }
    }
    
}
