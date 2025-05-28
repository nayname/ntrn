use rig::pipeline::{self, Op};
use rig::providers::openai;
use serde_json::Value;

#[tokio::main]
pub async fn ask_gpt(messages: &Value) -> anyhow::Result<String>  {
    // Requires `OPENAI_API_KEY` in the environment
    let openai_client = openai::Client::from_env();

    // GPT-4o agent (tweak temperature / system prompt here if you like)
    let agent = openai_client.agent("gpt-4o").build();

    // One-step pipeline → feed the input string to the agent
    let chain = pipeline::new().prompt(agent);

    // Run the pipeline; ? converts `PromptError` → `anyhow::Error`
    let response = chain.call(messages.to_string()).await?;

    Ok(response)
}


/// Build a single-message context for the LLM
pub fn get_context(query: &str, context: &str) -> Value {
    serde_json::json!([{ "role": "user",
        "content": format!("Context: {context}\n\nUser: {query}") }])
}
