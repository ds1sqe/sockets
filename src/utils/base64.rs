use std::vec;

pub struct Base64;

const OFFSET_UPPER: u8 = 65;
const OFFSET_LOWER: u8 = 71;
const OFFSET_DIGIT: u8 = 4;

impl Base64 {
    fn __base64_to_char(&self, base64: u8) -> Option<char> {
        let char = match base64 {
            0..=25 => base64 + OFFSET_UPPER,  // A-Z
            26..=51 => base64 + OFFSET_LOWER, // a-z
            52..=61 => base64 - OFFSET_DIGIT, // 0-9
            62 => 43,                         // +
            63 => 47,                         // /
            _ => return None,
        };
        Some(char as char)
    }
    fn __char_to_base64(&self, char: char) -> Option<u8> {
        let base64 = match char as u8 {
            43 => 62,                              // +
            47 => 63,                              // /
            48..=57 => char as u8 + OFFSET_DIGIT,  // 0-9
            65..=90 => char as u8 - OFFSET_UPPER,  // A-Z
            97..=122 => char as u8 - OFFSET_LOWER, // a-z
            _ => return None,
        };
        Some(base64)
    }

    fn __split(&self, chunk: &[u8]) -> Vec<u8> {
        match chunk.len() {
            1 => vec![&chunk[0] >> 2, (&chunk[0] & 0b00000011) << 4],
            2 => vec![
                &chunk[0] >> 2,
                ((&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4),
                (&chunk[1] & 0b00001111) << 2,
            ],
            3 => vec![
                &chunk[0] >> 2,
                ((&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4),
                ((&chunk[1] & 0b00001111) << 2 | &chunk[2] >> 6),
                &chunk[2] & 0b00111111,
            ],

            _ => unreachable!(),
        }
    }

    fn __encode_chunk(&self, chunk: Vec<u8>) -> Vec<char> {
        let mut out = vec!['='; 4];

        for idx in 0..chunk.len() {
            if let Some(char) = self.__base64_to_char(chunk[idx]) {
                out[idx] = char;
            }
        }
        out
    }

    pub fn encode(&self, data: &[u8]) -> String {
        let encoded = data
            .chunks(3)
            .map(|chunk| self.__split(chunk))
            .flat_map(|chunk| self.__encode_chunk(chunk));

        String::from_iter(encoded)
    }
}

#[cfg(test)]
mod test {
    use super::Base64;

    #[test]
    fn base64_encode() {
        let encoded = Base64.encode(b"hello world");
        println!("encoded: {}", encoded);
        assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
    }
}
