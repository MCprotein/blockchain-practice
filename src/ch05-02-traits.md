# 5.2 트레이트 (Traits)

## 트레이트란?

트레이트는 타입이 구현해야 하는 동작(메서드)의 집합을 정의합니다. TypeScript의 `interface`와 유사하지만 더 강력합니다.

```rust,ignore
// 트레이트 정의
trait Hashable {
    // 메서드 시그니처 (구현 필수)
    fn compute_hash(&self) -> String;

    // 기본 구현 (선택적으로 오버라이드)
    fn short_hash(&self) -> String {
        let full = self.compute_hash();
        full[..8].to_string()  // 앞 8자리
    }
}
```

TypeScript 인터페이스와 비교:

```typescript
// TypeScript interface
interface Hashable {
    computeHash(): string;

    // 기본 구현은 interface에 없음 (abstract class에서만)
    // 트레이트의 기본 구현은 Rust만의 기능
}
```

---

## 트레이트 구현

```rust,ignore
struct Block {
    index: u64,
    data: String,
    previous_hash: String,
    nonce: u64,
}

// Block에 Hashable 트레이트 구현
impl Hashable for Block {
    fn compute_hash(&self) -> String {
        // 실제로는 SHA-256 사용
        format!("{:x}",
            self.index as u64 * 1000
            + self.data.len() as u64
            + self.nonce
        )
    }
    // short_hash()는 기본 구현 사용
}

struct Transaction {
    from: String,
    to: String,
    amount: u64,
}

impl Hashable for Transaction {
    fn compute_hash(&self) -> String {
        format!("{}{}{}",
            self.from.len() + self.to.len(),
            self.amount,
            "tx"
        )
    }

    // 트랜잭션은 short_hash를 다르게 구현
    fn short_hash(&self) -> String {
        format!("TX:{}", &self.compute_hash()[..4])
    }
}

fn main() {
    let block = Block {
        index: 1,
        data: String::from("genesis"),
        previous_hash: String::from("0000"),
        nonce: 42,
    };

    let tx = Transaction {
        from: String::from("Alice"),
        to: String::from("Bob"),
        amount: 1000,
    };

    println!("Block hash: {}", block.compute_hash());
    println!("Block short: {}", block.short_hash());  // 기본 구현

    println!("TX hash: {}", tx.compute_hash());
    println!("TX short: {}", tx.short_hash());  // 오버라이드된 구현
}
```

---

## 트레이트 바운드

제네릭 함수에서 타입이 특정 트레이트를 구현해야 한다고 요구할 때:

```rust,ignore
// 방법 1: 인라인 트레이트 바운드
fn print_hash<T: Hashable>(item: &T) {
    println!("Hash: {}", item.compute_hash());
}

// 방법 2: where 절 (더 읽기 좋음, 복잡할 때 권장)
fn compare_hashes<T>(a: &T, b: &T) -> bool
where
    T: Hashable + std::fmt::Debug,
{
    println!("Comparing {:?} and {:?}", a, b);
    a.compute_hash() == b.compute_hash()
}

// 여러 트레이트 바운드
fn process<T: Hashable + Clone + std::fmt::Display>(item: T) {
    let cloned = item.clone();
    println!("Processing: {}", item);
    println!("Hash: {}", cloned.compute_hash());
}
```

### 트레이트 객체 (dyn Trait)

컴파일 타임에 타입을 모를 때, 런타임에 트레이트를 통해 동적으로 디스패치:

```rust,ignore
// 정적 디스패치 (제네릭) — 컴파일 타임에 타입 결정, 더 빠름
fn hash_static<T: Hashable>(item: &T) -> String {
    item.compute_hash()
}

// 동적 디스패치 (트레이트 객체) — 런타임에 타입 결정, 유연함
fn hash_dynamic(item: &dyn Hashable) -> String {
    item.compute_hash()
}

fn main() {
    let block = Block {
        index: 1,
        data: String::from("block data"),
        previous_hash: String::from("0000"),
        nonce: 7,
    };
    let tx = Transaction {
        from: String::from("Alice"),
        to: String::from("Bob"),
        amount: 1000,
    };

    // 정적 디스패치 — 각 호출에서 구체 타입을 앎
    hash_static(&block);
    hash_static(&tx);

    // 동적 디스패치 — 런타임에 결정
    let items: Vec<Box<dyn Hashable>> = vec![
        Box::new(Block { index: 0, data: String::from("g"), previous_hash: String::from("0"), nonce: 0 }),
        Box::new(Transaction { from: String::from("A"), to: String::from("B"), amount: 100 }),
    ];

    for item in &items {
        println!("{}", item.compute_hash());
    }
}
```

TypeScript에서는 항상 동적 디스패치(런타임 다형성):

```typescript
// TypeScript: 항상 런타임 타입 기반
function hashItem(item: Hashable): string {
    return item.computeHash();  // 런타임에 어떤 메서드인지 결정
}
```

---

## 표준 라이브러리의 주요 트레이트

