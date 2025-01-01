use pyano::model::{ ModelManager, ModelManagerServer };
use std::sync::Arc;

use log::{ info, error };
use pyano::model::config_loader::ModelRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the manager
    let manager = Arc::new(ModelManager::new());

    // List all models from registry
    let models = manager.list_models().await?;
    let registery: ModelRegistry = ModelRegistry::new();
    //TODO : Add models in listmodels from configs
    println!("\nAvailable Models in Registry:");
    println!("---------------------------");
    for model in models {
        println!(
            "Name: {}\nType: {:?}\nStatus: {:?}\nServer Port: {:?}\nLast Used: {}\n",
            model.name,
            model.model_type,
            model.status,
            model.server_port,
            model.last_used
        );
    }

    // Print configs from ModelRegistry
    println!("\nFrom ModelRegistry:");
    // Try getting configs for known models
    for model_name in ["qwen-7b", "llama-7b", "smolTalk", "granite"] {
        if let Some(config) = registery.get_config(model_name) {
            println!(
                "Name: {}\nType: {:?}\nKind: {}\nPath: {:?}\nMemory: {:?} GB (min) / {:?} GB (recommended)\nPort: {:?}\n",
                config.name,
                config.model_type,
                config.model_kind,
                config.model_path,
                config.memory_config.min_ram_gb,
                config.memory_config.recommended_ram_gb,
                config.server_config.port
            );
        }
    }

    Ok(())
}
