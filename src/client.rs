use crate::models::{LogitBias, Model, Role};
use log::debug;
use reqwest::{header::HeaderMap, Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main ChatGPTClient struct.
pub struct ChatGPTClient {
    base_url: String,
    api_key: String,
    client: Client,
}

/// Represents the input for the chat API call.
#[derive(Debug, Serialize)]
pub struct ChatInput {
    pub model: Model,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<LogitBias>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl Default for ChatInput {
    fn default() -> Self {
        Self {
            model: Model::Gpt_4,  // Set the default model
            messages: Vec::new(), // Set an empty vector for messages
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        }
    }
}

/// Represents the response from the chat API call.
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub usage: Usage,
    pub choices: Vec<Choice>,
}

/// Represents the usage information in the chat API response.
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

/// Represents a choice in the chat API response.
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: String,
}

/// Represents a message in the chat API call.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Enum representing possible errors in the ChatGPTClient.
#[derive(Error, Debug)]
pub enum ChatGPTError {
    #[error("Request failed with status code: {status_code}\nHeaders: {headers:?}\nBody: {body}")]
    RequestFailed {
        status_code: StatusCode,
        headers: HeaderMap,
        body: String,
    },
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl ChatGPTClient {
    /// Creates a new ChatGPTClient with the given API key and base URL.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for the ChatGPT API.
    /// * `base_url` - The base URL for the ChatGPT API.
    pub fn new(api_key: &str, base_url: &str) -> Self {
        let client = Client::builder()
            .use_rustls_tls()
            .build()
            .expect("New client");

        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            client,
        }
    }

    /// Sends a request to the ChatGPT API with the given input and returns the response.
    ///
    /// # Arguments
    ///
    /// * `input` - A ChatInput struct representing the input for the chat API call.
    ///
    /// # Examples
    ///
    /// ```
    /// use chat_gpt_lib_rs::{ChatGPTClient, ChatInput, Message, Model, Role};
    ///
    /// async fn example() {
    ///     let chat_gpt = ChatGPTClient::new("your_api_key", "https://api.openai.com");
    ///     let input = ChatInput {
    ///         model: Model::Gpt_4,
    ///         messages: vec![
    ///             Message {
    ///                 role: Role::System,
    ///                 content: "You are a helpful assistant.".to_string(),
    ///             },
    ///             Message {
    ///                 role: Role::User,
    ///                 content: "Who is the best field hockey player in the world".to_string(),
    ///             },
    ///         ],
    ///         ..Default::default()
    ///     };
    ///
    ///     let response = chat_gpt.chat(input).await.unwrap();
    /// }
    /// ```
    /// # Errors
    ///
    /// Returns a ChatGPTError if the request fails.
    pub async fn chat(&self, input: ChatInput) -> Result<ChatResponse, ChatGPTError> {
        let base_url = self.base_url.trim_end_matches('/').to_string();
        let url = format!("{}/v1/chat/completions", base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&input)
            .send()
            .await?;

        debug!(
            "API call to url: {}\n with json payload: {:?}",
            &url, &input
        );

        // Check if the status code is 200
        if response.status() == StatusCode::OK {
            response
                .json::<ChatResponse>()
                .await
                .map_err(ChatGPTError::from)
        } else {
            let status_code = response.status();
            let headers = response.headers().clone();
            let body = response.text().await?;
            Err(ChatGPTError::RequestFailed {
                status_code,
                headers,
                body,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a ChatGPTClient instance with a dummy API key and base URL
    fn create_dummy_client() -> ChatGPTClient {
        ChatGPTClient::new("dummy_api_key", "https://dummy-api-url.com")
    }

    #[tokio::test]
    async fn test_chat_gpt_client_new() {
        let client = create_dummy_client();
        assert_eq!(client.api_key, "dummy_api_key");
        assert_eq!(client.base_url, "https://dummy-api-url.com");
    }

    #[tokio::test]
    async fn test_chat_gpt_client_chat() {
        // Please note that this test will not actually make an API call to OpenAI,
        // but it will test the error handling of the `chat` function.
        let client = create_dummy_client();

        let input = ChatInput {
            model: Model::Gpt_4,
            messages: vec![
                Message {
                    role: Role::System,
                    content: "You are a helpful assistant.".to_string(),
                },
                Message {
                    role: Role::User,
                    content: "Who is the best field hockey player in the world?".to_string(),
                },
            ],
            ..Default::default()
        };

        let result = client.chat(input).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_usage_struct() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };

        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 20);
        assert_eq!(usage.total_tokens, 30);
    }

    #[test]
    fn test_choice_struct() {
        let choice = Choice {
            message: Message {
                role: Role::Assistant,
                content: "Sample response".to_string(),
            },
            finish_reason: "stop".to_string(),
        };

        assert_eq!(choice.message.role, Role::Assistant);
        assert_eq!(choice.message.content, "Sample response");
        assert_eq!(choice.finish_reason, "stop");
    }
}
