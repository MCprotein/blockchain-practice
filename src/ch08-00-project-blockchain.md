# 8장: 미니 프로젝트 — Rust로 블록체인 구현

## 프로젝트 개요

이 장에서는 1주차에 배운 내용을 중심으로 실제로 동작하는 블록체인을 Rust로 구현합니다.

> **읽는 방법**: 이 프로젝트에서는 `Result<T, E>`, `?` 연산자, `thiserror`, `Vec`의 이터레이터 등
> 아직 다루지 않은 개념이 일부 등장합니다. 지금은 문법을 모두 외우려고 하지 말고 **블록체인의 데이터 흐름**을 먼저 잡으세요.
> 문법은 2주차(에러 처리, 트레이트)와 3주차(컬렉션, 이터레이터)에서 본격적으로 배웁니다.

이 장에서 만들 프로그램은 실제 비트코인이나 이더리움처럼 네트워크 합의까지 구현하지 않습니다. 목표는 더 작습니다.

```text
데이터 문자열을 받는다
        ↓
Block 구조체에 담는다
        ↓
이전 블록 해시와 연결한다
        ↓
Proof of Work 조건을 만족할 때까지 nonce를 바꾼다
        ↓
체인 전체가 변조되지 않았는지 검증한다
```

즉, 이 장의 핵심은 “블록체인이 왜 변조를 감지할 수 있는가”를 코드로 확인하는 것입니다.

**구현 내용:**
- SHA-256 해싱
- Block 구조체와 Blockchain 구조체
- Proof of Work (PoW) 마이닝
- 체인 검증
- JSON 직렬화/역직렬화
- 커맨드라인 인터페이스

---

## 프로젝트 초기화

```bash
cargo new mini-blockchain
cd mini-blockchain
```

---

## Cargo.toml

```toml
[package]
name = "mini-blockchain"
version = "0.1.0"
edition = "2021"

[dependencies]
# SHA-256 해싱
sha2 = "0.10"

# 바이트 배열 ↔ 16진수 문자열
hex = "0.4"

# 직렬화/역직렬화
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 시간 처리
chrono = { version = "0.4", features = ["serde"] }

# 에러 처리
thiserror = "1.0"

# 로깅
log = "0.4"
env_logger = "0.10"
```

---

## 전체 프로젝트 구조

```text
mini-blockchain/
├── Cargo.toml
└── src/
    ├── main.rs          # 진입점, CLI
    ├── block.rs         # Block 구조체
    ├── blockchain.rs    # Blockchain 구조체
    ├── error.rs         # 에러 타입
    └── crypto.rs        # 해싱 유틸리티
```

파일별 책임을 먼저 잡고 들어가면 긴 코드가 덜 부담스럽습니다.

| 파일 | 책임 | 먼저 볼 질문 |
|------|------|--------------|
| `crypto.rs` | SHA-256 해시 계산 | 같은 입력이 항상 같은 해시가 되는가? |
| `block.rs` | 블록 하나의 데이터와 동작 | 블록 해시가 어떤 필드로 계산되는가? |
| `blockchain.rs` | 블록 목록과 검증 규칙 | 새 블록이 이전 블록과 어떻게 연결되는가? |
| `error.rs` | 실패 상황을 타입으로 표현 | 어떤 상황을 에러로 볼 것인가? |
| `main.rs` | CLI 실행 흐름 | 사용자가 어떤 명령으로 동작을 실행하는가? |

이 순서대로 읽으면 됩니다: `crypto.rs` → `block.rs` → `blockchain.rs` → `main.rs`.

---

## src/error.rs: 에러 타입

Rust는 예외를 던지는 대신 `Result<T, E>`로 성공과 실패를 값처럼 반환합니다. 아래 파일은 이 프로젝트에서 발생할 수 있는 실패를 `BlockchainError`라는 열거형으로 모아둡니다.

처음 보는 문법은 이렇게 읽으세요.

| 문법 | 뜻 |
|------|----|
| `enum BlockchainError` | 가능한 에러 종류를 하나의 타입으로 묶음 |
| `#[derive(Error, Debug)]` | `thiserror`가 에러 출력 코드를 자동 생성 |
| `#[error("...")]` | 사람이 읽을 에러 메시지 형식 |
| `pub type Result<T>` | 이 프로젝트 안에서 쓸 짧은 `Result` 별칭 |

