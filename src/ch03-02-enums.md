# 3.2 열거형 (Enums)

## Rust의 열거형은 강력하다

TypeScript의 `enum`은 단순한 상수 집합입니다. Rust의 `enum`은 **대수적 데이터 타입(Algebraic Data Type)**으로, 각 배리언트(variant)가 서로 다른 타입과 양의 데이터를 가질 수 있습니다.

```typescript
// TypeScript enum — 단순한 상수
enum Direction {
    North,
    South,
    East,
    West,
}

// TypeScript discriminated union — 데이터를 가진 유니온
type Message =
    | { type: "Text"; content: string }
    | { type: "Move"; x: number; y: number }
    | { type: "ChangeColor"; r: number; g: number; b: number };
```

```rust
// Rust enum — 단순 배리언트
enum Direction {
    North,
    South,
    East,
    West,
}

// Rust enum — 데이터를 가진 배리언트 (훨씬 자연스럽게)
enum Message {
    Text(String),                    // 튜플 배리언트
    Move { x: i32, y: i32 },        // 구조체 배리언트
    ChangeColor(u8, u8, u8),         // 여러 값을 가진 튜플 배리언트
    Quit,                            // 데이터 없는 배리언트
}
```

---

## 기본 열거형 정의와 사용

```rust,ignore
#[derive(Debug)]
enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

fn describe_status(status: &TransactionStatus) -> &str {
    match status {
        TransactionStatus::Pending   => "대기 중",
        TransactionStatus::Confirmed => "확정됨",
        TransactionStatus::Failed    => "실패",
        TransactionStatus::Cancelled => "취소됨",
    }
}

fn main() {
    let status = TransactionStatus::Pending;
    println!("Status: {}", describe_status(&status));

    // 비교
    let s2 = TransactionStatus::Confirmed;
    // status == s2  // 에러! PartialEq를 derive해야 함

    // 이렇게 사용
    if let TransactionStatus::Pending = status {
        println!("Transaction is pending");
    }
}
```

---

## 데이터를 가진 열거형

```rust
#[derive(Debug)]
enum Transaction {
    // 데이터 없음
    CoinbaseReward,

    // 단일 값
    Transfer(u64),  // 금액 (satoshi)

    // 여러 값 (튜플 배리언트)
    TransferWithFee(u64, u64),  // (금액, 수수료)

    // 이름 있는 필드 (구조체 배리언트)
    SmartContract {
        contract_address: String,
        value: u64,
        data: Vec<u8>,
    },
}

fn process_transaction(tx: &Transaction) {
    match tx {
        Transaction::CoinbaseReward => {
            println!("Coinbase reward transaction");
        }
        Transaction::Transfer(amount) => {
            println!("Transfer: {} satoshi", amount);
        }
        Transaction::TransferWithFee(amount, fee) => {
            println!("Transfer: {} satoshi, Fee: {} satoshi", amount, fee);
        }
        Transaction::SmartContract { contract_address, value, data } => {
            println!(
                "Smart contract call to {} with {} wei, data: {} bytes",
                contract_address,
                value,
                data.len()
            );
        }
    }
}

fn main() {
    let txs = vec![
        Transaction::CoinbaseReward,
        Transaction::Transfer(100_000),
        Transaction::TransferWithFee(50_000, 1_000),
        Transaction::SmartContract {
            contract_address: String::from("0xabcd..."),
            value: 0,
            data: vec![0x60, 0x80, 0x60, 0x40],
        },
    ];

    for tx in &txs {
        process_transaction(tx);
    }
}
```

---

## impl 블록을 가진 열거형

열거형에도 메서드를 구현할 수 있습니다:

```rust
#[derive(Debug)]
enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Localnet,
}

impl NetworkType {
    fn rpc_url(&self) -> &str {
        match self {
            NetworkType::Mainnet  => "https://api.mainnet-beta.solana.com",
            NetworkType::Testnet  => "https://api.testnet.solana.com",
            NetworkType::Devnet   => "https://api.devnet.solana.com",
            NetworkType::Localnet => "http://127.0.0.1:8899",
        }
    }

    fn is_production(&self) -> bool {
        matches!(self, NetworkType::Mainnet)
    }

    fn from_str(s: &str) -> Option<NetworkType> {
        match s {
            "mainnet" | "mainnet-beta" => Some(NetworkType::Mainnet),
            "testnet"                  => Some(NetworkType::Testnet),
            "devnet"                   => Some(NetworkType::Devnet),
            "localnet" | "localhost"   => Some(NetworkType::Localnet),
            _                          => None,
        }
    }
}

fn main() {
    let network = NetworkType::Devnet;
    println!("RPC URL: {}", network.rpc_url());
    println!("Production: {}", network.is_production());

    if let Some(net) = NetworkType::from_str("mainnet") {
        println!("Parsed: {:?}", net);
    }
}
```

