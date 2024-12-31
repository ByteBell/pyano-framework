use serde::{ Deserialize, Serialize };
use std::path::PathBuf;
use std::collections::HashMap;
use chrono::{ DateTime, Utc };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub model_path: PathBuf,
    pub model_type: ModelType,
    pub model_kind: String, // e.g. "Qwen", "LLaMA", smolVLM

    pub memory_config: ModelMemoryConfig,
    pub prompt_template: PromptTemplate,
    pub defaults: ModelDefaults,
    pub server_config: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    // Network configuration
    pub host: String,
    pub port: Option<u16>,

    // Model server settings
    pub ctx_size: usize,
    pub gpu_layers: i32,
    pub batch_size: usize,
    pub num_threads: Option<usize>,
    pub use_mmap: bool,
    pub use_gpu: bool,

    // Additional configuration
    pub extra_args: HashMap<String, String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: None,
            ctx_size: 2048,
            gpu_layers: 0,
            batch_size: 512,
            num_threads: None,
            use_mmap: true,
            use_gpu: false,
            extra_args: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    Text,
    Voice,
    Vision,
    #[serde(untagged)] Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextModelKind {
    Qwen,
    LLaMA,
    Mistral,
    // TODO Add more
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AudioModelKind {
    Whisper,
    Qwen2Audio,
    // Add more as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMemoryConfig {
    pub min_ram_gb: f32,
    pub recommended_ram_gb: f32,
    pub gpu_memory_gb: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub template: String,
    pub required_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDefaults {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    pub max_tokens: usize,
    pub repetition_penalty: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub server_port: Option<u16>,
    pub ctx_size: usize,
    pub gpu_layers: i32,
    pub batch_size: usize,
    pub extra_args: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub model_type: ModelType,
    pub status: ModelStatus,
    pub last_used: DateTime<Utc>,
    pub server_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelStatus {
    Loading,
    Running,
    Stopped,
    Error(String),
}
