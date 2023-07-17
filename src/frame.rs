use std::{
    any::Any,
    eprintln,
    fmt::{format, Display},
    io::{BufRead, Bytes, Cursor, Read, Write},
    println, vec,
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

#[derive(Debug, PartialEq)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct FrameHeader {
    // indicate FIN (is the frame is last one)
    pub fin: bool,
    // reserved
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub opcode: Opcode,
    pub masked: bool,
    pub payloadlength: u64,
    pub mask: Option<u32>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
    Data(Data),
    Control(Control),
    Reserved,
}
impl Opcode {
    fn parse(flags: u8) -> Self {
        match flags {
            0b0000 => Opcode::Data(Data::Continue),
            0b0001 => Opcode::Data(Data::Text),
            0b0010 => Opcode::Data(Data::Binary),
            0b1000 => Opcode::Control(Control::Close),
            0b1001 => Opcode::Control(Control::Ping),
            0b1010 => Opcode::Control(Control::Pong),
            _ => Opcode::Reserved,
        }
    }
    fn format(&self) -> u8 {
        match self {
            Opcode::Data(Data::Continue) => 0b0000,
            Opcode::Data(Data::Text) => 0b0001,
            Opcode::Data(Data::Binary) => 0b0010,
            Opcode::Control(Control::Close) => 0b1000,
            Opcode::Control(Control::Ping) => 0b1001,
            Opcode::Control(Control::Pong) => 0b1010,
            Opcode::Reserved => 0b1111,
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
                }
            }
            Opcode::Control(c) => {
                kind = "Control";
                match c {
                    Control::Close => detail = "Close",
                    Control::Ping => detail = "Ping",
                    Control::Pong => detail = "Pong",
                }
            }
            Opcode::Reserved => {
                kind = "Reserved";
                detail = "None"
            }
        };
        write!(f, "{kind}:{detail}")
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Data {
    Continue,
    Text,
    Binary,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Control {
    Close,
    Ping,
    Pong,
}

enum PayloadLength {
    // 7 bits
    U8(u8),
    // 16 bits
    U16,
    // 64 bits
    U64,
}
impl PayloadLength {
    fn get_format(length: u64) -> Self {
        if length <= 0b0111_1101 {
            PayloadLength::U8(length as u8)
        } else if length <= 0xFFFF {
            //65535
            PayloadLength::U16
        } else {
            PayloadLength::U64
        }
    }
    fn get_extra_bytes(&self) -> u8 {
        match self {
            Self::U8(_) => 0,
            Self::U16 => 2,
            Self::U64 => 8,
        }
    }
    fn get_payload_lenth_byte(&self) -> u8 {
        match self {
            Self::U8(bit) => *bit,
            Self::U16 => 126,
            Self::U64 => 127,
        }
    }
    fn parse(bytes: u8) -> Self {
        match bytes & 0b0111_1111 {
            127 => PayloadLength::U64,
            126 => PayloadLength::U16,
            any => PayloadLength::U8(any),
        }
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mask = match self.header.mask {
            Some(val) => format!("{:b}", val),
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
> Payload Length: {}",
            self.header.fin, self.header.opcode, mask, self.header.payloadlength
        )
        .unwrap();

        match self.header.opcode {
            Opcode::Data(Data::Text) => {
                write!(
                    f,
                    "
<Payload>
{}",
                    String::from_utf8(self.payload.clone()).unwrap()
                )
            }
            _ => {
                write!(
                    f,
                    "
<Payload>
{:?}",
                    self.payload.bytes()
                )
            }
        }
    }
}

impl Frame {
    /// parse bytes to internal Data
    pub fn parse(raw: &mut Vec<u8>) -> Self {
        let mut frame_cursor = Cursor::new(raw);
        let frame_header = FrameHeader::parse(&mut frame_cursor).unwrap().unwrap();

        let mut payload = vec![0; frame_header.payloadlength as usize];

        frame_cursor.read_exact(&mut payload).unwrap();

        Self::applymask(&mut payload, frame_header.mask.unwrap());

        Frame {
            header: frame_header,
            payload,
        }
    }
    /// write bytes to output form internal data
    pub fn format(&self, output: &mut impl Write) -> Result<(), std::io::Error> {
        self.header.format(output).unwrap();
        output.write_all(self.payload.as_slice()).unwrap();
        Ok(())
    }
    fn applymask(target: &mut [u8], mask: u32) {
        let b1: u8 = ((mask >> 24) & 0xff) as u8;
        let b2: u8 = ((mask >> 16) & 0xff) as u8;
        let b3: u8 = ((mask >> 8) & 0xff) as u8;
        let b4: u8 = (mask & 0xff) as u8;
        let mask_u8 = [b1, b2, b3, b4];

        for (idx, byte) in target.iter_mut().enumerate() {
            *byte ^= mask_u8[3 & idx];
        }
    }
    pub fn create_msg_frame(msg: String) -> Self {
        let header = FrameHeader {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: Opcode::Data(Data::Text),
            mask: None,
            masked: false,
            payloadlength: msg.len() as u64,
        };
        let payload = Vec::from(msg.as_bytes());
        Frame { header, payload }
    }
    pub fn create_pong_frame() -> Self {
        let header = FrameHeader {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: Opcode::Control(Control::Close),
            mask: None,
            masked: false,
            payloadlength: 0,
        };
        let payload = Vec::new();
        Frame { header, payload }
    }
}

impl Display for FrameHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mask = self.mask.unwrap();
        write!(
            f,
            "
<Header>
> FIN: {} Opcode: {}
> Masked: {} Payload length: {}
> Mask: {:#034b}
",
            self.fin, self.opcode, self.masked, self.payloadlength, mask,
        )
    }
}
impl FrameHeader {
    pub fn parse(cursor: &mut Cursor<impl AsRef<[u8]>>) -> Result<Option<Self>, std::io::Error> {
        let start = cursor.position();

        let mut head_buffer = [0u8; 2];
        if cursor.read(&mut head_buffer)? != 2 {
            return Ok(None);
        }
        let (first, second) = (head_buffer[0], head_buffer[1]);

        let fin = first & 0b1000_0000 != 0;
        let rsv1 = first & 0b0100_0000 != 0;
        let rsv2 = first & 0b0010_0000 != 0;
        let rsv3 = first & 0b0001_0000 != 0;

        let opcode = Opcode::parse(first & 0b1111);

        let payloadlength = {
            let length_bytes = second & 0b0111_1111;
            let extra_bytes = PayloadLength::parse(length_bytes).get_extra_bytes();
            if extra_bytes > 0 {
                if extra_bytes == 2 {
                    let mut buf = [0; 2];
                    match cursor.read_exact(&mut buf) {
                        Ok(_) => u16::from_be_bytes(buf) as u64,
                        Err(err) => return Err(err),
                    }
                } else {
                    // extra_bytes == 8
                    let mut buf = [0; 8];
                    match cursor.read_exact(&mut buf) {
                        Ok(_) => u64::from_be_bytes(buf),
                        Err(err) => return Err(err),
                    }
                }
            } else {
                u64::from(length_bytes)
            }
        };

        let masked = second & 0b1000_0000 != 0;
        let mask = if masked {
            let mut mask_bytes = [0u8; 4];
            if cursor.read(&mut mask_bytes)? == 4 {
                Some(u32::from_be_bytes(mask_bytes))
            } else {
                return Ok(None);
            }
        } else {
            None
        };

        let header = FrameHeader {
            fin,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            masked,
            payloadlength,
            mask,
        };

        Ok(Some(header))
    }
    fn format(&self, output: &mut impl Write) -> Result<(), std::io::Error> {
        let fin = if self.fin { 0b1000_0000 } else { 0 };
        let rsv1 = if self.rsv1 { 0b0100_0000 } else { 0 };
        let rsv2 = if self.rsv2 { 0b0010_0000 } else { 0 };
        let rsv3 = if self.rsv3 { 0b0001_0000 } else { 0 };
        let opcode = self.opcode.format();

        let first = fin | rsv1 | rsv2 | rsv3 | opcode;

        let length = PayloadLength::get_format(self.payloadlength);
        let maskflag = if self.masked { 0b1000_0000 } else { 0 };
        let lengthflag = length.get_payload_lenth_byte();

        let second = maskflag | lengthflag;

        output.write(&[first, second]);

        match length {
            PayloadLength::U8(_) => (),
            PayloadLength::U16 => {
                let buf = (self.payloadlength as u16).to_be_bytes();
                output.write(&buf);
            }
            PayloadLength::U64 => {
                let buf = (self.payloadlength).to_be_bytes();
                output.write(&buf);
            }
        };

        if self.mask.is_some() {
            output.write(self.mask.unwrap().to_be_bytes().as_ref());
        }

        Ok(())
    }

