# 3.1 구조체 (Structs)

## 구조체 정의

구조체는 관련된 데이터를 하나로 묶는 타입입니다.

```rust,ignore
// 기본 구조체 정의
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u64,
}
```

TypeScript의 interface/class와 비교:

```typescript
// TypeScript interface
interface Block {
    index: number;
    timestamp: number;
    data: string;
    previousHash: string;
    hash: string;
    nonce: number;
}

// TypeScript class
class Block {
    index: number;
    timestamp: number;
    data: string;
    previousHash: string;
    hash: string;
    nonce: number;

    constructor(index: number, data: string, previousHash: string) {
        this.index = index;
        this.timestamp = Date.now();
        this.data = data;
        this.previousHash = previousHash;
        this.hash = "";
        this.nonce = 0;
    }
}
```

---

## 구조체 인스턴스 생성

```rust,ignore
fn main() {
    // 모든 필드를 지정해서 생성
    let block = Block {
        index: 0,
        timestamp: 1700000000,
        data: String::from("Genesis Block"),
        previous_hash: String::from("0000000000000000"),
        hash: String::from("abc123"),
        nonce: 0,
    };

    // 필드 접근
    println!("Block #{}: {}", block.index, block.hash);
    println!("Data: {}", block.data);
}
```

**주의**: 구조체 인스턴스를 수정하려면 변수 자체가 `mut`이어야 합니다:

```rust,ignore
fn main() {
    let mut block = Block {
        index: 0,
        timestamp: 1700000000,
        data: String::from("Genesis Block"),
        previous_hash: String::from("0000000000000000"),
        hash: String::from(""),
        nonce: 0,
    };

    // 필드 수정 (mut이므로 가능)
    block.hash = String::from("computed_hash");
    block.nonce = 42;

    println!("Hash: {}, Nonce: {}", block.hash, block.nonce);
}
```

TypeScript는 `const` 객체도 내부 필드를 수정할 수 있지만, Rust는 변수가 `mut`이어야 합니다.

---

## 필드 초기화 단축 문법

함수 매개변수 이름과 구조체 필드 이름이 같으면 단축 문법을 쓸 수 있습니다:

```rust,ignore
fn create_block(index: u64, data: String, previous_hash: String) -> Block {
    let timestamp = get_current_timestamp();
    let nonce = 0;
    let hash = compute_hash(index, timestamp, &data, &previous_hash, nonce);

    Block {
        index,          // index: index 대신
        timestamp,      // timestamp: timestamp 대신
        data,           // data: data 대신
        previous_hash,  // previous_hash: previous_hash 대신
        hash,
        nonce,
    }
}
```

TypeScript의 shorthand property와 동일합니다:

```typescript
// TypeScript
function createBlock(index: number, data: string): Block {
    const timestamp = Date.now();
    return { index, timestamp, data };  // shorthand
}
```

---

## 구조체 업데이트 문법

기존 인스턴스의 일부 필드만 변경한 새 인스턴스를 만들 때:

```rust,ignore
fn main() {
    let block1 = Block {
        index: 0,
        timestamp: 1700000000,
        data: String::from("Genesis"),
        previous_hash: String::from("0000"),
        hash: String::from("abc"),
        nonce: 0,
    };

    // ..block1: 나머지 필드는 block1에서 가져옴
    let block2 = Block {
        index: 1,
        data: String::from("Second Block"),
        hash: String::from("def"),
        ..block1  // 나머지 필드 (timestamp, previous_hash, nonce)는 block1에서
    };

    // 주의: Copy가 아닌 필드(String)는 이동됨!
    // block1.previous_hash는 이제 block2가 소유
    // println!("{}", block1.previous_hash);  // 에러!
    println!("Block {}: {}", block2.index, block2.data);
}
```

TypeScript의 spread 연산자와 유사:

```typescript
const block2 = { ...block1, index: 1, data: "Second Block" };
```

---

## 튜플 구조체

이름 없는 필드를 가진 구조체입니다:

```rust,ignore
// 튜플 구조체 정의
struct Color(u8, u8, u8);      // RGB
struct Point(f64, f64, f64);   // 3D 좌표
struct Hash(String);           // newtype 패턴

fn main() {
    let red = Color(255, 0, 0);
    let origin = Point(0.0, 0.0, 0.0);
    let h = Hash(String::from("abc123"));

    // 인덱스로 접근
    println!("R: {}, G: {}, B: {}", red.0, red.1, red.2);
    println!("x: {}", origin.0);
    println!("Hash: {}", h.0);
}
```

**Newtype 패턴**: 타입 안전성을 위해 기본 타입을 감싸는 관용구입니다:

