# 2.2 참조와 빌림

## 참조가 왜 필요한가?

앞 챕터에서 소유권을 함수에 전달하면 원래 변수를 사용할 수 없게 된다는 걸 봤습니다:

```rust
fn calculate_length(s: String) -> usize {
    s.len()
}

fn main() {
    let s = String::from("hello");
    let len = calculate_length(s);  // s의 소유권이 이동!
    println!("The length of '???' is {}.", len);
    // println!("{}", s);  // 에러! s는 이미 이동됨
}
```

이 문제를 해결하는 방법이 **참조(reference)**입니다. 참조를 사용하면 소유권을 넘기지 않고 값을 빌려 사용할 수 있습니다.

---

## 불변 참조 `&T`

```rust
fn calculate_length(s: &String) -> usize {  // &String: String의 참조를 받음
    s.len()
}

fn main() {
    let s = String::from("hello");
    let len = calculate_length(&s);  // &s: s의 참조를 전달
    println!("The length of '{}' is {}.", s, len);  // s는 여전히 유효!
}
```

`&s`는 "s를 참조하지만 소유하지는 않는다"는 의미입니다. 소유권이 이동하지 않으므로, 참조가 스코프를 벗어나도 `s`는 해제되지 않습니다.

### 메모리 구조

```
 스택               힙
┌─────────┐        ┌─────────────┐
│   s     │        │             │
│  ptr ───┼───────►│  "hello"    │
│  len: 5 │        │             │
│  cap: 5 │        └─────────────┘
└─────────┘               ▲
     ▲                    │
┌─────────┐               │
│   r (&s)│               │
│  ptr ───┼───────────────┘
└─────────┘
  (s를 가리키는 참조)
```

참조 자체는 스택에 있고, s의 스택 데이터(포인터, 길이, 용량)를 가리킵니다.

### 참조는 불변이 기본

```rust
fn try_to_change(s: &String) {
    s.push_str(", world");  // 컴파일 에러!
    // error[E0596]: cannot borrow `*s` as mutable, as it is behind a `&` reference
}
```

참조를 통해서는 기본적으로 값을 변경할 수 없습니다. 변경하려면 가변 참조를 써야 합니다.

---

## 가변 참조 `&mut T`

```rust
fn change(s: &mut String) {
    s.push_str(", world");  // OK! 가변 참조로 변경 가능
}

fn main() {
    let mut s = String::from("hello");  // mut 키워드 필요!
    change(&mut s);  // &mut s: 가변 참조 전달
    println!("{}", s);  // "hello, world"
}
```

가변 참조를 사용하려면:
1. 변수 자체가 `mut`으로 선언되어야 함
2. 참조를 `&mut`으로 만들어야 함
3. 함수 인자 타입이 `&mut T`여야 함

TypeScript와 비교:

```typescript
// TypeScript: 객체는 기본적으로 가변
function change(s: string[]): void {
    s.push("world");  // 그냥 변경 가능
}

const arr = ["hello"];
change(arr);
console.log(arr);  // ["hello", "world"]

// readonly로 불변 강제
function readOnly(s: readonly string[]): void {
    s.push("world");  // 타입 에러! (컴파일 타임)
}
```

---

## 빌림 규칙 (Borrowing Rules)

Rust의 빌림 규칙은 참조가 항상 유효함을 보장합니다:

### 규칙 1: 가변 참조는 한 번에 하나만

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &mut s;
    let r2 = &mut s;  // 컴파일 에러!
    // error[E0499]: cannot borrow `s` as mutable more than once at a time

    println!("{}, {}", r1, r2);
}
```

**왜?** 두 개의 가변 참조가 동시에 존재하면 데이터 레이스(data race)가 발생할 수 있습니다.

> **데이터 레이스**: 두 포인터가 같은 데이터를 동시에 접근하고, 적어도 하나가 쓰기 작업을 하며, 접근을 동기화하는 메커니즘이 없는 상황.

Node.js는 싱글 스레드여서 이 문제를 신경 쓸 필요가 없었습니다. Rust는 멀티스레드 환경을 기본으로 고려합니다.

```rust
// 해결법 1: 스코프 분리
fn main() {
    let mut s = String::from("hello");

    {
        let r1 = &mut s;
        println!("{}", r1);
    }   // r1의 스코프 끝

    let r2 = &mut s;  // OK! r1은 이미 스코프를 벗어남
    println!("{}", r2);
}

// 해결법 2: NLL (Non-Lexical Lifetimes) — 마지막 사용 후 참조 종료
fn main() {
    let mut s = String::from("hello");
    let r1 = &mut s;
    println!("{}", r1);  // r1의 마지막 사용
    // r1은 여기서 더 이상 사용되지 않으므로 유효 범위 종료

    let r2 = &mut s;  // OK!
    println!("{}", r2);
}
```

### 규칙 2: 불변 참조와 가변 참조의 공존 불가

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;     // 불변 참조
    let r2 = &s;     // 불변 참조 (여러 개 OK)
    let r3 = &mut s; // 컴파일 에러!
    // error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable

    println!("{}, {}, {}", r1, r2, r3);
}
```

**왜?** 불변 참조를 사용하는 코드는 값이 변경되지 않을 거라고 기대합니다. 가변 참조가 동시에 존재하면 이 기대가 깨집니다.

```rust
// NLL 덕분에 이건 OK
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);  // r1, r2의 마지막 사용

    // r1, r2가 더 이상 사용되지 않으므로
    let r3 = &mut s;  // OK!
    println!("{}", r3);
}
```

### 규칙 정리

| 허용 여부 | 상황 |
|----------|------|
| ✅ 허용 | 불변 참조 여러 개 동시에 |
| ✅ 허용 | 가변 참조 하나만 (불변 참조 없을 때) |
| ❌ 불허 | 가변 참조 여러 개 동시에 |
| ❌ 불허 | 불변 참조 + 가변 참조 동시에 |

