use std::net::TcpListener;
use std::net::TcpStream;

use sockets::websockets::frame::Control;
use sockets::websockets::frame::Data;
use sockets::websockets::frame::Opcode;
use sockets::websockets::server::WebsocketConnection;
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
    let mut srv = WebsocketConnection::new(stream, None);
    srv.handshake().unwrap();

    loop {
        let packet = srv.receive();
        if packet.header.opcode == Opcode::Data(Data::Text) {
            let msg = String::from_utf8(packet.payload.clone()).unwrap();
            if msg == "ping" || msg == "Ping" {
                srv.send_msg(String::from("Pong"));
            } else {
                srv.send_msg(msg)
            }
        }

        if packet.header.opcode == Opcode::Control(Control::Ping) {
            srv.send_pong();
        }

        if packet.header.opcode == Opcode::Control(Control::Close) {
            let buf: [u8; 2] = packet.payload.try_into().unwrap();
            let code = u16::from_be_bytes(buf);
            println!(
                "
Got close call from client
>> Code: {}",
                code
            );
            break;
        }
    }

    Ok(())
}
