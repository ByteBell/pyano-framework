use std::error::Error as StdError;
use serde_json::{ json, Value };
use log::{ info, error };

use pyano::llm::Cloud::cloud_llm::{ cloudLLM };

#[tokio::main]
async fn main() {
    let cloud_agent_result = cloudLLM::new().client.build("<API_KEY".to_string()).await;
    print!("Cloud agent built successfully");
}
