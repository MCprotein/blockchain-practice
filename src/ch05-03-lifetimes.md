# 5.3 수명 (Lifetimes)

## 수명 어노테이션이 필요한 이유

수명(lifetime)은 Rust에서 가장 어렵고 생소한 개념입니다. 하지만 실제로 하는 일은 단순합니다: **참조가 얼마나 오래 유효한지 컴파일러에게 알려주는 것**입니다.

### 문제: 댕글링 참조

```rust,compile_fail
fn main() {
    let r;
    {
        let x = 5;
        r = &x;  // x의 참조를 r에 저장
    }   // x가 여기서 drop됨
    println!("{}", r);  // 에러! r이 가리키는 x는 이미 사라짐
}
```

컴파일러 에러:
```text
error[E0597]: `x` does not live long enough
 --> src/main.rs:5:13
  |
5 |         r = &x;
  |             ^^ borrowed value does not live long enough
6 |     }
  |     - `x` dropped here while still borrowed
7 |     println!("{}", r);
  |                    - borrow later used here
```

컴파일러가 각 변수의 수명을 추적해서 참조가 원본보다 오래 살 수 없도록 합니다.

---

## 함수에서 수명 어노테이션

두 참조를 받아 하나를 반환하는 함수에서 수명이 필요합니다:

```rust,compile_fail
// 컴파일 에러 — 반환하는 참조의 수명을 알 수 없음
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}
// error[E0106]: missing lifetime specifier
// help: this function's return type contains a borrowed value, but the
// signature does not say whether it is borrowed from `x` or `y`
```

컴파일러는 반환되는 `&str`이 `x`의 수명인지 `y`의 수명인지 알 수 없습니다. 수명 어노테이션으로 알려줍니다:

```rust
// 수명 어노테이션 추가
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

`'a`는 수명 매개변수입니다. 이 시그니처는 이렇게 읽습니다:
> "x와 y가 모두 수명 'a를 가질 때, 반환값도 수명 'a를 가진다."

실제로는 "x와 y 중 더 짧은 수명"을 의미합니다.

```rust,ignore
fn main() {
    let string1 = String::from("long string");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(string1.as_str(), string2.as_str());
        println!("Longest: {}", result);  // OK
    }
    // println!("{}", result);  // 에러! string2가 이미 drop됨
    // result의 수명이 string2에 제한됨
}
```

---

## 수명 어노테이션 문법

```rust,ignore
// 참조 타입에 수명 어노테이션
&i32         // 수명 없는 참조 (컴파일러가 추론)
&'a i32      // 수명 'a를 가진 참조
&'a mut i32  // 수명 'a를 가진 가변 참조

// 함수
fn function<'a>(x: &'a str) -> &'a str { x }

// 여러 수명
fn function2<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { x }

// 'static: 프로그램 전체 기간
let s: &'static str = "I live forever";  // 문자열 리터럴
```

---

## 구조체의 수명

구조체가 참조를 필드로 가질 때 수명 어노테이션이 필요합니다:

```rust,ignore
// 구조체가 참조를 소유하지 않고 빌림
struct BlockRef<'a> {
    // &str이 아닌 &'a str — 원본 데이터의 수명에 묶임
    hash: &'a str,
    data: &'a str,
}

impl<'a> BlockRef<'a> {
    fn new(hash: &'a str, data: &'a str) -> Self {
        BlockRef { hash, data }
    }

    fn display(&self) -> String {
        format!("Block[{}]: {}", &self.hash[..6], self.data)
    }
}

fn main() {
    let hash = String::from("abcdef1234567890");
    let data = String::from("Genesis Block");

    let block_ref = BlockRef::new(&hash, &data);
    println!("{}", block_ref.display());
    // block_ref는 hash, data보다 오래 살 수 없음

    // hash나 data가 여기서 drop되면 block_ref도 무효
}
```

**실용적인 조언**: 구조체 필드로 참조 대신 `String`, `Vec<T>` 등 소유된 타입을 사용하면 수명 어노테이션이 필요 없습니다. 성능이 매우 중요한 경우가 아니면 소유된 타입을 선호합니다:

```rust
// 수명 어노테이션 없음 — 소유된 데이터
struct Block {
    hash: String,   // String 소유 (힙에 할당)
    data: String,
}
```

---

## 수명 생략 규칙 (Lifetime Elision Rules)

자주 쓰이는 패턴에서 컴파일러가 수명을 자동으로 추론합니다. 수명을 명시하지 않아도 되는 경우:

**규칙 1**: 각 참조 매개변수는 고유한 수명을 가짐

```rust,ignore
fn foo(x: &str) -> &str { x }
// 컴파일러가 이렇게 처리:
fn foo<'a>(x: &'a str) -> &'a str { x }
```

**규칙 2**: 하나의 참조 입력만 있으면, 반환 참조는 그 수명을 가짐

