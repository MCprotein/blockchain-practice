# 3장: 구조체와 열거형

## 데이터를 모델링하는 방법

모든 프로그램은 데이터를 표현해야 합니다. TypeScript에서는 `class`와 `interface`로 데이터를 모델링했습니다. Rust에서는 **구조체(struct)**와 **열거형(enum)**을 사용합니다.

### TypeScript vs Rust 데이터 모델링

```typescript
// TypeScript: class로 데이터 + 메서드 묶기
class Block {
    index: number;
    timestamp: number;
    data: string;
    previousHash: string;
    hash: string;

    constructor(index: number, data: string, previousHash: string) {
        this.index = index;
        this.timestamp = Date.now();
        this.data = data;
        this.previousHash = previousHash;
        this.hash = this.calculateHash();
    }

    calculateHash(): string {
        return sha256(this.index + this.timestamp + this.data + this.previousHash);
    }
}
```

```rust,ignore
// Rust: struct로 데이터, impl 블록으로 메서드
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    previous_hash: String,
    hash: String,
}

impl Block {
    // 연관 함수 (TypeScript의 static 메서드)
    fn new(index: u64, data: String, previous_hash: String) -> Block {
        let timestamp = current_timestamp();
        let hash = calculate_hash(index, timestamp, &data, &previous_hash);
        Block { index, timestamp, data, previous_hash, hash }
    }

    // 메서드 (TypeScript의 인스턴스 메서드)
    fn calculate_hash(&self) -> String {
        calculate_hash(self.index, self.timestamp, &self.data, &self.previous_hash)
    }
}
```

이 코드를 처음 읽을 때는 문법보다 역할을 먼저 보세요.

| Rust 코드 | 역할 |
|-----------|------|
| `struct Block { ... }` | `Block`이라는 데이터 모양을 정의 |
| `index: u64` | 필드 이름은 `index`, 타입은 `u64` |
| `impl Block { ... }` | `Block`에 붙는 함수와 메서드를 정의 |
| `fn new(...) -> Block` | 새 `Block`을 만들어 반환하는 생성 함수 |
| `fn calculate_hash(&self) -> String` | 이미 만들어진 `Block`을 읽어 해시 문자열을 반환하는 메서드 |
| `&self` | 이 메서드가 `Block`을 소유하지 않고 읽기만 빌린다는 뜻 |
| `Block { index, timestamp, data, previous_hash, hash }` | 필드 값을 채워 새 구조체 인스턴스를 만드는 문법 |

TypeScript `class`는 데이터와 메서드가 한 덩어리지만, Rust는 데이터(`struct`)와 동작(`impl`)을 분리해서 읽습니다. 앞으로 블록체인 예제의 대부분은 `Transaction`, `Block`, `Blockchain` 같은 구조체를 먼저 정의하고, 그 아래 `impl`에서 생성, 해싱, 검증 동작을 붙이는 패턴으로 작성됩니다.

**핵심 차이점:**
- Rust의 `struct`는 데이터만 정의 (상속 없음)
- 메서드는 `impl` 블록에 별도로 정의
- 상속 대신 **트레이트(trait)**로 공통 동작 추상화

## 이 장의 구성

1. **구조체** (3.1): 데이터 정의, 메서드, 연관 함수
2. **열거형** (3.2): 대수적 데이터 타입, Option, Result
3. **패턴 매칭** (3.3): match, if let, while let

## 왜 이게 블록체인에서 중요한가?

블록체인 스마트 컨트랙트는 상태(state)를 정의하고 트랜잭션을 처리합니다. 상태를 올바르게 모델링하는 것이 보안의 핵심입니다.

예를 들어 Solana의 온체인 프로그램에서:

```rust,ignore
// Solana 프로그램의 계정 상태 (실제 패턴)
#[account]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub delegate: Option<Pubkey>,  // Option으로 null 안전하게 처리
    pub state: AccountState,       // 열거형으로 상태 표현
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority: Option<Pubkey>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccountState {
    Uninitialized,
    Initialized,
    Frozen,
}
```

구조체와 열거형을 제대로 이해해야 이런 코드를 읽고 작성할 수 있습니다.

다음 챕터에서 구조체를 자세히 배웁니다.
