pub struct Sha1 {
    // internal state 160(32*5)
    state: [u32; 5],
    // 512bits
    blocks: [u8; 64],
}

impl Default for Sha1 {
    fn default() -> Self {
        Self {
            state: [0; 5],
            blocks: [0; 64],
        }
    }
}

const HASH: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];

impl Sha1 {
    fn from<T: AsRef<u8>>(data: T) -> Self {
        let sha = Sha1::default();
        sha
    }
    // generate hash
    fn __generate(&mut self, data: &[u8]) {}
    // preprocess data into chunk
    fn __preprocess() {}
    // process data
    fn __process() {}
    // create output
    fn digest(&mut self) {}
}
