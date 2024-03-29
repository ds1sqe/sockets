use std::{
    io::{BufRead, BufReader, Cursor, Read, Write},
    println,
};

use crate::{
    http::header::{RequestHeader, ResponseHeader},
    utils::{base64::Base64, sha1::Sha1},
    websockets::frame::Frame,
};

#[derive(Debug)]
pub enum ConnectionState {
    NeedHandShake,
    MidHandShake,
    Connected,
    Failed,
    Closed,
}

#[derive(Debug)]
pub struct Connection {
    pub state: ConnectionState,
}

impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            state: ConnectionState::NeedHandShake,
        }
    }
    /// change state to mid handshake
    pub fn handshake(&mut self) {
        self.state = ConnectionState::MidHandShake
    }
    /// change state to connected
    pub fn connect(&mut self) {
        self.state = ConnectionState::Connected
    }
    /// change state to faild
    pub fn fail(&mut self) {
        self.state = ConnectionState::Failed
    }
    /// change state to closed
    pub fn close(&mut self) {
        self.state = ConnectionState::Closed
    }
}

#[derive(Debug)]
pub struct WebsocketConnection<Stream> {
    /// abstraction which represents the byte stream (Data stream)
    pub stream: Stream,
    // max payload size (default value : 16 MB)
    pub max_payload_size: usize,

    pub connection: Connection,
}

impl<Stream> WebsocketConnection<Stream> {
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

impl<Stream> WebsocketConnection<Stream>
where
    Stream: Unpin + Read + Write,
{
    /// handshake with client
    pub fn handshake(&mut self) -> std::io::Result<()> {
        println!("handshaking ...");

        self.connection.handshake();

        let mut reader = BufReader::new(&mut self.stream);

        let rsv: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(rsv.len());

        let req = String::from_utf8(rsv).unwrap();

        println!("Request: {}", req);

        let req_hdr = RequestHeader::from(&req);

        println!("Header: {:?}", req_hdr);

        let swk = req_hdr.get("Sec-WebSocket-Key").unwrap();

        println!("Sec-WebSocket-Key : {:?}", swk);

        const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

        let hash = Sha1::from([swk.as_bytes(), WS_GUID].concat())
            .digest()
            .as_byte();

        let swa = Base64.encode(&hash);

        let mut res = ResponseHeader::default();

        res.set("Sec-WebSocket-Accept", &swa);
        res.set("Upgrade", "websocket");
        res.set("Connection", "Upgrade");

        println!("Response created: {:?}", res);

        self.stream.write_all(res.format().as_bytes()).unwrap();

        println!("Responsed with");
        println!("{}", res.format());

        self.connection.connect();

        Ok(())
    }

    pub fn receive(&mut self) -> Frame {
        // match self.connection.state {
        //     ConnectionState::NeedHandShake => self.handshake(),
        //     ConnectionState::MidHandShake => {}
        //     ConnectionState::Connected => {}
        //     ConnectionState::Closed => {}
        //     _ => {}
        // }

        let mut reader = BufReader::new(&mut self.stream);

        let mut rsv: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(rsv.len());

        let frame = Frame::parse(&mut rsv);

        println!("{}", frame);

        frame
    }

    /// send msg to client
    pub fn send_msg(&mut self, msg: String) {
        let frame = Frame::create_msg_frame(msg);
        frame.format(&mut self.stream).unwrap();
    }
    pub fn send_pong(&mut self) {
        let frame = Frame::create_pong_frame();
        frame.format(&mut self.stream).unwrap();
    }
    /// close connection
    pub fn close(&self) {}
}
