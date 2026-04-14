# 3.3 패턴 매칭

## match 표현식

`match`는 Rust에서 가장 강력한 제어 흐름 구조입니다. TypeScript의 `switch`와 유사하지만 훨씬 강력합니다.

```rust
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: &Coin) -> u32 {
    match coin {
        Coin::Penny   => 1,
        Coin::Nickel  => 5,
        Coin::Dime    => 10,
        Coin::Quarter => 25,
    }
}
```

TypeScript `switch`와 비교:

```typescript
function valueInCents(coin: Coin): number {
    switch (coin) {
        case Coin.Penny:   return 1;
        case Coin.Nickel:  return 5;
        case Coin.Dime:    return 10;
        case Coin.Quarter: return 25;
        // default 없어도 TypeScript가 exhaustive check (enum의 경우)
    }
}
```

**`match`의 핵심 규칙: 모든 경우를 처리해야 한다 (exhaustive)**

```rust
fn value_in_cents(coin: &Coin) -> u32 {
    match coin {
        Coin::Penny   => 1,
        Coin::Nickel  => 5,
        // Dime, Quarter 누락!
    }
    // error[E0004]: non-exhaustive patterns: `&Coin::Dime` and `&Coin::Quarter` not covered
}
```

---

## match 팔(arm)의 구조

각 `match` 팔은 `패턴 => 표현식` 형태입니다:

```rust
fn describe_number(n: i32) -> &'static str {
    match n {
        // 단일 값
        0 => "zero",

        // 여러 값 (OR 패턴)
        1 | 2 | 3 => "small positive",

        // 범위
        4..=10 => "medium positive",

        // 조건 (match guard)
        n if n > 0 => "large positive",

        // 나머지 (와일드카드)
        _ => "negative",
    }
}

fn main() {
    println!("{}", describe_number(0));    // zero
    println!("{}", describe_number(2));    // small positive
    println!("{}", describe_number(7));    // medium positive
    println!("{}", describe_number(100));  // large positive
    println!("{}", describe_number(-5));   // negative
}
```

### 여러 줄 팔

```rust
fn process_block(block: &Block) -> String {
    match block.status {
        BlockStatus::Pending => {
            println!("Block is being processed...");
            let hash = compute_hash(block);
            format!("Pending block hash: {}", hash)
        }
        BlockStatus::Confirmed => {
            format!("Confirmed at height {}", block.height)
        }
        BlockStatus::Invalid(ref reason) => {
            eprintln!("Invalid block: {}", reason);
            String::from("invalid")
        }
    }
}
```

---

## 데이터를 가진 열거형 패턴 매칭

```rust
#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

fn process(msg: Message) {
    match msg {
        Message::Quit => {
            println!("Quit!");
        }
        Message::Move { x, y } => {
            // 구조 분해로 필드를 직접 꺼냄
            println!("Move to ({}, {})", x, y);
        }
        Message::Write(text) => {
            // 튜플 배리언트의 값을 꺼냄
            println!("Write: {}", text);
        }
        Message::ChangeColor(r, g, b) => {
            println!("Color: rgb({}, {}, {})", r, g, b);
        }
    }
}

fn main() {
    process(Message::Move { x: 10, y: 20 });
    process(Message::Write(String::from("hello")));
    process(Message::ChangeColor(255, 128, 0));
    process(Message::Quit);
}
```

---

## 구조 분해 (Destructuring)

패턴 매칭의 핵심 기능 중 하나는 구조 분해입니다.

### 구조체 구조 분해

```rust
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let p = Point { x: 3.0, y: 4.0 };

    // 구조 분해로 필드를 변수로 꺼냄
    let Point { x, y } = p;
    println!("x: {}, y: {}", x, y);

    // 다른 이름으로 꺼냄
    let Point { x: px, y: py } = Point { x: 1.0, y: 2.0 };
    println!("px: {}, py: {}", px, py);

    // 일부 필드만 (나머지는 무시)
    let Point { x, .. } = Point { x: 5.0, y: 6.0 };
    println!("x only: {}", x);

    // match에서 구조 분해
    let points = vec![
        Point { x: 0.0, y: 0.0 },
        Point { x: 1.0, y: 5.0 },
    ];

    for point in &points {
        match point {
            Point { x: 0.0, y: 0.0 } => println!("Origin"),
            Point { x, y: 0.0 }      => println!("On x-axis at {}", x),
            Point { x: 0.0, y }      => println!("On y-axis at {}", y),
            Point { x, y }           => println!("At ({}, {})", x, y),
        }
    }
}
```

