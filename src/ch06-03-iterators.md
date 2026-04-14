# 6.3 이터레이터 (Iterators)

## Iterator 트레이트

Rust의 이터레이터는 `Iterator` 트레이트를 구현합니다:

```rust
pub trait Iterator {
    type Item;  // 순회할 원소 타입

    fn next(&mut self) -> Option<Self::Item>;  // 유일하게 구현 필수인 메서드

    // 나머지 수백 개의 메서드는 기본 구현이 있음
    // map, filter, collect, fold, take, skip, ...
}
```

`next()`를 구현하면 `map`, `filter`, `collect` 등 모든 어댑터를 무료로 얻습니다.

---

## 이터레이터 생성

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];

    // iter(): 불변 참조 이터레이터 → &T
    let mut iter1 = v.iter();
    println!("{:?}", iter1.next());  // Some(1)
    println!("{:?}", iter1.next());  // Some(2)

    // into_iter(): 소유권을 가져가는 이터레이터 → T
    let v2 = vec![1, 2, 3];
    let mut iter2 = v2.into_iter();
    println!("{:?}", iter2.next());  // Some(1)
    // v2는 이제 사용 불가

    // iter_mut(): 가변 참조 이터레이터 → &mut T
    let mut v3 = vec![1, 2, 3];
    for x in v3.iter_mut() {
        *x *= 2;
    }
    println!("{:?}", v3);  // [2, 4, 6]

    // for 루프는 into_iter()를 자동 호출
    let v4 = vec![1, 2, 3];
    for x in &v4 {      // = v4.iter()
        print!("{} ", x);
    }
    for x in &mut v3 {  // = v3.iter_mut()
        *x += 1;
    }
    for x in v4 {       // = v4.into_iter()
        print!("{} ", x);
    }
}
```

---

## 이터레이터 어댑터

어댑터는 이터레이터를 받아 새 이터레이터를 반환합니다. **지연 평가(lazy evaluation)**입니다 — 최종 소비 메서드(`collect`, `sum`, `for_each` 등)가 호출될 때까지 실행되지 않습니다.

### map: 변환

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];

    // 각 원소에 함수 적용
    let doubled: Vec<i32> = v.iter().map(|&x| x * 2).collect();
    println!("{:?}", doubled);  // [2, 4, 6, 8, 10]

    // 타입 변환
    let strings: Vec<String> = v.iter().map(|x| x.to_string()).collect();
    println!("{:?}", strings);  // ["1", "2", "3", "4", "5"]

    // 블록체인: 트랜잭션 해시 계산
    let transactions = vec!["tx1_data", "tx2_data", "tx3_data"];
    let hashes: Vec<String> = transactions.iter()
        .map(|data| compute_hash(data))
        .collect();
}

fn compute_hash(data: &str) -> String {
    format!("{:x}", data.len())  // 실제로는 SHA-256
}
```

### filter: 조건 필터링

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let evens: Vec<&i32> = numbers.iter().filter(|&&x| x % 2 == 0).collect();
    println!("{:?}", evens);  // [2, 4, 6, 8, 10]

    // 소유된 값으로
    let evens_owned: Vec<i32> = numbers.iter().filter(|&&x| x % 2 == 0).copied().collect();

    // 블록체인: 특정 조건의 트랜잭션 필터링
    struct Transaction { amount: u64, confirmed: bool }

    let txs = vec![
        Transaction { amount: 100, confirmed: true },
        Transaction { amount: 500, confirmed: false },
        Transaction { amount: 1000, confirmed: true },
    ];

    let confirmed_large: Vec<&Transaction> = txs.iter()
        .filter(|tx| tx.confirmed && tx.amount >= 500)
        .collect();
    println!("Confirmed large txs: {}", confirmed_large.len());  // 1
}
```

### map + filter 체이닝

```rust
fn main() {
    let data = vec!["42", "abc", "100", "def", "7"];

    // 파싱 성공한 것만 필터링
    let valid_numbers: Vec<u32> = data.iter()
        .filter_map(|s| s.parse::<u32>().ok())  // filter + map 한번에
        .collect();
    println!("{:?}", valid_numbers);  // [42, 100, 7]

    // filter_map = filter(Some) + map(unwrap)
    let doubled_valid: Vec<u32> = data.iter()
        .filter_map(|s| s.parse::<u32>().ok())
        .map(|n| n * 2)
        .collect();
    println!("{:?}", doubled_valid);  // [84, 200, 14]
}
```

### collect: 이터레이터를 컬렉션으로

```rust
use std::collections::{HashMap, HashSet};

