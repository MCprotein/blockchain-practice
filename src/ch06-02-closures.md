# 6.2 클로저 (Closures)

## 클로저란?

클로저는 환경에서 변수를 캡처할 수 있는 익명 함수입니다. JavaScript의 화살표 함수와 매우 유사합니다.

```typescript
// JavaScript/TypeScript 화살표 함수
const add = (x: number, y: number): number => x + y;
const double = (x: number) => x * 2;
const greet = () => "Hello!";

// 환경 캡처
const multiplier = 3;
const multiplyBy = (x: number) => x * multiplier;  // 외부 변수 캡처
```

```rust,ignore
// Rust 클로저
let add = |x, y| x + y;
let double = |x| x * 2;
let greet = || "Hello!";

// 환경 캡처
let multiplier = 3;
let multiply_by = |x| x * multiplier;  // 외부 변수 캡처
println!("{}", multiply_by(5));  // 15
```

---

## 클로저 문법

```rust,ignore
fn main() {
    // 기본 형태: |매개변수| 표현식
    let add = |x, y| x + y;

    // 타입 명시 (보통 생략)
    let add2 = |x: i32, y: i32| -> i32 { x + y };

    // 여러 줄 블록
    let complex = |x: i32| {
        let doubled = x * 2;
        let added = doubled + 10;
        added  // 반환값 (return 키워드 없이)
    };

    // 매개변수 없음
    let greet = || String::from("Hello!");

    println!("{}", add(1, 2));      // 3
    println!("{}", complex(5));     // 20
    println!("{}", greet());        // Hello!
}
```

### 타입 추론

Rust는 사용 방법에서 클로저의 타입을 추론합니다:

```rust,ignore
fn main() {
    let add = |x, y| x + y;  // 타입 아직 미결정

    let result = add(1i32, 2i32);  // 이 줄에서 T = i32로 확정
    // add(1.0f64, 2.0f64);  // 에러! 이미 i32로 확정됨

    // 각 클로저 인스턴스는 고유한 타입 — 두 개의 클로저는 같은 타입이 아님
    let c1 = |x: i32| x + 1;
    let c2 = |x: i32| x + 1;  // c1과 동일해 보이지만 다른 타입
}
```

---

## 환경 캡처 방법

클로저는 외부 변수를 세 가지 방법으로 캡처합니다:

### 1. 불변 참조로 캡처 (기본)

```rust,ignore
fn main() {
    let x = 5;
    let print_x = || println!("x = {}", x);  // &x 캡처
    print_x();  // x = 5
    print_x();  // 여러 번 호출 가능
    println!("{}", x);  // x는 여전히 유효
}
```

### 2. 가변 참조로 캡처

```rust,ignore
fn main() {
    let mut count = 0;
    let mut increment = || {
        count += 1;  // &mut count 캡처
        println!("Count: {}", count);
    };
    increment();  // Count: 1
    increment();  // Count: 2
    // println!("{}", count);  // 에러! increment가 가변 참조를 보유 중
}
// count는 여기서 다시 접근 가능
```

### 3. 소유권 이동 (move)

```rust,ignore
fn main() {
    let data = vec![1, 2, 3];

    // move: data의 소유권을 클로저로 이동
    let owns_data = move || {
        println!("{:?}", data);
    };

    owns_data();
    // println!("{:?}", data);  // 에러! data의 소유권이 클로저로 이동됨

    // 스레드에 클로저를 보낼 때 특히 중요
    let text = String::from("hello");
    let thread = std::thread::spawn(move || {
        println!("In thread: {}", text);
        // text의 소유권이 스레드로 이동
    });
    thread.join().unwrap();
}
```

---

## Fn, FnMut, FnOnce 트레이트

클로저는 캡처 방식에 따라 세 가지 트레이트 중 하나(또는 여러 개)를 구현합니다:

### FnOnce: 소유권을 이동하는 클로저

```rust,ignore
fn call_once<F: FnOnce()>(f: F) {
    f();  // 한 번만 호출 가능
    // f();  // 에러! f는 소비됨
}

fn main() {
    let text = String::from("hello");
    let consume = move || {
        println!("{}", text);
        drop(text);  // text를 소비
    };

    call_once(consume);  // OK
    // call_once(consume);  // 에러! consume은 이미 소비됨
}
```

### FnMut: 가변 참조를 캡처하는 클로저

```rust,ignore
fn call_multiple_times<F: FnMut()>(mut f: F) {
    f();
    f();
    f();
}

fn main() {
    let mut count = 0;
    let mut counter = || {
        count += 1;
        println!("Count: {}", count);
    };

    call_multiple_times(&mut counter);
    // 또는
    // call_multiple_times(counter);
}
```

### Fn: 불변 참조만 캡처하는 클로저

```rust,ignore
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

fn main() {
    let offset = 5;
    let add_offset = |x| x + offset;  // &offset 캡처 (불변)

    println!("{}", apply_twice(add_offset, 10));  // 20
    println!("{}", apply_twice(add_offset, 10));  // 20 — 여러 번 사용 가능
}
```

### 트레이트 관계

```text
FnOnce  ←  FnMut  ←  Fn
(슈퍼트레이트)  (서브트레이트)
```

- 모든 `Fn`은 `FnMut`이기도 함
- 모든 `FnMut`은 `FnOnce`이기도 함
- 함수 인자에 `FnOnce`를 요구하면 가장 유연함 (모든 클로저 받을 수 있음)
- `Fn`을 요구하면 가장 제한적 (불변 캡처만 가능)

