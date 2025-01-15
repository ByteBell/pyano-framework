use dotenv::dotenv;

pub fn init() {
    dotenv().ok();
}

pub mod agent;
pub mod model;
pub mod types;
pub mod llm;
pub mod tools;
pub mod chain;
pub use model::manager::ModelManager;
pub mod embedding;
pub mod vectorstore;
pub mod schemas;
pub use types::*;
