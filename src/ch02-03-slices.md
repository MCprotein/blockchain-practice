# 2.3 슬라이스

## 슬라이스란?

슬라이스(slice)는 컬렉션의 일부를 소유권 없이 참조하는 타입입니다. 연속된 메모리의 특정 구간을 가리키는 "창문"이라고 생각하면 됩니다.

슬라이스는 두 가지 정보를 가집니다:
1. 시작 위치를 가리키는 포인터
2. 슬라이스의 길이

---

## 문자열 슬라이스 `&str`

앞서 `&str`이 "문자열 슬라이스"라고 했습니다. 이제 그 의미를 명확히 봅시다.

```rust
fn main() {
    let s = String::from("hello world");

    // 슬라이스: [시작 인덱스..끝 인덱스(미포함)]
    let hello = &s[0..5];   // "hello"
    let world = &s[6..11];  // "world"

    println!("{}", hello);  // hello
    println!("{}", world);  // world

    // 처음부터: [..5] == [0..5]
    let hello2 = &s[..5];

    // 끝까지: [6..] == [6..s.len()]
    let world2 = &s[6..];

    // 전체: [..] == [0..s.len()]
    let whole = &s[..];

    println!("{} {}", hello2, world2);
    println!("{}", whole);
}
```

### 메모리 구조

```
String s:
 스택                힙
┌─────────┐         ┌─────────────────────────┐
│  ptr ───┼────────►│ h  e  l  l  o     w  o  r  l  d │
│  len:11 │         │ 0  1  2  3  4  5  6  7  8  9 10 │
│  cap:11 │         └─────────────────────────┘
└─────────┘                 ▲         ▲
                            │         │
&s[0..5] "hello":           │         │
┌─────────┐                 │         │
│  ptr ───┼─────────────────┘         │
│  len: 5 │                           │
└─────────┘                           │
                                      │
&s[6..11] "world":                    │
┌─────────┐                           │
│  ptr ───┼───────────────────────────┘
│  len: 5 │
└─────────┘
```

슬라이스는 새 메모리를 할당하지 않습니다. 원본 String의 특정 구간을 참조할 뿐입니다.

### 슬라이스와 소유권

```rust
fn main() {
    let mut s = String::from("hello world");

    let word = first_word(&s);  // &s의 슬라이스를 반환
    // word는 s의 일부를 참조

    s.clear();  // 컴파일 에러!
    // error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
    // word가 s를 불변으로 빌리고 있으므로 s를 수정할 수 없음

    println!("The first word is: {}", word);
}

fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[..i];
        }
    }
    s
}
```

슬라이스가 원본 데이터의 참조를 유지하므로, 원본이 수정되면 슬라이스가 무효화될 수 있습니다. Rust는 이를 컴파일 타임에 방지합니다.

### 문자열 리터럴은 슬라이스

```rust
let s = "hello world";  // 타입: &str
```

문자열 리터럴은 프로그램 바이너리에 저장된 문자열 데이터를 가리키는 슬라이스입니다. 그래서 타입이 `&str`이고 불변입니다.

### 함수에서 &String 대신 &str 선호

```rust
// 덜 유연한 버전
fn first_word_v1(s: &String) -> &str {
    // String 참조만 받을 수 있음
    &s[..5]
}

// 더 유연한 버전 (권장)
fn first_word_v2(s: &str) -> &str {
    // String 참조(&String)도 받을 수 있고,
    // 문자열 리터럴(&str)도 받을 수 있음
    &s[..5]
}

fn main() {
    let owned = String::from("hello world");
    let literal = "hello world";

    // &String → &str 자동 변환 (deref coercion)
    first_word_v2(&owned);    // OK
    first_word_v2(literal);   // OK (이미 &str)

    // &String만 받는 함수
    first_word_v1(&owned);    // OK
    // first_word_v1(literal); // 에러! 타입 불일치
}
```

TypeScript에서 `string`과 `String` 객체를 구분하지 않는 것과 달리, Rust에서는 `&str`이 더 범용적인 타입입니다.

---

## 배열 슬라이스 `&[T]`

문자열 슬라이스와 동일한 개념이 배열에도 적용됩니다:

```rust
fn main() {
    let a = [1, 2, 3, 4, 5];

    // 배열의 슬라이스
    let slice = &a[1..3];  // [2, 3]
    println!("{:?}", slice);

    // 슬라이스의 타입은 &[i32]
    let first_three: &[i32] = &a[..3];
    println!("{:?}", first_three);  // [1, 2, 3]

    // 슬라이스의 길이
    println!("Length: {}", slice.len());

    // 슬라이스 반복
    for item in slice {
        println!("{}", item);
    }
}
```

### Vec에서 슬라이스

```rust
fn sum(numbers: &[i32]) -> i32 {  // &[i32]: i32 배열의 슬라이스
    numbers.iter().sum()
}

fn main() {
    // Vec에서 슬라이스
    let v = vec![1, 2, 3, 4, 5];
    let total = sum(&v);           // Vec → &[i32] 자동 변환
    let partial = sum(&v[1..4]);   // 일부만

    // 배열에서 슬라이스
    let a = [1, 2, 3, 4, 5];
    let total2 = sum(&a);          // [i32; 5] → &[i32] 자동 변환

    println!("{}, {}, {}", total, partial, total2);
}
```

