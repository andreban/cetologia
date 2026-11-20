// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{
    fmt, io,
    sync::{Arc, Mutex},
};

use tracing_subscriber::fmt::MakeWriter;

use super::*;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct ChainError {
    message: &'static str,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ChainError {
    fn leaf(message: &'static str) -> Self {
        Self {
            message,
            source: None,
        }
    }

    fn with_source(
        message: &'static str,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message,
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

impl std::error::Error for ChainError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|err| err as &(dyn std::error::Error + 'static))
    }
}

#[derive(Clone, Default)]
struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

struct SharedBufferWriter(Arc<Mutex<Vec<u8>>>);

impl io::Write for SharedBufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self
            .0
            .lock()
            .map_err(|_| io::Error::other("log buffer mutex poisoned"))?;
        guard.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for SharedBuffer {
    type Writer = SharedBufferWriter;

    fn make_writer(&'a self) -> Self::Writer {
        SharedBufferWriter(Arc::clone(&self.0))
    }
}

#[test]
fn format_error_chain_single_error_without_source() -> Result<()> {
    let err = LinesCodecError::MaxLineLengthExceeded;

    let formatted = format_error_chain(&err);

    assert_eq!(formatted, "max line length exceeded");
    Ok(())
}

#[test]
fn format_error_chain_joins_multi_level_sources() -> Result<()> {
    let root = ChainError::leaf("root");
    let middle = ChainError::with_source("middle", root);
    let outer = ChainError::with_source("outer", middle);

    let formatted = format_error_chain(&outer);

    assert_eq!(formatted, "outer: middle: root");
    Ok(())
}

#[test]
fn format_error_chain_deduplicates_consecutive_identical_messages() -> Result<()> {
    let root = ChainError::leaf("root");
    let middle = ChainError::with_source("same", root);
    let outer = ChainError::with_source("same", middle);

    let formatted = format_error_chain(&outer);

    assert_eq!(formatted, "same: root");
    Ok(())
}

#[test]
fn format_lines_codec_error_unwraps_io_error_source_chain() -> Result<()> {
    let root = ChainError::leaf("connection reset by peer");
    let middle = ChainError::with_source("unexpected EOF during chunk size line", root);
    let outer = ChainError::with_source("error decoding response body", middle);
    let io_err = LinesCodecError::Io(io::Error::other(outer));

    assert_eq!(
        format_lines_codec_error(&io_err),
        "error decoding response body: unexpected EOF during chunk size line: connection reset by peer"
    );
    assert_eq!(
        format_lines_codec_error(&LinesCodecError::MaxLineLengthExceeded),
        "max line length exceeded"
    );
    Ok(())
}

#[test]
fn parse_sse_event_logs_warn_and_returns_full_error_chain() -> Result<()> {
    let buffer = SharedBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_ansi(false)
        .with_writer(buffer.clone())
        .finish();

    let root = ChainError::leaf("connection reset by peer");
    let middle = ChainError::with_source("unexpected EOF during chunk size line", root);
    let outer = ChainError::with_source("error decoding response body", middle);
    let err = LinesCodecError::Io(io::Error::other(outer));

    let parsed = tracing::subscriber::with_default(subscriber, || {
        CetologiaClient::parse_sse_event(Err(err))
    });

    let Some(Err(client_err)) = parsed else {
        return Err("expected Some(Err(_)) from parse_sse_event".into());
    };
    assert!(client_err.to_string().starts_with("Stream error: "));
    let CetologiaError::Stream(msg) = client_err else {
        return Err(format!("expected CetologiaError::Stream, got {client_err:?}").into());
    };
    assert_eq!(
        msg,
        "error decoding response body: unexpected EOF during chunk size line: connection reset by peer"
    );

    let bytes = buffer
        .0
        .lock()
        .map_err(|_| io::Error::other("log buffer mutex poisoned"))?
        .clone();
    let logs = String::from_utf8(bytes)?;
    assert!(logs.contains("WARN"), "expected WARN level in logs: {logs}");
    assert!(
        logs.contains("SSE stream error"),
        "expected SSE stream error message in logs: {logs}"
    );
    assert!(
        logs.contains("error decoding response body"),
        "expected outer error in logs: {logs}"
    );
    assert!(
        logs.contains("unexpected EOF during chunk size line"),
        "expected intermediate cause in logs: {logs}"
    );
    assert!(
        logs.contains("connection reset by peer"),
        "expected root cause in logs: {logs}"
    );

    Ok(())
}
