//! OpenAI chat-completions adapter — serves OpenAI, DeepSeek, Kimi, GLM, and
//! Gemini (via its OpenAI-compatible endpoint). One adapter, five providers;
//! only the base URL differs (carried in [`ProviderKind`]).
//!
//! Wire shape: `POST {base}/chat/completions` with `Authorization: Bearer`,
//! all roles (including system) in `messages`, and SSE `data:` lines carrying
//! `choices[0].delta.content`, terminated by `data: [DONE]`.

use serde_json::{Value, json};
use tracing::{debug, trace};

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

    pub(crate) fn body(kind: ProviderKind, req: &ChatRequest) -> Value {
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

        let mut body = json!({
            "model": req.model,
            "messages": messages,
            "stream": true,
        });

        // Output-length cap: OpenAI has deprecated `max_tokens` in favour of
        // `max_completion_tokens` (o-series/GPT-5 reject `max_tokens` outright).
        // `max_completion_tokens` is accepted by every current OpenAI model, so use
        // it there. The other compat providers still expect `max_tokens`, so this is
        // gated per provider rather than renamed globally.
        match kind {
            ProviderKind::OpenAi => body["max_completion_tokens"] = json!(req.max_tokens),
            _ => body["max_tokens"] = json!(req.max_tokens),
        }

        // Several compat providers (DeepSeek V4, GLM auto-deciders, Kimi K2.6) default
        // thinking ON, toggled by a `thinking:{type:"disabled"}` body param they share.
        // The studio never surfaces a chain-of-thought and it would burn the token
        // budget, so force it off — but only for models flagged as safe to disable
        // (per-model, since some reason compulsorily and would reject the param).
        if crate::ModelInfo::lookup(kind, &req.model).is_some_and(|m| m.disable_thinking) {
            body["thinking"] = json!({ "type": "disabled" });
        }

        body
    }
}

impl Provider for OpenAiCompatAdapter {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    fn stream_chat(&self, req: ChatRequest) -> async_channel::Receiver<ChatDelta> {
        let url = format!("{}/chat/completions", self.kind.resolved_base_url());
        let body = Self::body(self.kind, &req);
        let token_field = if body.get("max_completion_tokens").is_some() {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
        let thinking_disabled = body.get("thinking").is_some();
        debug!(
            url = %url,
            provider = ?self.kind,
            model = %req.model,
            token_field,
            thinking_disabled,
            "openai-compat stream_chat request"
        );
        let prompt = crate::render_prompt(&req.messages);
        debug!(len = prompt.len(), preview = %crate::preview(&prompt, 400), "openai-compat prompt");
        trace!(prompt = %prompt, "openai-compat full prompt");

        let request = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body);
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
            model: "deepseek-v4-flash".into(),
            max_tokens: 512,
            messages: vec![ChatMessage::system("card"), ChatMessage::user("hello")],
        };
        let body = OpenAiCompatAdapter::body(ProviderKind::DeepSeek, &req);
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn token_field_is_gated_per_provider() {
        let mk = |model: &str| ChatRequest {
            model: model.into(),
            max_tokens: 512,
            messages: vec![ChatMessage::user("hi")],
        };
        // OpenAI: the deprecated max_tokens is replaced by max_completion_tokens;
        // gpt-5.6 reasons off by default, so no thinking param.
        let openai = OpenAiCompatAdapter::body(ProviderKind::OpenAi, &mk("gpt-5.6-terra"));
        assert_eq!(openai["max_completion_tokens"], 512);
        assert!(openai.get("max_tokens").is_none());
        assert!(openai.get("thinking").is_none());

        // DeepSeek V4 flash: keeps max_tokens and forces thinking off (defaults on).
        let deepseek = OpenAiCompatAdapter::body(ProviderKind::DeepSeek, &mk("deepseek-v4-flash"));
        assert_eq!(deepseek["max_tokens"], 512);
        assert!(deepseek.get("max_completion_tokens").is_none());
        assert_eq!(deepseek["thinking"]["type"], "disabled");

        // Kimi K3 reasons compulsorily → no thinking param (it would reject it).
        let kimi = OpenAiCompatAdapter::body(ProviderKind::Kimi, &mk("kimi-k3"));
        assert!(kimi.get("thinking").is_none());

        // Gemini: plain max_tokens, thinking controlled elsewhere → no param.
        let gemini = OpenAiCompatAdapter::body(ProviderKind::Gemini, &mk("gemini-3.6-flash"));
        assert_eq!(gemini["max_tokens"], 512);
        assert!(gemini.get("thinking").is_none());
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
