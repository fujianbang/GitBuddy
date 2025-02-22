use crate::config::ModelParameters;
use crate::llm::LLMResult;
use crate::prompt::Prompt;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct OpenAICompatible {
    pub(crate) url: String,
    pub(crate) model: String,
    pub(crate) prompt: String,
    pub(crate) api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String, // the model used to generate this completion
    object: String,
    system_fingerprint: String, // This fingerprint represents the backend configuration that the model runs with.
    choices: Vec<OpenAIResponseChoice>,
    usage: OpenAIResponseUsage, // the usage information of the request
    created: i64, // the Unix timestamp when the request was created
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoice {
    index: i64, // the index of the choice in the choices list
    message: OpenAIResponseChoiceMessage,
    finish_reason: String, // the reason why the model stopped generating tokens
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoiceMessage {
    role: String, // the role of the message
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
}

impl OpenAICompatible {
    pub(crate) fn request(&self, diff_content: &str, option: ModelParameters) -> Result<LLMResult> {
        let client = reqwest::blocking::Client::new();

        let api_key = self.api_key.clone();

        let response = client
            .post(format!("{}/v1/chat/completions", self.url))
            .timeout(Duration::from_secs(120))
            .header("Authorization", format!("Bearer {api_key}",))
            .json(&json!({
                "model": &self.model,
                "messages": [
                    {
                        "role": "system",
                        "content": self.prompt,
                    },
                    {
                        "role": "user",
                        "content": format!("diff content: \n{diff_content}"),
                    }
                ],
                "options": {
                    "temperature": option.temperature,
                    "top_p": option.top_p,
                    "top_k": option.top_k,
                },
                "options": option,
                "keep_alive": "30m",
                "max_tokens": option.max_tokens,
                // "format": {
                //     "type": "object",
                //     "properties": {"subject": {"type":"string"}, "scope": {"type":"string"}, "summary": {"type":"string"}},
                //     "required": ["subject", "scope", "summary"]
                // },
            }))
            .send()
            .expect("Error sending request");

        return if response.status().is_success() {
            let _response_json = OpenAIResponse {
                id: "".to_string(),
                model: "".to_string(),
                object: "".to_string(),
                system_fingerprint: "".to_string(),
                choices: vec![],
                usage: OpenAIResponseUsage {
                    completion_tokens: 0,
                    prompt_tokens: 0,
                    total_tokens: 0,
                },
                created: 0,
            };
            let response_json: OpenAIResponse = response.json().expect("Failed to parse response as JSON");

            if response_json.choices.is_empty() {
                panic!("No choices returned from OpenAI API");
            }
            let choice = &response_json.choices[0];
            Ok(LLMResult {
                commit_message: choice.message.content.clone().trim().to_string(),
                total_tokens: response_json.usage.total_tokens,
                prompt_tokens: response_json.usage.prompt_tokens,
                completion_tokens: response_json.usage.completion_tokens,
            })
        } else {
            let reason = match response.text() {
                Ok(text) => text,
                Err(e) => {
                    return Err(anyhow!("Error: {:?}", e.to_string().truncate(100)));
                }
            };
            return Err(anyhow!("Error: {}", reason));
        };
    }
}
