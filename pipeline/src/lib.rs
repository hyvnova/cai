pub mod prompts;

use cai_core::client::{ Client };
use serde_json::Value;


// ! This enum definition is tied to the `eval_user_prompt` function and therefore `prompts::USER_PROMPT_EVAL`
// ! If either it's changed this enum should be updated accordingly
#[derive(Debug, Clone, PartialEq)]
enum EvalInputResult {
    Proceed,
    Reject,
    Clarify(Vec<String>),
    Error(String),
}

async fn eval_user_prompt(user_prompt: &str, client: &mut Client) -> EvalInputResult {
    println!("{}", format!("Evaluating input: {}", user_prompt));

    let sys_prompt: String = prompts::get_user_prompt_eval(&client.perform_history_summary().await.unwrap_or_default(), user_prompt);

    // `res` should be of a JSON with the following structure:
    // {
    //   "step": "prompt_assessment",
    //   "result": "<clarify | proceed | reject>",
    //   "confidence": <number 0-1>,
    //   "missing": [ "<string>", … ]   # required ONLY if result == "clarify"
    // }
    //
    let res: String = match
        &client.send_message(
            &sys_prompt,
            Some("gpt-4.1-nano".to_string())
        ).await
    {
        Ok(response) => {
            println!("{}", format!("Received response: {}", response));
            response.clone()
        }
        Err(e) => {
            return EvalInputResult::Error(
                format!("[ERROR] Failed to make independent request: {}", e)
            );
        }
    };

    // Try to parse the response as JSON
    let res_obj: Value = match serde_json::from_str::<serde_json::Value>(&res) {
        Ok(json) => {
            println!("{}", format!("Parsed JSON: {}", json));
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
    println!("{}", format!("Action determined: {}", action));

    match action {
        "clarify" => {
            let default_vec: Vec<Value> = Vec::with_capacity(0); // Default value for missing information -- used in case no vector it's provided

            let missing: Vec<String> = res_obj
                .get("missing")
                .and_then(Value::as_array)
                .unwrap_or(&default_vec)
                .iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect::<Vec<String>>();

            println!("{}", format!("Missing information: {:?}", missing));
            return EvalInputResult::Clarify(missing);
        }

        "proceed" => {
            let confidence = res_obj.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
            println!("{}", format!("Confidence level: {}", confidence));
            return EvalInputResult::Proceed;
        }

        "reject" => {
            let confidence = res_obj.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
            println!("{}", format!("Confidence level: {}", confidence));
            return EvalInputResult::Reject;
        }
        _ => EvalInputResult::Error(format!("[ERROR] Unknown action: {}", action)),
    }
}

/// Entry point for the pipeline, receives the prompt, chat history, and other parameters.
/// If everything goes as expected, after possible asking the user for more information, and doing some socratic reasoning,
/// It will return a polished response that can be used in the chat.
/// Notice that, as long as the AI finds it necessary, it will keep "doing things" and re-prompting itself until it reaches a point where it can return a response.
pub async fn launch(prompt: &str, client: &mut Client) -> String {
    let eval_result: EvalInputResult = eval_user_prompt(prompt, client).await;

    println!("{}", format!("Evaluation result: {:?}", eval_result));

    String::new()
}
