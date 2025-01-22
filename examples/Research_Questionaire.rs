use std::fmt::format;
use std::{ collections::HashMap, error::Error as StdError };
use axum::Json;
use pyano::{
    llm::options::LLMHTTPCallOptions,
    agent::agent_builder::AgentBuilder,
    chain::sequential_chain::Chain,
    ModelManager,
};
use log::{ info, error };
use std::sync::{ Arc, Mutex };
use pdf::file::File as PdfFile;
use pdf::object::Resolve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    let model_manager = Arc::new(ModelManager::new());

    let researcher_llm = model_manager
        .clone()
        .get_llm("DeepSeek-R1", None).await
        .map_err(|e| {
            error!("Failed to Get DeepSeek model: {}", e);
            e
        })?;

    researcher_llm.clone().load().await;

    let novice_llm = model_manager
        .clone()
        .get_llm("Qwen2.5-1.5B", None).await
        .map_err(|e| {
            error!("Failed to Get Qwen model: {}", e);
            e
        })?;

    novice_llm.clone().load().await;

    let pdf_path = "/home/deadbytes/Downloads/DeepSeek_R1.pdf";
    let paper_content = std::fs::read_to_string(pdf_path).map_err(|e| {
        error!("Failed to read PDF file: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    let answer = Arc::new(
        Mutex::new(
            AgentBuilder::new()
                .with_name(String::from("Researcher Agent"))
                .with_system_prompt(
                    format!("You are an excellent Researcher who has read the Following Paper thoroughly and only replies to the questions related to this paper otherwise you are not interested in answering stuff. However, while answering about the paper you are nerdy about it! Here is the paper content: \n{} \n", paper_content).to_string()
                )
                .with_user_prompt("Reply to the Question based on the Read Paper".to_string())
                .with_stream(true)
                .with_llm(researcher_llm)
                .build()
        )
    );

    let question = Arc::new(
        Mutex::new(
            AgentBuilder::new()
                .with_name(String::from("Novice Agent"))
                .with_system_prompt(
                    format!("You ask very ntellectual questions about serious topics but also like to goof around a little. Please Ask Questions based on the base of the followng paper \n {} \n you will get replies and based on that replies keep asking questions also you can generate new questions to start a new conversation ", paper_content).to_string()
                )
                .with_user_prompt("Analyze the generated content.".to_string())
                .with_stream(true)
                .with_llm(novice_llm)
                .build()
        )
    );
    // Create a chain and add agents
    let mut chain = Chain::new().add_agent(question).add_agent(answer);
    // Run the chain
    if let Err(e) = chain.run().await {
        eprintln!("Error executing chain: {}", e);
    }
    model_manager.show_model_details().await;

    // Access the memory logs
    let logs = chain.memory_logs();
    for log in logs {
        println!(
            "Agent: {}, Input: {}, Output: {}, Timestamp: {:?}",
            log.agent_name,
            log.input,
            log.output,
            log.timestamp
        );
    }
    Ok(())
}