```rust,ignore
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Invalid block at index {index}: {reason}")]
    InvalidBlock { index: u64, reason: String },

    #[error("Chain validation failed at block {0}")]
    ValidationFailed(u64),

    #[error("Block not found at height {0}")]
    BlockNotFound(u64),

    #[error("Mining failed: {0}")]
    MiningError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Empty blockchain")]
    EmptyChain,
}

pub type Result<T> = std::result::Result<T, BlockchainError>;
```

---

## src/crypto.rs: SHA-256 해싱

블록체인의 변조 감지는 해시에서 시작합니다. 이 파일은 “바이트 또는 문자열을 넣으면 SHA-256 해시 문자열을 돌려주는 작은 유틸리티”입니다.

```rust,ignore
use sha2::{Sha256, Digest};

/// 입력 데이터의 SHA-256 해시를 16진수 문자열로 반환
pub fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// 문자열의 SHA-256 해시
pub fn sha256_str(data: &str) -> String {
    sha256(data.as_bytes())
}

/// 여러 데이터를 연결한 SHA-256 해시
pub fn sha256_concat(parts: &[&str]) -> String {
    let combined = parts.join("");
    sha256_str(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_known_value() {
        // SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        let hash = sha256_str("hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_sha256_deterministic() {
        let h1 = sha256_str("blockchain");
        let h2 = sha256_str("blockchain");
        assert_eq!(h1, h2, "Same input must produce same hash");
    }

    #[test]
    fn test_sha256_different_inputs() {
        let h1 = sha256_str("block1");
        let h2 = sha256_str("block2");
        assert_ne!(h1, h2, "Different inputs must produce different hashes");
    }
}
```

---

## src/block.rs: Block 구조체

이 파일이 프로젝트의 중심입니다. `Block`은 하나의 블록을 표현합니다.

블록 필드는 다음 뜻입니다.

| 필드 | 뜻 |
|------|----|
| `index` | 체인에서 몇 번째 블록인지 나타내는 높이 |
| `timestamp` | 블록 생성 시각 |
| `data` | 이 미니 프로젝트에서 트랜잭션 대신 저장하는 문자열 |
| `previous_hash` | 바로 앞 블록의 해시 |
| `hash` | 이 블록 자체의 해시 |
| `nonce` | Proof of Work 조건을 맞추기 위해 바꾸는 숫자 |

실제 블록체인에서는 `data` 자리에 트랜잭션 목록과 머클 루트가 들어갑니다. 여기서는 처음 배우는 독자가 구조를 볼 수 있도록 문자열 하나로 단순화했습니다.