### 튜플 구조 분해

```rust
fn main() {
    let (a, b, c) = (1, 2, 3);
    println!("{} {} {}", a, b, c);

    // 일부 무시
    let (first, _, last) = (1, 2, 3);
    println!("{} {}", first, last);

    // 중첩 구조 분해
    let ((x1, y1), (x2, y2)) = ((1, 2), (3, 4));
    println!("({},{}) to ({},{})", x1, y1, x2, y2);
}
```

### 열거형 구조 분해

```rust
#[derive(Debug)]
enum TransactionResult {
    Success { txid: String, block_height: u64 },
    Failure { code: u32, reason: String },
    Pending(String),  // tx hash
}

fn handle_result(result: TransactionResult) {
    match result {
        TransactionResult::Success { txid, block_height } => {
            println!("TX {} confirmed at block {}", txid, block_height);
        }
        TransactionResult::Failure { code, reason } => {
            println!("TX failed ({}): {}", code, reason);
        }
        TransactionResult::Pending(hash) => {
            println!("TX {} is pending...", hash);
        }
    }
}
```

---

## 와일드카드와 변수 바인딩

```rust
fn main() {
    let num = 7u32;

    match num {
        // 값 무시
        _ => println!("anything"),
    }

    // 변수 바인딩과 와일드카드
    match num {
        n @ 1..=10 => println!("Got {} (1-10)", n),  // @ 바인딩
        n @ 11..=20 => println!("Got {} (11-20)", n),
        _ => println!("Out of range"),
    }

    // 참조 패턴
    let reference = &4;
    match reference {
        &val => println!("Got a value via destructuring: {}", val),
    }

    // 또는 ref 키워드로
    let value = 5;
    match value {
        ref r => println!("Got a reference to {}", r),
    }
}
```

### @ 바인딩 활용

```rust
fn categorize_block_height(height: u64) -> String {
    match height {
        // 값을 n에 바인딩하면서 범위 검사
        n @ 0 => format!("Genesis block"),
        n @ 1..=99 => format!("Early block #{}", n),
        n @ 100..=999 => format!("Block #{} (hundreds)", n),
        n => format!("Block #{} (large)", n),
    }
}
```

---

## match 가드 (Match Guards)

패턴에 추가 조건을 붙일 수 있습니다:

```rust
fn classify_transaction(amount: u64, is_confirmed: bool) -> &'static str {
    match (amount, is_confirmed) {
        (0, _) => "zero-value transaction",
        (_, false) => "unconfirmed",
        (amt, true) if amt > 1_000_000 => "large confirmed",
        (amt, true) if amt > 10_000 => "medium confirmed",
        (_, true) => "small confirmed",
    }
}

fn main() {
    println!("{}", classify_transaction(0, true));          // zero-value
    println!("{}", classify_transaction(5_000, false));     // unconfirmed
    println!("{}", classify_transaction(2_000_000, true));  // large confirmed
    println!("{}", classify_transaction(50_000, true));     // medium confirmed
    println!("{}", classify_transaction(100, true));        // small confirmed
}
```

---

## if let: 단일 패턴 매칭

`match`가 한 패턴만 처리할 때, `if let`이 더 간결합니다:

```rust
fn main() {
    let some_value: Option<u32> = Some(42);

    // match로 쓰면
    match some_value {
        Some(v) => println!("Got: {}", v),
        None => {}  // 아무것도 안 함
    }

    // if let으로 더 간결하게
    if let Some(v) = some_value {
        println!("Got: {}", v);
    }

    // else 추가 가능
    if let Some(v) = some_value {
        println!("Got: {}", v);
    } else {
        println!("Nothing");
    }

    // 열거형과 함께
    let event = WalletEvent::Deposit { amount: 1000, from: String::from("Alice") };

    if let WalletEvent::Deposit { amount, from } = event {
        println!("Deposit {} from {}", amount, from);
    }
}
```

### 블록체인 코드에서 if let 활용

