use serde::Deserialize;

use crate::config::Config;

#[derive(Deserialize, Debug)]
pub struct LLMResponse {
    pub text: String,
    pub vote: Option<u32>,
}

pub async fn call_llm(config: &Config, system_prompt: &str, prompt: &str) -> LLMResponse {
    let api_key = &config.groq_api_key;

    let client = reqwest::Client::new();

    println!("[DEBUG] System prompt: {}", system_prompt);
    println!("[DEBUG] Prompt: {}", prompt);

    let res = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "meta-llama/llama-4-scout-17b-16e-instruct",
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "Response",
                    "schema": {
                        "properties": {
                            "response": {
                                "type": "object",
                                "properties": {
                                    "text": {
                                        "type": "string",
                                        "description": "Your text response that will be made public to the other players"
                                    },
                                    "vote": {
                                        "type": "number",
                                        "description": "Your vote (player number). Never vote for yourself. Only vote when instructed to. When instructed to vote, you must vote."
                                    }
                                },
                                "required": ["text"]
                            }
                        },
                        "required": ["response"],
                        "type": "object"
                    }
                }
            }
        }))
        .send()
        .await
        .unwrap();
    let json: serde_json::Value = res.json().await.unwrap();
    let response_json = serde_json::from_str::<serde_json::Value>(
        json["choices"][0]["message"]["content"].as_str().unwrap(),
    )
    .unwrap();
    let response = response_json["response"].to_string();
    println!("\n[DEBUG] Response:\n{}", &response);
    return serde_json::from_str::<LLMResponse>(&response).unwrap();
}

pub fn build_prompt<T: AsRef<str>>(log: T, current: T) -> String {
    return format!(
        "{{\"chat_log\": \"{}\", \"current_prompt\":\"{}\"}}",
        log.as_ref(),
        current.as_ref()
    );
}
