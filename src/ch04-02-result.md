# 4.2 Result\<T, E\>

## Result 타입이란?

`Result<T, E>`는 성공(`Ok(T)`) 또는 실패(`Err(E)`)를 나타내는 열거형입니다:

```rust
// 표준 라이브러리에 이렇게 정의되어 있음
enum Result<T, E> {
    Ok(T),   // 성공: 값 T를 담고 있음
    Err(E),  // 실패: 에러 E를 담고 있음
}
```

TypeScript의 `Promise<T>`와 비슷하지만, 비동기가 아닙니다. 단순히 "성공 또는 실패"를 타입으로 표현합니다.

```typescript
// TypeScript: 에러는 예외로 처리 (타입에 드러나지 않음)
async function parseBlockHeight(s: string): Promise<number> {
    const n = parseInt(s);
    if (isNaN(n)) throw new Error(`Invalid height: ${s}`);
    return n;
}

// 또는 명시적으로 Result 패턴을 흉내내기도 함
type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };
```

```rust
// Rust: 에러가 반환 타입에 명시됨
fn parse_block_height(s: &str) -> Result<u64, String> {
    s.parse::<u64>().map_err(|e| format!("Invalid height: {}", e))
}
```

---

## Result 반환하기

```rust
use std::num::ParseIntError;

fn parse_height(s: &str) -> Result<u64, ParseIntError> {
    let n = s.parse::<u64>()?;  // ? 연산자: 에러면 즉시 반환
    Ok(n)
}

// 또는 직접 Ok/Err 반환
fn validate_block_index(index: u64, chain_len: usize) -> Result<(), String> {
    if index as usize != chain_len {
        return Err(format!(
            "Expected index {}, got {}",
            chain_len, index
        ));
    }
    Ok(())  // 성공, 반환할 값 없음
}
```

---

## Result 처리하기

### 1. match로 처리

```rust
fn main() {
    let result = parse_height("42");

    match result {
        Ok(height) => println!("Height: {}", height),
        Err(e)     => println!("Error: {}", e),
    }

    // 에러에 따른 다른 처리
    match "abc".parse::<u64>() {
        Ok(n)  => println!("Parsed: {}", n),
        Err(e) => {
            eprintln!("Parse error: {}", e);
            // 기본값 사용, 재시도, 로그 등
        }
    }
}
```

### 2. unwrap() — 빠르게 쓰되 주의

```rust
fn main() {
    // Ok이면 값 반환, Err이면 panic!
    let height = "42".parse::<u64>().unwrap();
    println!("{}", height);  // 42

    // Err이면 panic
    // let bad = "abc".parse::<u64>().unwrap();
    // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: ...'
}
```

`unwrap()`은 프로토타입, 테스트, 또는 절대 실패하지 않는다고 확신할 때만 씁니다.

### 3. expect() — 더 나은 에러 메시지

```rust
fn main() {
    let config_path = std::env::var("CONFIG_PATH")
        .expect("CONFIG_PATH environment variable must be set");

    let port: u16 = std::env::var("PORT")
        .unwrap_or(String::from("8080"))
        .parse()
        .expect("PORT must be a valid port number (1-65535)");
}
```

### 4. unwrap_or() — 기본값

```rust
fn main() {
    let height: u64 = "abc".parse().unwrap_or(0);
    println!("{}", height);  // 0 (파싱 실패 시 기본값)

    let height2: u64 = "abc".parse().unwrap_or_default();
    println!("{}", height2);  // 0 (u64의 기본값)

    let height3: u64 = "abc".parse().unwrap_or_else(|e| {
        eprintln!("Parse failed: {}, using default", e);
        0
    });
}
```

### 5. map() — Ok 값 변환

```rust
fn main() {
    // Ok(42) → Ok("42 blocks")
    let result: Result<String, _> = "42".parse::<u64>()
        .map(|n| format!("{} blocks", n));
    println!("{:?}", result);  // Ok("42 blocks")

    // Err는 그대로 통과
    let result2: Result<String, _> = "abc".parse::<u64>()
        .map(|n| format!("{} blocks", n));
    println!("{:?}", result2);  // Err(...)
}
```

### 6. map_err() — Err 값 변환

```rust
#[derive(Debug)]
enum AppError {
    ParseError(String),
    NetworkError(String),
}

fn parse_height(s: &str) -> Result<u64, AppError> {
    s.parse::<u64>()
        .map_err(|e| AppError::ParseError(format!("Cannot parse '{}': {}", s, e)))
}
```

### 7. and_then() — Ok일 때 다음 연산 (flatMap)

```rust
fn get_block_hash(height_str: &str, blockchain: &Blockchain) -> Result<String, AppError> {
    "42".parse::<u64>()
        .map_err(|e| AppError::ParseError(e.to_string()))
        .and_then(|height| {
            blockchain.get_block(height)
                .map(|block| block.hash.clone())
                .ok_or(AppError::NotFound(format!("Block {} not found", height)))
        })
}
```

### 8. is_ok(), is_err()

```rust
fn main() {
    let ok: Result<i32, &str> = Ok(42);
    let err: Result<i32, &str> = Err("oops");

    println!("{}", ok.is_ok());   // true
    println!("{}", ok.is_err());  // false
    println!("{}", err.is_ok());  // false
    println!("{}", err.is_err()); // true
}
```

---

## 커스텀 에러 타입 만들기

### 방법 1: 단순 String 에러 (간단하지만 제한적)

```rust
fn parse(s: &str) -> Result<u64, String> {
    s.parse::<u64>().map_err(|e| e.to_string())
}
```

### 방법 2: 열거형 에러 타입 (권장)

