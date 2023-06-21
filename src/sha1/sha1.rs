pub struct Sha1 {
    // internal state 160(32*5)
    state: [u32; 5],
    // 512bits
    blocks: [u8; 64],

    len: usize,
    blocklen: usize,
}

impl Default for Sha1 {
    fn default() -> Sha1 {
        Sha1 {
            state: HASH,
            len: 0,
            blocks: [0; 64],
            blocklen: 0,
        }
    }
}
impl Sha1 {
    pub fn new() -> Sha1 {
        Sha1::default()
    }
}
pub fn __rotate_left_u32(value: u32, bits: usize) -> u32 {
    value << (bits) | ((value) >> (32 - (bits)))
}

pub fn __trim_to_64(input: &[u8]) -> &[u8; 64] {
    unsafe {
        assert!(input.len() == 64);
        let arr: &[u8; 64] = &*(input.as_ptr() as *const [u8; 64]);
        arr
    }
}
// initial states
const HASH: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
// Round key constants
const K0: u32 = 0x5A827999u32;
const K1: u32 = 0x6ED9EBA1u32;
const K2: u32 = 0x8F1BBCDCu32;
const K3: u32 = 0xCA62C1D6u32;

impl Sha1 {
    pub fn from<T: AsRef<[u8]>>(data: T) -> Self {
        let mut sha = Sha1::default();
        sha.__generate(data.as_ref());
        sha
    }
    // generate hash
    fn __generate(&mut self, data: &[u8]) {
        self.__preprocess(data);
    }
    // preprocess data
    fn __preprocess(&mut self, mut data: &[u8]) {
        if self.blocklen > 0 {
            let len = self.blocklen;
            let offset = core::cmp::min(data.len(), self.blocks.len() - len);
            // put data into block
            self.blocks[len..len + offset].clone_from_slice(&data[..offset]);
            if len + offset == self.blocks.len() {
                let lenptr = &mut self.len;
                *lenptr += &self.blocks.len();

                self.__process(&self.blocks.clone());

                self.blocklen = 0;
                data = &data[offset..];
            } else {
                self.blocklen += offset;
                return;
            }
        }
        assert_eq!(self.blocklen, 0);
        for chunk in data.chunks(64) {
            // if chunk size is 512 bits
            if chunk.len() == 64 {
                let lenptr = &mut self.len;

                *lenptr += chunk.len();
                // same with += 64

                self.__process(__trim_to_64(chunk));
            } else {
                self.blocks[..chunk.len()].clone_from_slice(chunk);
                self.blocklen = chunk.len()
            }
        }
    }

    // process data (512bit)
    fn __process(&mut self, block: &[u8; 64]) {
        let mut words = [0u32; 80];

        // break block into u32 * 16
        for (widx, word) in words[..16].iter_mut().enumerate() {
            // u32.len() = u8.len() * 4
            let offset = widx * 4;
            *word = (block[offset + 3] as u32)
                | ((block[offset + 2] as u32) << 8)
                | ((block[offset + 1] as u32) << 16)
                | ((block[offset] as u32) << 24);
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
            let temp = __rotate_left_u32(a, 5) + f + e + k + word;
            e = d;
            d = c;
            c = __rotate_left_u32(b, 30);
            b = c;
            a = temp;
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
    }
    // create output (160 bits)
    pub fn digest(&mut self) -> [u8; 20] {
        let mut state = self.state;
        let bits = (self.len + (self.blocklen)) * 8;
        let lengths = [
            (bits >> 56) as u8,
            (bits >> 48) as u8,
            (bits >> 40) as u8,
            (bits >> 32) as u8,
            (bits >> 24) as u8,
            (bits >> 16) as u8,
            (bits >> 8) as u8,
            (bits >> 0) as u8,
        ];
        let mut tail = [0; 128];

        tail[..self.blocklen].clone_from_slice(&self.blocks[..self.blocklen]);
        tail[self.blocklen] = 0x80;

        // add paddings
        if self.blocklen < 56 {
            tail[56..64].clone_from_slice(&lengths);
            self.__process(__trim_to_64(&tail[0..64]));
        } else {
            tail[120..128].clone_from_slice(&lengths);
            self.__process(__trim_to_64(&tail[0..64]));
            self.__process(__trim_to_64(&tail[64..128]));
        }

        [
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
        ]
    }
}

#[cfg(test)]
mod test {
    use std::println;

    use super::Sha1;

    #[test]
    fn sha1_test1() {
        let text = "The quick brown fox jumps over the lazy dog";
        let bytes = Sha1::from(text.as_bytes()).digest();
        for byte in bytes {
            print!("{:#04x}", byte);
        }
        println!();

        let text = "The quick brown fox jumps over the lazy cog";
        let bytes = Sha1::from(text.as_bytes()).digest();
        for byte in bytes {
            print!("{:#04x}", byte);
        }
    }
}
