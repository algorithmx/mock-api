use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub async fn query_llm(
    api_url: &str, 
    api_model: &str,
    api_key: &str, 
    system_prompt: &str,
    assistant_prompt: &str,
    user_prompt: &str
) -> Result<String, reqwest::Error> {
    let client = Client::new();
    
    let request = ChatRequest {
        model: api_model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: assistant_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            }
        ],
    };

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    Ok(response.choices[0].message.content.clone())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_query_llm() {
        let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY must be set");
        let api_url = "https://api.openai.com/v1/chat/completions";
        let api_model = "gpt-3.5-turbo";
        
        let result = query_llm(
            api_url,
            api_model,
            &api_key,
            "You are a helpful assistant.",
            "I am ready to help.",
            "Say 'Hello, World!'",
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
    }
}



