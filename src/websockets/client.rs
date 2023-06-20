use std::io::{Read, Write};

#[derive(Debug)]
pub enum ConnectionState {
    NeedHandShake,
    MidHandShake,
    Accepted,
    Close,
}

#[derive(Debug)]
pub struct Client<Stream> {
    /// abstraction which represents the byte stream (Data stream)
    pub stream: Stream,
    // max payload size (default value : 16 MB)
    pub max_payload_size: usize,

    pub state: ConnectionState,
}

impl<Stream> Client<Stream> {
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

impl<Stream> Client<Stream>
where
    Stream: Unpin + Read + Write,
{
    /// handshake with sever
    fn handshake(mut self) {
        let header = String::from(
            "
GET / HTTP/1.1\r\n
Host: 127.0.0.1:8001\r\n
Upgrade: websocket\r\n
Connection: Upgrade\r\n
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n
Sec-WebSocket-Version: 13\r\n
",
        );

        self.stream.write(header.as_bytes());
        self.stream.flush();
    }

    /// connect with sever
    pub async fn connect(self) {
        self.handshake()
    }
    /// send msg to client
    pub async fn send(&self) {}
    /// close connection
    pub async fn close(&self) {}
}