    fn get_random_mask(&self) -> u32 {
        rand::random()
    }
    fn set_random_mask(&mut self) {
        self.mask = Some(self.get_random_mask());
        self.masked = true;
    }
}

#[cfg(test)]
mod test {
    use std::{
        io::{Cursor, Read},
        print, println,
    };

    use crate::frame::Opcode;

    use super::FrameHeader;

    #[test]
    fn header_format() {
        let mut header = FrameHeader {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: super::Opcode::Control(super::Control::Close),
            mask: None,
            masked: false,
            payloadlength: 123,
        };
        header.set_random_mask();
        println!("Header: {}", header);
        let mut formatted = Vec::new();
        header.format(&mut formatted);

        for bytes in formatted.bytes() {
            let bt = bytes.unwrap();
            print!("{:#010b} ", bt)
        }
    }
    #[test]
    fn header_format_and_parse_2() {
        use super::*;
        for i in 1..5 {
            let test_length;
            if i == 1 {
                test_length = 100;
            } else if i == 2 {
                test_length = 10000;
            } else if i == 3 {
                test_length = 1000000;
            } else {
                test_length = 0;
            }

            for j in 1..8 {
                let test_opcode = match j {
                    1 => Opcode::Data(Data::Continue),
                    2 => Opcode::Data(Data::Text),
                    3 => Opcode::Data(Data::Binary),
                    4 => Opcode::Control(Control::Ping),
                    5 => Opcode::Control(Control::Pong),
                    6 => Opcode::Control(Control::Close),
                    _ => Opcode::Reserved,
                };

                for k in 1..3 {
                    let test_fin = match k {
                        1 => true,
                        2 => false,
                        _ => false,
                    };

                    let mut header = FrameHeader {
                        fin: test_fin,
                        rsv1: false,
                        rsv2: false,
                        rsv3: false,
                        opcode: test_opcode,
                        mask: None,
                        masked: false,
                        payloadlength: test_length,
                    };
                    header.set_random_mask();
                    println!("Header: {}", header);
                    let mut formatted = Vec::new();
                    header.format(&mut formatted);

                    for bytes in formatted.bytes() {
                        let bt = bytes.unwrap();
                        print!("{:#010b} ", bt)
                    }

                    let mut parse_target = Cursor::new(formatted);
                    let parsed = FrameHeader::parse(&mut parse_target).unwrap().unwrap();
                    println!("{}", parsed);

                    assert_eq!(header, parsed);
                }
            }
        }
    }
}
