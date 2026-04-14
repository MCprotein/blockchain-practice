# 2.1 소유권 규칙

## 세 가지 소유권 규칙

Rust의 소유권 시스템은 세 가지 규칙으로 요약됩니다:

1. Rust의 각 값(value)은 **소유자(owner)**라고 불리는 변수를 가진다.
2. 한 번에 소유자는 **하나**만 존재할 수 있다.
3. 소유자가 **스코프(scope)를 벗어나면** 값은 드롭(drop)된다.

이 세 규칙이 모든 것의 기초입니다. 하나씩 살펴봅시다.

---

## 스택 vs 힙 메모리

소유권을 이해하려면 스택과 힙의 차이를 알아야 합니다.

### 스택 (Stack)

- **크기가 컴파일 타임에 알려진** 값들을 저장
- LIFO(Last In, First Out) 구조
- 매우 빠름 (포인터를 밀고 빼는 것뿐)
- 함수 호출 시 자동 할당, 함수 종료 시 자동 해제

스택에 저장되는 타입들:
- `i32`, `u64`, `f64`, `bool`, `char` 등 기본 타입
- 고정 크기 배열 `[i32; 5]`
- 튜플 `(i32, bool)`
- 포인터/참조 자체 (가리키는 데이터가 아닌 포인터)

### 힙 (Heap)

- **크기가 런타임에 결정**되는 값들을 저장
- 운영체제에 메모리를 요청하고, 포인터를 받음
- 스택보다 느림 (할당/해제 비용 있음)
- 명시적으로 해제해야 함 (Rust는 소유권으로 자동화)

힙에 저장되는 타입들:
- `String` (가변 길이 문자열)
- `Vec<T>` (가변 길이 벡터)
- `Box<T>` (힙에 할당된 값)
- `HashMap<K, V>` 등

```
스택                    힙
┌─────────┐            ┌──────────────────┐
│  ptr ───┼───────────►│  "hello, world"  │
│  len: 5 │            └──────────────────┘
│  cap: 11│
└─────────┘
  String s
```

`String`은 스택에 (포인터, 길이, 용량)을 저장하고, 실제 문자 데이터는 힙에 저장합니다.

---

## 이동(Move)

### Copy 타입과 Move 타입

TypeScript에서는 모든 객체가 참조로 공유됩니다:

```typescript
// TypeScript
let a = { value: 42 };
let b = a;  // 참조 복사 (같은 객체를 가리킴)
b.value = 100;
console.log(a.value);  // 100 (a도 변경됨!)
```

Rust는 다릅니다. 값을 변수에 할당하거나 함수에 전달하면 기본적으로 **소유권이 이동**합니다:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1의 소유권이 s2로 이동 (move)

    println!("{}", s2);  // OK
    println!("{}", s1);  // 컴파일 에러!
    // error[E0382]: borrow of moved value: `s1`
}
```

왜 이렇게 동작할까요?

`String`은 (포인터, 길이, 용량)으로 구성됩니다. `s2 = s1`을 하면:

```
이전:                    이동 후:
s1: ptr → "hello"       s1: (무효화됨)
                         s2: ptr → "hello"
```

만약 두 변수가 같은 포인터를 가지고 있다면, 스코프가 끝날 때 같은 메모리를 두 번 해제하는 double-free 버그가 생깁니다. Rust는 이를 막기 위해 이동 후 s1을 무효화합니다.

### 함수 인자로 전달 시 이동

```rust
fn takes_ownership(s: String) {
    println!("Got: {}", s);
}   // s의 스코프 끝 → drop 호출 → 메모리 해제

fn main() {
    let s = String::from("hello");
    takes_ownership(s);  // s의 소유권이 함수로 이동

    println!("{}", s);  // 에러! s는 이미 이동됨
}
```

TypeScript에서는 이런 문제가 없습니다:

```typescript
function takesString(s: string): void {
    console.log(`Got: ${s}`);
}

const s = "hello";
takesString(s);
console.log(s);  // 정상 동작 — JS는 문자열을 복사
```

### 함수에서 소유권 반환

소유권을 함수에서 돌려받을 수 있습니다:

```rust
fn gives_ownership() -> String {
    let s = String::from("hello");
    s  // 소유권이 호출자에게 이동 (return 키워드 생략 가능)
}

fn takes_and_gives_back(s: String) -> String {
    s  // 받은 소유권을 그대로 반환
}

