//! OpenAI chat-completions adapter — serves OpenAI, DeepSeek, Kimi, GLM, and
//! Gemini (via its OpenAI-compatible endpoint). One adapter, five providers;
//! only the base URL differs (carried in [`ProviderKind`]).
//!
//! Wire shape: `POST {base}/chat/completions` with `Authorization: Bearer`,
//! all roles (including system) in `messages`, and SSE `data:` lines carrying
//! `choices[0].delta.content`, terminated by `data: [DONE]`.

use serde_json::{Value, json};

use crate::{ChatDelta, ChatRequest, Provider, ProviderKind, Role, stream};

pub struct OpenAiCompatAdapter {
    client: reqwest::Client,
    kind: ProviderKind,
    api_key: String,
}

impl OpenAiCompatAdapter {
    pub fn new(kind: ProviderKind, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            kind,
            api_key,
        }
    }

    pub(crate) fn body(req: &ChatRequest) -> Value {
        let messages: Vec<Value> = req
            .messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                json!({ "role": role, "content": m.content })
            })
            .collect();

        json!({
            "model": req.model,
            "messages": messages,
            "max_tokens": req.max_tokens,
            "stream": true,
        })
    }
}

impl Provider for OpenAiCompatAdapter {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    fn stream_chat(&self, req: ChatRequest) -> async_channel::Receiver<ChatDelta> {
        let request = self
            .client
            .post(format!("{}/chat/completions", self.kind.base_url()))
            .bearer_auth(&self.api_key)
            .json(&Self::body(&req));
        stream::run_stream(request, interpret)
    }
}

/// Map one chat-completions SSE payload to deltas. `[DONE]` closes the stream;
/// otherwise pull the incremental `choices[0].delta.content`.
fn interpret(data: &str) -> Vec<ChatDelta> {
    if data == "[DONE]" {
        return vec![ChatDelta::Done];
    }
    let Ok(v) = serde_json::from_str::<Value>(data) else {
        return vec![];
    };
    v.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("delta"))
        .and_then(|d| d.get("content"))
        .and_then(Value::as_str)
        .filter(|t| !t.is_empty())
        .map(|t| vec![ChatDelta::Text(t.to_string())])
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChatMessage;

    #[test]
    fn keeps_system_role_in_messages() {
        let req = ChatRequest {
            model: "deepseek-chat".into(),
            max_tokens: 512,
            messages: vec![ChatMessage::system("card"), ChatMessage::user("hello")],
        };
        let body = OpenAiCompatAdapter::body(&req);
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn interprets_content_delta_and_done() {
        let d = interpret(r#"{"choices":[{"delta":{"content":"Hi"}}]}"#);
        assert_eq!(d, vec![ChatDelta::Text("Hi".into())]);
        assert_eq!(interpret("[DONE]"), vec![ChatDelta::Done]);
        // Role-only opening delta carries no content.
        assert!(interpret(r#"{"choices":[{"delta":{"role":"assistant"}}]}"#).is_empty());
    }
}