```rust,ignore
struct BlockHeight(u64);
struct TransactionId(String);
struct Wei(u128);  // Ethereum 최소 단위

fn process_block(height: BlockHeight) {
    println!("Processing block at height {}", height.0);
}

fn main() {
    let height = BlockHeight(12345);
    process_block(height);

    // process_block(12345u64);  // 에러! 타입이 다름
    // 실수로 잘못된 숫자를 넣는 것을 컴파일 타임에 방지
}
```

---

## 유닛 구조체

필드가 없는 구조체입니다. 트레이트를 구현하기 위한 타입으로 자주 사용됩니다:

```rust,ignore
struct AlwaysEqual;  // 필드 없음

// 트레이트 구현 시 유용
struct Block {
    index: u64,
    data: String,
}

struct GenesisBlock;

impl GenesisBlock {
    fn create() -> Block {
        Block {
            index: 0,
            data: String::from("Genesis Block"),
        }
    }
}
```

---

## impl 블록: 메서드와 연관 함수

```rust,ignore
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    // === 연관 함수 (Associated Functions) ===
    // self 매개변수 없음 → TypeScript의 static 메서드

    /// 제네시스 블록 생성
    fn genesis() -> Block {
        Block {
            index: 0,
            timestamp: 0,
            data: String::from("Genesis Block"),
            previous_hash: String::from("0000000000000000"),
            hash: String::from(""),
            nonce: 0,
        }
    }

    /// 새 블록 생성
    fn new(index: u64, data: String, previous_hash: String) -> Block {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let nonce = 0;
        let hash = Block::compute_hash(index, timestamp, &data, &previous_hash, nonce);

        Block { index, timestamp, data, previous_hash, hash, nonce }
    }

    /// 해시 계산 (내부용 연관 함수)
    fn compute_hash(
        index: u64,
        timestamp: u64,
        data: &str,
        previous_hash: &str,
        nonce: u64,
    ) -> String {
        format!("{}{}{}{}{}", index, timestamp, data, previous_hash, nonce)
        // 실제로는 SHA-256 해시를 써야 함
    }

    // === 메서드 (Methods) ===
    // &self: 불변 참조 — 읽기 전용
    // &mut self: 가변 참조 — 수정 가능
    // self: 소유권 이동 — 인스턴스 소비

    /// 블록 해시 반환 (&self: 읽기 전용)
    fn get_hash(&self) -> &str {
        &self.hash
    }

    /// 블록 인덱스 반환
    fn index(&self) -> u64 {
        self.index
    }

    /// 블록이 유효한지 검증
    fn is_valid(&self) -> bool {
        let expected = Block::compute_hash(
            self.index,
            self.timestamp,
            &self.data,
            &self.previous_hash,
            self.nonce,
        );
        self.hash == expected
    }

    /// 해시 재계산 (&mut self: 수정)
    fn recalculate_hash(&mut self) {
        self.hash = Block::compute_hash(
            self.index,
            self.timestamp,
            &self.data,
            &self.previous_hash,
            self.nonce,
        );
    }

    /// nonce 증가 (&mut self)
    fn increment_nonce(&mut self) {
        self.nonce += 1;
        self.recalculate_hash();
    }

    /// 블록 요약 문자열 (self를 소비: 재사용 불가)
    fn into_summary(self) -> String {
        format!("Block #{} [{}]: {}", self.index, &self.hash[..8], self.data)
        // self가 소비됨
    }
}
```

### TypeScript class와 비교

```typescript
// TypeScript
class Block {
    index: number;
    timestamp: number;
    data: string;
    previousHash: string;
    hash: string;
    nonce: number;

    // static 메서드 (Rust의 연관 함수)
    static genesis(): Block {
        return new Block(0, "Genesis", "0000000000000000");
    }

    constructor(index: number, data: string, previousHash: string) {
        this.index = index;
        this.timestamp = Date.now();
        this.data = data;
        this.previousHash = previousHash;
        this.nonce = 0;
        this.hash = this.computeHash();
    }

    // 인스턴스 메서드 (Rust의 &self 메서드)
    computeHash(): string {
        return sha256(this.index + this.timestamp + this.data + this.previousHash + this.nonce);
    }

    isValid(): boolean {
        return this.hash === this.computeHash();
    }
}
```

**핵심 차이:**

| TypeScript | Rust |
|-----------|------|
| `constructor` | 연관 함수 `fn new()` (관례) |
| `static method` | 연관 함수 (`self` 없음) |
| `this.field` | `self.field` |
| 메서드는 항상 `this` 변경 가능 | `&self` (읽기), `&mut self` (쓰기) 명시 |
| 상속 가능 | 상속 없음, 트레이트로 대체 |

---

## 여러 impl 블록

하나의 구조체에 여러 `impl` 블록을 가질 수 있습니다. 트레이트 구현이나 코드 구성에 유용합니다:

