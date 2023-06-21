use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;

use sockets::websockets::server::Server;
use sockets::worker::ThreadPool;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8001").unwrap();
    let poll = ThreadPool::build(16);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        poll.excute(|| handle_connection(stream).unwrap());
    }
    println!("Shutting down main thread on server");

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut srv = Server::new(stream, None);
    srv.handshake();
    Ok(())
}
