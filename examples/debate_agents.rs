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
use tokio::time::Duration;
use std::io::{ self, Write };
use env_logger::Builder;
use pyano::agent::agent_trait::AgentTrait;
use colored::Colorize;

pub fn setup_logger() {
    Builder::from_default_env()
        .format(|buf, record| {
            if record.level() == log::Level::Info {
                writeln!(buf, "{}", record.args())
            } else {
                writeln!(buf, "[{}] {}: {}", record.level(), record.target(), record.args())
            }
        })
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError + Send + Sync>> {
    // std::env::set_var("RUST_LOG", "info");
    setup_logger();

    let model_manager = Arc::new(ModelManager::new());
    model_manager.show_models();

    // Setup first LLM for pro-argument agent
    let pro_llm = model_manager
        .clone()
        .get_llm("smolTalk", None).await
        .map_err(|e| {
            error!("Failed to Get SmolTalk model: {}", e);
            e
        })?;

    let pro_llm_for_updates = pro_llm.clone();

    // Setup second LLM for counter-argument agent with custom template

    let contra_llm = model_manager
        .clone()
        .get_llm("granite", None).await
        .map_err(|e| {
            error!("Failed to Get Granite model: {}", e);
            e
        })?;

    let contra_llm_for_updates = contra_llm.clone();

    println!("Enter the debate topic:");
    let mut topic = String::new();
    io::stdin().read_line(&mut topic)?;
    let topic = topic.trim().to_string();

    let mut current_context = topic.clone();
    let mut round = 1;

    // Create pro-argument agent
    let pro_agent = Arc::new(
        Mutex::new(
            AgentBuilder::new()
                .with_name(String::from("Pro Agent"))
                .with_system_prompt(
                    "You are a debater who makes compelling arguments in favor of the given topic. \
                     Respond to counter-arguments thoughtfully and maintain a respectful tone.".to_string()
                )
                .with_user_prompt(topic.clone())
                .with_stream(true)
                .with_llm(pro_llm)
                .build()
        )
    );

    // Create counter-argument agent
    let contra_agent = Arc::new(
        Mutex::new(
            AgentBuilder::new()
                .with_name(String::from("Contra Agent"))
                .with_system_prompt(
                    "You are a debater who presents thoughtful counter-arguments to the given topic. \
                     Challenge assumptions while maintaining a respectful and constructive tone.".to_string()
                )
                .with_user_prompt(topic.clone())
                .with_stream(true)
                .with_llm(contra_llm)
                .build()
        )
    );

    println!("\nDebate starting on topic: {}\n", topic);
    println!("Press Ctrl+C to stop the debate\n");

    loop {
        println!("\nRound {}", round);
        println!("---------");

        // Pro agent's turn
        {
            let mut agent = pro_agent.lock().unwrap();
            // Create new agent with updated prompt
            *agent = AgentBuilder::new()
                .with_name(String::from("Pro Agent"))
                .with_system_prompt(
                    "You are a debater who makes compelling arguments in favor of the given topic. \
                     Respond to counter-arguments thoughtfully and maintain a respectful tone. Respond in less than 100 words.".to_string()
                )
                .with_user_prompt(
                    format!("Topic: {}. Previous context: {}", topic, current_context)
                )
                .with_stream(true)
                .with_llm(pro_llm_for_updates.clone())
                .build();

            println!("\n {}", "Pro Agent:".green());
            let pro_response = agent.invoke().await?;
            // println!("\nPro Agent: {}", pro_response);
            current_context = pro_response;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Contra agent's turn
        {
            let mut agent = contra_agent.lock().unwrap();
            // Create new agent with updated prompt
            *agent = AgentBuilder::new()
                .with_name(String::from("Contra Agent"))
                .with_system_prompt(
                    "You are a debater who presents thoughtful counter-arguments to the given topic. \
                     Challenge assumptions while maintaining a respectful and constructive tone. Respond in less than 100 words.".to_string()
                )
                .with_user_prompt(
                    format!("Topic: {}. Previous context: {}", topic, current_context)
                )
                .with_stream(true)
                .with_llm(contra_llm_for_updates.clone())
                .build();

            println!("\n{}", "Contra Agent:".bright_blue());

            let contra_response = agent.invoke().await?;
            current_context = contra_response;
        }

        round += 1;
        // tokio::time::sleep(Duration::from_secs(2)).await;

        // println!("\nContinue debate? (y/n):");
        // let mut continue_debate = String::new();
        // io::stdin().read_line(&mut continue_debate)?;
        // if continue_debate.trim().to_lowercase() != "y" {
        //     break;
        // }
    }

    println!("\nDebate concluded after {} rounds", round - 1);
    Ok(())
}