fn main() {
    let numbers = vec![1, 2, 3, 2, 1];

    // Vec로 수집
    let v: Vec<i32> = numbers.iter().copied().collect();

    // HashSet으로 수집 (중복 제거)
    let set: HashSet<i32> = numbers.iter().copied().collect();
    println!("{:?}", set);  // {1, 2, 3}

    // HashMap으로 수집
    let words = vec!["hello", "world", "rust"];
    let word_lengths: HashMap<&str, usize> = words.iter()
        .map(|&w| (w, w.len()))
        .collect();
    println!("{:?}", word_lengths);

    // String으로 수집
    let chars = vec!['h', 'e', 'l', 'l', 'o'];
    let s: String = chars.into_iter().collect();
    println!("{}", s);  // "hello"

    // Result<Vec, E>로 수집 (하나라도 Err이면 전체 Err)
    let strings = vec!["1", "2", "3"];
    let numbers: Result<Vec<u32>, _> = strings.iter().map(|s| s.parse::<u32>()).collect();
    println!("{:?}", numbers);  // Ok([1, 2, 3])

    let strings2 = vec!["1", "abc", "3"];
    let numbers2: Result<Vec<u32>, _> = strings2.iter().map(|s| s.parse::<u32>()).collect();
    println!("{:?}", numbers2);  // Err(ParseIntError { ... })
}
```

### fold와 reduce: 누적

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // fold: 초기값 + 누적 함수
    let sum = numbers.iter().fold(0i32, |acc, &x| acc + x);
    let product = numbers.iter().fold(1i32, |acc, &x| acc * x);
    println!("Sum: {}, Product: {}", sum, product);  // 15, 120

    // sum(), product(): 특화된 fold
    let sum2: i32 = numbers.iter().sum();
    let product2: i32 = numbers.iter().product();
    println!("{}, {}", sum2, product2);

    // reduce: 초기값 없음 (첫 원소가 초기값)
    let max = numbers.iter().copied().reduce(|a, b| if a > b { a } else { b });
    println!("{:?}", max);  // Some(5)

    // max(), min()
    println!("{:?}", numbers.iter().max());  // Some(5)
    println!("{:?}", numbers.iter().min());  // Some(1)

    // 블록체인: 총 트랜잭션 금액
    struct Tx { amount: u64 }
    let txs = vec![Tx { amount: 100 }, Tx { amount: 200 }, Tx { amount: 150 }];
    let total: u64 = txs.iter().map(|tx| tx.amount).sum();
    println!("Total: {}", total);  // 450
}
```

### take, skip, enumerate, zip

```rust
fn main() {
    let numbers: Vec<i32> = (1..=10).collect();

    // take: 처음 n개
    let first_three: Vec<i32> = numbers.iter().copied().take(3).collect();
    println!("{:?}", first_three);  // [1, 2, 3]

    // skip: 처음 n개 건너뛰기
    let after_three: Vec<i32> = numbers.iter().copied().skip(3).collect();
    println!("{:?}", after_three);  // [4, 5, 6, 7, 8, 9, 10]

    // enumerate: 인덱스와 함께
    for (i, &n) in numbers.iter().enumerate().take(3) {
        println!("[{}] = {}", i, n);  // [0] = 1, [1] = 2, [2] = 3
    }

    // zip: 두 이터레이터를 쌍으로 묶기
    let names = vec!["Alice", "Bob", "Carol"];
    let scores = vec![100, 200, 150];
    let pairs: Vec<(&&str, &i32)> = names.iter().zip(scores.iter()).collect();
    println!("{:?}", pairs);

    // 더 실용적으로
    let combined: Vec<String> = names.iter().zip(scores.iter())
        .map(|(name, score)| format!("{}: {}", name, score))
        .collect();
    println!("{:?}", combined);

    // chain: 두 이터레이터를 이어 붙이기
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let chained: Vec<i32> = a.iter().chain(b.iter()).copied().collect();
    println!("{:?}", chained);  // [1, 2, 3, 4, 5, 6]

    // flat_map: map 후 평탄화
    let words = vec!["hello world", "foo bar"];
    let all_words: Vec<&str> = words.iter()
        .flat_map(|s| s.split_whitespace())
        .collect();
    println!("{:?}", all_words);  // ["hello", "world", "foo", "bar"]
}
```