```rust,ignore
fn first_word(s: &str) -> &str {
    // 수명 명시 불필요
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[..i];
        }
    }
    s
}
// 컴파일러가 이렇게 처리:
fn first_word<'a>(s: &'a str) -> &'a str {
    for (i, byte) in s.as_bytes().iter().enumerate() {
        if *byte == b' ' {
            return &s[..i];
        }
    }
    s
}
```

**규칙 3**: 메서드에서 `&self` 또는 `&mut self`가 있으면, 반환 참조는 self의 수명

```rust,ignore
impl Block {
    fn get_data(&self) -> &str {
        // 수명 명시 불필요 — self의 수명으로 자동 처리
        &self.data
    }
}
// 컴파일러가 이렇게 처리:
// fn get_data<'a>(&'a self) -> &'a str { &self.data }
```

---

## 수명이 필요한 실제 상황

### 상황 1: 두 참조 중 하나를 반환

```rust
// 수명 어노테이션 필요
fn longest_prefix<'a>(s: &'a str, prefix: &'a str) -> &'a str {
    if s.starts_with(prefix) { prefix } else { s }
}
```

### 상황 2: 구조체가 참조를 포함할 때

```rust
struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn current_slice(&self) -> &'a str {
        &self.input[self.position..]
    }
}

fn main() {
    let tx_data = String::from("FROM:Alice TO:Bob AMOUNT:1000");
    let mut parser = Parser::new(&tx_data);
    println!("{:?}", parser.peek());  // Some('F')
    println!("{}", parser.current_slice());
}
```

### 상황 3: 입력과 출력 수명이 다를 때

```rust
fn get_prefix<'a, 'b>(s: &'a str, _separator: &'b str) -> &'a str {
    // _separator의 수명은 반환값과 무관
    // 반환값의 수명은 s에만 묶임
    s.split_once(':').map(|(prefix, _)| prefix).unwrap_or(s)
}
```

---

## 'static 수명

`'static`은 프로그램 전체 기간 동안 유효한 수명입니다:

```rust,ignore
// 문자열 리터럴은 'static
let s: &'static str = "Hello, World!";

// static 상수
static MAX_BLOCK_SIZE: usize = 1_000_000;

// 'static 바운드 — 소유된 타입이거나 'static 참조여야 함
fn spawn_thread<F: Fn() + Send + 'static>(f: F) {
    std::thread::spawn(f);
}

// 오류 타입에서 흔히 보임
fn may_fail() -> Result<(), Box<dyn std::error::Error + 'static>> {
    Ok(())
}
```

---

## Node.js 개발자를 위한 실용 조언

수명을 처음 배울 때 느끼는 답답함의 대부분은:
1. 구조체 필드에 참조를 쓰려다 발생
2. 여러 함수 걸쳐 참조를 전달하려다 발생

**가장 실용적인 해결책**: 소유된 데이터를 사용하세요.

```rust
// 수명 문제를 만날 때의 선택지:

// 1. 소유된 타입 사용 (가장 간단)
struct Config {
    network: String,  // &str 대신 String
    rpc_url: String,
}

// 2. Arc<str> 사용 (공유 소유권)
use std::sync::Arc;
struct Config2 {
    network: Arc<str>,
    rpc_url: Arc<str>,
}

// 3. 수명 어노테이션 (성능이 중요할 때)
struct Config3<'a> {
    network: &'a str,
    rpc_url: &'a str,
}
```

Node.js에서는 객체를 참조로 자유롭게 공유합니다. Rust에서는 소유권이 있으므로 데이터를 복제(`clone()`)하거나, 참조 카운팅(`Rc<T>`, `Arc<T>`)을 사용하거나, 수명을 관리하는 세 가지 선택지가 있습니다.

---

## 수명 관련 자주 보는 에러와 해결법

```text
error[E0106]: missing lifetime specifier
```
→ 함수가 참조를 반환하는데 수명이 불명확. 입력 참조 중 어느 것에서 나오는지 명시하거나, 소유된 타입(`String`) 반환 고려.

```text
error[E0597]: `x` does not live long enough
```
→ 참조가 원본 데이터보다 오래 살려고 함. 데이터의 스코프를 늘리거나, 소유권을 이동(clone).

```text
error[E0502]: cannot borrow `x` as mutable because it is also borrowed as immutable
```
→ 불변 참조가 살아있는 동안 가변 참조 생성 시도. 불변 참조를 먼저 끝내고 가변 참조 사용.

---

## 요약

- 수명: 참조가 유효한 기간을 컴파일러에게 알려주는 어노테이션
- `'a`, `'b` 등으로 표기 (알파벳 소문자, 관례상 짧게)
- 구조체가 참조 필드를 가질 때 수명 어노테이션 필요
- 수명 생략 규칙: 흔한 패턴에서 컴파일러가 자동 추론
- `'static`: 프로그램 전체 기간 동안 유효 (문자열 리터럴, 상수)
- **실용 팁**: 수명이 어려우면 소유된 타입(`String`, `Vec`)으로 해결

다음으로는 Solidity 기초를 배웁니다. 컬렉션(Vec, HashMap 등)은 3주차에서 다룹니다.