---

## Option\<T\>: null을 대체하는 타입

Rust에는 `null`이 없습니다. 대신 `Option<T>` 열거형을 사용합니다.

```rust
// 표준 라이브러리에 이렇게 정의되어 있음
enum Option<T> {
    Some(T),  // 값이 있음
    None,     // 값이 없음
}
```

TypeScript의 `null`/`undefined`와 비교:

```typescript
// TypeScript
function findBlock(index: number): Block | null {
    const blocks = [
        { index: 0, hash: "genesis" },
        { index: 1, hash: "abc123" },
    ];
    const found = blocks.find((block) => block.index === index);
    if (!found) {
        return null;
    }
    return found;
}

const block = findBlock(5);
// 개발자가 null 체크를 잊어도 TypeScript는 타입 에러를 냄 (strict mode)
if (block !== null) {
    console.log(block.hash);
}
// 하지만 런타임에 null이 올 수 있는 상황이 많음
```

```rust
// Rust
struct Block {
    index: u64,
    hash: String,
}

fn find_block(blocks: &[Block], index: u64) -> Option<&Block> {
    blocks.iter().find(|b| b.index == index)
}

fn main() {
    let blocks = vec![
        Block { index: 0, hash: String::from("genesis") },
        Block { index: 1, hash: String::from("abc123") },
        Block { index: 5, hash: String::from("def456") },
    ];
    let result = find_block(&blocks, 5);

    // Option을 처리하지 않으면 컴파일 에러!
    match result {
        Some(block) => println!("Found: {}", block.hash),
        None        => println!("Block not found"),
    }

    // 또는 if let
    if let Some(block) = find_block(&blocks, 3) {
        println!("Block 3 hash: {}", block.hash);
    }
}
```

### Option 관련 주요 메서드

```rust
fn main() {
    let some_val: Option<i32> = Some(42);
    let no_val: Option<i32> = None;

    // unwrap(): Some이면 값 반환, None이면 panic!
    // 프로덕션 코드에서 주의해서 사용
    let val = some_val.unwrap();  // 42

    // unwrap_or(): None일 때 기본값
    let val2 = no_val.unwrap_or(0);  // 0
    let val3 = no_val.unwrap_or_default();  // 타입의 기본값 (i32 → 0)

    // unwrap_or_else(): None일 때 클로저 실행
    let val4 = no_val.unwrap_or_else(|| compute_default());

    // expect(): None이면 커스텀 메시지로 panic
    // let val5 = no_val.expect("값이 있어야 합니다");

    // is_some(), is_none()
    println!("some_val has value: {}", some_val.is_some());  // true
    println!("no_val is none: {}", no_val.is_none());        // true

    // map(): Some이면 변환, None이면 None 유지
    let doubled: Option<i32> = some_val.map(|v| v * 2);  // Some(84)
    let nothing: Option<i32> = no_val.map(|v| v * 2);    // None

    // and_then(): Some이면 Option 반환하는 함수 실행 (flatMap)
    let result: Option<String> = some_val.and_then(|v| {
        if v > 0 { Some(v.to_string()) } else { None }
    });

    // filter(): 조건이 참이면 Some 유지, 아니면 None
    let filtered: Option<i32> = some_val.filter(|&v| v > 100);  // None (42 <= 100)

    // or(): None이면 다른 Option으로 대체
    let result2 = no_val.or(Some(99));  // Some(99)

    // as_ref(): Option<T> → Option<&T> (소유권 유지)
    let s: Option<String> = Some(String::from("hello"));
    let r: Option<&String> = s.as_ref();  // s의 소유권을 유지하면서 참조
    println!("{:?}", r);
    println!("{:?}", s);  // s 여전히 유효
}

fn compute_default() -> i32 { 42 }
```

### 블록체인에서 Option 활용

