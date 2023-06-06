use std::{
    eprintln,
    fmt::Display,
    io::{Cursor, Read},
};

//  Data frame spec from RFC6455
//  0                   1                   2                   3
//  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
// +-+-+-+-+-------+-+-------------+-------------------------------+
// |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
// |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
// |N|V|V|V|       |S|             |   (if payload len==126/127)   |
// | |1|2|3|       |K|             |                               |
// +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
// |     Extended payload length continued, if payload len == 127  |
// + - - - - - - - - - - - - - - - +-------------------------------+
// |                               |Masking-key, if MASK set to 1  |
// +-------------------------------+-------------------------------+
// | Masking-key (continued)       |          Payload Data         |
// +-------------------------------- - - - - - - - - - - - - - - - +
// :                     Payload Data continued ...                :
// + - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +
// |                     Payload Data continued ...                |
// +---------------------------------------------------------------+
pub struct Frame {
    header: FrameHeader,
    body: Vec<u8>,
}

pub struct FrameHeader {
    // indicate FIN (is the frame is last one)
    pub fin: bool,
    // reserved
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub opcode: Opcode,
    pub mask: Option<u32>,
    pub payloadlength: u64,
}
pub enum Opcode {
    Data(Data),
    Control(Control),
}
impl Opcode {
    fn parse(flags: u8) -> Self {
        match flags {
            0b0000 => Opcode::Data(Data::Continue),
            0b0001 => Opcode::Data(Data::Text),
            0b0010 => Opcode::Data(Data::Binary),
            0b1000 => Opcode::Control(Control::Ping),
            0b1001 => Opcode::Control(Control::Pong),
            0b1010 => Opcode::Control(Control::Close),
            rsv => {
                if rsv < 7 {
                    Opcode::Data(Data::Reserved)
                } else {
                    Opcode::Control(Control::Reserved)
                }
            }
        }
    }
    fn format(&self) -> u8 {
        match self {
            Opcode::Data(Data::Continue) => 0b0000,
            Opcode::Data(Data::Text) => 0b0001,
            Opcode::Data(Data::Binary) => 0b0010,
            Opcode::Control(Control::Ping) => 0b1000,
            Opcode::Control(Control::Pong) => 0b1001,
            Opcode::Control(Control::Close) => 0b1010,
            Opcode::Data(Data::Reserved) => 0b0111,
            Opcode::Control(Control::Reserved) => 0b1111,
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind;
        let detail;

        match self {
            Opcode::Data(d) => {
                kind = "DATA";
                match d {
                    Data::Continue => detail = "Continue",
                    Data::Text => detail = "Text",
                    Data::Binary => detail = "Binary",
                    Data::Reserved => detail = "Reserved",
                }
            }
            Opcode::Control(c) => {
                kind = "Control";
                match c {
                    Control::Close => detail = "Close",
                    Control::Ping => detail = "Ping",
                    Control::Pong => detail = "Pong",
                    Control::Reserved => detail = "Reserved",
                }
            }
        };
        write!(f, "{kind}:{detail}")
    }
}
pub enum Data {
    Continue,
    Text,
    Binary,
    Reserved,
}
pub enum Control {
    Close,
    Ping,
    Pong,
    Reserved,
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mask = match self.header.mask {
            Some(val) => val.to_string(),
            None => String::from("None"),
        };
        write!(
            f,
            "
<Frame>
Header
> FIN: {}
> Opcode: {}
> Mask: {}
> Payload Length: {}
Payload
> {:?}
",
            self.header.fin, self.header.opcode, mask, self.header.payloadlength, self.body
        )
    }
}

impl Frame {
    // parse bytes to internal Data
    fn parse() {}
    // internal Data to bytes
    fn format() {}
}

impl FrameHeader {
    fn parse(cursor: &mut Cursor<impl AsRef<[u8]> + Read>) -> Result<Option<Self>, std::io::Error> {
        let start = cursor.position();

        let mut head_buffer = [0u8; 2];
        if cursor.read(&mut head_buffer)? != 2 {
            return Ok(None);
        }
        let (first, second) = (head_buffer[0], head_buffer[1]);

        let fin = first & 0b0001 != 0;
        let rsv1 = first & 0b0010 != 0;
        let rsv2 = first & 0b0100 != 0;
        let rsv3 = first & 0b1000 != 0;

        let opcode = Opcode::parse(first >> 4);

        let mask = None;
        let payloadlength = 10;

        let header = FrameHeader {
            fin,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            mask,
            payloadlength,
        };

        Ok(Some(header))
    }
    fn format(&self) -> u32 {
        let fin = if self.fin { 0b0001 } else { 0 };
        let rsv1 = if self.rsv1 { 0b0010 } else { 0 };
        let rsv2 = if self.rsv2 { 0b0100 } else { 0 };
        let rsv3 = if self.rsv3 { 0b1000 } else { 0 };

        let opcode = self.opcode.format() << 4;

        // let mask = None;
        // let payloadlength = 10;

        let bits = fin | rsv1 | rsv2 | rsv3 | opcode;
        bits as u32
    }
}