```rust,ignore
struct Block {
    index: u64,
    data: String,
    previous_hash: String,
    hash: String,
}

impl Block {
    // 생성자들
    fn genesis() -> Block {
        Block {
            index: 0,
            data: String::from("Genesis Block"),
            previous_hash: String::from("0000"),
            hash: String::from("genesis-hash"),
        }
    }

    fn new(index: u64, data: String, previous_hash: String) -> Block {
        let hash = format!("hash-{index}");
        Block { index, data, previous_hash, hash }
    }
}

impl Block {
    // 검증 메서드들
    fn is_valid(&self) -> bool {
        !self.hash.is_empty() && !self.previous_hash.is_empty()
    }

    fn verify_hash(&self) -> bool {
        self.hash == format!("hash-{}", self.index) || self.index == 0
    }
}

impl Block {
    // 변환 메서드들
    fn to_json(&self) -> String {
        format!(
            r#"{{"index":{},"data":"{}","hash":"{}"}}"#,
            self.index, self.data, self.hash
        )
    }

    fn into_summary(self) -> String {
        format!("Block #{}: {}", self.index, self.hash)
    }
}
```

---

## 구조체 출력: Debug와 Display

구조체를 `println!`으로 출력하려면 트레이트를 구현해야 합니다:

```rust,ignore
// derive 매크로로 자동 구현 (간단하지만 포맷이 고정)
#[derive(Debug)]
struct Block {
    index: u64,
    hash: String,
    data: String,
}

fn main() {
    let block = Block {
        index: 0,
        hash: String::from("abc123"),
        data: String::from("Genesis"),
    };

    // Debug 출력 ({:?})
    println!("{:?}", block);
    // Block { index: 0, hash: "abc123", data: "Genesis" }

    // 예쁜 Debug 출력 ({:#?})
    println!("{:#?}", block);
    // Block {
    //     index: 0,
    //     hash: "abc123",
    //     data: "Genesis",
    // }
}
```

커스텀 출력 형식을 원하면 `Display` 트레이트를 직접 구현합니다:

```rust,ignore
use std::fmt;

struct Block {
    index: u64,
    hash: String,
    data: String,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Block #{} [{}...]: {}", self.index, &self.hash[..6], self.data)
    }
}

fn main() {
    let block = Block {
        index: 0,
        hash: String::from("abc123def456"),
        data: String::from("Genesis"),
    };

    println!("{}", block);  // Block #0 [abc123...]: Genesis
}
```

---

## 블록체인 전체 구조체 예시

```rust,ignore
use std::fmt;

#[derive(Debug, Clone)]
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    fn new(index: u64, data: String, previous_hash: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

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

    fn calculate_hash(&self) -> String {
        // 실제 구현에서는 SHA-256 사용
        format!("{:x}", self.index + self.timestamp + self.nonce)
    }

    fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined! Nonce: {}, Hash: {}", self.nonce, self.hash);
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block #{}\n  Timestamp: {}\n  Data: {}\n  Hash: {}\n  Nonce: {}",
            self.index, self.timestamp, self.data, self.hash, self.nonce
        )
    }
}

#[derive(Debug)]
struct Blockchain {
    blocks: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    fn new() -> Self {
        let genesis = Block::new(0, String::from("Genesis Block"), String::from("0"));
        Blockchain {
            blocks: vec![genesis],
            difficulty: 2,
        }
    }

    fn add_block(&mut self, data: String) {
        let previous_hash = self.blocks.last()
            .map(|b| b.hash.clone())
            .unwrap_or_default();
        let index = self.blocks.len() as u64;
        let mut block = Block::new(index, data, previous_hash);
        block.mine(self.difficulty);
        self.blocks.push(block);
    }

    fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            if current.hash != current.calculate_hash() {
                return false;
            }
            if current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }
}

fn main() {
    let mut blockchain = Blockchain::new();
    blockchain.add_block(String::from("Alice sends 1 BTC to Bob"));
    blockchain.add_block(String::from("Bob sends 0.5 BTC to Carol"));

    for block in &blockchain.blocks {
        println!("{}\n", block);
    }

    println!("Blockchain valid: {}", blockchain.is_valid());
}
```

---

## 요약

- `struct`로 관련 데이터를 묶음 (TypeScript의 interface/class의 데이터 부분)
- `impl` 블록에 메서드와 연관 함수를 정의
- 연관 함수: `self` 없음 → TypeScript `static`에 해당, `Type::function()` 으로 호출
- 메서드: `&self`(읽기), `&mut self`(쓰기), `self`(소비) 세 가지
- `#[derive(Debug)]`로 자동 디버그 출력
- 커스텀 출력은 `fmt::Display` 트레이트 구현
- 상속 없음 — 대신 트레이트(5장)로 공통 동작 추상화

다음 챕터에서는 더 강력한 타입인 열거형(enum)을 배웁니다.