```rust,ignore
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::crypto::sha256_concat;

/// 블록체인의 단일 블록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// 블록 높이 (체인에서의 위치)
    pub index: u64,

    /// 블록 생성 시각 (Unix timestamp)
    pub timestamp: i64,

    /// 블록에 담긴 데이터 (실제 블록체인에서는 트랜잭션 목록)
    pub data: String,

    /// 이전 블록의 해시 (체인 연결)
    pub previous_hash: String,

    /// 이 블록의 해시
    pub hash: String,

    /// Proof of Work에서 사용한 논스값
    pub nonce: u64,
}

impl Block {
    /// 제네시스(첫 번째) 블록 생성
    pub fn genesis() -> Self {
        let mut block = Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            data: String::from("Genesis Block"),
            previous_hash: String::from("0000000000000000000000000000000000000000000000000000000000000000"),
            hash: String::new(),
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    /// 새 블록 생성 (아직 마이닝 전)
    pub fn new(index: u64, data: String, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    /// 블록의 SHA-256 해시 계산
    /// 해시 = SHA256(index + timestamp + data + previous_hash + nonce)
    pub fn calculate_hash(&self) -> String {
        sha256_concat(&[
            &self.index.to_string(),
            &self.timestamp.to_string(),
            &self.data,
            &self.previous_hash,
            &self.nonce.to_string(),
        ])
    }

    /// 해시가 올바른지 검증
    pub fn has_valid_hash(&self) -> bool {
        self.hash == self.calculate_hash()
    }

    /// 해시가 요구 난이도를 만족하는지 확인
    /// 난이도 N = 해시가 N개의 '0'으로 시작해야 함
    pub fn meets_difficulty(&self, difficulty: usize) -> bool {
        let target = "0".repeat(difficulty);
        self.hash.starts_with(&target)
    }

    /// Proof of Work 마이닝
    /// 요구 난이도를 만족하는 해시를 찾을 때까지 nonce 증가
    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        let started_at = std::time::Instant::now();

        println!(
            "Mining block #{} (difficulty: {}, target prefix: {})...",
            self.index, difficulty, target
        );

        // nonce를 0부터 증가시키며 목표 해시 탐색
        loop {
            self.hash = self.calculate_hash();
            if self.hash.starts_with(&target) {
                break;
            }
            self.nonce += 1;
        }

        let elapsed = started_at.elapsed();
        println!(
            "Block #{} mined in {:.2}s! Nonce: {}, Hash: {}...",
            self.index,
            elapsed.as_secs_f64(),
            self.nonce,
            &self.hash[..10]
        );
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block #{}
  Timestamp:     {}
  Data:          {}
  Previous Hash: {}...
  Hash:          {}...
  Nonce:         {}",
            self.index,
            self.timestamp,
            self.data,
            &self.previous_hash[..10],
            &self.hash[..10],
            self.nonce
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.index, 0);
        assert!(!genesis.hash.is_empty());
        assert!(genesis.has_valid_hash());
    }

    #[test]
    fn test_block_hash_changes_with_nonce() {
        let mut block = Block::new(1, "test".to_string(), "prev".to_string());
        let hash1 = block.calculate_hash();
        block.nonce += 1;
        let hash2 = block.calculate_hash();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_mining_meets_difficulty() {
        let mut block = Block::new(1, "test".to_string(), "0000".to_string());
        block.mine(2);  // difficulty 2 (해시가 "00"으로 시작)
        assert!(block.hash.starts_with("00"));
        assert!(block.has_valid_hash());
    }

    #[test]
    fn test_invalid_hash_detection() {
        let mut block = Block::genesis();
        block.data = String::from("tampered data");  // 데이터 변조
        // 해시를 재계산하지 않았으므로 has_valid_hash()는 false
        assert!(!block.has_valid_hash());
    }
}
```

---

## src/blockchain.rs: Blockchain 구조체

`Blockchain`은 `Block` 여러 개를 순서대로 보관하고, 새 블록을 추가하기 전에 연결 규칙을 검증합니다.

이 파일에서 확인할 핵심 규칙은 네 가지입니다.

1. 새 블록의 `index`는 마지막 블록보다 정확히 1 커야 한다.
2. 새 블록의 `previous_hash`는 마지막 블록의 `hash`와 같아야 한다.
3. 새 블록의 `hash`는 실제 필드값으로 다시 계산한 해시와 같아야 한다.
4. 새 블록의 `hash`는 현재 난이도 조건을 만족해야 한다.

이 네 가지가 지켜지면 “체인에 새 블록을 붙여도 된다”고 판단합니다.

