use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;

use sockets::websockets::client::Client;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8001").unwrap();
    handle_connection(stream);
}

fn handle_connection(mut stream: TcpStream) {
    let ws = Client::new(&mut stream, None);
    let _ = ws.connect();
    let read = BufReader::new(&mut stream);
    for lines in read.lines() {
        match lines {
            Ok(line) => {
                println!("{}", line);
            }
            Err(_) => {}
        }
    }
}
