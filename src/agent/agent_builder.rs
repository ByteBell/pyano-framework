use crate::llm::llm_builder::LLM;
use super::agent::Agent;
use crate::tools::Tool;
use std::sync::Arc;
use crate::llm::Cloud::cloud_llm::cloudLLM; // Import cloud_llm
pub struct AgentBuilder {
    system_prompt: Option<String>,
    user_prompt: Option<String>,
    stream: Option<bool>,
    llm: Option<LLM>,
    cloud_llm: Option<cloudLLM>, // Add a field for cloud LLM, which will only contain client and AgentBuilder Object should either have a llm or a cloud llm
    name: Option<String>,
    tools: Option<Vec<Arc<dyn Tool>>>, // New field for tools
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            system_prompt: None,
            user_prompt: None,
            stream: Some(false),
            llm: None,
            cloud_llm: None, // Initialize cloud_llm as None
            name: None,
            tools: None, // Initialize tools as None
        }
    }

    pub fn with_system_prompt(mut self, system_prompt: String) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    pub fn with_user_prompt(mut self, user_prompt: String) -> Self {
        self.user_prompt = Some(user_prompt);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn with_llm(mut self, llm: LLM) -> Self {
        self.llm = Some(llm);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_tools(mut self, tools: Vec<Arc<dyn Tool>>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn build(self) -> Agent {
        if self.llm.is_none() && self.cloud_llm.is_none() {
            panic!("Both LLM and cloudLLM cannot be none; one of the values must be provided");
        }
        if self.user_prompt.is_none() {
            panic!("User prompt must be provided before building the Agent");
        }
        if self.system_prompt.is_none() {
            panic!("System prompt must be provided before building the Agent");
        }

        Agent {
            system_prompt: self.system_prompt,
            user_prompt: self.user_prompt,
            stream: self.stream,
            llm: self.llm,
            name: self.name,
            tools: self.tools, // Set tools field
        }
    }
}
