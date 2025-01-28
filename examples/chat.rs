use std::error::Error as StdError;
use std::io::{ self, Write };

use pyano::{
    llm::{
        options::LLMHTTPCallOptions,
        llm_builder::LLM,
        stream_processing::llamacpp_process_stream,
    },
    agent::{ agent_builder::AgentBuilder, agent_trait::AgentTrait },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let prompt_template =
        "
        <｜begin▁of▁sentence｜>{system_prompt}<｜User｜>{user_prompt}<｜Assistant｜>
    ";
    let system_prompt = "Answer the user questions";

    // Create LLMHTTPCallOptions with required configurations
    let options = LLMHTTPCallOptions::new()
        .with_server_url("http://localhost:52555".to_string())
        .with_prompt_template(prompt_template.to_string())
        .with_temperature(0.7)
        .build();

    println!("Welcome to the LLM CLI! Type 'exit' to quit.");
    // Define the user prompt

    // Execute the LLM call with the user prompt
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;

        let user_prompt = user_input.trim();

        if user_prompt.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }

        // Build a new LLM instance for each interaction
        let llm = LLM::builder()
            .with_options(options.clone())
            .with_process_response(|stream| Box::pin(llamacpp_process_stream(stream)))
            .build();

        // Create a new agent for each interaction
        let agent = AgentBuilder::new()
            .with_system_prompt(system_prompt.to_string())
            .with_user_prompt(user_prompt.to_string())
            .with_stream(true)
            .with_llm(llm)
            .build();

        match agent.invoke().await {
            Ok(_) => println!("\n---"),
            Err(e) => eprintln!("Error during processing: {}", e),
        }
    }

    Ok(())
}
