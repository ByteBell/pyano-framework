use async_trait::async_trait;
use crate::llm::llm_builder::LLM;
use super::types::{ ModelConfig, ModelInfo, ModelStatus };
use super::error::ModelResult;
use super::state::ModelState;

#[async_trait]
pub trait ModelManagerInterface: Send + Sync {
    async fn load_model(&self, config: ModelConfig) -> ModelResult<()>;
    async fn unload_model(&self, name: &str) -> ModelResult<()>;
    async fn get_model_status(&self, name: &str) -> ModelResult<ModelStatus>;
    async fn list_models(&self) -> ModelResult<Vec<ModelInfo>>;
    async fn get_or_create_llm(&self, model_name: &str, auto_load: bool) -> ModelResult<LLM>;
    async fn get_or_create_llm_with_state(
        &self,
        model_name: &str,
        state: ModelState,
        auto_load: bool
    ) -> ModelResult<LLM>;
    async fn load_model_by_name(&self, name: &str) -> ModelResult<()>;
}
