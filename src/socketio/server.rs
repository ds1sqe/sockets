use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::{
    websockets::server::{self, WebsocketConnection},
    worker::ThreadPool,
};

struct Event {
    name: String,
    id: Option<usize>,
    room_id: Option<usize>,
    payload: Option<String>,
}

impl Event {
    fn to_string(&self) -> String {
        String::from(format!(
            "event: {:?},\npayload: {:?}\n",
            self.name, self.payload
        ))
    }
}

type Callback = fn(Event) -> Option<()>;

#[derive(Debug)]
struct Config {
    url: String,
    threads: usize,
    max_payload_size: usize,
}

#[derive(Debug)]
pub struct Server<Stream> {
    connections: HashMap<String, Arc<Mutex<WebsocketConnection<Stream>>>>,
    config: Config,
    listeners: HashMap<String, Callback>,
    rooms: HashMap<usize, HashSet<usize>>,
}

trait SocketIoServer<Stream> {
    fn new(config: Config) -> Self;
    fn on(&mut self, event: String, callback: Callback);
    fn enter(&mut self, id: usize, room_id: usize);
    fn exit(&mut self, id: usize, room_id: usize);
    fn emit(&mut self, event: Event);
    fn send(&mut self, id: usize, msg: String);
    fn on_close(&mut self, callback: Callback);
    fn on_connect(&mut self, callback: Callback);
    // fn listen(&self);
}

impl<Stream> SocketIoServer<Stream> for Server<Stream>
where
    Stream: Unpin + Read + Write,
{
    fn new(config: Config) -> Self {
        Self {
            connections: HashMap::new(),
            config,
            rooms: HashMap::new(),
            listeners: HashMap::new(),
        }
    }
    fn on(&mut self, event: String, callback: Callback) {
        self.listeners.insert(event, callback);
    }
    fn enter(&mut self, id: usize, room_id: usize) {
        self.rooms
            .entry(room_id)
            .and_modify(|r| {
                r.insert(id);
            })
            .or_insert({
                let mut new_set = HashSet::new();
                new_set.insert(id);
                new_set
            });
    }
    fn exit(&mut self, id: usize, room_id: usize) {
        self.rooms.entry(room_id).and_modify(|r| {
            r.remove(&id);
        });
    }
    fn send(&mut self, id: usize, msg: String) {
        let tc = self.connections.get_mut(&id.to_string()).unwrap();
        tc.lock().unwrap().send_msg(msg)
    }
    fn emit(&mut self, event: Event) {
        if event.room_id.is_some() {
            if let Some(target_ids) = self.rooms.get_mut(&event.room_id.unwrap()) {
                for target in target_ids.clone().into_iter() {
                    self.send(target, event.to_string())
                }
            }
        }
    }
    fn on_close(&mut self, callback: Callback) {
        self.listeners.insert(String::from("close"), callback);
    }
    fn on_connect(&mut self, callback: Callback) {
        self.listeners.insert(String::from("connect"), callback);
    }
}

impl Server<TcpStream> {
    pub fn create(config: Config) -> Self {
        let srv = Self::new(config);
        srv
    }
    fn manage_connection(&mut self, stream: TcpStream) {
        // TODO: manage ID
        let id = stream.peer_addr().unwrap().to_string();
        let wc = Arc::new(Mutex::new(WebsocketConnection::new(
            stream.try_clone().expect("clone faild"),
            Some(self.config.max_payload_size),
        )));
        println!("peer: {id}");
        wc.lock().unwrap().handshake().unwrap();
        // TODO: add socketio spec handshake
        println!("sio handshake...");
        let ans = String::from(
"0{\"sid\":\"lv_VI97HAXpY6yYWAAAC\",\"upgrades\":[\"websocket\"],\"pingInterval\":25000,\"pingTimeout\":20000,\"maxPayload\":1000000}"
        );
        wc.lock().unwrap().send_msg(ans);
        self.connections.insert(id, wc.clone());

        // TODO: manage received event
        loop {
            let packet = wc.lock().unwrap().receive();
            let msg = String::from_utf8(packet.payload.clone()).unwrap();
            if msg == "40" {
                let ans = String::from("40{\"sid\":\"lv_VI97HAXpY6yYWAAAC\"}");
                wc.lock().unwrap().send_msg(ans);
            } else if msg != "3" {
                if msg == "42[\"test\",\"\"]" {
                    println!("{:?}", self);
                    let msg = String::from("42[\"testing\",\"hello socket.io\"]");
                    wc.lock().unwrap().send_msg(msg);
                } else {
                    let ping = String::from("2");
                    wc.lock().unwrap().send_msg(ping);
                }
            }
        }
    }

    pub fn listen(&'static mut self) {
        let listener = TcpListener::bind(self.config.url.clone()).unwrap();
        let threads = ThreadPool::build(self.config.threads);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            threads.excute(|| self.manage_connection(stream))
        }
    }
}
