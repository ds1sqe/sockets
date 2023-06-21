use std::{collections::HashMap, fmt::Display, format};

#[derive(Debug, PartialEq)]
pub struct RequestHeader {
    method: String,
    route: String,
    protocol: String,
    data: HashMap<String, String>,
}

impl RequestHeader {
    fn new() -> Self {
        RequestHeader {
            method: String::from("GET"),
            route: String::from("/"),
            protocol: String::from("HTTP/1.1"),
            data: HashMap::new(),
        }
    }
    pub fn from(str: &str) -> Self {
        Self::__from_internal(str)
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
    pub fn set(&mut self, key: &str, val: &str) {
        self.data.insert(String::from(key), String::from(val));
    }

    fn __from_internal(str: &str) -> Self {
        let mut hdr = Self::new();

        let mut offset = 0;
        for (lidx, line) in str.lines().enumerate() {
            if line.is_empty() {
                offset += 1;
                continue;
            }
            if lidx == offset {
                // first line
                for (widx, words) in line.split(" ").enumerate() {
                    if widx == 0 {
                        hdr.method = String::from(words);
                    } else if widx == 1 {
                        hdr.route = String::from(words);
                    } else if widx == 2 {
                        hdr.protocol = String::from(words);
                    }
                }
            } else {
                let mut words = line.split(": ");
                let key = words.next().unwrap();
                let val = words.next().unwrap();
                hdr.set(key, val);
            }
        }
        hdr
    }

    pub fn format(&self) -> String {
        let first_line = format!("{} {} {}\r\n", self.method, self.route, self.protocol);
        let lines: String = self
            .data
            .iter()
            .map(|(k, v)| format!("{k}: {v}\r\n"))
            .collect();

        first_line + &lines
    }
}

#[derive(Debug, PartialEq)]
pub struct ResponseHeader {
    protocol: String,
    status: String,
    data: HashMap<String, String>,
}
impl Default for ResponseHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseHeader {
    fn new() -> Self {
        ResponseHeader {
            protocol: String::from("HTTP/1.1"),
            status: String::from("101 Switching Protocols"),
            data: HashMap::new(),
        }
    }
    pub fn from(str: &str) -> Self {
        Self::__from_internal(str)
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
    pub fn set(&mut self, key: &str, val: &str) {
        self.data.insert(String::from(key), String::from(val));
    }

    fn __from_internal(str: &str) -> Self {
        let mut hdr = Self::new();

        let mut offset = 0;
        for (lidx, line) in str.lines().enumerate() {
            if line.is_empty() {
                offset += 1;
                continue;
            }
            if lidx == offset {
                // first line
                let (protocol, status) = line.split_once(" ").unwrap();
                hdr.protocol = String::from(protocol);
                hdr.status = String::from(status);
            } else {
                let mut words = line.split(": ");
                let key = words.next().unwrap();
                let val = words.next().unwrap();
                hdr.set(key, val);
            }
        }
        hdr
    }

    pub fn format(&self) -> String {
        let first_line = format!("{} {}\r\n", self.protocol, self.status);
        let lines: String = self
            .data
            .iter()
            .map(|(k, v)| format!("{k}: {v}\r\n"))
            .collect();

        first_line + &lines + "\r\n"
    }
}

#[cfg(test)]
mod test {
    use std::println;

    use super::RequestHeader;

    #[test]
    fn header_request_test1() {
        println!("header1 \n");
        let header1 = RequestHeader::from(
            "
GET / HTTP/1.1\r\n
Host: 127.0.0.1:8001\r\n
Upgrade: websocket\r\n
Connection: Upgrade\r\n
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n
Sec-WebSocket-Version: 13\r\n
",
        );
        println!("header created(repr) {:?}", header1);

        println!("header formatted {:?}", header1.format());

        println!("header2 \n");
        let header2 = RequestHeader::from(
            "
GET / HTTP/1.1\r\nHost: 127.0.0.1:8001\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n
",
        );
        println!("header created(repr) {:?}", header2);

        println!("header formatted {:?}", header2.format());

        assert_eq!(header1, header2);
    }
}
