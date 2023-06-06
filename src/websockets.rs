use std::io::{Read, Write};

#[derive(Debug)]
pub enum ConnectionState {
    NeedHandShake,
    Accepted,
    Close,
}

#[derive(Debug)]
pub struct WebSocketServer<Stream> {
    /// abstraction which represents the byte stream (Data stream)
    pub stream: Stream,
    // max payload size (default value : 16 MB)
    pub max_payload_size: usize,

    pub state: ConnectionState,
}

impl<Stream> WebSocketServer<Stream> {
    /// create new stream
    /// `stream` : Abstraction represents data stream
    /// `max_size` : max size of payload (default : 16 MB)
    pub fn new(stream: Stream, max_size: Option<usize>) -> Self {
        match max_size {
            Some(size) => Self {
                stream,
                max_payload_size: size,
                state: ConnectionState::NeedHandShake,
            },
            None => Self {
                stream,
                max_payload_size: 16 * 1024 * 1024,
                state: ConnectionState::NeedHandShake,
            },
        }
    }
}

impl<Stream> WebSocketServer<Stream>
where
    Stream: Unpin + Read + Write,
{
    /// handshake with client
    pub async fn handshake(&self) {}
    /// send msg to client
    pub async fn send(&self) {}
    /// close connection
    pub async fn close(&self) {}
}