### any, all, find, position

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // any: 하나라도 조건 만족하면 true (단락 평가)
    println!("{}", numbers.iter().any(|&x| x > 4));   // true
    println!("{}", numbers.iter().any(|&x| x > 10));  // false

    // all: 모두 조건 만족하면 true (단락 평가)
    println!("{}", numbers.iter().all(|&x| x > 0));  // true
    println!("{}", numbers.iter().all(|&x| x > 2));  // false

    // find: 조건 만족하는 첫 번째 원소
    let found = numbers.iter().find(|&&x| x > 3);
    println!("{:?}", found);  // Some(4)

    // position: 조건 만족하는 첫 번째 인덱스
    let pos = numbers.iter().position(|&x| x > 3);
    println!("{:?}", pos);  // Some(3)

    // count
    let count = numbers.iter().filter(|&&x| x % 2 == 0).count();
    println!("{}", count);  // 2 (2, 4)
}
```

---

## 지연 평가 (Lazy Evaluation)

이터레이터 어댑터는 지연 평가됩니다. `collect()` 같은 소비 메서드가 호출될 때까지 실행되지 않습니다:

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];

    // 이 코드는 아직 아무것도 실행하지 않음
    let iter = v.iter()
        .map(|x| {
            println!("mapping {}", x);  // 아직 실행 안 됨!
            x * 2
        })
        .filter(|x| x > &4);

    println!("Iterator created, nothing executed yet");

    // collect()를 호출할 때 실제 실행
    let result: Vec<i32> = iter.collect();
    println!("{:?}", result);
}
// 출력:
// Iterator created, nothing executed yet
// mapping 1
// mapping 2
// mapping 3   (filter에서 거름)
// mapping 4
// mapping 5
// [6, 8, 10]
```

**장점**: 불필요한 중간 컬렉션을 만들지 않아 메모리 효율적입니다.

```rust
// take(3)이 있으면 세 개 찾은 후 중단 (나머지는 실행 안 됨)
let first_three_even: Vec<i32> = (0..)  // 무한 이터레이터!
    .filter(|x| x % 2 == 0)
    .take(3)
    .collect();
println!("{:?}", first_three_even);  // [0, 2, 4]
```

---

## 커스텀 이터레이터 구현

```rust
struct BlockHeightRange {
    current: u64,
    end: u64,
}

impl BlockHeightRange {
    fn new(start: u64, end: u64) -> Self {
        BlockHeightRange { current: start, end }
    }
}

impl Iterator for BlockHeightRange {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.current < self.end {
            let height = self.current;
            self.current += 1;
            Some(height)
        } else {
            None
        }
    }
}

fn main() {
    let range = BlockHeightRange::new(100, 105);

    // Iterator 트레이트를 구현했으므로 모든 어댑터 사용 가능
    let hashes: Vec<String> = range
        .map(|height| format!("block_{}", height))
        .collect();
    println!("{:?}", hashes);
    // ["block_100", "block_101", "block_102", "block_103", "block_104"]

    // for 루프에서도 사용 가능
    for height in BlockHeightRange::new(200, 203) {
        println!("Processing block {}", height);
    }
}
```

---

## JavaScript Array 메서드와 비교

