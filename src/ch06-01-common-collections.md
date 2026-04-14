# 6.1 공통 컬렉션: Vec, String, HashMap

## Vec\<T\>: 동적 배열

`Vec<T>`는 JavaScript의 `Array`에 해당하는 동적 크기 배열입니다. 힙에 할당되며, 크기가 런타임에 변경됩니다.

### 생성

```rust
fn main() {
    // 빈 Vec 생성
    let v1: Vec<i32> = Vec::new();

    // 타입 추론 가능하면 생략
    let mut v2 = Vec::new();
    v2.push(1);  // 컴파일러가 Vec<i32>로 추론

    // vec! 매크로로 초기값 지정
    let v3 = vec![1, 2, 3, 4, 5];

    // 특정 크기와 초기값으로
    let v4: Vec<u8> = vec![0; 32];  // [0, 0, 0, ..., 0] (32개)

    // 범위에서 생성
    let v5: Vec<i32> = (0..10).collect();  // [0, 1, 2, ..., 9]

    // 다른 컬렉션에서 변환
    let v6: Vec<String> = vec!["a", "b", "c"]
        .iter()
        .map(|s| s.to_string())
        .collect();
}
```

TypeScript와 비교:
```typescript
const v1: number[] = [];
const v2 = new Array<number>();
const v3 = [1, 2, 3, 4, 5];
const v4 = new Array(32).fill(0);
const v5 = Array.from({ length: 10 }, (_, i) => i);
```

### 읽기와 수정

```rust
fn main() {
    let mut v = vec![10, 20, 30, 40, 50];

    // 인덱스 접근 — 범위 초과 시 panic!
    println!("{}", v[0]);   // 10
    println!("{}", v[4]);   // 50

    // 안전한 접근 — Option 반환
    println!("{:?}", v.get(2));    // Some(30)
    println!("{:?}", v.get(100));  // None (panic 없음)

    // 수정
    v[0] = 100;
    println!("{:?}", v);  // [100, 20, 30, 40, 50]

    // 추가
    v.push(60);
    println!("{:?}", v);  // [100, 20, 30, 40, 50, 60]

    // 마지막 원소 제거
    let last = v.pop();  // Some(60)
    println!("{:?}", last);

    // 특정 위치에 삽입 (O(n) 비용)
    v.insert(1, 15);  // 인덱스 1에 15 삽입
    println!("{:?}", v);  // [100, 15, 20, 30, 40, 50]

    // 특정 위치 제거 (O(n) 비용)
    let removed = v.remove(1);  // 인덱스 1 제거
    println!("Removed: {}", removed);  // 15

    // 마지막과 교환 후 제거 (O(1), 순서 바뀜)
    let swapped = v.swap_remove(0);
    println!("Swap-removed: {}", swapped);  // 100
}
```

### 반복

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];

    // 불변 참조로 반복
    for item in &v {
        println!("{}", item);
    }
    // v는 여전히 유효

    // 인덱스와 함께
    for (i, item) in v.iter().enumerate() {
        println!("[{}] = {}", i, item);
    }

    // 소유권 이동으로 반복 (v는 이후 사용 불가)
    for item in v {
        println!("{}", item);
    }
    // println!("{:?}", v);  // 에러! v는 소비됨

    // 가변 참조로 수정하며 반복
    let mut v2 = vec![1, 2, 3];
    for item in &mut v2 {
        *item *= 2;  // 역참조로 값 수정
    }
    println!("{:?}", v2);  // [2, 4, 6]
}
```

### 유용한 메서드

```rust
fn main() {
    let mut v = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];

    // 길이와 비어있는지
    println!("len: {}", v.len());       // 10
    println!("empty: {}", v.is_empty()); // false

    // 정렬
    v.sort();
    println!("{:?}", v);  // [1, 1, 2, 3, 3, 4, 5, 5, 6, 9]

    // 역순 정렬
    v.sort_by(|a, b| b.cmp(a));
    println!("{:?}", v);  // [9, 6, 5, 5, 4, 3, 3, 2, 1, 1]

    // 커스텀 정렬 (key 기반)
    let mut words = vec!["banana", "apple", "cherry"];
    words.sort_by_key(|s| s.len());
    println!("{:?}", words);  // ["apple", "banana", "cherry"]

    // 중복 제거 (정렬 후)
    v.sort();
    v.dedup();
    println!("{:?}", v);  // [1, 2, 3, 4, 5, 6, 9]

    // 검색
    println!("{:?}", v.contains(&5));   // true
    println!("{:?}", v.iter().position(|&x| x == 5));  // Some(4)

    // 분할
    let (left, right) = v.split_at(3);
    println!("{:?} | {:?}", left, right);

    // 연결
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let combined: Vec<i32> = a.into_iter().chain(b.into_iter()).collect();
    println!("{:?}", combined);

    // 확장
    let mut c = vec![1, 2, 3];
    c.extend([4, 5, 6]);
    println!("{:?}", c);

    // 잘라내기 (길이 제한)
    c.truncate(4);
    println!("{:?}", c);  // [1, 2, 3, 4]

    // 전체 지우기
    c.clear();
    println!("{:?}", c);  // []
}
```

### 블록체인에서의 Vec 활용

```rust
struct MerkleTree {
    leaves: Vec<String>,  // 트랜잭션 해시들
}

