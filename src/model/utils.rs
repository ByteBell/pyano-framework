use std::collections::HashMap;
use std::env;
use dotenv::dotenv;

pub fn get_env_vars() -> HashMap<String, String> {
    // Load .env file
    dotenv().ok();
    
    // Collect all environment variables into HashMap
    env::vars().collect()
}

// Example usage function
pub fn get_env_var(key: &str) -> Option<String> {
    dotenv().ok();
    env::var(key).ok()
}