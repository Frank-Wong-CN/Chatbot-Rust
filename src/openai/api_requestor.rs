use super::types::*;
use reqwest;
use serde_json::json;

pub async fn get_response(
    context: Vec<Message>,
    api_key: &str,
) -> Result<CompletionResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let request = CompletionRequest {
        model: "gpt-3.5-turbo".into(),
        messages: context,
    };

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await;

    return match response {
        Ok(success) => Ok(success.json::<CompletionResponse>().await?),
        Err(failed) => Ok(CompletionResponse {
            id: String::new(),
            object: String::new(),
            created: 0,
            model: String::new(),
            usage: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            choices: vec![ResponseChoice {
                index: 0,
                finish_reason: String::new(),
                message: Message {
                    role: MessageRole::System,
                    content: failed.to_string(),
                },
            }],
        }),
    };
}