fn main() {
    let s1 = gives_ownership();          // 소유권 획득
    let s2 = String::from("world");
    let s3 = takes_and_gives_back(s2);   // s2 이동 → s3로 돌아옴
    // s2는 더 이상 사용 불가
    println!("{} {}", s1, s3);
}
```

이 패턴은 번거롭습니다. 이 문제를 해결하는 것이 다음 챕터의 **참조(Reference)**입니다.

---

## Copy 타입

일부 타입은 이동 대신 **복사(copy)**됩니다:

```rust
fn main() {
    let x = 5;
    let y = x;  // Copy! x는 여전히 유효

    println!("{}", x);  // OK — x가 복사됨
    println!("{}", y);  // OK
}
```

`i32`는 스택에만 저장되고 크기가 고정되어 있습니다. 복사 비용이 매우 낮으므로 Rust는 자동으로 복사합니다.

**Copy 트레이트를 구현하는 타입들:**

```rust
// 모든 정수 타입
let a: i8 = 1;
let b: i16 = 2;
let c: i32 = 3;
let d: i64 = 4;
let e: i128 = 5;
let f: u8 = 6;
// u16, u32, u64, u128, usize, isize 등

// 부동소수점
let g: f32 = 1.0;
let h: f64 = 2.0;

// bool
let i: bool = true;

// char
let j: char = 'A';

// 튜플 (모든 요소가 Copy인 경우)
let k: (i32, bool) = (1, true);

// 고정 크기 배열 (요소가 Copy인 경우)
let l: [i32; 3] = [1, 2, 3];
```

**Copy가 아닌 타입들 (이동됨):**

```rust
// String — 힙에 데이터가 있음
let s = String::from("hello");

// Vec<T> — 힙에 데이터가 있음
let v = vec![1, 2, 3];

// Box<T> — 힙 할당
let b = Box::new(42);

// 힙 데이터를 포함하는 모든 타입
```

### Copy vs Clone

명시적으로 깊은 복사(deep copy)를 원하면 `clone()`을 사용합니다:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();  // 힙 데이터까지 복사 (비용이 있음)

    println!("{}", s1);  // OK
    println!("{}", s2);  // OK — 완전히 독립된 복사본
}
```

TypeScript와 비교:

```typescript
// TypeScript에서 깊은 복사
const obj1 = { name: "hello" };
const obj2 = { ...obj1 };  // 얕은 복사 (spread)
const obj3 = JSON.parse(JSON.stringify(obj1));  // 깊은 복사
```

---

## String vs &str

Rust에서 가장 혼란스러운 부분 중 하나가 문자열 타입이 두 가지라는 점입니다.

### String: 소유된 문자열

```rust
let s: String = String::from("hello");
let s2: String = "hello".to_string();
let s3: String = String::new();  // 빈 String
```

- 힙에 할당됨
- 가변(내용 변경 가능)
- 소유권이 있음
- 크기를 런타임에 알 수 있음

### &str: 문자열 슬라이스 (참조)

```rust
let s: &str = "hello";  // 문자열 리터럴 — 프로그램 바이너리에 저장
```

- 어딘가의 문자열 데이터를 가리키는 참조
- 불변
- 소유권 없음 (빌려온 것)
- 크기가 고정됨

### 언제 무엇을 쓰나?

```rust
// 함수 인자: &str 를 선호 (더 유연함)
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    let owned = String::from("Alice");
    greet(&owned);      // String → &str 자동 변환 (deref coercion)
    greet("Bob");       // &str 리터럴 직접 전달
}

// 반환값이나 구조체 필드: 소유권이 필요하면 String
struct User {
    name: String,   // 소유된 데이터
    email: String,
}

// 임시 참조만 필요하면 &str (수명 어노테이션 필요할 수 있음)
struct Config<'a> {
    name: &'a str,  // 수명이 있는 참조 (나중에 자세히 배움)
}
```

### TypeScript와 비교

TypeScript의 `string`은 Rust의 두 타입을 모두 커버합니다. Rust가 둘로 나눈 이유는 소유권과 성능 때문입니다:

```typescript
// TypeScript: string은 항상 불변, 새 문자열 생성 시 새 할당
let s = "hello";
let s2 = s + " world";  // 새 문자열 할당

// JavaScript 엔진이 내부적으로 최적화해줌
```

