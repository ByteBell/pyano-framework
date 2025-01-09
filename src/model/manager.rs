use async_trait::async_trait;
use log::{ debug, error, info };
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use std::sync::atomic::{ AtomicBool, Ordering };
use std::time::{ Duration, SystemTime, UNIX_EPOCH };
use parking_lot::Mutex;
use super::process::ModelProcess;
use super::config_loader::ModelRegistry;
use super::error::{ ModelError, ModelResult };
use super::{ ModelConfig, ModelInfo, ModelStatus, ModelType, SystemMemory };
use crate::llm::llm_builder::LLM;
use crate::llm::options::LLMHTTPCallOptions;
use crate::llm::stream_processing::llamacpp_process_stream;
use crate::llm::types::AccumulatedStream;

use std::pin::Pin;
use bytes::Bytes;
use futures::Stream;
use super::manager_trait::ModelManagerInterface;

type StreamProcessor = Arc<dyn (Fn(AccumulatedStream) -> AccumulatedStream) + Send + Sync>;

pub struct ModelManager {
    models: Arc<RwLock<HashMap<String, ModelProcess>>>,
    registry: ModelRegistry,
    system_memory: SystemMemory,

    lock_in_progress: Arc<AtomicBool>,
    last_lock_holder: Arc<Mutex<Option<String>>>, // For debugging
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            registry: ModelRegistry::new(),
            system_memory: SystemMemory::new(),

