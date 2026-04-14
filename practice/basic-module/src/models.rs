pub struct Block {
    pub index: u64,
    pub hash: String,
}

impl Block {
    pub fn new() -> Self {
        Block {
            index: 0,
            hash: String::new(),
        }
    }
}