```rust
// Rust: 명시적으로 선택
let s1 = "hello";              // &str: 복사 비용 없음, 불변
let s2 = String::from("hello"); // String: 힙 할당, 가변 가능
let s3 = s2 + " world";        // s2를 소비하고 새 String 반환
```

### 문자열 조작

```rust
fn main() {
    // 문자열 생성
    let mut s = String::from("Hello");

    // 이어붙이기
    s.push_str(", world");  // 문자열 이어붙이기
    s.push('!');            // 문자 하나 이어붙이기
    println!("{}", s);      // "Hello, world!"

    // + 연산자
    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1 + &s2;  // s1의 소유권이 이동됨! s1은 더 이상 사용 불가
    // s1을 소비하고 s2의 참조를 받아 새 String 반환

    // format! 매크로 (소유권 이동 없음, 더 명확)
    let s1 = String::from("Hello");
    let s2 = String::from("world");
    let s3 = format!("{}, {}!", s1, s2);  // s1, s2 모두 여전히 유효
    println!("{}", s1);  // OK
    println!("{}", s3);  // "Hello, world!"

    // 길이
    let len = s3.len();  // 바이트 수
    println!("Length: {}", len);

    // 포함 여부
    println!("{}", s3.contains("world"));  // true

    // 분리
    let parts: Vec<&str> = "a,b,c".split(',').collect();
    println!("{:?}", parts);  // ["a", "b", "c"]

    // 변환
    let upper = "hello".to_uppercase();  // "HELLO"
    let lower = "HELLO".to_lowercase();  // "hello"
    let trimmed = "  hello  ".trim();    // "hello"
}
```

---

## 실제 블록체인 코드에서의 소유권

소유권이 실제 코드에서 어떻게 적용되는지 블록체인 예시로 봅시다:

```rust
#[derive(Debug)]
struct Block {
    index: u64,
    data: String,       // 소유된 데이터
    previous_hash: String,
    hash: String,
}

impl Block {
    fn new(index: u64, data: String, previous_hash: String) -> Block {
        // data와 previous_hash의 소유권이 이 함수로 이동됨
        let hash = calculate_hash(index, &data, &previous_hash);
        // hash 계산 시 참조(&)를 사용 → 소유권 이동 없이 읽기만
        Block {
            index,
            data,
            previous_hash,
            hash,
        }
    }

    fn get_data(&self) -> &str {
        // self.data의 참조를 반환 (&str)
        // 소유권을 넘기지 않음
        &self.data
    }
}

fn calculate_hash(index: u64, data: &str, previous_hash: &str) -> String {
    // &str로 받으므로 소유권 이동 없음
    format!("{}:{}{}", index, data, previous_hash)
    // 새 String을 생성해서 반환
}

fn main() {
    let data = String::from("Genesis Block");
    let prev_hash = String::from("0000000000000000");

    // data와 prev_hash의 소유권이 Block::new로 이동
    let block = Block::new(0, data, prev_hash);

    // data와 prev_hash는 이제 Block이 소유
    // println!("{}", data);  // 에러! 이미 이동됨

    // Block의 데이터는 참조로 읽기
    let block_data: &str = block.get_data();
    println!("Block data: {}", block_data);

    println!("{:#?}", block);
}
```

---

## 소유권 규칙 정리

| 상황 | 동작 |
|------|------|
| `let y = x` (Copy 타입) | 복사 — x, y 모두 유효 |
| `let y = x` (Move 타입) | 이동 — y만 유효, x는 무효 |
| `func(x)` (Copy 타입) | 복사 — x는 여전히 유효 |
| `func(x)` (Move 타입) | 이동 — x는 무효 |
| `let y = x.clone()` | 깊은 복사 — x, y 모두 유효 |
| `let y = &x` | 참조 — 소유권 이동 없음 |

---

## 요약

- Rust는 세 가지 소유권 규칙으로 메모리 안전성을 보장
- 스택 타입(i32, bool 등)은 Copy, 힙 타입(String, Vec 등)은 Move
- Move 후에는 원래 변수를 사용할 수 없음
- `clone()`으로 깊은 복사 가능 (비용 있음)
- `String`은 소유된 가변 문자열, `&str`은 불변 참조
- 함수 인자로 `&str`을 선호, 소유권이 필요하면 `String`

다음 챕터에서는 이 번거로움을 해결하는 **참조(Reference)**를 배웁니다.