```rust
fn get_block_data(blockchain: &Blockchain, index: u64) -> Option<String> {
    blockchain.blocks.get(index as usize).map(|b| b.data.clone())
}

fn main() {
    let blockchain = Blockchain::new();

    // if let으로 깔끔하게 처리
    if let Some(data) = get_block_data(&blockchain, 0) {
        println!("Genesis data: {}", data);
    } else {
        println!("Block not found");
    }

    // 체이닝
    if let Some(block) = blockchain.blocks.first() {
        if let Some(hash) = block.hash.get(..6) {
            println!("Short hash: {}...", hash);
        }
    }
}
```

---

## while let: 조건부 반복

```rust
fn main() {
    let mut stack = vec![1, 2, 3, 4, 5];

    // stack.pop()이 Some을 반환하는 동안 반복
    while let Some(top) = stack.pop() {
        println!("Popped: {}", top);
    }
    // 5, 4, 3, 2, 1 순서로 출력

    // 채널에서 메시지 받기 (tokio/std 채널 패턴)
    // while let Ok(msg) = receiver.recv() {
    //     handle_message(msg);
    // }
}
```

---

## let else (Rust 1.65+)

패턴이 매칭되지 않으면 early return하는 패턴:

```rust
fn process_transaction(tx_data: &str) -> Result<(), String> {
    // tx_data를 파싱
    let parts: Vec<&str> = tx_data.split(':').collect();

    // 패턴 매칭 실패시 else 블록 실행 (return/break/continue/panic 필요)
    let [from, to, amount_str] = parts.as_slice() else {
        return Err(String::from("Invalid transaction format"));
    };

    let Ok(amount) = amount_str.parse::<u64>() else {
        return Err(format!("Invalid amount: {}", amount_str));
    };

    println!("Transfer {} from {} to {}", amount, from, to);
    Ok(())
}

fn main() {
    match process_transaction("Alice:Bob:1000") {
        Ok(()) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }

    match process_transaction("invalid") {
        Ok(()) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }
}
```

---

## matches! 매크로

bool을 반환하는 패턴 매칭 단축형:

```rust
#[derive(PartialEq)]
enum Status { Active, Inactive, Suspended }

fn main() {
    let status = Status::Active;

    // match로
    let is_active = match status {
        Status::Active => true,
        _ => false,
    };

    // matches! 매크로로 (더 간결)
    let is_active2 = matches!(status, Status::Active);

    // 여러 패턴
    let is_problematic = matches!(status, Status::Inactive | Status::Suspended);

    // 조건 포함
    let num = 42i32;
    let in_range = matches!(num, 1..=100);

    println!("{} {} {} {}", is_active, is_active2, is_problematic, in_range);
}
```

---

## 전체 패턴 종류 요약

```rust
fn all_patterns(x: i32) {
    match x {
        // 1. 리터럴
        0 => println!("zero"),

        // 2. 변수 (모든 값을 n에 바인딩)
        n => println!("n = {}", n),
    }

    let pair = (1, -1);
    match pair {
        // 3. 튜플 패턴
        (0, y) => println!("First is zero, y={}", y),
        (x, 0) => println!("x={}, Second is zero", x),
        (x, y) => println!("({}, {})", x, y),
    }

    // 4. 열거형 패턴 (앞서 설명)
    // 5. 구조체 패턴 (앞서 설명)
    // 6. 범위 패턴 (앞서 설명)
    // 7. @ 바인딩 (앞서 설명)
    // 8. 와일드카드 _ (앞서 설명)
    // 9. OR 패턴 |
    // 10. 가드 if
    // 11. ref/ref mut (참조 바인딩)
}
```

---

## 요약

- `match`: 강력한 패턴 매칭 — exhaustive (모든 케이스 강제 처리)
- `if let`: 단일 패턴에 간결하게 사용
- `while let`: 패턴이 매칭되는 동안 반복
- `let else`: 매칭 실패 시 early return
- `matches!`: bool 반환하는 패턴 매칭 단축형
- 구조 분해: 튜플, 구조체, 열거형을 분해해서 내부 값 꺼내기
- `@` 바인딩: 패턴 매칭하면서 값을 변수에 바인딩
- match 가드: `if` 조건으로 패턴에 추가 조건 부여

다음으로는 합의 알고리즘을 배운 뒤, 1주차 미니프로젝트로 블록체인을 직접 구현합니다. 에러 처리는 2주차에서 본격적으로 다룹니다.