```rust,ignore
// 가장 유연: FnOnce (어떤 클로저든 받음)
fn run_once<F: FnOnce() -> String>(f: F) -> String { f() }

// 중간: FnMut (반복 호출, 내부 상태 변경 가능)
fn run_many<F: FnMut() -> String>(mut f: F) { let _ = f(); let _ = f(); }

// 가장 제한적: Fn (반복 호출, 내부 상태 변경 불가)
fn run_shared<F: Fn() -> String>(f: F) { println!("{}", f()); println!("{}", f()); }
```

---

## 일반 함수도 클로저 트레이트를 구현함

```rust,ignore
fn double(x: i32) -> i32 { x * 2 }

fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

fn main() {
    // 일반 함수도 Fn 트레이트를 구현
    println!("{}", apply(double, 5));       // 10
    println!("{}", apply(|x| x + 1, 5));   // 6
}
```

---

## 클로저를 반환하는 함수

```rust,ignore
// 클로저 반환 — impl Fn 사용
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y  // x를 캡처해서 반환
}

fn make_multiplier(factor: f64) -> impl Fn(f64) -> f64 {
    move |x| x * factor
}

fn main() {
    let add5 = make_adder(5);
    let add10 = make_adder(10);

    println!("{}", add5(3));   // 8
    println!("{}", add10(3));  // 13

    let double = make_multiplier(2.0);
    let triple = make_multiplier(3.0);

    println!("{}", double(5.0));  // 10.0
    println!("{}", triple(5.0));  // 15.0
}
```

여러 다른 클로저 타입을 반환해야 할 때는 `Box<dyn Fn>`:

```rust,ignore
fn make_validator(min: u64, max: u64) -> Box<dyn Fn(u64) -> bool> {
    Box::new(move |value| value >= min && value <= max)
}

fn main() {
    let valid_amount = make_validator(1, 1_000_000);
    println!("{}", valid_amount(500));        // true
    println!("{}", valid_amount(2_000_000));  // false
}
```

---

## 블록체인에서 클로저 활용

```rust,ignore
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    fee: u64,
}

struct Mempool {
    pending: Vec<Transaction>,
}

impl Mempool {
    fn new() -> Self {
        Mempool { pending: Vec::new() }
    }

    fn add(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    // 클로저로 필터링 기준 주입 (전략 패턴)
    fn select_transactions<F>(&self, predicate: F) -> Vec<&Transaction>
    where
        F: Fn(&Transaction) -> bool,
    {
        self.pending.iter().filter(|tx| predicate(tx)).collect()
    }

    // 클로저로 정렬 기준 주입
    fn get_sorted<F>(&self, key_fn: F) -> Vec<&Transaction>
    where
        F: Fn(&Transaction) -> u64,
    {
        let mut txs: Vec<&Transaction> = self.pending.iter().collect();
        txs.sort_by_key(|tx| key_fn(tx));
        txs
    }
}

fn main() {
    let mut mempool = Mempool::new();
    mempool.add(Transaction { from: "A".into(), to: "B".into(), amount: 100, fee: 10 });
    mempool.add(Transaction { from: "C".into(), to: "D".into(), amount: 5000, fee: 50 });
    mempool.add(Transaction { from: "E".into(), to: "F".into(), amount: 50, fee: 5 });

    // 최소 금액 필터 (클로저로 기준 주입)
    let min_amount = 100u64;
    let large_txs = mempool.select_transactions(|tx| tx.amount >= min_amount);
    println!("Large transactions: {}", large_txs.len());

    // 수수료 기준 정렬 (클로저로 기준 주입)
    let by_fee = mempool.get_sorted(|tx| tx.fee);
    for tx in by_fee {
        println!("Fee: {}, Amount: {}", tx.fee, tx.amount);
    }

    // 인라인 클로저로 복잡한 필터
    let profitable = mempool.select_transactions(|tx| {
        let fee_rate = tx.fee * 100 / tx.amount.max(1);
        fee_rate >= 10 && tx.amount >= 100
    });
    println!("Profitable: {}", profitable.len());
}
```

---

## JavaScript 화살표 함수와 비교 정리

| JavaScript/TypeScript | Rust |
|----------------------|------|
| `const f = (x) => x + 1` | `let f = \|x\| x + 1;` |
| `const f = (x, y) => x + y` | `let f = \|x, y\| x + y;` |
| `const f = () => {}` | `let f = \|\| {};` |
| 외부 변수 자동 캡처 | 불변 참조 우선, `move`로 소유권 이동 |
| 항상 여러 번 호출 가능 | Fn (여러 번), FnMut (가변), FnOnce (한 번) |
| 반환 시 클로저 타입 명시 불필요 | `impl Fn(i32) -> i32` 또는 `Box<dyn Fn>` |

---

## 요약

- 클로저: `|매개변수| 표현식` 문법의 익명 함수
- 환경 캡처: 불변 참조(기본), 가변 참조(`mut` 필요), 소유권 이동(`move`)
- `Fn`: 불변 참조, 여러 번 호출 가능
- `FnMut`: 가변 참조, 여러 번 호출 가능
- `FnOnce`: 소유권 이동, 한 번만 호출 가능
- 클로저 반환: `impl Fn(...)`, 다형적이면 `Box<dyn Fn(...)>`
- 일반 함수도 Fn 트레이트를 구현함

다음 챕터에서 이터레이터를 배웁니다.
