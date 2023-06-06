#[derive(Debug)]
pub struct WebSocket<Stream> {
    /// abstraction which represents the byte stream (Data stream)
    pub stream: Stream,
    // max payload size (default value : 16 MB)
    pub max_payload_size: usize,
}

impl<Stream> WebSocket<Stream> {
    /// create new stream
    /// `stream` : Abstraction represents data stream
    /// `max_size` : max size of payload (default : 16 MB)
    pub fn new(stream: Stream, max_size: Option<usize>) -> Self {
        match max_size {
            Some(size) => Self {
                stream,
                max_payload_size: size,
            },
            None => Self {
                stream,
                max_payload_size: 16 * 1024 * 1024,
            },
        }
    }
    pub async fn connect() {}
    pub async fn send() {}
    pub async fn push() {}
    pub async fn flush() {}
    pub async fn close() {}
}