`&[T]`를 인자로 받으면 `Vec<T>`와 `[T; N]` 모두 받을 수 있습니다. `&str`이 `String`과 `&str` 리터럴 모두를 받을 수 있는 것과 동일한 패턴입니다.

---

## 슬라이스 관련 주요 메서드

```rust
fn main() {
    let v = vec![3, 1, 4, 1, 5, 9, 2, 6];
    let s = v.as_slice();  // Vec → &[i32]

    // 길이
    println!("len: {}", s.len());

    // 비어있는지
    println!("empty: {}", s.is_empty());

    // 첫/마지막 원소
    println!("first: {:?}", s.first());  // Some(3)
    println!("last: {:?}", s.last());    // Some(6)

    // 인덱스 접근 (안전한 방법)
    println!("get(2): {:?}", s.get(2));  // Some(4)
    println!("get(100): {:?}", s.get(100));  // None (panic 없음!)

    // 인덱스 접근 (위험한 방법 — 범위 초과시 panic)
    println!("s[2]: {}", s[2]);  // 4

    // 포함 여부
    println!("contains 9: {}", s.contains(&9));

    // 정렬 (슬라이스에서 직접 정렬하면 원본이 바뀜)
    let mut v2 = vec![3, 1, 4, 1, 5];
    v2.sort();
    println!("{:?}", v2);  // [1, 1, 3, 4, 5]

    // 분할
    let (left, right) = s.split_at(4);
    println!("left: {:?}", left);   // [3, 1, 4, 1]
    println!("right: {:?}", right); // [5, 9, 2, 6]

    // 청크로 나누기
    for chunk in s.chunks(3) {
        println!("{:?}", chunk);
    }
    // [3, 1, 4]
    // [1, 5, 9]
    // [2, 6]

    // 윈도우 슬라이딩
    for window in s.windows(3) {
        println!("{:?}", window);
    }
    // [3, 1, 4]
    // [1, 4, 1]
    // ...
}
```

### 블록체인에서 슬라이스 활용

```rust
fn verify_chain(blocks: &[Block]) -> bool {
    // &[Block]: Block 슬라이스 (소유권 없이 검증)
    if blocks.is_empty() {
        return true;
    }

    // 연속한 두 블록 쌍을 윈도우로 검사
    for window in blocks.windows(2) {
        let prev = &window[0];
        let curr = &window[1];

        if curr.previous_hash != prev.hash {
            return false;
        }
        if curr.index != prev.index + 1 {
            return false;
        }
    }
    true
}

struct Block {
    index: u64,
    hash: String,
    previous_hash: String,
}

fn main() {
    let chain = vec![
        Block { index: 0, hash: "abc".to_string(), previous_hash: "000".to_string() },
        Block { index: 1, hash: "def".to_string(), previous_hash: "abc".to_string() },
        Block { index: 2, hash: "ghi".to_string(), previous_hash: "def".to_string() },
    ];

    println!("Chain valid: {}", verify_chain(&chain));
    // verify_chain이 chain의 소유권을 가져가지 않음
    println!("Chain length: {}", chain.len());
}
```

---

## 슬라이스 인덱싱 주의사항

Rust의 문자열은 UTF-8로 인코딩됩니다. 멀티바이트 문자를 바이트 인덱스로 자르면 패닉이 발생합니다:

```rust
fn main() {
    let s = String::from("안녕하세요");

    // 한글은 UTF-8에서 3바이트
    // "안"은 바이트 0..3, "녕"은 3..6, ...

    // 이건 OK (바이트 경계에 맞게)
    let an = &s[0..3];  // "안"
    println!("{}", an);

    // 이건 패닉! (바이트 경계 중간을 자름)
    // let wrong = &s[0..1];  // panic!
    // thread 'main' panicked at 'byte index 1 is not a char boundary'

    // 안전한 방법: chars()로 문자 단위 접근
    let first_char: Option<char> = s.chars().next();
    println!("{:?}", first_char);  // Some('안')

    // 문자 단위 슬라이싱
    let first_two: String = s.chars().take(2).collect();
    println!("{}", first_two);  // "안녕"
}
```

TypeScript와 비교:

```typescript
const s = "안녕하세요";
const first = s[0];        // "안" — JS는 문자 단위로 인덱싱
const slice = s.slice(0, 2); // "안녕" — 문자 단위
```

JavaScript/TypeScript는 내부적으로 UTF-16을 사용하고 인덱싱이 코드 유닛 기준이라 이모지 같은 서로게이트 페어에서도 문제가 생길 수 있습니다. Rust는 더 명시적입니다.

---

## 요약

| 개념 | 타입 | 설명 |
|------|------|------|
| 문자열 소유 | `String` | 힙에 할당된 가변 문자열 |
| 문자열 슬라이스 | `&str` | 문자열 데이터의 참조 |
| 문자열 리터럴 | `&str` | 바이너리에 저장된 데이터의 참조 |
| 배열 슬라이스 | `&[T]` | 배열/Vec의 일부 참조 |

- 슬라이스는 새 메모리를 할당하지 않음 (참조)
- `&str`을 인자로 받으면 `String`과 `&str` 리터럴 모두 받을 수 있음
- `&[T]`를 인자로 받으면 `Vec<T>`와 `[T; N]` 모두 받을 수 있음
- UTF-8 문자열의 바이트 인덱싱은 주의 필요

다음으로는 블록과 체인 구조를 알아본 후, 구조체와 열거형으로 데이터를 모델링하는 방법을 배웁니다.
