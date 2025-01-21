use std::error::Error as StdError;
use serde_json::{ json, Value };
use log::{ info, error };

use pyano::llm::Cloud::cloud_llm::{ cloudLLM };
use std::env;

#[tokio::main]
async fn main() {
    env::set_var("OPENAI_API_KEY", "<YOUR_API_KEY>");
    let cloud_agent_result = cloudLLM::new().client.build().await;
    print!("Cloud agent built successfully");
    let cloud_agent = cloud_agent_result.unwrap();
    match cloud_agent.client.chat().create(cloud_agent.request).await {
        Ok(response) => {
            println!("\nResponse:\n");
            for choice in response.choices {
                println!(
                    "{}: Role: {}  Content: {:?}",
                    choice.index,
                    choice.message.role,
                    choice.message.content
                );
            }
        }
        Err(e) => {
            print!("Error creating chat completion: {}", e);
        }
    }
}