```rust,ignore
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::block::Block;
use crate::error::{BlockchainError, Result};

/// 블록들의 체인
#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    /// 블록 목록 (인덱스 0이 제네시스)
    pub blocks: Vec<Block>,

    /// 채굴 난이도 (해시 앞의 0 개수)
    pub difficulty: usize,
}

impl Blockchain {
    /// 새 블록체인 생성 (제네시스 블록 포함)
    pub fn new(difficulty: usize) -> Self {
        println!("Creating new blockchain with difficulty {}...", difficulty);
        let genesis = Block::genesis();
        println!("Genesis block created: {}", &genesis.hash[..10]);

        Blockchain {
            blocks: vec![genesis],
            difficulty,
        }
    }

    /// 가장 마지막(최신) 블록 반환
    pub fn last_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    /// 블록 높이 반환
    pub fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// 새 블록 추가 (자동으로 마이닝)
    pub fn add_block(&mut self, data: String) -> Result<&Block> {
        let last = self.last_block()
            .ok_or(BlockchainError::EmptyChain)?;

        let index = last.index + 1;
        let previous_hash = last.hash.clone();

        let mut new_block = Block::new(index, data, previous_hash);
        new_block.mine(self.difficulty);

        // 추가 전 검증
        self.validate_new_block(&new_block)?;

        self.blocks.push(new_block);
        Ok(self.blocks.last().unwrap())
    }

    /// 새로 추가될 블록의 유효성 검증
    fn validate_new_block(&self, block: &Block) -> Result<()> {
        let last = self.last_block()
            .ok_or(BlockchainError::EmptyChain)?;

        // 1. 인덱스 확인
        if block.index != last.index + 1 {
            return Err(BlockchainError::InvalidBlock {
                index: block.index,
                reason: format!(
                    "Expected index {}, got {}",
                    last.index + 1,
                    block.index
                ),
            });
        }

        // 2. 이전 해시 확인
        if block.previous_hash != last.hash {
            return Err(BlockchainError::InvalidBlock {
                index: block.index,
                reason: format!(
                    "Invalid previous hash: expected {}, got {}",
                    &last.hash[..10],
                    &block.previous_hash[..10]
                ),
            });
        }

        // 3. 해시 유효성 확인
        if !block.has_valid_hash() {
            return Err(BlockchainError::InvalidBlock {
                index: block.index,
                reason: String::from("Hash does not match block data"),
            });
        }

        // 4. 난이도 충족 확인
        if !block.meets_difficulty(self.difficulty) {
            return Err(BlockchainError::InvalidBlock {
                index: block.index,
                reason: format!(
                    "Hash does not meet difficulty {}",
                    self.difficulty
                ),
            });
        }

        Ok(())
    }

    /// 전체 체인 유효성 검증
    pub fn validate(&self) -> Result<()> {
        // 제네시스 블록 검증
        if self.blocks.is_empty() {
            return Err(BlockchainError::EmptyChain);
        }

        let genesis = &self.blocks[0];
        if !genesis.has_valid_hash() {
            return Err(BlockchainError::ValidationFailed(0));
        }

        // 나머지 블록들 검증
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            // 해시 유효성
            if !current.has_valid_hash() {
                return Err(BlockchainError::ValidationFailed(current.index));
            }

            // 이전 해시 연결성
            if current.previous_hash != previous.hash {
                return Err(BlockchainError::ValidationFailed(current.index));
            }

            // 인덱스 순서
            if current.index != previous.index + 1 {
                return Err(BlockchainError::ValidationFailed(current.index));
            }
        }

        Ok(())
    }

    /// 특정 높이의 블록 조회
    pub fn get_block(&self, height: u64) -> Result<&Block> {
        self.blocks.get(height as usize)
            .ok_or(BlockchainError::BlockNotFound(height))
    }

    /// JSON으로 직렬화
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(BlockchainError::from)
    }

    /// JSON에서 역직렬화
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(BlockchainError::from)
    }

    /// 체인 요약 출력
    pub fn print_summary(&self) {
        println!("\n=== Blockchain Summary ===");
        println!("Height:     {} blocks", self.height());
        println!("Difficulty: {}", self.difficulty);
        println!("Valid:      {}", self.validate().is_ok());
        println!("==========================\n");
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Blockchain (difficulty: {}) ===", self.difficulty)?;
        for block in &self.blocks {
            writeln!(f, "{}", block)?;
            writeln!(f, "  {}", "-".repeat(50))?;
        }
        write!(f, "Total blocks: {}", self.blocks.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chain() -> Blockchain {
        let mut chain = Blockchain::new(1);  // 낮은 난이도로 빠른 테스트
        chain.add_block("Alice → Bob: 1 BTC".to_string()).unwrap();
        chain.add_block("Bob → Carol: 0.5 BTC".to_string()).unwrap();
        chain
    }

    #[test]
    fn test_new_blockchain_has_genesis() {
        let chain = Blockchain::new(1);
        assert_eq!(chain.height(), 1);
        assert_eq!(chain.blocks[0].index, 0);
    }

    #[test]
    fn test_add_block_increases_height() {
        let mut chain = Blockchain::new(1);
        chain.add_block("tx1".to_string()).unwrap();
        chain.add_block("tx2".to_string()).unwrap();
        assert_eq!(chain.height(), 3);
    }

    #[test]
    fn test_chain_validation_passes_for_valid_chain() {
        let chain = create_test_chain();
        assert!(chain.validate().is_ok());
    }

    #[test]
    fn test_chain_validation_fails_after_tampering() {
        let mut chain = create_test_chain();

        // 블록 데이터 변조
        chain.blocks[1].data = String::from("TAMPERED: Alice → Attacker: 1 BTC");
        // 해시는 재계산하지 않음 → 검증 실패

        assert!(chain.validate().is_err());
    }

    #[test]
    fn test_blocks_are_linked() {
        let chain = create_test_chain();
        for i in 1..chain.blocks.len() {
            assert_eq!(
                chain.blocks[i].previous_hash,
                chain.blocks[i - 1].hash,
                "Block {} should reference block {}'s hash",
                i, i - 1
            );
        }
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let chain = create_test_chain();
        let json = chain.to_json().unwrap();
        let restored = Blockchain::from_json(&json).unwrap();

        assert_eq!(chain.blocks.len(), restored.blocks.len());
        assert_eq!(chain.blocks[0].hash, restored.blocks[0].hash);
    }

    #[test]
    fn test_get_block_by_height() {
        let chain = create_test_chain();
        assert!(chain.get_block(0).is_ok());
        assert!(chain.get_block(1).is_ok());
        assert!(chain.get_block(99).is_err());
    }
}
```

