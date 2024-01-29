use crate::models::general::llm::{APIResponse, ChatCompletion, Message};
use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder, RequestBuilder, Response};
use std::env;

pub async fn call_gpt(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();

    let api_key: String = env::var("OPEN_AI_KEY").expect("Did not find the secret key.");
    let api_org: String = env::var("OPEN_AI_ORG").expect("Did not find the organisation key.");

    let url: &str = "https://api.openai.com/v1/chat/completions";

    let mut headers: HeaderMap = HeaderMap::new();

    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );

    headers.insert(
        "OpenAI-Organization",
        HeaderValue::from_str(api_org.as_str())
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );

    let client: Client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    let chat_completion: ChatCompletion = ChatCompletion {
        model: "gpt-4-turbo-preview".to_string(),
        messages,
        temperature: 0.1,
    };

    let res: APIResponse = client
        .post(url)
        .json(&chat_completion)
        .send()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?
        .json()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    Ok(res.choices[0].message.content.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test]
    // async fn tests_call_to_openai() {
    //     let message = Message {
    //         role: "user".to_string(),
    //         content: "This is a test, give me a short response.".to_string(),
    //     };

    //     let messages = vec![message];

    //     let res = call_gpt(messages).await;

    //     if let Ok(res_str) = res {
    //         dbg!(res_str);
    //         assert!(true)
    //     } else {
    //         assert!(false)
    //     }
    // }
}
