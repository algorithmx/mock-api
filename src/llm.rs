pub mod query;
use serde::Deserialize;

use crate::llm::query::query_llm;

#[derive(Deserialize)]
struct LLMRequestBody {
    api_url: String,
    api_model: String,
    api_key: String,
    prompt: String,
}

pub fn compose_config(
    request_body: &str
) -> Result<String, reqwest::Error> {
    let body: LLMRequestBody = serde_json::from_str(request_body).unwrap();
    let system_prompt = "You are a outstanding API designer and backend engineer. \
    Your task is to help the user to write a json configuration file for the API mock server. \
    The configuration file should be in the format of the `ProjectConfig` struct in the `schema.rs` file. \
    You should consider the provided example. \
    Your reply must contains only the configuration as valid json, no extra words or comments.
    Special attention to match the top-level keys to the provided example.
    ";
    
    let schema_content = include_str!("./schema.rs");
    let example_content = include_str!("./llm/example_prompt.txt");
    
    let assistant_prompt = format!(
        "## `schema.rs` file content\n\n{}\n\n{}",
        schema_content,
        example_content
    );

    // block the thread until the llm response is ready
    let result = tokio::runtime::Runtime::new().unwrap().block_on(
        query_llm(
            &body.api_url, 
            &body.api_model, 
            &body.api_key, 
            system_prompt, 
            &assistant_prompt, 
            &body.prompt
        )
    );
    remove_quotes_if_exists(result)
}


fn remove_quotes_if_exists(s: Result<String, reqwest::Error>) -> Result<String, reqwest::Error> {
    match s {
        Ok(result) => {
            let mut result1 = &result[..];
            result1 = result1.strip_prefix("```").unwrap_or(result1);
            result1 = result1.strip_suffix("```").unwrap_or(result1);
            result1 = result1.strip_prefix("json").unwrap_or(result1);
            result1 = result1.strip_suffix('\n').unwrap_or(result1);
            result1 = result1.strip_prefix('\n').unwrap_or(result1);
            Ok(result1.to_string())
        }
        Err(e) => Err(e),
    }
}

