use cai_core::client::Client;
use pipeline::launch;



#[tokio::main]
async fn main() {   
    let mut client: Client = Client::new();

    let tests_prompts = vec!["What is going on this codebase?"];

    for prompt in tests_prompts {
        let result: String = launch(prompt, &mut client).await;
        println!("Prompt: {}\nResponse: {}", prompt, result);
        println!("----------------------------------------------------------------\n");
    }
}