### Display와 Debug

```rust,ignore
use std::fmt;

struct Block {
    index: u64,
    hash: String,
}

// Debug: {:?} 포맷 — 개발자용, derive로 자동 구현 가능
impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("index", &self.index)
            .field("hash", &self.hash)
            .finish()
    }
}

// Display: {} 포맷 — 사용자용, 직접 구현 필요
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Block #{} [{}]", self.index, &self.hash[..6])
    }
}

fn main() {
    let block = Block { index: 0, hash: String::from("abcdef1234") };
    println!("{:?}", block);  // Debug
    println!("{}", block);    // Display
}
```

### Clone과 Copy

```rust,ignore
// Clone: 명시적 깊은 복사 (.clone() 호출)
#[derive(Clone)]
struct Block {
    index: u64,
    hash: String,  // String은 Clone이지만 Copy가 아님
}

// Copy: 암묵적 복사 (스택에만 있는 간단한 타입)
// Copy는 Clone을 내포함
#[derive(Clone, Copy)]
struct Point {
    x: f64,  // f64는 Copy
    y: f64,
}

fn main() {
    let b1 = Block { index: 0, hash: String::from("abc") };
    let b2 = b1.clone();  // 명시적 복사
    println!("{}", b1.index);  // OK, b1도 유효

    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1;  // 암묵적 Copy
    println!("{}", p1.x);  // OK, p1도 유효 (Copy 타입이므로)
}
```

### PartialEq, Eq, PartialOrd, Ord

```rust,ignore
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct BlockHeight(u64);

fn main() {
    let h1 = BlockHeight(100);
    let h2 = BlockHeight(200);
    let h3 = BlockHeight(100);

    // PartialEq: == 연산자
    println!("{}", h1 == h3);  // true
    println!("{}", h1 != h2);  // true

    // PartialOrd: <, >, <=, >= 연산자
    println!("{}", h1 < h2);   // true

    // Ord: 전순서 비교 (sort 등에 필요)
    let mut heights = vec![BlockHeight(300), BlockHeight(100), BlockHeight(200)];
    heights.sort();
    println!("{:?}", heights);  // [BlockHeight(100), BlockHeight(200), BlockHeight(300)]

    let max = heights.iter().max().unwrap();
    println!("Max: {:?}", max);
}
```

### Default

```rust,ignore
#[derive(Debug, Default)]
struct BlockConfig {
    difficulty: usize,      // 기본값: 0
    max_transactions: u32,  // 기본값: 0
    mining_reward: u64,     // 기본값: 0
    network: String,        // 기본값: ""
}

// 커스텀 기본값
impl Default for BlockConfig {
    fn default() -> Self {
        BlockConfig {
            difficulty: 4,
            max_transactions: 1000,
            mining_reward: 625_000_000,  // 6.25 BTC in satoshi
            network: String::from("mainnet"),
        }
    }
}

fn main() {
    let config = BlockConfig::default();
    println!("{:?}", config);

    // 일부만 변경
    let custom = BlockConfig {
        difficulty: 6,
        ..BlockConfig::default()  // 나머지는 기본값
    };
    println!("{:?}", custom);

    // unwrap_or_default()와 함께
    let maybe_config: Option<BlockConfig> = None;
    let config2 = maybe_config.unwrap_or_default();
}
```

### Iterator 트레이트 (6.3장에서 자세히)

```rust,ignore
struct CountingIterator {
    current: u64,
    max: u64,
}

impl CountingIterator {
    fn new(max: u64) -> Self {
        CountingIterator { current: 0, max }
    }
}

impl Iterator for CountingIterator {
    type Item = u64;  // 연관 타입

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.max {
            self.current += 1;
            Some(self.current)
        } else {
            None
        }
    }
}

fn main() {
    let counter = CountingIterator::new(5);
    let sum: u64 = counter.sum();
    println!("Sum 1..5 = {}", sum);  // 15

    // Iterator를 구현하면 map, filter, collect 등 모두 자동으로 사용 가능
    let evens: Vec<u64> = CountingIterator::new(10)
        .filter(|n| n % 2 == 0)
        .collect();
    println!("{:?}", evens);  // [2, 4, 6, 8, 10]
}
```

---

## derive 매크로

`#[derive(...)]`는 표준 트레이트의 기계적인 구현을 자동으로 생성합니다:

```rust,ignore
#[derive(
    Debug,      // {:?} 출력
    Clone,      // .clone() 메서드
    PartialEq,  // == 연산자
    Eq,         // 완전 동등 (Hash 구현에 필요)
    Hash,       // HashMap의 키로 사용
    Default,    // ::default() 생성자
    serde::Serialize,    // serde 크레이트
    serde::Deserialize,  // serde 크레이트
)]
struct TransactionId {
    value: String,
}
```

**derive 가능한 표준 트레이트:**

