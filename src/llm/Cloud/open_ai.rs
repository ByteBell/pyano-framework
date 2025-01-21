// ToDo Set up a client as a llm object which can go inside Agentbuilder

use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
        CreateChatCompletionRequest,
    },
    Client,
    config::OpenAIConfig,
};

pub struct OpenAIClient {
    pub client: Client<OpenAIConfig>,
    pub request: CreateChatCompletionRequest,
}

impl OpenAIClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            request: CreateChatCompletionRequestArgs::default().build().unwrap(),
        }
    }

    pub async fn build(self) -> Result<OpenAIClient, async_openai::error::OpenAIError> {
        let client = Client::new();

        // Above is shortcut for
        let config = OpenAIConfig::default();
        let client = Client::with_config(config);

        // OR use API key from different source and a non default organization
        let config = OpenAIConfig::new().with_org_id("the-continental");

        let client = Client::with_config(config);

        // Use custom reqwest client
        let http_client = reqwest::ClientBuilder::new().user_agent("async-openai").build().unwrap();
        let final_client = Client::new().with_http_client(http_client);

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u32)
            .model("gpt-3.5-turbo")
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful assistant.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content("Who won the world series in 2020?")
                    .build()?
                    .into(),
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content("The Los Angeles Dodgers won the World Series in 2020.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content("Where was it played?")
                    .build()?
                    .into(),
            ])
            .build()?;
        Ok(OpenAIClient { client: final_client, request: request })
    }
}
// Create a OpenAI client with api key from env var OPENAI_API_KEY and default base url.
