pub struct Sha1 {
    // internal state 160(32*5)
    state: [u32; 5],
    // 512bits
    blocks: [u8; 64],

    len: usize,
    blocklen: usize,
}

impl Default for Sha1 {
    fn default() -> Self {
        Self {
            state: [0; 5],
            len: 0,
            blocks: [0; 64],
            blocklen: 0,
        }
    }
}

const HASH: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];

impl Sha1 {
    fn from<T: AsRef<[u8]>>(data: T) -> Self {
        let mut sha = Sha1::default();
        sha.__generate(data.as_ref());
        sha
    }
    // generate hash
    fn __generate(&mut self, data: &[u8]) {}
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
                // process(block);

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

                *lenptr += chunk.len()
                // same with += 64

                // process(chunk);
            } else {
                self.blocks[..chunk.len()].clone_from_slice(chunk);
                self.blocklen = chunk.len()
            }
        }
    }
    // process data
    fn __process() {}
    // create output
    fn digest(&mut self) {}
}