---

## src/main.rs: 진입점과 CLI

```rust,ignore
mod block;
mod blockchain;
mod crypto;
mod error;

use blockchain::Blockchain;
use std::env;

fn print_usage() {
    println!("Usage: mini-blockchain <command>");
    println!();
    println!("Commands:");
    println!("  demo          — Run a demonstration");
    println!("  mine <data>   — Mine a new block with the given data");
    println!("  verify        — Create and verify a chain");
    println!("  bench         — Benchmark different difficulties");
}

fn run_demo() {
    println!("╔═══════════════════════════════════╗");
    println!("║    Mini Blockchain in Rust 🦀      ║");
    println!("╚═══════════════════════════════════╝\n");

    // 난이도 2로 블록체인 생성
    let mut chain = Blockchain::new(2);

    // 트랜잭션 데이터 추가
    let transactions = vec![
        "Alice → Bob: 1.5 BTC",
        "Bob → Carol: 0.5 BTC",
        "Carol → Dave: 0.1 BTC",
    ];

    for tx in transactions {
        println!("Adding transaction: {}", tx);
        match chain.add_block(tx.to_string()) {
            Ok(block) => println!("  ✓ Block #{} added (hash: {}...)\n", block.index, &block.hash[..10]),
            Err(e)    => eprintln!("  ✗ Failed: {}\n", e),
        }
    }

    // 체인 출력
    println!("{}", chain);
    chain.print_summary();

    // 체인 검증
    println!("Validating chain...");
    match chain.validate() {
        Ok(()) => println!("✓ Chain is valid!\n"),
        Err(e) => println!("✗ Chain is invalid: {}\n", e),
    }

    // 변조 시도
    println!("Attempting to tamper with block #1...");
    chain.blocks[1].data = String::from("ATTACKER → Attacker: 1000 BTC");
    // 해시를 재계산하지 않음

    println!("Validating tampered chain...");
    match chain.validate() {
        Ok(()) => println!("✗ Chain accepted tampered data! (This should not happen)"),
        Err(e) => println!("✓ Tampering detected: {}\n", e),
    }

    // JSON 직렬화 데모
    println!("Serializing blockchain to JSON...");
    // 원본 체인으로 복원
    let mut clean_chain = Blockchain::new(2);
    clean_chain.add_block("Alice → Bob: 1 BTC".to_string()).unwrap();

    match clean_chain.to_json() {
        Ok(json) => {
            println!("JSON (first 200 chars): {}...\n", &json[..200.min(json.len())]);

            // 역직렬화
            match Blockchain::from_json(&json) {
                Ok(restored) => println!("✓ Deserialized: {} blocks\n", restored.blocks.len()),
                Err(e)       => println!("✗ Deserialization failed: {}\n", e),
            }
        }
        Err(e) => println!("✗ Serialization failed: {}\n", e),
    }
}

fn run_verify() {
    println!("Creating and verifying a 3-block chain...\n");

    let mut chain = Blockchain::new(2);
    chain.add_block("Block 1 data".to_string()).unwrap();
    chain.add_block("Block 2 data".to_string()).unwrap();

    println!("{}", chain);

    match chain.validate() {
        Ok(())  => println!("✓ Chain valid"),
        Err(e)  => println!("✗ Invalid: {}", e),
    }
}

fn run_bench() {
    println!("Benchmarking mining at different difficulties...\n");

    for difficulty in 1..=4 {
        let start = std::time::Instant::now();
        let mut chain = Blockchain::new(difficulty);
        chain.add_block(format!("Benchmark block at difficulty {}", difficulty)).unwrap();
        let elapsed = start.elapsed();

        let block = chain.last_block().unwrap();
        println!(
            "Difficulty {}: {:.3}s, nonce={}, hash={}...",
            difficulty,
            elapsed.as_secs_f64(),
            block.nonce,
            &block.hash[..10]
        );
    }
}

fn main() {
    // 로거 초기화 (RUST_LOG 환경변수로 제어)
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("demo");

    match command {
        "demo" => run_demo(),
        "mine" => {
            let data = args.get(2).cloned().unwrap_or_else(|| "Default block data".to_string());
            let mut chain = Blockchain::new(2);
            match chain.add_block(data) {
                Ok(block) => println!("Mined: {}", block),
                Err(e)    => eprintln!("Error: {}", e),
            }
        }
        "verify" => run_verify(),
        "bench"  => run_bench(),
        _        => print_usage(),
    }
}
```