| 트레이트 | 기능 |
|---------|------|
| `Debug` | `{:?}` 포맷 |
| `Clone` | `.clone()` 메서드 |
| `Copy` | 암묵적 복사 (Clone도 필요) |
| `PartialEq` | `==`, `!=` 연산자 |
| `Eq` | 완전 동등 (PartialEq 필요) |
| `PartialOrd` | `<`, `>`, `<=`, `>=` (PartialEq 필요) |
| `Ord` | 전순서 (Eq, PartialOrd 필요) |
| `Hash` | HashMap/HashSet 키 (Eq 필요) |
| `Default` | `::default()` |

---

## 트레이트의 고아 규칙 (Orphan Rule)

트레이트를 구현할 수 있는 조건:
- **내가 만든 타입**에 **외부 트레이트** 구현 가능
- **외부 타입**에 **내가 만든 트레이트** 구현 가능
- **외부 타입**에 **외부 트레이트** 구현 불가능 (고아 규칙 위반)

```rust,ignore
use std::fmt;

// OK: 내 타입(Block)에 외부 트레이트(Display) 구현
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Block #{}", self.index)
    }
}

// OK: 외부 타입(Vec<Block>)에 내 트레이트(Hashable) 구현
impl Hashable for Vec<Block> {
    fn compute_hash(&self) -> String {
        self.iter()
            .map(|block| block.compute_hash())
            .collect::<Vec<_>>()
            .join("")
    }
}

// 에러: 외부 타입(String)에 외부 트레이트(Display) 구현 불가
// impl fmt::Display for String {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{self}")
//     }
// }
// error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

---

## TypeScript interface와 Rust trait 상세 비교

```typescript
// TypeScript
interface Serializable {
    serialize(): string;
    deserialize(data: string): void;  // 메서드 반환 타입에 void
}

// TypeScript는 클래스가 여러 인터페이스를 구현할 수 있음
class Block implements Serializable, Hashable {
    serialize(): string { return JSON.stringify(this); }
    deserialize(data: string): void { Object.assign(this, JSON.parse(data)); }
    computeHash(): string { return this.serialize(); }
}

// TypeScript는 인터페이스 확장 가능
interface ExtendedHashable extends Hashable {
    verifyHash(): boolean;
}
```

```rust,ignore
// Rust
trait Serializable {
    fn serialize(&self) -> String;

    // Rust trait의 기본 구현
    fn serialize_pretty(&self) -> String {
        format!("```\n{}\n```", self.serialize())
    }
}

// 트레이트 확장 (supertrait)
trait ExtendedHashable: Hashable {
    fn verify_hash(&self) -> bool;
}

// Block에 여러 트레이트 구현
impl Serializable for Block {
    fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl ExtendedHashable for Block {
    fn verify_hash(&self) -> bool {
        self.hash == self.compute_hash()
    }
}
// Hashable도 별도로 구현해야 함 (ExtendedHashable의 supertrait이므로)
impl Hashable for Block {
    fn compute_hash(&self) -> String {
        format!("{}:{}:{}", self.index, self.previous_hash, self.nonce)
    }
}
```

**핵심 차이:**

| TypeScript interface | Rust trait |
|---------------------|-----------|
| 기본 구현 없음 (abstract class에서는 가능) | 기본 구현 가능 |
| 기존 클래스에 나중에 인터페이스 추가 불가 | 기존 타입에 트레이트 구현 가능 |
| 런타임 타입 체크 | 컴파일 타임 체크 |
| 항상 동적 디스패치 | 정적(제네릭) 또는 동적(dyn) 선택 |

---

## 연관 타입 (Associated Types)

트레이트에서 출력 타입을 정의하는 또 다른 방법:

```rust,ignore
trait BlockStore {
    type Block;    // 연관 타입
    type Error;

    fn get(&self, height: u64) -> Result<Option<Self::Block>, Self::Error>;
    fn height(&self) -> u64;
}

struct InMemoryStore {
    blocks: Vec<Block>,
}

impl BlockStore for InMemoryStore {
    type Block = Block;
    type Error = String;

    fn get(&self, height: u64) -> Result<Option<Block>, String> {
        Ok(self.blocks.get(height as usize).cloned())
    }

    fn height(&self) -> u64 {
        self.blocks.len() as u64
    }
}
```

---

## 요약

- 트레이트: 타입이 구현해야 하는 동작의 집합 정의
- `impl Trait for Type`: 특정 타입에 트레이트 구현
- 기본 구현: 트레이트에서 제공, 오버라이드 가능
- 트레이트 바운드: `<T: Trait>` 또는 `where T: Trait`
- 동적 디스패치: `dyn Trait` (런타임 결정, 유연함)
- 표준 트레이트: `Debug`, `Clone`, `Copy`, `Display`, `PartialEq`, `Default` 등
- `#[derive(...)]`: 표준 트레이트 자동 구현
- 고아 규칙: 내 타입 또는 내 트레이트 중 하나는 현재 크레이트에 있어야 함

다음 챕터에서 수명(lifetime) 어노테이션을 배웁니다.
