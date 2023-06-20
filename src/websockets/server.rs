use std::io::{Read, Write};

trait ConnectionState {
    fn try_handshake(self: Box<Self>) -> Box<dyn ConnectionState>;
    fn verify_handshake(self: Box<Self>) -> Box<dyn ConnectionState>;
    fn abort(self: Box<Self>) -> Box<dyn ConnectionState>;
}

struct NeedHandShake {}
impl ConnectionState for NeedHandShake {
    fn try_handshake(self: Box<Self>) -> Box<dyn ConnectionState> {
        Box::new(MidHandShake {})
    }
    fn verify_handshake(self: Box<Self>) -> Box<dyn ConnectionState> {
        self
    }
    fn abort(self: Box<Self>) -> Box<dyn ConnectionState> {
        Box::new(Closed {})
    }
}

struct MidHandShake {}
impl ConnectionState for MidHandShake {
    fn try_handshake(self: Box<Self>) -> Box<dyn ConnectionState> {
        self
    }
    fn verify_handshake(self: Box<Self>) -> Box<dyn ConnectionState> {
        self
    }
    fn abort(self: Box<Self>) -> Box<dyn ConnectionState> {
        Box::new(Closed {})
    }
}

struct Accepted {}
impl ConnectionState for Accepted {
    fn try_handshake(self: Box<Self>) -> Box<dyn ConnectionState> {
        self
    }
    fn verify_handshake(self: Box<Self>) -> Box<dyn ConnectionState> {
        Box::new(Accepted {})
    }
    fn abort(self: Box<Self>) -> Box<dyn ConnectionState> {
        self
    }
}

struct Closed {}
impl ConnectionState for Closed {
    fn try_handshake(self: Box<Self>) -> Box<dyn ConnectionState> {
        self
    }
    fn verify_handshake(self: Box<Self>) -> Box<dyn ConnectionState> {
        self
    }
    fn abort(self: Box<Self>) -> Box<dyn ConnectionState> {
        self
    }
}

pub struct Server<Stream> {
    /// abstraction which represents the byte stream (Data stream)
    pub stream: Stream,
    // max payload size (default value : 16 MB)
    pub max_payload_size: usize,

    state: Option<Box<dyn ConnectionState>>,
}

impl<Stream> Server<Stream> {
    /// create new stream
    /// `stream` : Abstraction represents data stream
    /// `max_size` : max size of payload (default : 16 MB)
    pub fn new(stream: Stream, max_size: Option<usize>) -> Self {
        match max_size {
            Some(size) => Self {
                stream,
                max_payload_size: size,
                state: Some(Box::new(NeedHandShake {})),
            },
            None => Self {
                stream,
                max_payload_size: 16 * 1024 * 1024,
                state: Some(Box::new(NeedHandShake {})),
            },
        }
    }
}

impl<Stream> Server<Stream>
where
    Stream: Unpin + Read + Write,
{
    /// handshake with client
    pub fn handshake(mut self) {
        let header = String::from(
            "
HTTP/1.1 101 Switching Protocols\r\n
Upgrade: websocket\r\n
Connection: Upgrade\r\n
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n
",
        );

        if let Some(state) = self.state.take() {
            self.state = Some(state.try_handshake())
        }
    }
    pub async fn receive(self) {
        self.handshake();
    }
    /// send msg to client
    pub async fn send(&self) {}
    /// close connection
    pub async fn close(&self) {}
}
