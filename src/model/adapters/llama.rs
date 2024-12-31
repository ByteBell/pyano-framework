// todo this will use the llama.cpp
// the address to adapter will be given in config to the model manager

use std::process::{ Child, Command };
use std::thread::sleep;
use std::time::Duration;
use chrono::{ DateTime, Utc };
use log::{ info, error };
use tokio::sync::oneshot;

use super::super::{ ModelConfig, ModelStatus };
use super::super::error::{ ModelError, ModelResult };

pub(crate) struct LlamaProcess {
    pub config: ModelConfig,
    pub cmd: Option<Command>,
}

impl LlamaProcess {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            cmd: None,
        }
    }

    pub async fn getcmd(&mut self) {
        /* ToDO Implement server based on machine type */
        let mut cmd = if cfg!(target_os = "macos") {
            Command::new("./src/model/adapters/llama/arm64/llama-server")
        } else {
            Command::new("./src/model/adapters/llama/ubuntu/llama-server")
        };

        // Configure command based on adapter config
        cmd.arg("-m")
            .arg(&self.config.model_path)
            .arg("--ctx-size")
            .arg(self.config.server_config.ctx_size.to_string());

        if let Some(port) = self.config.server_config.port {
            cmd.arg("--port").arg(port.to_string());
        }

        if let Some(threads) = self.config.server_config.num_threads {
            cmd.arg("--threads").arg(threads.to_string());
        }

        if self.config.server_config.gpu_layers > 0 {
            cmd.arg("--n-gpu-layers").arg(self.config.server_config.gpu_layers.to_string());
        }

        if !self.config.server_config.use_mmap {
            cmd.arg("--no-mmap");
        }

        // Add batch size
        cmd.arg("--batch-size").arg(self.config.server_config.batch_size.to_string());

        // Add extra arguments
        for (key, value) in &self.config.server_config.extra_args {
            cmd.arg(format!("--{}", key)).arg(value);
        }

        self.cmd = Some(cmd);
    }
}
