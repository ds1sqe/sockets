use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;

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
    let reader = BufReader::new(&mut stream);

    let request: String = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    print!("{:#?}", request);

    //
    // let header= Response::builder().status(StatusCode::SWITCHING_PROTOCOLS)
    //     .header("Connection","Upgrade").header("Upgrade", "websocket").header("Sec-Websocket-Accept", derive_accept_key(req_key))
    //
    //
    //
    // stream.write_all(header.try_into())?;
    //
    // println!("{}", header);

    Ok(())
}
