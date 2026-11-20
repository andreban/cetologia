// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use cetologia::prelude::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
};
use tokio_stream::StreamExt;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
async fn chat_stream_truncated_chunked_response_preserves_error_source_chain() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let (abort_tx, abort_rx) = oneshot::channel::<()>();

    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await?;
        let mut req_buf = Vec::new();
        let mut chunk = [0u8; 1024];

        let header_end = loop {
            let n = socket.read(&mut chunk).await?;
            if n == 0 {
                return Err("client closed connection before completing HTTP headers".into());
            }
            req_buf.extend_from_slice(&chunk[..n]);
            if let Some(pos) = req_buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos + 4;
            }
        };

        let headers_str = std::str::from_utf8(&req_buf[..header_end])?;
        let content_length = headers_str
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.trim()
                    .eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>())
            })
            .transpose()?
            .unwrap_or(0);

        let mut remaining = content_length.saturating_sub(req_buf.len() - header_end);
        while remaining > 0 {
            let n = socket.read(&mut chunk).await?;
            if n == 0 {
                return Err("client closed connection before completing HTTP body".into());
            }
            remaining = remaining.saturating_sub(n);
        }

        let headers = concat!(
            "HTTP/1.1 200 OK\r\n",
            "Content-Type: text/event-stream\r\n",
            "Transfer-Encoding: chunked\r\n",
            "\r\n"
        );
        let sse_event = "data: {\"id\":\"chunk-1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"deepseek-chat\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n";
        let first_chunk = format!("{:x}\r\n{sse_event}\r\n", sse_event.len());

        socket.write_all(headers.as_bytes()).await?;
        socket.write_all(first_chunk.as_bytes()).await?;
        socket.flush().await?;

        let _ = abort_rx.await;

        socket.write_all(b"ff\r\ndata: {\"id\":\"chunk-2\"").await?;
        socket.flush().await?;
        drop(socket);

        Result::Ok(())
    });

    let http_client = reqwest::Client::builder().no_proxy().build()?;
    let client = CetologiaClient::builder("test-key")
        .base_url(format!("http://{addr}"))
        .client(http_client)
        .build();
    let request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(vec![ChatMessage::user("Hello")])
        .build();

    let stream =
        tokio::time::timeout(Duration::from_secs(5), client.chat_stream(&request)).await??;
    let mut stream = std::pin::pin!(stream);

    let first = tokio::time::timeout(Duration::from_secs(5), stream.next())
        .await?
        .ok_or("stream ended before emitting first chunk")??;
    assert_eq!(
        first
            .choices
            .first()
            .and_then(|c| c.delta.content.as_deref()),
        Some("Hello")
    );

    abort_tx
        .send(())
        .map_err(|_| "mock server stopped before abort signal")?;

    let next = tokio::time::timeout(Duration::from_secs(5), stream.next())
        .await?
        .ok_or("stream ended cleanly instead of returning a mid-stream error")?;
    let Err(err) = next else {
        return Err("expected mid-stream error after truncated chunk".into());
    };

    server.await??;

    assert!(err.to_string().starts_with("Stream error: "));
    let CetologiaError::Stream(msg) = err else {
        return Err(format!("expected CetologiaError::Stream, got {err:?}").into());
    };
    let rest = msg
        .strip_prefix("error decoding response body: ")
        .ok_or_else(|| {
            format!("expected 'error decoding response body: <cause>' prefix, got: {msg}")
        })?;
    assert!(
        !rest.is_empty(),
        "expected non-empty underlying error cause after prefix in: {msg}"
    );
    assert!(
        !rest.starts_with("error decoding response body"),
        "expected deduplicated error chain without repeated prefix in: {msg}"
    );

    Ok(())
}
