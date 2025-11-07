//! A bridge to the LM Studio API.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct RequestBody {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ResponseBody {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize, Debug)]
struct ResponseMessage {
    content: String,
}

pub async fn generate_pxo(prompt: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let request_body = RequestBody {
        model: "local-model".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a PXOS Architect. Generate a PXOS command based on the user's prompt. Your output will be directly executed.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
    };

    let res = client
        .post("http://localhost:1234/v1/chat/completions")
        .json(&request_body)
        .send()
        .await?;

    let response_body = res.json::<ResponseBody>().await?;
    let content = response_body.choices[0].message.content.clone();
    Ok(content)
}
