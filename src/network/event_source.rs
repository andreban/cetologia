// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Server-Sent Events (SSE) decoder for streaming HTTP responses.

use reqwest::Response;
use std::mem;
use tokio_stream::{Stream, StreamExt};
use tokio_util::{
    codec::{Decoder, FramedRead, LinesCodec, LinesCodecError},
    io::StreamReader,
};
use tracing::warn;

static EVENT: &str = "event: ";
static DATA: &str = "data: ";
static ID: &str = "id: ";
static RETRY: &str = "retry: ";

/// Extension trait for converting an HTTP response into a stream of [`ServerSentEvent`]s.
pub trait EventSource {
    /// Consumes the response and returns a stream of parsed SSE events.
    fn event_stream(self) -> impl Stream<Item = std::result::Result<ServerSentEvent, LinesCodecError>>;
}

impl EventSource for Response {
    fn event_stream(self) -> impl Stream<Item = std::result::Result<ServerSentEvent, LinesCodecError>> {
        stream_response(self)
    }
}

/// A parsed Server-Sent Event.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ServerSentEvent {
    /// The event type (from the `event:` field).
    pub event: Option<String>,
    /// The event payload (from one or more `data:` fields, joined by `\n`).
    pub data: Option<String>,
    /// The event ID (from the `id:` field).
    pub id: Option<String>,
    /// The reconnection time in milliseconds (from the `retry:` field).
    pub retry: Option<usize>,
}

/// A [`Decoder`] that parses a byte stream of SSE-formatted data into [`ServerSentEvent`]s.
pub struct ServerSentEventsCodec {
    lines_codec: LinesCodec,
    next: ServerSentEvent,
}

impl Default for ServerSentEventsCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerSentEventsCodec {
    /// Creates a new SSE codec.
    pub fn new() -> Self {
        Self {
            lines_codec: LinesCodec::new(),
            next: Default::default(),
        }
    }
}

impl Decoder for ServerSentEventsCodec {
    type Item = ServerSentEvent;
    type Error = LinesCodecError;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        let res = self.lines_codec.decode(src)?;

        let Some(mut line) = res else {
            return Ok(None);
        };

        // Empty line indicates dispatch of the current event.
        if line.is_empty() {
            if self.next.data.is_some() || self.next.event.is_some() {
                let result = mem::take(&mut self.next);
                return Ok(Some(result));
            }
            return Ok(None);
        }

        // SSE comment lines (e.g. keep-alive ping : ping)
        if line.starts_with(':') {
            return Ok(None);
        }

        if line.starts_with(EVENT) {
            line.drain(..EVENT.len());
            self.next.event = Some(line);
        } else if line.starts_with(DATA) {
            line.drain(..DATA.len());
            if let Some(ref mut existing) = self.next.data {
                existing.push('\n');
                existing.push_str(&line);
            } else {
                self.next.data = Some(line);
            }
        } else if line.starts_with(ID) {
            line.drain(..ID.len());
            self.next.id = Some(line);
        } else if line.starts_with(RETRY) {
            line.drain(..RETRY.len());
            let Ok(retry) = line.parse() else {
                warn!(line, "Received invalid retry value");
                return Ok(None);
            };
            self.next.retry = Some(retry);
        }

        Ok(None)
    }
}

/// Converts a [`Response`] into a stream of [`ServerSentEvent`]s.
pub fn stream_response(
    response: Response,
) -> impl Stream<Item = std::result::Result<ServerSentEvent, LinesCodecError>> {
    let bytes_stream = response.bytes_stream();
    let body_reader = StreamReader::new(bytes_stream.map(|res| res.map_err(std::io::Error::other)));
    FramedRead::new(body_reader, ServerSentEventsCodec::new())
}
