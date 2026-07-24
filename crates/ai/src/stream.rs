//! Drives one streaming HTTP request to completion on a dedicated thread.
//!
//! `stream_chat` must return synchronously (the studio wires the receiver into
//! a gpui task), and reqwest is async — so we spawn a thread with a
//! current-thread tokio runtime, mirroring the preview crate's detached-window
//! pattern. Deltas are forwarded over an unbounded channel; the caller's
//! executor never blocks on ours.

use tracing::{debug, error, info};

use crate::{ChatDelta, sse::SseDecoder};

/// Interpret one SSE `data:` payload into zero or more deltas. Stateless per
/// provider, so a plain fn pointer suffices.
pub(crate) type Interpret = fn(&str) -> Vec<ChatDelta>;

pub(crate) fn run_stream(
    request: reqwest::RequestBuilder,
    interpret: Interpret,
) -> async_channel::Receiver<ChatDelta> {
    let (tx, rx) = async_channel::unbounded::<ChatDelta>();

    info!("dispatching streaming chat request");

    let spawned = std::thread::Builder::new()
        .name("wf-ai-stream".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send_blocking(ChatDelta::Error(format!("runtime init failed: {e}")));
                    return;
                }
            };
            rt.block_on(drive(request, interpret, tx));
        });

    if spawned.is_err() {
        // Extremely unlikely (OS thread exhaustion); surface it rather than
        // returning a channel that never yields.
        let _ = rx; // keep the receiver end alive for the caller
        let (etx, erx) = async_channel::unbounded();
        let _ = etx.send_blocking(ChatDelta::Error("failed to spawn stream thread".into()));
        return erx;
    }
    rx
}

async fn drive(
    request: reqwest::RequestBuilder,
    interpret: Interpret,
    tx: async_channel::Sender<ChatDelta>,
) {
    use futures_util::StreamExt;

    let resp = match request.send().await {
        Ok(r) => r,
        Err(e) => {
            error!(error = %e, "request dispatch failed");
            let _ = tx
                .send(ChatDelta::Error(format!("request failed: {e}")))
                .await;
            return;
        }
    };

    debug!(status = %resp.status(), "received HTTP response");

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let detail = body.trim();
        error!(status = %status, body = %crate::preview(detail, 400), "non-success HTTP status");
        let _ = tx
            .send(ChatDelta::Error(format!("HTTP {status}: {detail}")))
            .await;
        return;
    }

    let mut stream = Box::pin(resp.bytes_stream());
    let mut decoder = SseDecoder::default();
    let mut done = false;
    let mut emitted = 0usize;

    'outer: while let Some(chunk) = stream.next().await {
        let bytes = match chunk {
            Ok(b) => b,
            Err(e) => {
                error!(error = %e, "byte-stream error");
                let _ = tx
                    .send(ChatDelta::Error(format!("stream error: {e}")))
                    .await;
                return;
            }
        };

        let mut deltas = Vec::new();
        decoder.push(bytes.as_ref(), |data| deltas.extend(interpret(data)));

        for delta in deltas {
            let terminal = matches!(delta, ChatDelta::Done | ChatDelta::Error(_));
            if tx.send(delta).await.is_err() {
                return; // receiver dropped — stop pulling from the network
            }
            emitted += 1;
            if terminal {
                done = true;
                break 'outer;
            }
        }
    }

    // Providers signal completion explicitly (message_stop / [DONE]); if the
    // connection ended without it, close the stream cleanly ourselves.
    if !done {
        let _ = tx.send(ChatDelta::Done).await;
    }

    debug!(deltas = emitted, terminal = done, "stream complete");
}
