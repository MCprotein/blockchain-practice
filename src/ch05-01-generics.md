# 5.1 제네릭

## 제네릭이 필요한 이유

같은 로직인데 타입만 다른 코드를 반복 작성하는 것은 나쁜 설계입니다:

```rust,ignore
// 반복 코드 — 나쁜 패턴
fn largest_i32(list: &[i32]) -> &i32 {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn largest_f64(list: &[f64]) -> &f64 {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

제네릭으로 하나로 합칩니다:

```rust,ignore
// 제네릭 함수 — 좋은 패턴
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    println!("Largest number: {}", largest(&numbers));

    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Largest char: {}", largest(&chars));
}
```

`T: PartialOrd`는 "T는 비교 가능해야 한다"는 트레이트 바운드입니다. 5.2에서 자세히 다룹니다.

---

## 함수의 제네릭

```rust,ignore
// 단일 타입 매개변수
fn identity<T>(value: T) -> T {
    value
}

// 여러 타입 매개변수
fn zip_first<T, U>(a: Vec<T>, b: Vec<U>) -> Vec<(T, U)> {
    a.into_iter().zip(b.into_iter()).collect()
}

// 참조와 제네릭
fn print_value<T: std::fmt::Display>(value: &T) {
    println!("{}", value);
}

fn main() {
    let x = identity(42);          // T = i32
    let s = identity("hello");     // T = &str

    let pairs = zip_first(vec![1, 2, 3], vec!["a", "b", "c"]);
    println!("{:?}", pairs);  // [(1, "a"), (2, "b"), (3, "c")]

    print_value(&42);
    print_value(&"hello");
}
```

TypeScript와 비교:

```typescript
// TypeScript 제네릭 함수
function identity<T>(value: T): T {
    return value;
}

function zipFirst<T, U>(a: T[], b: U[]): [T, U][] {
    return a.map((item, i) => [item, b[i]]);
}
```

문법이 매우 유사합니다. 핵심 차이는 Rust에서 제네릭을 사용할 때 타입이 어떤 동작을 지원하는지 트레이트 바운드로 명시해야 한다는 점입니다.

---

## 구조체의 제네릭

```rust,ignore
// 단일 타입 매개변수
#[derive(Debug)]
struct Wrapper<T> {
    value: T,
}

impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        Wrapper { value }
    }

    fn get(&self) -> &T {
        &self.value
    }

    fn into_inner(self) -> T {
        self.value
    }
}

// 특정 타입에 대한 추가 메서드
impl Wrapper<String> {
    fn uppercase(&self) -> String {
        self.value.to_uppercase()
    }
}

fn main() {
    let w1 = Wrapper::new(42);
    let w2 = Wrapper::new("hello");
    let w3 = Wrapper::new(String::from("world"));

    println!("{}", w1.get());  // 42
    println!("{}", w2.get());  // hello
    println!("{}", w3.uppercase());  // WORLD (String 전용 메서드)
}
```

### 여러 타입 매개변수를 가진 구조체

```rust,ignore
#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V,
}

impl<K: std::fmt::Display, V: std::fmt::Display> KeyValue<K, V> {
    fn print(&self) {
        println!("{}: {}", self.key, self.value);
    }
}

fn main() {
    let kv1 = KeyValue { key: "height", value: 12345u64 };
    let kv2 = KeyValue { key: 1u32, value: "genesis" };

    kv1.print();  // height: 12345
    kv2.print();  // 1: genesis
}
```

### 블록체인에서의 제네릭 구조체

```rust,ignore
/// 제네릭 트랜잭션 — 다양한 페이로드를 담을 수 있음
#[derive(Debug, Clone)]
struct Transaction<T> {
    id: String,
    from: String,
    to: String,
    payload: T,  // 트랜잭션 데이터 타입이 유연
    timestamp: u64,
}

impl<T: Clone> Transaction<T> {
    fn new(from: String, to: String, payload: T) -> Self {
        Transaction {
            id: generate_id(),
            from,
            to,
            payload,
            timestamp: current_timestamp(),
        }
    }
}

// 구체적인 페이로드 타입들
#[derive(Debug, Clone)]
struct TokenTransfer {
    token: String,
    amount: u64,
}

#[derive(Debug, Clone)]
struct ContractCall {
    contract: String,
    method: String,
    args: Vec<String>,
}

fn main() {
    let transfer_tx: Transaction<TokenTransfer> = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        TokenTransfer { token: "USDC".to_string(), amount: 1000 },
    );

    let contract_tx: Transaction<ContractCall> = Transaction::new(
        "Alice".to_string(),
        "0xContract".to_string(),
        ContractCall {
            contract: "0xabcd".to_string(),
            method: "transfer".to_string(),
            args: vec!["Bob".to_string(), "1000".to_string()],
        },
    );

    println!("{:?}", transfer_tx);
    println!("{:?}", contract_tx);
}