impl MerkleTree {
    fn new(transactions: Vec<String>) -> Self {
        MerkleTree { leaves: transactions }
    }

    fn root(&self) -> Option<String> {
        if self.leaves.is_empty() {
            return None;
        }

        let mut current = self.leaves.clone();

        while current.len() > 1 {
            let mut next = Vec::new();
            // 두 개씩 묶어서 해시
            for chunk in current.chunks(2) {
                let combined = match chunk {
                    [left, right] => format!("{}{}", left, right),
                    [left]        => left.clone(),  // 홀수 개면 마지막은 그대로
                    _             => unreachable!(),
                };
                next.push(hash(&combined));
            }
            current = next;
        }

        current.into_iter().next()
    }
}

fn hash(s: &str) -> String {
    format!("{:x}", s.len())  // 실제로는 SHA-256
}
```

---

## String: UTF-8 문자열

`String`은 힙에 할당된 가변 UTF-8 문자열입니다.

### 생성과 변환

```rust
fn main() {
    // 생성 방법들
    let s1 = String::new();
    let s2 = String::from("hello");
    let s3 = "hello".to_string();
    let s4 = "hello".to_owned();  // to_string()과 동일

    // 숫자 → 문자열
    let n = 42;
    let s5 = n.to_string();
    let s6 = format!("{}", n);

    // 문자열 → 숫자
    let parsed: Result<i32, _> = "42".parse();
    let num: i32 = "42".parse().unwrap();
}
```

### 이어붙이기

```rust
fn main() {
    // push_str: 문자열 추가
    let mut s = String::from("Hello");
    s.push_str(", World");
    s.push('!');
    println!("{}", s);  // "Hello, World!"

    // + 연산자 (s1의 소유권이 이동됨!)
    let s1 = String::from("Hello");
    let s2 = String::from(", World!");
    let s3 = s1 + &s2;  // s1은 더 이상 유효하지 않음
    println!("{}", s3);

    // format! (소유권 이동 없음, 권장)
    let s1 = String::from("Hello");
    let s2 = String::from(", World!");
    let s3 = format!("{}{}", s1, s2);
    println!("{} {}", s1, s2);  // s1, s2 모두 유효
    println!("{}", s3);
}
```

### 인덱싱과 슬라이싱

```rust
fn main() {
    let s = String::from("hello");

    // 인덱스로 접근 불가! (UTF-8 때문)
    // let c = s[0];  // 에러!

    // 바이트 슬라이스 (ASCII는 OK, 멀티바이트 문자는 위험)
    let slice = &s[0..3];  // "hel" (바이트 단위)

    // 문자 단위 반복
    for c in s.chars() {
        print!("{} ", c);  // h e l l o
    }

    // 바이트 단위 반복
    for b in s.bytes() {
        print!("{} ", b);  // 104 101 108 108 111
    }

    // 한글 처리
    let korean = String::from("안녕하세요");
    println!("len (bytes): {}", korean.len());     // 15 (한글 = 3바이트)
    println!("chars: {}", korean.chars().count()); // 5 (문자 수)

    // 첫 번째 문자
    let first: Option<char> = korean.chars().next();
    println!("{:?}", first);  // Some('안')

    // n번째 문자 (O(n))
    let third: Option<char> = korean.chars().nth(2);
    println!("{:?}", third);  // Some('하')
}
```

### 주요 String 메서드

```rust
fn main() {
    let s = String::from("  Hello, World!  ");

    // 공백 제거
    println!("{}", s.trim());         // "Hello, World!"
    println!("{}", s.trim_start());   // "Hello, World!  "
    println!("{}", s.trim_end());     // "  Hello, World!"

    // 대소문자
    println!("{}", s.trim().to_uppercase());  // "HELLO, WORLD!"
    println!("{}", s.trim().to_lowercase());  // "hello, world!"

    // 검색
    println!("{}", s.contains("World"));          // true
    println!("{}", s.starts_with("  Hello"));     // true
    println!("{}", s.ends_with("!  "));           // true
    println!("{:?}", s.find("World"));            // Some(9)

    // 분리
    let csv = "Alice,Bob,Carol";
    let names: Vec<&str> = csv.split(',').collect();
    println!("{:?}", names);  // ["Alice", "Bob", "Carol"]

    // 줄 분리
    let text = "line1\nline2\nline3";
    for line in text.lines() {
        println!("{}", line);
    }

    // 교체
    let replaced = "hello world".replace("world", "Rust");
    println!("{}", replaced);  // "hello Rust"

    // 반복
    let repeated = "abc".repeat(3);
    println!("{}", repeated);  // "abcabcabc"

    // 문자 확인
    println!("{}", "123".chars().all(|c| c.is_ascii_digit()));   // true
    println!("{}", "abc".chars().all(|c| c.is_alphabetic()));    // true

    // 분할 후 수집
    let words: Vec<&str> = "one two three".split_whitespace().collect();
    println!("{:?}", words);  // ["one", "two", "three"]
}
```

---

## HashMap\<K, V\>: 키-값 저장소

`HashMap<K, V>`는 JavaScript의 `Map`에 해당합니다.

### 생성과 삽입

```rust
use std::collections::HashMap;

