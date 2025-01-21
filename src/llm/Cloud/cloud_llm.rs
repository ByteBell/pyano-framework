use super::open_ai::OpenAIClient;

pub struct cloudLLM {
    pub client: OpenAIClient,
}
impl cloudLLM {
    pub fn new() -> Self {
        let client = OpenAIClient::new();
        Self { client }
    }
}
