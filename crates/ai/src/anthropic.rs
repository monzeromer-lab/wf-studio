//! Anthropic Messages API adapter (native protocol, raw HTTP).
//!
//! Wire shape per the Anthropic API reference: `POST /v1/messages` with
//! `x-api-key` + `anthropic-version: 2023-06-01`, the system prompt as a
//! **top-level** field (not a message), and SSE `content_block_delta` events
//! carrying `delta.text`. Rust has no official Anthropic SDK, so we speak HTTP
//! directly.

use serde_json::{Value, json};

use crate::{ChatDelta, ChatRequest, Provider, ProviderKind, Role, stream};

const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicAdapter {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    /// Build the request body. System messages are hoisted into the top-level
    /// `system` field (the Messages API rejects `role: "system"` entries);
    /// user/assistant turns stay in `messages`.
    pub(crate) fn body(req: &ChatRequest) -> Value {
        let mut system = String::new();
        let mut messages = Vec::new();
        for m in &req.messages {
            match m.role {
                Role::System => {
                    if !system.is_empty() {
                        system.push_str("\n\n");
                    }
                    system.push_str(&m.content);
                }
                Role::User => messages.push(json!({ "role": "user", "content": m.content })),
                Role::Assistant => {
                    messages.push(json!({ "role": "assistant", "content": m.content }))
                }
            }
        }

        let mut body = json!({
            "model": req.model,
            "max_tokens": req.max_tokens,
            "messages": messages,
            "stream": true,
        });
        if !system.is_empty() {
            body["system"] = Value::String(system);
        }
        body
    }
}

impl Provider for AnthropicAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    fn stream_chat(&self, req: ChatRequest) -> async_channel::Receiver<ChatDelta> {
        let request = self
            .client
            .post(format!("{}/messages", ProviderKind::Anthropic.base_url()))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&Self::body(&req));
        stream::run_stream(request, interpret)
    }
}

/// Map one Messages-API SSE payload to deltas. Text arrives on
/// `content_block_delta`; the stream ends on `message_stop`; server errors
/// arrive as an `error` event.
fn interpret(data: &str) -> Vec<ChatDelta> {
    let Ok(v) = serde_json::from_str::<Value>(data) else {
        return vec![];
    };
    match v.get("type").and_then(Value::as_str) {
        Some("content_block_delta") => v
            .get("delta")
            .and_then(|d| d.get("text"))
            .and_then(Value::as_str)
            .map(|t| vec![ChatDelta::Text(t.to_string())])
            .unwrap_or_default(),
        Some("message_stop") => vec![ChatDelta::Done],
        Some("error") => {
            let msg = v
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            vec![ChatDelta::Error(msg.to_string())]
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChatMessage;

    #[test]
    fn hoists_system_prompt_out_of_messages() {
        let req = ChatRequest {
            model: "claude-opus-4-8".into(),
            max_tokens: 1024,
            messages: vec![
                ChatMessage::system("You are WebFluent."),
                ChatMessage::user("مطعم صغير في القاهرة"),
            ],
        };
        let body = AnthropicAdapter::body(&req);
        assert_eq!(body["system"], "You are WebFluent.");
        assert_eq!(body["stream"], true);
        assert_eq!(body["messages"].as_array().unwrap().len(), 1);
        assert_eq!(body["messages"][0]["role"], "user");
        // Arabic survives JSON serialization.
        assert_eq!(body["messages"][0]["content"], "مطعم صغير في القاهرة");
    }

    #[test]
    fn interprets_text_delta_and_stop() {
        let d = interpret(
            r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"Hi"}}"#,
        );
        assert_eq!(d, vec![ChatDelta::Text("Hi".into())]);
        assert_eq!(
            interpret(r#"{"type":"message_stop"}"#),
            vec![ChatDelta::Done]
        );
        assert!(interpret(r#"{"type":"ping"}"#).is_empty());
    }
}
