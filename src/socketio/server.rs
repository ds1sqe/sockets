use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
};

use crate::websockets::server;

struct Event {
    name: String,
    id: Option<usize>,
    room_id: Option<usize>,
    payload: Option<String>,
}

type Callback = fn(Event) -> Option<()>;

struct Config {
    url: String,
}

struct Server<Stream> {
    stream: Stream,
    config: Config,
    listeners: HashMap<String, Callback>,
    rooms: HashMap<usize, HashSet<usize>>,
}

trait SocketIoServer<Stream> {
    fn spawn(stream: Stream, config: Config) -> Self;
    fn on(&mut self, event: String, callback: Callback);
    fn enter(&mut self, id: usize, room_id: usize);
    fn exit(&mut self, id: usize, room_id: usize);
    fn emit(&self, event: Event);
    fn on_close(&mut self, callback: Callback);
    fn on_connect(&mut self, callback: Callback);
    fn listen(&self);
}

impl<Stream> SocketIoServer<Stream> for Server<Stream>
where
    Stream: Unpin + Read + Write,
{
    fn spawn(stream: Stream, config: Config) -> Self {
        Self {
            stream,
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
    fn emit(&self, event: Event) {
        if event.room_id.is_some() {
            if let Some(target_ids) = self.rooms.get(&event.room_id.unwrap()) {
                for target in target_ids.into_iter() {
                    // TODO:
                    // send event to target
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
    fn listen(&self) {
        // TODO: manage connection
    }
}