---

## 댕글링 참조 방지

Rust의 컴파일러는 댕글링 참조(dangling reference)를 방지합니다:

```rust
fn dangle() -> &String {  // 컴파일 에러!
    let s = String::from("hello");
    &s  // s의 참조를 반환하려고 시도
}   // s는 여기서 drop됨! — 반환된 참조가 가리키는 메모리가 해제됨

// error[E0106]: missing lifetime specifier
// help: this function's return type contains a borrowed value,
//       but there is no value for it to be borrowed from
```

**해결책**: 소유권을 반환

```rust
fn no_dangle() -> String {
    let s = String::from("hello");
    s   // 소유권을 반환 — 메모리가 해제되지 않음
}
```

---

## 실용적인 참조 패턴

### 패턴 1: 읽기만 할 때 `&T`

```rust
struct Block {
    index: u64,
    hash: String,
    data: String,
}

impl Block {
    // self를 참조로 받음 — 소유권 이동 없음
    fn get_hash(&self) -> &str {
        &self.hash
    }

    fn get_data(&self) -> &str {
        &self.data
    }

    fn is_valid(&self) -> bool {
        !self.hash.is_empty()
    }
}

fn print_block(block: &Block) {  // 참조로 받음
    println!("Block {}: {}", block.index, block.hash);
}

fn main() {
    let block = Block {
        index: 0,
        hash: String::from("abc123"),
        data: String::from("genesis"),
    };

    print_block(&block);  // 소유권 이동 없음
    println!("Hash: {}", block.get_hash());
    println!("Valid: {}", block.is_valid());
    // block은 여전히 유효
}
```

### 패턴 2: 수정이 필요할 때 `&mut T`

```rust
struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    // &mut self: 자신을 가변으로 빌림
    fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    fn get_last_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    fn len(&self) -> usize {
        self.blocks.len()
    }
}

fn main() {
    let mut chain = Blockchain { blocks: vec![] };

    chain.add_block(Block {
        index: 0,
        hash: String::from("000abc"),
        data: String::from("genesis"),
    });

    if let Some(last) = chain.get_last_block() {
        println!("Last block: {}", last.index);
    }

    println!("Chain length: {}", chain.len());
}
```

### 패턴 3: 여러 필드를 동시에 가변 참조

```rust
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let mut p = Point { x: 1.0, y: 2.0 };

    // 구조체의 서로 다른 필드를 동시에 가변 참조 — OK
    let rx = &mut p.x;
    let ry = &mut p.y;
    *rx += 1.0;
    *ry += 1.0;
    println!("({}, {})", p.x, p.y);
}
```

---

## 역참조 연산자 `*`

참조를 통해 실제 값에 접근하려면 `*`(역참조)를 씁니다:

```rust
fn main() {
    let x = 5;
    let r = &x;       // r은 x의 참조

    println!("{}", r);   // 자동 역참조 — 5 출력
    println!("{}", *r);  // 명시적 역참조 — 5 출력

    let mut y = 10;
    let ry = &mut y;
    *ry += 1;            // 역참조 후 수정
    println!("{}", y);   // 11
}
```

대부분의 경우 Rust가 자동으로 역참조합니다(deref coercion). `.` 연산자는 자동으로 역참조합니다:

```rust
fn main() {
    let s = String::from("hello");
    let r = &s;

    // 다음 두 줄은 동일
    println!("{}", r.len());    // 자동 역참조
    println!("{}", (*r).len()); // 명시적 역참조
}
```

---

## 함수에서 참조 반환 시 주의사항

```rust
// OK: 입력 참조의 수명을 그대로 반환
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[..i];
        }
    }
    s
}

fn main() {
    let sentence = String::from("hello world");
    let word = first_word(&sentence);
    // sentence.clear();  // 에러! word가 sentence를 참조 중
    println!("First word: {}", word);
}
```

반환되는 참조의 수명은 입력 참조의 수명과 연결됩니다. `sentence`가 살아있는 동안만 `word`를 사용할 수 있습니다.

---

## TypeScript/Node.js 개발자를 위한 핵심 차이점

```typescript
// TypeScript: 참조는 항상 공유 가능
class Block {
    data: string;
    constructor(data: string) { this.data = data; }
}

const block = new Block("genesis");
const ref1 = block;  // 같은 객체를 가리킴
const ref2 = block;  // 같은 객체를 가리킴
ref1.data = "modified";
console.log(ref2.data);  // "modified" — 같은 객체!
```

```rust
// Rust: 불변 참조는 여러 개, 가변 참조는 하나만
let mut block = Block { data: String::from("genesis") };

let ref1 = &block;        // 불변 참조
let ref2 = &block;        // 불변 참조 — OK
// let ref3 = &mut block; // 에러! 불변 참조가 살아있는 동안 불가

// 불변 참조를 다 쓴 후
let ref3 = &mut block;    // OK
ref3.data = String::from("modified");
```

이 규칙이 멀티스레드 데이터 레이스를 컴파일 타임에 방지합니다.

---

## 요약

- `&T`: 불변 참조 — 소유권 이동 없이 읽기만
- `&mut T`: 가변 참조 — 소유권 이동 없이 읽기/쓰기
- 빌림 규칙: 불변 참조 여러 개 OR 가변 참조 하나 (동시에 둘 다 안 됨)
- 댕글링 참조: 컴파일러가 방지
- `*`: 역참조 연산자 (`.` 연산자는 자동 역참조)
- 함수 인자는 가능하면 `&T`나 `&str`로 받아서 소유권 이동 방지

다음 챕터에서는 슬라이스(slice)를 배웁니다.
