use std::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use super::core::{Army, Unit};

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

#[derive(Debug)]
struct Config {
    url: String,
    max_connection: usize,
    max_payload_size: usize,
}

pub struct Signal {
    something: String,
}

pub struct Connection {
    thd: thread::JoinHandle<()>,
}
impl Connection {
    pub fn build(
        job_rx: Arc<Mutex<Receiver<TcpStream>>>,
        mut sig_tx: Option<Sender<Box<dyn Send>>>,
        mut sig_rx: Option<Receiver<Box<dyn Send>>>,
    ) -> Connection {
        let thread = thread::spawn(move || loop {
            let sig = job_rx.lock().unwrap().recv();
            match sig {
                Ok(stream) => {
                    self::Connection::handle(
                        sig_tx.take().unwrap(),
                        sig_rx.take().unwrap(),
                        stream,
                    );
                }
                Err(detail) => {
                    println!("ERR>> {detail}");
                    break;
                }
            }
        });

        Connection { thd: thread }
    }
    pub fn handle(
        sig_tx: Sender<Box<dyn Send>>,
        sig_rx: Receiver<Box<dyn Send>>,
        stream: TcpStream,
    ) {
        loop {}
    }
}

pub struct ConnectionPool {
    connections: Vec<Connection>,
    job_tx: Sender<TcpStream>,
}
impl ConnectionPool {
    pub fn build(
        size: usize,
        mut transceivers: Vec<Option<Unit<Box<dyn Send>>>>,
    ) -> ConnectionPool {
        let (job_tx, job_rx) = mpsc::channel();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let mut connections = Vec::with_capacity(size);

        for idx in 0..size {
            let con = Connection::build(
                Arc::clone(&job_rx),
                Some(transceivers[idx].take().unwrap().sender),
                Some(transceivers[idx].take().unwrap().receiver),
            );
            connections.push(con)
        }

        ConnectionPool {
            connections,
            job_tx,
        }
    }

    pub fn catch_connection(&self, stream: TcpStream) {
        self.job_tx.send(stream);
    }
}

pub struct Server {
    config: Config,

    senders: Option<Vec<mpsc::Sender<Box<dyn Send>>>>,
    receiver: Option<mpsc::Receiver<Box<dyn Send>>>,

    connections: ConnectionPool,
}

impl Server {
    pub fn build(config: Config) -> Self {
        let army = Army::build(config.max_connection);

        let root = army.commander;

        let connections = ConnectionPool::build(config.max_connection, army.units);

        Server {
            config,
            senders: Some(root.senders),
            receiver: Some(root.receiver),
            connections,
        }
    }

    pub fn manage(
        senders: Vec<mpsc::Sender<Box<dyn Send>>>,
        receiver: mpsc::Receiver<Box<dyn Send>>,
    ) {
        loop {
            let sig = receiver.recv().unwrap();

            // manage signals here
            if true {
                senders[0].send(Box::new(Signal {
                    something: String::from("something"),
                }));
            }
        }
    }

    pub fn listen(&mut self) {
        let listener = TcpListener::bind(self.config.url.clone()).unwrap();

        // create event manager
        let snd = self.senders.take().unwrap();
        let rsv = self.receiver.take().unwrap();
        thread::spawn(move || self::Server::manage(snd.clone(), rsv));

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.connections.catch_connection(stream)
        }
    }
}
