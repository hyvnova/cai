use async_openai::{ config::OpenAIConfig, Client as APIClient };
use serde_json::Value;
use tokio::time::{ sleep, Duration };

const MAX_RETRIES: usize = 5; // Max retries for API requests

/// Keep retrying with exponential back-off until the call succeeds.
///
/// Every failure is treated as “rate-limited”.
pub async fn call_with_backoff(
    ai: &APIClient<OpenAIConfig>,
    payload: serde_json::Value
) -> Result<String, ()> {
    const BASE_DELAY: Duration = Duration::from_secs(2); // 1st wait
    const MAX_DELAY: Duration = Duration::from_secs(60); // cap

    let mut delay = BASE_DELAY;
    for _ in 0..MAX_RETRIES {

        let response: Value =  match ai.chat().create_byot(payload.clone()).await {
            Ok(res) => {
               res
            } // 🎉 success → return

            Err(err) => {
                eprintln!("[Retry] Request failed ({}). Sleeping {:?}…", err, delay);
                sleep(delay).await;
                delay = (delay * 2).min(MAX_DELAY); // exponential growth, capped
                continue; // retry
            }

        };

        if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
            // self.history.add_message(MessageRole::Assistant, content.to_string());

            // ui.print_message(
            //     MsgRole::Assistant,
            //     MsgType::Plain(content.to_string()),
            // );

            return Ok(content.to_string());
        } else {
            // println!("[ERROR] No content in response.");
            // self.history.add_message(
            //     MessageRole::System,
            //     format!("[ERROR] No message recieved from assistant")
            // );
            return Err(());
        }

    }
    Err(())
}