```typescript
// JavaScript
const arr = [1, 2, 3, 4, 5];

arr.map(x => x * 2);                      // [2, 4, 6, 8, 10]
arr.filter(x => x % 2 === 0);             // [2, 4]
arr.reduce((acc, x) => acc + x, 0);       // 15
arr.find(x => x > 3);                     // 4
arr.findIndex(x => x > 3);               // 3
arr.some(x => x > 4);                     // true
arr.every(x => x > 0);                    // true
arr.includes(3);                          // true
arr.slice(1, 3);                          // [2, 3]
arr.flat();
arr.flatMap(x => [x, x * 2]);
arr.forEach(x => console.log(x));
```

```rust
// Rust
let v = vec![1, 2, 3, 4, 5];

v.iter().map(|&x| x * 2).collect::<Vec<_>>();
v.iter().filter(|&&x| x % 2 == 0).collect::<Vec<_>>();
v.iter().fold(0i32, |acc, &x| acc + x);
v.iter().find(|&&x| x > 3);
v.iter().position(|&x| x > 3);
v.iter().any(|&x| x > 4);
v.iter().all(|&x| x > 0);
v.contains(&3);
v[1..3].to_vec();   // 또는 v.iter().skip(1).take(2).collect()
v.iter().flatten();
v.iter().flat_map(|&x| vec![x, x * 2]).collect::<Vec<_>>();
v.iter().for_each(|x| println!("{}", x));
```

**핵심 차이:**
1. Rust는 `iter()`, `iter_mut()`, `into_iter()` 구분
2. Rust는 지연 평가 — `collect()` 등이 없으면 실행 안 됨
3. Rust는 타입을 명시해야 하는 경우 많음 (`collect::<Vec<_>>()`)
4. 참조(`&x`, `&&x`)를 역참조하는 패턴이 자주 필요

---

## 실용 패턴: 블록체인 이터레이터 체이닝

```rust
#[derive(Debug)]
struct Transaction {
    id: String,
    from: String,
    to: String,
    amount: u64,
    fee: u64,
    confirmed: bool,
}

fn analyze_transactions(transactions: &[Transaction]) {
    // 1. 확정된 트랜잭션의 총 금액
    let confirmed_total: u64 = transactions.iter()
        .filter(|tx| tx.confirmed)
        .map(|tx| tx.amount)
        .sum();

    // 2. 수수료 순으로 상위 3개 선택
    let mut sorted_by_fee: Vec<&Transaction> = transactions.iter().collect();
    sorted_by_fee.sort_by_key(|tx| std::cmp::Reverse(tx.fee));
    let top_3: Vec<&Transaction> = sorted_by_fee.into_iter().take(3).collect();

    // 3. 주소별 거래량 집계
    use std::collections::HashMap;
    let volume_by_address: HashMap<&str, u64> = transactions.iter()
        .flat_map(|tx| {
            // 송신자와 수신자 모두 집계
            vec![
                (tx.from.as_str(), tx.amount),
                (tx.to.as_str(), tx.amount),
            ]
        })
        .fold(HashMap::new(), |mut map, (addr, amount)| {
            *map.entry(addr).or_insert(0) += amount;
            map
        });

    // 4. 대용량 미확정 트랜잭션 ID 목록
    let pending_large: Vec<&str> = transactions.iter()
        .filter(|tx| !tx.confirmed && tx.amount > 10_000)
        .map(|tx| tx.id.as_str())
        .collect();

    println!("Confirmed total: {}", confirmed_total);
    println!("Pending large count: {}", pending_large.len());
    println!("Unique addresses: {}", volume_by_address.len());
}
```

---

## 요약

- `Iterator` 트레이트: `next()` 하나만 구현하면 수백 개의 메서드 무료
- `iter()` (불변 참조), `iter_mut()` (가변 참조), `into_iter()` (소유권) 구분
- 어댑터: `map`, `filter`, `filter_map`, `flat_map`, `take`, `skip`, `zip`, `chain`, `enumerate`
- 소비 메서드: `collect`, `sum`, `product`, `fold`, `reduce`, `any`, `all`, `find`, `count`, `for_each`
- 지연 평가: 소비 메서드 호출 전까지 실행되지 않음
- `collect::<Vec<_>>()` — 타입 힌트가 필요할 때 turbofish 문법 사용

다음으로는 스마트 컨트랙트 심화 주제를 다룬 후, 비동기 프로그래밍을 배웁니다.
