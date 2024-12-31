use std::process::{ Child, Command };
use std::thread::sleep;
use std::time::Duration;
use chrono::{ DateTime, Utc };
use log::{ info, error };
use tokio::sync::oneshot;
use super::adapters::llama::LlamaProcess;
use super::{ ModelConfig, ModelStatus };
use super::error::{ ModelError, ModelResult };
pub(crate) struct ModelProcess {
    pub config: ModelConfig,
    pub child: Option<Child>,
    pub status: ModelStatus,
    pub last_used: DateTime<Utc>,
    pub shutdown_signal: Option<oneshot::Sender<()>>,
    pub model_process: Option<LlamaProcess>,
}

impl ModelProcess {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            child: None,
            status: ModelStatus::Stopped,
            last_used: Utc::now(),
            shutdown_signal: None,
            model_process: None,
        }
    }

    pub async fn start(&mut self) -> ModelResult<()> {
        if self.status == ModelStatus::Running {
            return Ok(());
        }
        info!("Starting model {}", self.config.name);
        self.status = ModelStatus::Loading;
        self.model_process = Some(LlamaProcess::new(self.config.clone()));
        self.model_process.as_mut().unwrap().getcmd().await;
        let mut cmd = self.model_process.as_mut().unwrap().cmd.as_mut().unwrap();
        info!("Starting model with command: {:?}", cmd);
        match self.model_process.as_mut().unwrap().cmd.as_mut().unwrap().spawn() {
            Ok(child) => {
                sleep(Duration::from_secs(10));
                self.child = Some(child);
                self.status = ModelStatus::Running;
                self.last_used = Utc::now();
                Ok(())
            }
            Err(e) => {
                self.status = ModelStatus::Error(e.to_string());
                Err(ModelError::ProcessError(e.to_string()))
            }
        }
    }
    pub async fn stop(&mut self) -> ModelResult<()> {
        if let Some(mut child) = self.child.take() {
            let pid = child.id();

            // Try graceful shutdown first
            if let Err(e) = child.kill() {
                error!("Failed to kill process gracefully: {}", e);
                // Force kill as backup
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
            sleep(Duration::from_secs(5));
            // Wait for process to exit with timeout
            let _ = tokio::time::timeout(std::time::Duration::from_secs(1), async {
                let mut child = child;
                let _ = child.wait();
            }).await;

            // Force kill again if still running
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }

        self.status = ModelStatus::Stopped;
        self.child = None;

        Ok(())
    }
}
