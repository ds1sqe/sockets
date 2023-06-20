use std::io::{Read, Write};

#[derive(Debug)]
pub enum __ConnectionState {
    NeedHandShake,
    MidHandShake,
    Connected,
    Failed,
    Closed,
}

#[derive(Debug)]
pub struct Connection {
    pub state: __ConnectionState,
}

impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            state: __ConnectionState::NeedHandShake,
        }
    }
    pub fn handshake(&mut self) {
        self.state = __ConnectionState::MidHandShake
    }
    pub fn connect(&mut self) {
        self.state = __ConnectionState::Connected
    }
    pub fn fail(&mut self) {
        self.state = __ConnectionState::Failed
    }
    pub fn close(&mut self) {
        self.state = __ConnectionState::Closed
    }
}

#[derive(Debug)]
pub struct Server<Stream> {
    /// abstraction which represents the byte stream (Data stream)
    pub stream: Stream,
    // max payload size (default value : 16 MB)
    pub max_payload_size: usize,

    pub connection: Connection,
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
                connection: Connection::new(),
            },
            None => Self {
                stream,
                max_payload_size: 16 * 1024 * 1024,
                connection: Connection::new(),
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
        self.stream.write(header.as_bytes());
        self.stream.flush();
    }
    pub async fn receive(self) {
        self.handshake();
    }
    /// send msg to client
    pub async fn send(&self) {}
    /// close connection
    pub async fn close(&self) {}
}