fn generate_id() -> String { uuid::Uuid::new_v4().to_string() }
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
```

---

## 열거형의 제네릭

이미 `Option<T>`와 `Result<T, E>`에서 봤습니다. 직접 만들어봅시다:

```rust,ignore
// 이진 트리 (블록체인 Merkle Tree의 기초)
#[derive(Debug)]
enum Tree<T> {
    Leaf(T),
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

impl<T: std::fmt::Display> Tree<T> {
    fn depth(&self) -> usize {
        match self {
            Tree::Leaf(_) => 0,
            Tree::Node { left, right, .. } => {
                1 + left.depth().max(right.depth())
            }
        }
    }

    fn print_inorder(&self) {
        match self {
            Tree::Leaf(v) => print!("{} ", v),
            Tree::Node { value, left, right } => {
                left.print_inorder();
                print!("{} ", value);
                right.print_inorder();
            }
        }
    }
}

fn main() {
    let tree: Tree<i32> = Tree::Node {
        value: 4,
        left: Box::new(Tree::Node {
            value: 2,
            left: Box::new(Tree::Leaf(1)),
            right: Box::new(Tree::Leaf(3)),
        }),
        right: Box::new(Tree::Leaf(5)),
    };

    println!("Depth: {}", tree.depth());  // 2
    tree.print_inorder();  // 1 2 3 4 5
    println!();
}
```

---

## 모노모피제이션: 런타임 비용 없음

Rust의 제네릭은 **모노모피제이션(monomorphization)**으로 구현됩니다. 컴파일 시 각 구체 타입에 대한 특화 버전이 생성됩니다.

```rust,ignore
fn identity<T>(x: T) -> T { x }

// 컴파일러가 이 두 호출을 감지하고:
identity(42);      // T = i32
identity(3.14);    // T = f64

// 이렇게 두 개의 함수를 생성:
// fn identity_i32(x: i32) -> i32 { x }
// fn identity_f64(x: f64) -> f64 { x }
```

**결과**: 런타임에 타입 체크가 없습니다. C++의 템플릿과 동일한 방식으로, 제네릭 코드가 구체 타입과 동일한 성능을 냅니다.

TypeScript와 비교:

```typescript
// TypeScript/JavaScript: 런타임에 타입이 동적으로 처리됨
// 제네릭은 컴파일 타임만의 개념, 런타임에는 지워짐 (type erasure)
function identity<T>(x: T): T { return x; }
// 컴파일 후: function identity(x) { return x; }
```

**트레이드오프**:
- 장점: 런타임 오버헤드 없음, C 수준 성능
- 단점: 컴파일 시간이 길어짐, 바이너리 크기 증가

---

## 타입 매개변수 기본값 (Rust 1.0+)

```rust,ignore
// HashMap은 기본 해셔를 가짐
// pub struct HashMap<K, V, S = RandomState> { ... }

use std::collections::HashMap;

let map: HashMap<String, u64> = HashMap::new();
// S = RandomState (기본값)

// 커스텀 해셔 사용
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;
let custom_map: HashMap<String, u64, BuildHasherDefault<DefaultHasher>> =
    HashMap::with_hasher(BuildHasherDefault::default());
```

---

## 제네릭 타입 추론

Rust의 타입 추론이 강력하기 때문에 대부분의 경우 타입을 명시하지 않아도 됩니다:

```rust,ignore
fn main() {
    // 타입 추론으로 명시 불필요
    let v = vec![1, 2, 3];          // Vec<i32>
    let first = v.first();          // Option<&i32>
    let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();  // Vec<i32>

    // 명확하지 않을 때는 타입 힌트
    let parsed = "42".parse::<u64>().unwrap();  // turbofish 문법
    let parsed: u64 = "42".parse().unwrap();    // 또는 타입 어노테이션

    // 컬렉션 타입이 모호할 때
    let chars: Vec<char> = "hello".chars().collect();
    let set: std::collections::HashSet<_> = v.iter().collect();
}
```

---

## TypeScript와 Rust 제네릭 비교 정리

| 기능 | TypeScript | Rust |
|------|-----------|------|
| 기본 문법 | `<T>` | `<T>` |
| 여러 타입 | `<T, U>` | `<T, U>` |
| 타입 제약 | `<T extends Hashable>` | `<T: Hashable>` (트레이트 바운드) |
| 여러 제약 | `<T extends A & B>` | `<T: A + B>` |
| 기본값 | `<T = string>` | `<T = DefaultType>` (일부 지원) |
| 런타임 동작 | 타입 소거 (erasure) | 모노모피제이션 |
| 성능 | 단일 함수로 동작 | 타입별 특화 함수 생성 |

---

## 요약

- 제네릭으로 타입에 무관한 범용 코드 작성
- 함수, 구조체, 열거형, `impl` 블록 모두에 제네릭 사용 가능
- 모노모피제이션: 컴파일 시 구체 타입별 특화 버전 생성 → 런타임 비용 없음
- 트레이트 바운드(`T: Trait`)로 제네릭 타입에 요구 동작 명시
- 타입 추론으로 대부분 타입 명시 불필요

다음 챕터에서 트레이트(Traits)를 자세히 배웁니다.