---

## 실행 방법

```bash
# 데모 실행
cargo run -- demo

# 특정 데이터로 마이닝
cargo run -- mine "Alice → Bob: 2.5 BTC"

# 체인 검증
cargo run -- verify

# 성능 벤치마크
cargo run -- bench

# 릴리스 빌드 (훨씬 빠름, PoW 마이닝은 꼭 릴리스로)
cargo build --release
./target/release/mini-blockchain bench

# 테스트 실행
cargo test

# 특정 테스트만
cargo test blockchain::tests::test_chain_validation
cargo test block::tests

# 로그 출력 포함
RUST_LOG=debug cargo run -- demo
```

---

## 예상 출력

```text
╔═══════════════════════════════════╗
║    Mini Blockchain in Rust 🦀      ║
╚═══════════════════════════════════╝

Creating new blockchain with difficulty 2...
Genesis block created: 4b227777d...

Adding transaction: Alice → Bob: 1.5 BTC
Mining block #1 (difficulty: 2, target prefix: 00)...
Block #1 mined in 0.001s! Nonce: 127, Hash: 003f7a2b1...
  ✓ Block #1 added (hash: 003f7a2b1...)

Adding transaction: Bob → Carol: 0.5 BTC
Mining block #2 (difficulty: 2, target prefix: 00)...
Block #2 mined in 0.003s! Nonce: 432, Hash: 00ab12c3d...
  ✓ Block #2 added (hash: 00ab12c3d...)

Adding transaction: Carol → Dave: 0.1 BTC
Mining block #3 (difficulty: 2, target prefix: 00)...
Block #3 mined in 0.002s! Nonce: 89, Hash: 006ef4a11...
  ✓ Block #3 added (hash: 006ef4a11...)

=== Blockchain (difficulty: 2) ===
Block #0
  Timestamp:     1700000000
  Data:          Genesis Block
  Previous Hash: 0000000000...
  Hash:          4b227777d4...
  Nonce:         0
  --------------------------------------------------
...

=== Blockchain Summary ===
Height:     4 blocks
Difficulty: 2
Valid:      true
==========================

Validating chain...
✓ Chain is valid!

Attempting to tamper with block #1...
Validating tampered chain...
✓ Tampering detected: Chain validation failed at block 1
```

---

## 핵심 개념 설명

### Proof of Work (작업 증명)

PoW는 "이 정도의 계산 작업을 했음"을 증명하는 메커니즘입니다:

```text
목표: 해시가 "00..."으로 시작하는 nonce 찾기
→ 평균적으로 256번 시도 (difficulty=2)
→ 계산 비용이 있어서 악의적인 체인 재작성을 어렵게 함
→ 검증은 한 번의 해시 계산으로 O(1)
```

```rust,ignore
// 마이닝: O(2^(4*difficulty)) 평균 시도
fn mine(&mut self, difficulty: usize) {
    let target = "0".repeat(difficulty);
    while !self.hash.starts_with(&target) {
        self.nonce += 1;
        self.hash = self.calculate_hash();
    }
}

// 검증: O(1)
fn has_valid_hash(&self) -> bool {
    self.hash == self.calculate_hash()
}
```

### 해시 체인의 불변성

