use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub index: u64,
    pub data: String,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}

impl Block {
    fn mine(index: u64, data: String, previous_hash: String, difficulty: usize) -> Self {
        let target = "0".repeat(difficulty);
        let mut nonce = 0;

        loop {
            let hash = calculate_hash(index, &data, &previous_hash, nonce);
            if hash.starts_with(&target) {
                return Self {
                    index,
                    data,
                    previous_hash,
                    nonce,
                    hash,
                };
            }
            nonce = nonce.checked_add(1).expect("nonce 범위를 초과했습니다");
        }
    }

    fn has_valid_hash(&self, difficulty: usize) -> bool {
        self.hash == calculate_hash(self.index, &self.data, &self.previous_hash, self.nonce)
            && self.hash.starts_with(&"0".repeat(difficulty))
    }
}

#[derive(Debug)]
pub struct Blockchain {
    blocks: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        assert!(
            (1..=4).contains(&difficulty),
            "학습용 난이도는 1부터 4까지만 지원합니다"
        );
        let genesis = Block::mine(0, "genesis".into(), "0".repeat(64), difficulty);
        Self {
            blocks: vec![genesis],
            difficulty,
        }
    }

    pub fn add_block(&mut self, data: impl Into<String>) {
        let previous = self
            .blocks
            .last()
            .expect("체인에는 제네시스 블록이 있습니다");
        let next = Block::mine(
            previous.index + 1,
            data.into(),
            previous.hash.clone(),
            self.difficulty,
        );
        self.blocks.push(next);
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn is_valid(&self) -> bool {
        let Some(genesis) = self.blocks.first() else {
            return false;
        };

        if genesis.index != 0
            || genesis.data != "genesis"
            || genesis.previous_hash != "0".repeat(64)
            || !genesis.has_valid_hash(self.difficulty)
        {
            return false;
        }

        self.blocks.windows(2).all(|pair| {
            let previous = &pair[0];
            let current = &pair[1];
            current.index == previous.index + 1
                && current.previous_hash == previous.hash
                && current.has_valid_hash(self.difficulty)
        })
    }
}

fn calculate_hash(index: u64, data: &str, previous_hash: &str, nonce: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(index.to_be_bytes());
    hasher.update((data.len() as u64).to_be_bytes());
    hasher.update(data.as_bytes());
    hasher.update(previous_hash.as_bytes());
    hasher.update(nonce.to_be_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 새_블록을_추가하면_체인이_유효하다() {
        let mut chain = Blockchain::new(2);
        chain.add_block("alice -> bob: 10");
        chain.add_block("bob -> carol: 3");

        assert_eq!(chain.blocks().len(), 3);
        assert!(chain.is_valid());
    }

    #[test]
    fn 과거_데이터를_바꾸면_검증에_실패한다() {
        let mut chain = Blockchain::new(2);
        chain.add_block("alice -> bob: 10");
        chain.blocks[1].data = "alice -> bob: 1000".into();

        assert!(!chain.is_valid());
    }

    #[test]
    fn 인덱스를_바꾸면_검증에_실패한다() {
        let mut chain = Blockchain::new(1);
        chain.add_block("first");
        chain.blocks[1].index = 9;

        assert!(!chain.is_valid());
    }

    #[test]
    fn 이전_해시를_바꾸면_검증에_실패한다() {
        let mut chain = Blockchain::new(1);
        chain.add_block("first");
        chain.blocks[1].previous_hash = "f".repeat(64);

        assert!(!chain.is_valid());
    }

    #[test]
    fn nonce를_바꾸면_검증에_실패한다() {
        let mut chain = Blockchain::new(1);
        chain.add_block("first");
        chain.blocks[1].nonce += 1;

        assert!(!chain.is_valid());
    }

    #[test]
    fn 저장된_해시를_바꾸면_검증에_실패한다() {
        let mut chain = Blockchain::new(1);
        chain.add_block("first");
        chain.blocks[1].hash = "0".repeat(64);

        assert!(!chain.is_valid());
    }

    #[test]
    fn 이전_해시가_블록을_연결한다() {
        let mut chain = Blockchain::new(1);
        chain.add_block("first");
        chain.add_block("second");

        assert_eq!(chain.blocks()[2].previous_hash, chain.blocks()[1].hash);
    }

    #[test]
    #[should_panic(expected = "학습용 난이도는 1부터 4까지만 지원합니다")]
    fn 지나치게_큰_난이도를_거부한다() {
        Blockchain::new(5);
    }

    #[test]
    #[should_panic(expected = "학습용 난이도는 1부터 4까지만 지원합니다")]
    fn 난이도_0을_거부한다() {
        Blockchain::new(0);
    }
}