```rust
use std::fmt;

#[derive(Debug)]
enum BlockchainError {
    InvalidBlockIndex { expected: u64, got: u64 },
    HashMismatch { expected: String, actual: String },
    InvalidProofOfWork { hash: String, difficulty: usize },
    EmptyChain,
    SerializationError(String),
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockchainError::InvalidBlockIndex { expected, got } => {
                write!(f, "Invalid block index: expected {}, got {}", expected, got)
            }
            BlockchainError::HashMismatch { expected, actual } => {
                write!(f, "Hash mismatch: expected {}, got {}", expected, actual)
            }
            BlockchainError::InvalidProofOfWork { hash, difficulty } => {
                write!(f, "Hash '{}' doesn't meet difficulty {}", hash, difficulty)
            }
            BlockchainError::EmptyChain => {
                write!(f, "Blockchain is empty")
            }
            BlockchainError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
        }
    }
}

// std::error::Error 트레이트 구현 (선택사항이지만 관례)
impl std::error::Error for BlockchainError {}
```

### 방법 3: thiserror 크레이트 (가장 편리, 권장)

`thiserror`는 에러 타입 정의를 매크로로 단순화합니다:

```toml
# Cargo.toml
[dependencies]
thiserror = "1.0"
```

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum BlockchainError {
    #[error("Invalid block index: expected {expected}, got {got}")]
    InvalidBlockIndex { expected: u64, got: u64 },

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Hash '{hash}' doesn't meet difficulty {difficulty}")]
    InvalidProofOfWork { hash: String, difficulty: usize },

    #[error("Blockchain is empty")]
    EmptyChain,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    // 다른 에러를 감싸기 (#[from] — From 트레이트 자동 구현)
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

`thiserror`를 쓰면 `Display`와 `Error` 트레이트가 자동으로 구현됩니다.

---

## 실제 블록체인 코드에서의 에러 처리

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Block not found at height {0}")]
    BlockNotFound(u64),

    #[error("Chain validation failed at block {0}")]
    ValidationFailed(u64),

    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct Blockchain {
    blocks: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    pub fn add_block(&mut self, data: String) -> Result<&Block, BlockchainError> {
        let last_block = self.blocks.last()
            .ok_or(BlockchainError::InvalidBlock("Chain is empty".to_string()))?;

        let new_index = last_block.index + 1;
        let previous_hash = last_block.hash.clone();

        let mut new_block = Block::new(new_index, data, previous_hash);
        new_block.mine(self.difficulty);

        // 검증
        if !new_block.is_valid() {
            return Err(BlockchainError::InvalidBlock(
                format!("Block {} failed validation", new_index)
            ));
        }

        self.blocks.push(new_block);
        Ok(self.blocks.last().unwrap()) // unwrap OK: 방금 push했으므로 항상 Some
    }

    pub fn get_block(&self, height: u64) -> Result<&Block, BlockchainError> {
        self.blocks.get(height as usize)
            .ok_or(BlockchainError::BlockNotFound(height))
    }

    pub fn validate(&self) -> Result<(), BlockchainError> {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            if current.previous_hash != previous.hash {
                return Err(BlockchainError::ValidationFailed(current.index));
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, BlockchainError> {
        // serde_json::Error가 #[from]으로 BlockchainError::Serialization으로 자동 변환
        Ok(serde_json::to_string(self)?)
    }
}

fn main() {
    let mut chain = Blockchain::new();

    match chain.add_block(String::from("Alice sends 1 BTC to Bob")) {
        Ok(block) => println!("Added block #{}", block.index),
        Err(e) => eprintln!("Failed to add block: {}", e),
    }

    match chain.validate() {
        Ok(()) => println!("Chain is valid"),
        Err(BlockchainError::ValidationFailed(height)) => {
            eprintln!("Chain invalid at block {}", height);
        }
        Err(e) => eprintln!("Validation error: {}", e),
    }
}
```

---

## anyhow: 애플리케이션 코드용 에러 처리

`thiserror`가 라이브러리 에러 타입 정의에 쓰인다면, `anyhow`는 애플리케이션의 main/bin 코드에서 편리하게 씁니다:

```toml
[dependencies]
anyhow = "1.0"
```

```rust
use anyhow::{Context, Result, anyhow, bail};

fn main() -> Result<()> {  // anyhow::Result = Result<T, anyhow::Error>
    let config_path = std::env::args().nth(1)
        .ok_or_else(|| anyhow!("Usage: program <config-path>"))?;

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config from '{}'", config_path))?;

    let config: serde_json::Value = serde_json::from_str(&content)
        .context("Config file must be valid JSON")?;

    let port = config["port"].as_u64()
        .ok_or_else(|| anyhow!("Config must have 'port' field"))?;

    if port > 65535 {
        bail!("Port {} is out of range", port);  // bail! = return Err(anyhow!(...))
    }

    println!("Starting server on port {}", port);
    Ok(())
}
```

| 크레이트 | 사용 케이스 |
|---------|-----------|
| `thiserror` | 라이브러리, 구체적인 에러 타입이 필요한 경우 |
| `anyhow` | 바이너리/애플리케이션, 에러 타입이 다양하게 섞이는 경우 |

---

## 요약

- `Result<T, E>`: 성공(`Ok(T)`) 또는 실패(`Err(E)`)를 타입으로 표현
- 처리 방법: `match`, `unwrap()`, `expect()`, `unwrap_or()`, `map()`, `and_then()`
- 커스텀 에러: `thiserror` 크레이트 권장 (Display, Error 자동 구현)
- 애플리케이션 코드: `anyhow` 크레이트 편리
- `unwrap()`은 테스트/프로토타입에서만, 프로덕션에서는 `?` 연산자 사용

다음 챕터에서 `?` 연산자로 에러를 간결하게 전파하는 방법을 배웁니다.