블록 N의 해시는 블록 N-1의 해시를 포함합니다:

```text
Block 0 (Genesis)
  hash = SHA256("0" + timestamp + "Genesis Block" + "000...0" + "0")
  hash = "4b22..."

Block 1
  hash = SHA256("1" + timestamp + "Alice → Bob" + "4b22..." + nonce)
  hash = "003f..."

Block 2
  hash = SHA256("2" + timestamp + "Bob → Carol" + "003f..." + nonce)
  hash = "00ab..."
```

블록 1의 데이터를 바꾸면:
- 블록 1의 해시가 바뀜
- 블록 2의 `previous_hash`가 틀려짐
- 블록 2 이후 모든 블록을 다시 마이닝해야 함 → 실용적으로 불가능

---

## 확장 아이디어

이 미니 블록체인을 확장해볼 수 있는 방향들:

### 1. 트랜잭션 구조체 추가

```rust,ignore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub signature: String,
}

pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,  // data 대신
}
```

### 2. Merkle Tree 루트 해시

```rust,ignore
impl Block {
    pub fn merkle_root(&self) -> String {
        let tx_hashes: Vec<String> = self.transactions.iter()
            .map(|tx| sha256_str(&serde_json::to_string(tx).unwrap()))
            .collect();
        compute_merkle_root(tx_hashes)
    }
}
```

### 3. 비동기 마이닝 (Tokio)

```rust,ignore
use tokio::task;

impl Block {
    pub async fn mine_async(&mut self, difficulty: usize) {
        // CPU 집약적 작업은 spawn_blocking으로 별도 스레드에서
        let block = self.clone();
        let mined = task::spawn_blocking(move || {
            let mut b = block;
            b.mine(difficulty);
            b
        }).await.unwrap();

        *self = mined;
    }
}
```

### 4. P2P 네트워크 (Tokio TCP)

```rust,ignore
async fn handle_peer(
    mut stream: TcpStream,
    state: Arc<RwLock<Blockchain>>,
) {
    // 피어로부터 새 블록 수신 및 검증
    let mut buf = vec![0u8; 65536];
    let n = stream.read(&mut buf).await.unwrap();
    let block: Block = serde_json::from_slice(&buf[..n]).unwrap();

    let mut chain = state.write().await;
    // 블록 검증 및 추가
}
```

### 5. 지갑과 서명 (ed25519)

```toml
[dependencies]
ed25519-dalek = "2.0"
rand = "0.8"
```

```rust,ignore
use ed25519_dalek::{Keypair, Signer, Verifier};

fn create_wallet() -> Keypair {
    let mut rng = rand::thread_rng();
    Keypair::generate(&mut rng)
}

fn sign_transaction(keypair: &Keypair, tx_data: &str) -> String {
    let signature = keypair.sign(tx_data.as_bytes());
    hex::encode(signature.to_bytes())
}
```

---

## 요약

이 장에서 구현한 것:

1. **SHA-256 해싱** (`sha2` 크레이트) — 암호학적 해시 함수
2. **Block 구조체** — 인덱스, 타임스탬프, 데이터, 이전 해시, 현재 해시, 논스
3. **Proof of Work** — 목표 난이도를 충족하는 해시 탐색
4. **Blockchain 구조체** — 블록 목록, 검증 로직
5. **체인 불변성** — 해시 체인으로 변조 감지
6. **JSON 직렬화** — `serde`/`serde_json`으로 영속화
7. **테스트** — 단위 테스트로 핵심 로직 검증
8. **CLI** — `env::args()`로 커맨드라인 인터페이스

**사용된 Rust 개념들:**
- 구조체와 `impl` 블록
- 열거형과 패턴 매칭 (에러 처리)
- `Result<T, E>`와 `?` 연산자
- `thiserror`로 커스텀 에러 타입
- `#[derive(Debug, Clone, Serialize, Deserialize)]`
- 이터레이터와 클로저
- 소유권과 참조 (`&self`, `&mut self`, `.clone()`)
- 모듈 시스템 (`mod`, `pub`, `use`)

---

*1주차를 완료했습니다! Rust 기초와 블록체인 핵심 구조를 직접 구현해봤습니다. 2주차에서는 에러 처리, 트레이트 등 Rust를 더 깊이 배우면서 이더리움과 Solidity를 시작합니다.*
