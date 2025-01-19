use std::{ collections::HashMap, error::Error as StdError };
use axum::Json;
use pyano::{ agent::agent_builder::AgentBuilder, chain::sequential_chain::Chain, ModelManager };
use log::{ info, error };
use std::sync::{ Arc, Mutex };

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    info!("Initializing ModelManager");
    let model_manager = Arc::new(ModelManager::new());
    model_manager.show_registry(); // get_model_registry
    info!("Loading SmolTalk model");
    let content_llm = model_manager
        .clone()
        .get_or_create_llm("smolTalk", true).await
        .map_err(|e| {
            error!("Failed to load SmolTalk model: {}", e);
            e
        })?;
    let mut state: HashMap<String, serde_json::Value> = HashMap::new();
    state.insert("Port".to_string(), serde_json::json!(5010));
    state.insert("Temprature".to_string(), serde_json::json!(0.8));
    let llama_llm = model_manager
        .clone()
        .get_or_create_llm_with_state("granite", state, true).await
        .map_err(|e| {
            error!("Failed to load Granite model: {}", e);
            e
        })?;

    model_manager.show_model_details().await;

    // Create agents
    let agent_1 = Arc::new(
        Mutex::new(
            AgentBuilder::new()
                .with_name(String::from("Content Generator Agent"))
                .with_system_prompt("You are an excellent content generator.".to_string())
                .with_user_prompt(
                    "Generate content on the topic - Future of AI agentix framework".to_string()
                )
                .with_stream(true)
                .with_llm(content_llm.clone())
                .build()
        )
    );
    // Get LLM for LLaMA (Qwen will be unloaded if memory is low)
    let agent_2 = Arc::new(
        Mutex::new(
            AgentBuilder::new()
                .with_name(String::from("Analyzer Agent"))
                .with_system_prompt("You are a great analyzer of generated content.".to_string())
                .with_user_prompt("Analyze the generated content.".to_string())
                .with_stream(true)
                .with_llm(llama_llm.clone())
                .build()
        )
    );
    // Create a chain and add agents
    let mut chain = Chain::new().add_agent(agent_1).add_agent(agent_2);
    model_manager.show_model_details().await;
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
    model_manager.show_model_details().await;

    Ok(())
}
