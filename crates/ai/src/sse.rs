//! Minimal Server-Sent Events framing, shared by both provider adapters.
//!
//! We only need the `data:` field — both the Anthropic Messages API and the
//! OpenAI chat-completions API carry their JSON payloads there. Bytes are
//! buffered and decoded a whole line at a time, so a multi-byte UTF-8 sequence
//! split across two network chunks is never decoded mid-character (a line is
//! always complete before we decode it, and SSE payload lines never contain a
//! raw newline).

#[derive(Default)]
pub struct SseDecoder {
    buf: Vec<u8>,
}

impl SseDecoder {
    /// Feed a network chunk; invoke `on_data` with the trimmed payload of each
    /// complete `data:` line the chunk completes.
    pub fn push(&mut self, chunk: &[u8], mut on_data: impl FnMut(&str)) {
        self.buf.extend_from_slice(chunk);
        while let Some(nl) = self.buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.buf.drain(..=nl).collect();
            let line = String::from_utf8_lossy(&line);
            let line = line.trim_end_matches(['\r', '\n']);
            if let Some(rest) = line.strip_prefix("data:") {
                on_data(rest.trim());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(chunks: &[&[u8]]) -> Vec<String> {
        let mut dec = SseDecoder::default();
        let mut out = Vec::new();
        for c in chunks {
            dec.push(c, |d| out.push(d.to_string()));
        }
        out
    }

    #[test]
    fn extracts_data_lines_and_ignores_event_lines() {
        let out = collect(&[b"event: delta\ndata: {\"a\":1}\n\ndata: [DONE]\n\n"]);
        assert_eq!(out, vec!["{\"a\":1}".to_string(), "[DONE]".to_string()]);
    }

    #[test]
    fn reassembles_a_payload_split_across_chunks() {
        // Split mid-line and mid-multibyte-char ("مرحبا" bytes cut in two).
        let full = "data: مرحبا\n".as_bytes();
        let (a, b) = full.split_at(9);
        assert_eq!(collect(&[a, b]), vec!["مرحبا".to_string()]);
    }
}