            lock_in_progress: Arc::new(AtomicBool::new(false)),
            last_lock_holder: Arc::new(Mutex::new(None)),
        }
    }

    async fn acquire_models_lock<'a>(
        &'a self,
        operation: &str,
        timeout: Duration
    ) -> ModelResult<tokio::sync::RwLockWriteGuard<'a, HashMap<String, ModelProcess>>> {
        info!("Starting lock acquisition for operation: {}", operation);

        // First, try to get read lock to check current state
        info!("Attempting to get read lock to check state...");
        if let Ok(read_guard) = self.models.try_read() {
            let count = read_guard.len();
            drop(read_guard); // Explicitly drop the read guard
            info!("Current models count (from read lock): {}", count);
        } else {
            info!("Could not get immediate read lock - might be exclusively locked");
        }

        // Try immediate write lock
        info!("Attempting immediate write lock acquisition...");
        if let Ok(guard) = self.models.try_write() {
            info!("Successfully acquired immediate write lock");
            return Ok(guard);
        } else {
            info!("Immediate write lock not available, falling back to timed attempt");
        }

        // If immediate acquisition fails, try with timeout
        info!("Starting timed write lock acquisition...");

        let start = std::time::Instant::now();
        let mut attempts = 0;

        while start.elapsed() < timeout {
            attempts += 1;
            info!("Write lock acquisition attempt #{}", attempts);

            match tokio::time::timeout(Duration::from_secs(1), self.models.write()).await {
                Ok(guard) => {
                    info!(
                        "Successfully acquired write lock after {} attempts and {:?}",
                        attempts,
                        start.elapsed()
                    );
                    return Ok(guard);
                }
                Err(_) => {
                    if attempts % 2 == 0 {
                        // Log every other attempt to avoid spam
                        info!(
                            "Still trying to acquire write lock - attempt #{}, elapsed: {:?}",
                            attempts,
                            start.elapsed()
                        );
                    }
                    // Small delay before next attempt
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }

        error!(
            "Failed to acquire write lock after {} attempts and {:?}",
            attempts,
            start.elapsed()
        );

        Err(
            ModelError::ProcessError(
                format!(
                    "Lock acquisition timeout after {} attempts for operation: {}",
                    attempts,
                    operation
                )
            )
        )
    }
    pub async fn load_model(&self, config: ModelConfig) -> ModelResult<()> {
        self.record_lock_event(&format!("Starting load_model for {}", config.name));

        // First check if model is already loaded without holding write lock
        {
            let read_guard = self.models.read().await;
            if let Some(process) = read_guard.get(&config.name) {
                if process.status == ModelStatus::Running {
                    self.record_lock_event(&format!("Model {} already loaded", config.name));
                    return Ok(());
                }
            }
        }

        self.record_lock_event("Checking memory requirements");
        self.system_memory.debug_memory_info().await;

        // Memory management with proper lock release
        match self.manage_memory(config.memory_config.min_ram_gb).await {
            Ok(_) => {
                info!("Memory requirements satisfied for model {}", config.name);
            }
            Err(e) => {
                error!("Failed to allocate memory for model {}: {}", config.name, e);
                return Err(e);
            }
        }

        self.record_lock_event("Acquiring write lock for model insertion");
        let mut models = match
            tokio::time::timeout(std::time::Duration::from_secs(5), self.models.write()).await
        {
            Ok(guard) => guard,
            Err(_) => {
                self.record_lock_event("Timeout acquiring write lock in load_model");
                return Err(ModelError::ProcessError("Timeout acquiring write lock".to_string()));
            }
        };

        let mut process = ModelProcess::new(config.clone());
        match process.start().await {
            Ok(_) => {
                info!("Successfully started model process: {}", config.name);
                models.insert(config.name.clone(), process);
                self.record_lock_event(&format!("Successfully loaded model {}", config.name));
                Ok(())
            }
            Err(e) => {
                error!("Failed to start model process: {}", e);
                self.record_lock_event(&format!("Failed to start model process: {}", e));
                Err(e)
            }
        }
    }

    async fn load_model_by_name(&self, name: &str) -> ModelResult<()> {
        let config = self.registry
            .get_config(name)
            .ok_or_else(|| {
                ModelError::ModelNotFound(format!("Configuration not found for model: {}", name))
            })?;
        self.load_model(config.clone()).await
    }

    pub async fn unload_model(&self, name: &str) -> ModelResult<()> {
        let mut models = self.models.write().await;

        if let Some(process) = models.get_mut(name) {
            process.stop().await?;
            models.remove(name);
            Ok(())
        } else {
            Err(ModelError::ModelNotFound(name.to_string()))
        }
    }

    pub async fn get_model_status(&self, name: &str) -> ModelResult<ModelStatus> {
        let models = self.models.read().await;

        match models.get(name) {
            Some(process) => Ok(process.status.clone()),
            None => Err(ModelError::ModelNotFound(name.to_string())),
        }
    }

    pub async fn list_models(&self) -> ModelResult<Vec<ModelInfo>> {
        let models = self.models.read().await;

        Ok(
            models
                .values()
                .map(|process| ModelInfo {
                    name: process.config.name.clone(),
                    model_type: process.config.model_type.clone(),
                    status: process.status.clone(),
                    last_used: process.last_used,
                    server_port: process.config.server_config.port,
                })
                .collect()
        )
    }

    fn get_processor_for_model(config: &ModelConfig) -> StreamProcessor {
        match config.model_type {
            ModelType::Text =>
                match config.model_kind.as_str() {
                    "LLaMA" =>
                        Arc::new(move |stream: AccumulatedStream| -> AccumulatedStream {
                            Box::pin(llamacpp_process_stream(stream))
                        }),
                    "Qwen" =>
                        Arc::new(move |stream: AccumulatedStream| -> AccumulatedStream {
                            Box::pin(qwen_process_stream(stream))
                        }),
                    _ =>
                        Arc::new(move |stream: AccumulatedStream| -> AccumulatedStream {
                            Box::pin(llamacpp_process_stream(stream))
                        }),
                }
            _ =>
                Arc::new(move |stream: AccumulatedStream| -> AccumulatedStream {
                    Box::pin(llamacpp_process_stream(stream))
                }),
        }
    }

    pub async fn get_or_create_llm(
        self: Arc<Self>,
        model_name: &str,
        options: Option<LLMHTTPCallOptions>,
        auto_load: bool
    ) -> ModelResult<LLM> {
        let config = self.registry.get_config(model_name).ok_or_else(|| {
            error!("Model configuration not found for: {}", model_name);
            ModelError::ModelNotFound(format!("Configuration not found for model: {}", model_name))
        })?;

        let model_status = self.get_model_status(model_name).await;
        // info!("Current model status: {:?}", model_status);

        // Only load the model immediately if auto_load is true
        if auto_load {
            let model_status = self.get_model_status(model_name).await;
            match model_status {
                Ok(ModelStatus::Running) => {
                    info!("Model {} is already running", model_name);
                }
                _ => {
                    info!("Loading model: {}", model_name);
                    self.load_model(config.clone()).await?;
                    // Verify model was loaded successfully
                    match self.get_model_status(model_name).await? {
                        ModelStatus::Running => {
                            info!("Model {} loaded successfully", model_name);
                        }
                        status => {
                            error!("Model {} failed to load properly", model_name);
                            return Err(
                                ModelError::ProcessError(
                                    format!(
                                        "Failed to load model: {}. Status: {:?}",
                                        model_name,
                                        status
                                    )
                                )
                            );
                        }
                    }
                }
            }
        }

        // Create LLM with model-specific configurations
        let mut llm_options = options.unwrap_or_default();
        llm_options = llm_options
            .with_server_url(
                format!(
                    "http://{}:{}",
                    config.server_config.host,
                    config.server_config.port.unwrap_or(8000)
                )
            )
            .with_prompt_template(config.prompt_template.template.clone());

        // Apply model defaults if not overridden
        if llm_options.temperature.is_none() {
            llm_options = llm_options.with_temperature(config.defaults.temperature);
        }

        let processor = Self::get_processor_for_model(&config);
        // let manager: Arc<dyn ModelManagerInterface> = Arc::new(self.clone());
        Ok(
            LLM::builder()
                .with_model_manager(self.clone(), model_name.to_string(), auto_load)
                .with_options(llm_options)
                .with_process_response(move |stream| processor(stream))
                .build()
        )
    }

    async fn manage_memory(&self, required_gb: f32) -> ModelResult<()> {
        info!("Starting memory management for {:.1} GB", required_gb);

        // Get initial memory status
        let initial_status = self.system_memory.get_memory_status().await;
        info!(
            "Initial memory status:\n\
             Available: {:.1} GB\n\
             Total: {:.1} GB\n\
             Usage: {:.1}%",
            initial_status.available_gb,
            initial_status.total_gb,
            initial_status.usage_percentage
        );

        if self.system_memory.has_available_memory(required_gb).await {
            info!("Sufficient memory available ({:.1} GB required)", required_gb);
            return Ok(());
        }

        // Try to get models lock with detailed diagnostics
        info!("Attempting to acquire models lock for memory management...");
        let models_result = self.acquire_models_lock(
            "manage_memory",
            Duration::from_secs(10) // Increased timeout
        ).await;

        let mut models = match models_result {
            Ok(guard) => {
                info!("Successfully acquired models lock for memory management");
                guard
            }
            Err(e) => {
                error!("Failed to acquire models lock: {}", e);
                return Err(e);
            }
        };
        if models.is_empty() {
            info!("No models currently loaded to unload");
            return Err(
                ModelError::MemoryError(
                    "No models available to unload for freeing memory".to_string()
                )
            );
        }

        // Convert to vec for sorting
        let mut model_times: Vec<_> = models
            .iter()
            .map(|(k, v)| (k.clone(), v.last_used))
            .collect();

        // Sort by last used time (oldest first)
        model_times.sort_by_key(|(_k, v)| *v);

        // Track unloading results
        let mut freed_memory = 0.0;
        let mut unloaded_models = Vec::new();
        let mut failed_unloads = Vec::new();

        // Unload models until we have enough memory
        for (model_name, _) in model_times {
            if let Some(process) = models.get_mut(&model_name) {
                let model_memory = process.config.memory_config.min_ram_gb;

                info!("Attempting to unload model: {}", model_name);

                match process.stop().await {
                    Ok(()) => {
                        freed_memory += model_memory;
                        unloaded_models.push(model_name.clone());

                        // Remove from models map
                        models.remove(&model_name);

                        info!(
                            "Unloaded model: {} - Total freed memory: {:.1} GB",
                            model_name,
                            freed_memory
                        );

                        if self.system_memory.has_available_memory(required_gb).await {
                            info!("Successfully freed enough memory");
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        error!("Failed to unload model {}: {}", model_name, e);
                        failed_unloads.push((model_name.clone(), e.to_string()));
                    }
                }
            }
        }

        // If we get here, we couldn't free enough memory
        let mem_status = self.system_memory.get_memory_status().await;
        Err(
            ModelError::MemoryError(
                format!(
                    "Could not allocate enough memory ({:.1} GB required) after unloading attempt.\n\
             Memory Status:\n\
             - Available: {:.1} GB\n\
             - Total: {:.1} GB\n\
             - Usage: {:.1}%\n\
             Unloading Results:\n\
             - Successfully unloaded: {:?} (freed {:.1} GB)\n\
             - Failed to unload: {:?}",
                    required_gb,
                    mem_status.available_gb,
                    mem_status.total_gb,
                    mem_status.usage_percentage,
                    unloaded_models,
                    freed_memory,
                    failed_unloads
                )
            )
        )
    }

    // Add this method to help diagnose lock issues
    pub async fn debug_lock_status(&self) -> String {
        format!(
            "Lock in progress: {}, Last operation: {:?}",
            self.lock_in_progress.load(Ordering::SeqCst),
            self.last_lock_holder.lock().clone()
        )
    }

    // Add diagnostic method
    pub async fn diagnose_locks(&self) -> String {
        let read_result = self.models.try_read();
        let write_result = self.models.try_write();

        match (read_result, write_result) {
            (Ok(_), _) => "Read lock available - no write lock held".to_string(),
            (_, Ok(_)) => "Write lock available - no locks held".to_string(),
            _ => "All locks currently held".to_string(),
        }
    }

    // Add lock tracking
    fn record_lock_event(&self, event: &str) {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        info!("Lock Event [{}]: {}", timestamp, event);
    }

    // Add show registry function

    pub fn show_registry(&self) {
        info!("Available models in the registry:");
        for (name, config) in self.registry.get_all_configs() {
            info!("Model Name: {}, Type: {:?}", name, config.model_type);
        }
    }
}

#[async_trait]
impl ModelManagerInterface for ModelManager {
    async fn load_model(&self, config: ModelConfig) -> ModelResult<()> {
        // Existing implementation
        self.load_model(config).await
    }

    async fn unload_model(&self, name: &str) -> ModelResult<()> {
        // Existing implementation
        self.unload_model(name).await
    }

    async fn get_model_status(&self, name: &str) -> ModelResult<ModelStatus> {
        // Existing implementation
        self.get_model_status(name).await
    }

    async fn list_models(&self) -> ModelResult<Vec<ModelInfo>> {
        // Existing implementation
        self.list_models().await
    }

    async fn get_or_create_llm(
        &self,
        model_name: &str,
        options: Option<LLMHTTPCallOptions>,
        auto_load: bool
    ) -> ModelResult<LLM> {
        // Existing implementation
        self.get_or_create_llm(model_name, options, auto_load).await
    }
    async fn load_model_by_name(&self, name: &str) -> ModelResult<()> {
        self.load_model_by_name(name).await
    }
}

pub fn qwen_process_stream(
    stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>
) -> Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>> {
    // Implementation similar to llamacpp_process_stream but for Qwen
    // For now, we can use the same implementation
    llamacpp_process_stream(stream)
}
