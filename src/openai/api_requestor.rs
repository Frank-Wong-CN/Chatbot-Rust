use reqwest;

use super::error::*;
use super::types::*;

pub async fn get_response(
    context: &Vec<Message>,
    api_key: &str,
) -> Result<OpenAIResponse, String> {
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let request = CompletionRequest {
        model: "gpt-3.5-turbo".into(),
        messages: context.clone().to_vec(),
    };

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await;

    return match response {
        Ok(success) => match success.text().await {
            Ok(body) => match serde_json::from_str::<CompletionResponse>(&body) {
				Ok(completion) => Ok(OpenAIResponse::Success(completion)),
				Err(json_error) => match serde_json::from_str::<OpenAIError>(&body) {
					Ok(openai_response) => Ok(OpenAIResponse::Failure(openai_response)),
					Err(_) => Err(JSONParseError::new(
						json_error.to_string(),
						body,
					)
					.to_string())
				}
			},
            Err(request_error) => Err(RequestError::new(request_error).to_string())
        },
        Err(request_error) => Err(RequestError::new(request_error).to_string()),
    };
}