fn main() {
    // 빈 HashMap 생성
    let mut scores: HashMap<String, u64> = HashMap::new();

    // 삽입
    scores.insert(String::from("Alice"), 100);
    scores.insert(String::from("Bob"), 200);
    scores.insert(String::from("Carol"), 150);

    // 리터럴로 생성 (collect 이용)
    let map: HashMap<&str, i32> = [
        ("one", 1),
        ("two", 2),
        ("three", 3),
    ].iter().cloned().collect();

    // Rust 1.56+ 방법
    let map2 = HashMap::from([
        ("Alice", 100),
        ("Bob", 200),
    ]);

    println!("{:?}", scores);
}
```

### 읽기

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::from([
        (String::from("Alice"), 100u64),
        (String::from("Bob"), 200u64),
    ]);

    // 인덱스 접근 — 키가 없으면 panic!
    // let score = map["Charlie"];  // panic!

    // 안전한 접근 — Option 반환
    let alice_score = map.get("Alice");    // Some(&100)
    let charlie_score = map.get("Charlie"); // None

    println!("{:?}", alice_score);   // Some(100)
    println!("{:?}", charlie_score); // None

    // 참조 없이 값만 얻기
    let score = map.get("Alice").copied();  // Option<u64> (Copy 타입이므로)
    let score2 = map.get("Alice").cloned(); // Option<u64> (Clone으로)

    // 키 존재 확인
    println!("{}", map.contains_key("Alice"));  // true

    // 기본값으로 가져오기
    let score = map.get("Charlie").copied().unwrap_or(0);
    println!("{}", score);  // 0
}
```

### 업데이트

```rust
use std::collections::HashMap;

fn main() {
    let mut map: HashMap<String, u64> = HashMap::new();

    // 1. 덮어쓰기
    map.insert(String::from("Alice"), 100);
    map.insert(String::from("Alice"), 200);  // 기존 값 교체
    println!("{:?}", map.get("Alice"));  // Some(200)

    // 2. 없을 때만 삽입 (entry API)
    map.entry(String::from("Bob")).or_insert(150);
    map.entry(String::from("Bob")).or_insert(999);  // 이미 있으므로 무시
    println!("{:?}", map.get("Bob"));  // Some(150)

    // 3. 기존 값 기반 업데이트
    let text = "hello world hello rust world hello";
    let mut word_count: HashMap<&str, u32> = HashMap::new();

    for word in text.split_whitespace() {
        let count = word_count.entry(word).or_insert(0);
        *count += 1;  // 역참조로 값 수정
    }
    println!("{:?}", word_count);
    // {"hello": 3, "world": 2, "rust": 1}

    // 4. 조건부 업데이트
    map.entry(String::from("Carol"))
        .or_insert_with(|| 300);  // 없을 때 클로저 실행

    // 5. 삭제
    map.remove("Alice");
    println!("{}", map.contains_key("Alice"));  // false
}
```

