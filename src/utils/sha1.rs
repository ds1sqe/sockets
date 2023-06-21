use std::{fmt::Display, format};

// initial states
const HASH: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
// Round key constants
const K0: u32 = 0x5A827999u32;
const K1: u32 = 0x6ED9EBA1u32;
const K2: u32 = 0x8F1BBCDCu32;
const K3: u32 = 0xCA62C1D6u32;

#[derive(Debug)]
pub struct Sha1 {
    // internal state 160(32*5)
    state: [u32; 5],
    // 512bits
    blocks: [u8; 64],

    len: usize,
}

pub fn __rotate_left_u32(value: u32, bits: usize) -> u32 {
    (value << (bits)) | (value) >> (32 - (bits))
}

pub fn __trim_to_64(input: &[u8]) -> &[u8; 64] {
    unsafe {
        assert!(input.len() == 64);
        let arr: &[u8; 64] = &*(input.as_ptr() as *const [u8; 64]);
        arr
    }
}

impl Default for Sha1 {
    fn default() -> Sha1 {
        Sha1 {
            state: HASH,
            len: 0,
            blocks: [0; 64],
        }
    }
}
impl Sha1 {
    pub fn new() -> Sha1 {
        Sha1::default()
    }
}
impl Sha1 {
    pub fn from<T: AsRef<[u8]>>(data: T) -> Self {
        let mut sha = Sha1::default();
        sha.__generate(data.as_ref());
        sha
    }

    /// pre -> process -> digest

    fn __preprocess_tail(&mut self, tail: &mut [u8; 128], tailbytes: usize) {
        let bits = self.len;

        tail[0] = 0x80;
        tail[tailbytes - 8] = ((bits >> 56) & 0xFF) as u8;
        tail[tailbytes - 7] = ((bits >> 48) & 0xFF) as u8;
        tail[tailbytes - 6] = ((bits >> 40) & 0xFF) as u8;
        tail[tailbytes - 5] = ((bits >> 32) & 0xFF) as u8;
        tail[tailbytes - 4] = ((bits >> 24) & 0xFF) as u8;
        tail[tailbytes - 3] = ((bits >> 16) & 0xFF) as u8;
        tail[tailbytes - 2] = ((bits >> 8) & 0xFF) as u8;
        tail[tailbytes - 1] = (bits & 0xFF) as u8;
    }

    fn __generate(&mut self, data: &[u8]) {
        let data_bytes = data.len();
        self.len = data_bytes * 8;

        let iter_count = (data_bytes + 8) / 64 + 1;
        let tailbytes = 64 * iter_count - data_bytes;
        let mut tail: [u8; 128] = [0; 128];

        self.__preprocess_tail(&mut tail, tailbytes);

        let mut didx = 0;
        for _ in 0..iter_count {
            let mut words = [0u32; 80];

            for widx in 0..16 {
                let mut wcount = 24;

                while didx < data_bytes && wcount >= 0 {
                    words[widx] += u32::from(data[didx]) << wcount;
                    didx += 1;
                    wcount -= 8;
                }

                while wcount >= 0 {
                    words[widx] += u32::from(tail[didx - data_bytes]) << wcount;
                    didx += 1;
                    wcount -= 8;
                }
            }

            //* extend u32*16 into u32*80
            //* Extend the sixteen 32-bit words into eighty 32-bit words, with potential optimization from:
            //* "Improving the Performance of the Secure Hash Algorithm (SHA-1)" by Max Locktyukhin
            for widx in 16..32 {
                words[widx] = __rotate_left_u32(
                    words[widx - 3] ^ words[widx - 8] ^ words[widx - 14] ^ words[widx - 16],
                    1,
                );
            }
            for widx in 32..80 {
                words[widx] = __rotate_left_u32(
                    words[widx - 6] ^ words[widx - 16] ^ words[widx - 28] ^ words[widx - 32],
                    2,
                );
            }

            let mut a = self.state[0];
            let mut b = self.state[1];
            let mut c = self.state[2];
            let mut d = self.state[3];
            let mut e = self.state[4];
            let mut f = 0;
            let mut k = 0;

            for (widx, word) in words.iter().enumerate() {
                if widx < 20 {
                    f = (b & c) | ((!b) & d);
                    k = K0;
                } else if widx < 40 {
                    f = b ^ c ^ d;
                    k = K1;
                } else if widx < 60 {
                    f = (b & c) | (b & d) | (c & d);
                    k = K2;
                } else if widx < 80 {
                    f = b ^ c ^ d;
                    k = K3;
                }
                let mut temp: u32 = 0;
                temp = temp
                    .wrapping_add(__rotate_left_u32(a, 5))
                    .wrapping_add(f)
                    .wrapping_add(e)
                    .wrapping_add(k)
                    .wrapping_add(*word);
                e = d;
                d = c;
                c = __rotate_left_u32(b, 30);
                b = a;
                a = temp;
            }

            self.state[0] = self.state[0].wrapping_add(a);
            self.state[1] = self.state[1].wrapping_add(b);
            self.state[2] = self.state[2].wrapping_add(c);
            self.state[3] = self.state[3].wrapping_add(d);
            self.state[4] = self.state[4].wrapping_add(e);
        }
    }

    // create output (160 bits)
    pub fn digest(&mut self) -> ShaOutput {
        ShaOutput::from([
            (self.state[0] >> 24) as u8,
            (self.state[0] >> 16) as u8,
            (self.state[0] >> 8) as u8,
            (self.state[0]) as u8,
            (self.state[1] >> 24) as u8,
            (self.state[1] >> 16) as u8,
            (self.state[1] >> 8) as u8,
            (self.state[1]) as u8,
            (self.state[2] >> 24) as u8,
            (self.state[2] >> 16) as u8,
            (self.state[2] >> 8) as u8,
            (self.state[2]) as u8,
            (self.state[3] >> 24) as u8,
            (self.state[3] >> 16) as u8,
            (self.state[3] >> 8) as u8,
            (self.state[3]) as u8,
            (self.state[4] >> 24) as u8,
            (self.state[4] >> 16) as u8,
            (self.state[4] >> 8) as u8,
            (self.state[4]) as u8,
        ])
    }
}

#[derive(Debug)]
pub struct ShaOutput {
    state: [u8; 20],
}

impl Default for ShaOutput {
    fn default() -> Self {
        ShaOutput::new()
    }
}

impl ShaOutput {
    pub fn new() -> Self {
        ShaOutput { state: [0; 20] }
    }
    pub fn from(state: [u8; 20]) -> Self {
        ShaOutput { state }
    }
    pub fn to_hexstring(&self) -> String {
        let str: String = self
            .state
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();
        str
    }
}

#[cfg(test)]
mod test {
    use std::println;

    use super::Sha1;

    #[test]
    fn sha1_test_from() {
        let texts = [
            "The quick brown fox jumps over the lazy dog",
            "The quick brown fox jumps over the lazy cog",
        ];
        let expected = [
            "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12",
            "de9f2c7fd25e1b3afad3e85a0bd17d9b100db4b3",
        ];

        for (idx, text) in texts.iter().enumerate() {
            let output = Sha1::from(text).digest().to_hexstring();
            println!("target: {}", text);
            println!("SHA-1: {}", output);
            assert_eq!(expected[idx], output);
        }
    }
}