```rust
#[derive(Debug)]
struct Block {
    index: u64,
    hash: String,
    previous_hash: Option<String>,  // 제네시스 블록은 이전 해시 없음
    data: String,
}

impl Block {
    fn genesis() -> Self {
        Block {
            index: 0,
            hash: String::from("0000abc"),
            previous_hash: None,  // 제네시스는 이전 블록 없음
            data: String::from("Genesis"),
        }
    }

    fn new(index: u64, data: String, previous_hash: String) -> Self {
        Block {
            index,
            hash: String::from("computed"),
            previous_hash: Some(previous_hash),
            data,
        }
    }

    fn is_genesis(&self) -> bool {
        self.previous_hash.is_none()
    }

    fn get_previous_hash(&self) -> &str {
        self.previous_hash.as_deref().unwrap_or("N/A")
    }
}

fn main() {
    let genesis = Block::genesis();
    let block1 = Block::new(1, String::from("tx1"), genesis.hash.clone());

    println!("Genesis? {}", genesis.is_genesis());  // true
    println!("Prev hash: {}", genesis.get_previous_hash());  // N/A

    println!("Block1 genesis? {}", block1.is_genesis());  // false
    println!("Block1 prev: {}", block1.get_previous_hash());  // 0000abc
}
```

---

## TypeScript discriminated union과 비교

```typescript
// TypeScript discriminated union
type WalletEvent =
    | { kind: "deposit";    amount: number; from: string }
    | { kind: "withdraw";   amount: number; to: string }
    | { kind: "swap";       fromToken: string; toToken: string; amount: number }
    | { kind: "error";      message: string };

function handleEvent(event: WalletEvent): void {
    switch (event.kind) {
        case "deposit":
            console.log(`Deposit ${event.amount} from ${event.from}`);
            break;
        case "withdraw":
            console.log(`Withdraw ${event.amount} to ${event.to}`);
            break;
        case "swap":
            console.log(`Swap ${event.amount} ${event.fromToken} → ${event.toToken}`);
            break;
        case "error":
            console.log(`Error: ${event.message}`);
            break;
        // TypeScript는 모든 케이스를 처리했는지 확인 (exhaustive check)
    }
}
```

```rust
// Rust enum — 훨씬 간결하고 타입 안전
#[derive(Debug)]
enum WalletEvent {
    Deposit  { amount: u64, from: String },
    Withdraw { amount: u64, to: String },
    Swap     { from_token: String, to_token: String, amount: u64 },
    Error    (String),
}

fn handle_event(event: &WalletEvent) {
    match event {
        WalletEvent::Deposit { amount, from } => {
            println!("Deposit {} from {}", amount, from);
        }
        WalletEvent::Withdraw { amount, to } => {
            println!("Withdraw {} to {}", amount, to);
        }
        WalletEvent::Swap { from_token, to_token, amount } => {
            println!("Swap {} {} → {}", amount, from_token, to_token);
        }
        WalletEvent::Error(msg) => {
            println!("Error: {}", msg);
        }
        // 모든 배리언트를 처리하지 않으면 컴파일 에러!
        // non-exhaustive patterns
    }
}

fn main() {
    let events = vec![
        WalletEvent::Deposit { amount: 1_000_000, from: String::from("Alice") },
        WalletEvent::Withdraw { amount: 500_000, to: String::from("Bob") },
        WalletEvent::Error(String::from("Insufficient balance")),
    ];

    for event in &events {
        handle_event(event);
    }
}
```

**Rust enum의 장점:**
1. `kind` 필드 없이 배리언트 자체가 구분자
2. 각 배리언트마다 다른 타입과 양의 데이터
3. `match`에서 모든 케이스 처리 강제 (exhaustive)
4. 패턴 매칭으로 데이터를 바로 꺼낼 수 있음

---

## 요약

- Rust `enum`은 TypeScript `enum`보다 훨씬 강력 — 각 배리언트가 데이터를 가질 수 있음
- TypeScript의 discriminated union과 유사하지만 더 간결하고 타입 안전
- `Option<T>`: null/undefined 대체 — `Some(T)` 또는 `None`
- `Option` 메서드: `unwrap()`, `unwrap_or()`, `map()`, `and_then()`, `filter()`
- `match`로 모든 배리언트를 처리해야 함 (exhaustive check — 컴파일 타임)
- 열거형에도 `impl` 블록으로 메서드 추가 가능

다음 챕터에서는 열거형과 가장 잘 어울리는 패턴 매칭을 자세히 배웁니다.