### 반복

```rust
use std::collections::HashMap;

fn main() {
    let map = HashMap::from([
        ("Alice", 100),
        ("Bob", 200),
        ("Carol", 150),
    ]);

    // 키-값 쌍 반복 (순서 불보장!)
    for (name, score) in &map {
        println!("{}: {}", name, score);
    }

    // 키만
    for name in map.keys() {
        println!("{}", name);
    }

    // 값만
    for score in map.values() {
        println!("{}", score);
    }

    // 정렬된 순서로 (BTreeMap 사용하거나 Vec으로 변환)
    let mut sorted: Vec<(&str, &i32)> = map.iter().collect();
    sorted.sort_by_key(|(k, _)| *k);
    for (name, score) in sorted {
        println!("{}: {}", name, score);
    }
}
```

### 블록체인에서의 HashMap 활용

```rust
use std::collections::HashMap;

struct UTXO {
    txid: String,
    vout: u32,
    amount: u64,
}

struct UTXOSet {
    // txid:vout → UTXO
    utxos: HashMap<String, UTXO>,
}

impl UTXOSet {
    fn new() -> Self {
        UTXOSet { utxos: HashMap::new() }
    }

    fn add(&mut self, utxo: UTXO) {
        let key = format!("{}:{}", utxo.txid, utxo.vout);
        self.utxos.insert(key, utxo);
    }

    fn spend(&mut self, txid: &str, vout: u32) -> Option<UTXO> {
        let key = format!("{}:{}", txid, vout);
        self.utxos.remove(&key)
    }

    fn get_balance(&self, address: &str) -> u64 {
        self.utxos.values()
            .filter(|u| u.txid.starts_with(address))  // 실제로는 주소 비교
            .map(|u| u.amount)
            .sum()
    }
}
```

---

## JavaScript Array/Map vs Rust Vec/HashMap 총 비교

```typescript
// JavaScript
const arr = [1, 2, 3];
arr.push(4);
arr.pop();
arr[0];
arr.includes(2);
arr.indexOf(2);
arr.slice(0, 2);
arr.splice(1, 1);
arr.sort();
arr.reverse();
arr.find(x => x > 2);
arr.filter(x => x > 1);
arr.map(x => x * 2);
arr.reduce((acc, x) => acc + x, 0);
arr.forEach(x => console.log(x));
arr.some(x => x > 2);
arr.every(x => x > 0);
arr.flat();
arr.flatMap(x => [x, x * 2]);

const map = new Map();
map.set("key", "value");
map.get("key");
map.has("key");
map.delete("key");
map.size;
```

```rust
// Rust
let mut v = vec![1, 2, 3];
v.push(4);
v.pop();
v[0];
v.contains(&2);
v.iter().position(|&x| x == 2);
v[0..2].to_vec();   // 슬라이스 후 Vec으로
v.remove(1);
v.sort();
v.reverse();
v.iter().find(|&&x| x > 2);
v.iter().filter(|&&x| x > 1);
v.iter().map(|&x| x * 2);
v.iter().fold(0, |acc, &x| acc + x);
v.iter().for_each(|x| println!("{}", x));
v.iter().any(|&x| x > 2);
v.iter().all(|&x| x > 0);
v.iter().flatten();
v.iter().flat_map(|&x| vec![x, x * 2]);

let mut map = HashMap::new();
map.insert("key", "value");
map.get("key");
map.contains_key("key");
map.remove("key");
map.len();
```

---

## 요약

- `Vec<T>`: 동적 배열, `push`/`pop`/`insert`/`remove`
- `String`: UTF-8 문자열, 인덱스 접근 불가 (chars()로 문자 단위 접근)
- `HashMap<K, V>`: 키-값 저장, `entry()` API로 조건부 삽입/업데이트
- 컬렉션 반복: `&v`(불변 참조), `&mut v`(가변 참조), `v`(소유권 이동)
- `get()`으로 안전하게 접근 (Option 반환), 인덱스는 panic 위험

다음 챕터에서 클로저를 배웁니다.
