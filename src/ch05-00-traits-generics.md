# 5장: 제네릭과 트레이트

## 추상화의 두 축

Rust에서 코드 재사용과 추상화는 두 개념을 중심으로 이루어집니다:

- **제네릭(Generics)**: "어떤 타입이든 동작하는 코드"를 작성
- **트레이트(Traits)**: "이 동작을 할 수 있는 타입"을 정의

TypeScript에서는 제네릭은 비슷하게 존재하지만, 트레이트는 `interface`와 유사하면서도 중요한 차이가 있습니다.

### 제네릭 — TypeScript와 유사

```typescript
// TypeScript 제네릭
function first<T>(arr: T[]): T | undefined {
    return arr[0];
}

first([1, 2, 3]);       // number
first(["a", "b"]);      // string
```

```rust
// Rust 제네릭
fn first<T>(slice: &[T]) -> Option<&T> {
    slice.first()
}

first(&[1, 2, 3]);       // Option<&i32>
first(&["a", "b"]);      // Option<&&str>
```

### 트레이트 — interface보다 강력

```typescript
// TypeScript interface
interface Hashable {
    computeHash(): string;
}

class Block implements Hashable {
    computeHash(): string { return "..."; }
}
```

```rust
// Rust trait
trait Hashable {
    fn compute_hash(&self) -> String;
}

struct Block { /* ... */ }

impl Hashable for Block {
    fn compute_hash(&self) -> String {
        // 구현
        String::from("...")
    }
}
```

**핵심 차이**: Rust 트레이트는 기존 타입에도 구현할 수 있습니다 (TypeScript interface는 불가). 예를 들어 `String`에 내가 만든 트레이트를 구현할 수 있습니다 (단, 고아 규칙 제한 있음).

## 이 장의 구성

1. **제네릭** (5.1): 함수, 구조체, 열거형의 제네릭, 모노모피제이션
2. **트레이트** (5.2): 정의, 구현, 바운드, 표준 트레이트, derive
3. **수명** (5.3): 수명 어노테이션, 생략 규칙

## 블록체인에서의 활용

```rust
// 제네릭 + 트레이트로 범용적인 블록체인 저장소 구현
trait BlockStore {
    type Block;
    type Error;

    fn get(&self, height: u64) -> Result<Option<Self::Block>, Self::Error>;
    fn insert(&mut self, block: Self::Block) -> Result<(), Self::Error>;
    fn height(&self) -> u64;
}

// 인메모리 구현
struct InMemoryStore {
    blocks: Vec<Block>,
}

// 데이터베이스 구현
struct RocksDbStore {
    db: rocksdb::DB,
}

// 동일한 BlockStore 인터페이스로 두 구현체 모두 사용 가능
fn process_new_block<S: BlockStore<Block = Block>>(
    store: &mut S,
    block: Block,
) -> Result<(), S::Error> {
    store.insert(block)
}
```

다음 챕터에서 제네릭부터 자세히 배웁니다.
