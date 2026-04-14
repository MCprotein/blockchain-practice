# 4.1 panic!

## panic!이란?

`panic!`은 프로그램이 계속 실행될 수 없는 상황에서 즉시 종료하는 메커니즘입니다. 스택을 풀어내며(unwinding) 정리 코드를 실행하거나, 즉시 중단(abort)합니다.

```rust
fn main() {
    panic!("Something went terribly wrong!");
    // thread 'main' panicked at 'Something went terribly wrong!', src/main.rs:2:5
}
```

## panic!이 발생하는 상황들

### 1. 명시적 panic! 호출

```rust
fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        panic!("Division by zero!");
    }
    a / b
}
```

### 2. 배열/Vec 범위 초과

```rust
fn main() {
    let v = vec![1, 2, 3];
    println!("{}", v[10]);
    // thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 10'
}
```

### 3. unwrap()이 None에 호출될 때

```rust
fn main() {
    let value: Option<i32> = None;
    let x = value.unwrap();
    // thread 'main' panicked at 'called `Option::unwrap()` on a `None` value'
}
```

### 4. expect()

```rust
fn main() {
    let value: Option<i32> = None;
    let x = value.expect("value must exist here");
    // thread 'main' panicked at 'value must exist here'
}
```

### 5. 정수 오버플로 (debug 모드)

```rust
fn main() {
    let x: u8 = 255;
    let y = x + 1;  // debug 모드에서 panic! (overflow)
                    // release 모드에서는 wrapping (0이 됨)
}
```

### 6. assert! 매크로

```rust
fn main() {
    let x = 5;
    assert!(x > 10, "x must be greater than 10, got {}", x);
    // thread 'main' panicked at 'x must be greater than 10, got 5'

    assert_eq!(x, 5);   // x == 5이면 통과
    assert_ne!(x, 10);  // x != 10이면 통과
}
```

---

## panic!을 쓸 때와 쓰지 말 때

### 써야 하는 상황

**1. 불변식(invariant) 위반**

```rust
fn add_block(&mut self, block: Block) {
    // 이 조건이 깨지면 버그 — 프로그램 자체가 잘못된 것
    assert!(
        block.index == self.blocks.len() as u64,
        "Block index mismatch: expected {}, got {}",
        self.blocks.len(),
        block.index
    );
    self.blocks.push(block);
}
```

**2. 테스트 코드**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_block_hash() {
        let block = Block::genesis();
        assert!(!block.hash.is_empty(), "Genesis block must have a hash");
        assert_eq!(block.index, 0, "Genesis block index must be 0");
    }
}
```

**3. 예제/프로토타입 코드 (todo!, unimplemented!)**

```rust
fn mine_block(&mut self) -> Block {
    todo!("PoW mining not yet implemented")
    // thread 'main' panicked at 'not yet implemented: PoW mining not yet implemented'
}

fn verify_signature(&self) -> bool {
    unimplemented!("Signature verification")
}

fn unused_function() {
    unreachable!("This code should never be reached");
}
```

**4. 외부 입력이 아닌, 프로그래머의 실수로만 발생할 수 있는 상황**

```rust
// 컴파일러가 None이 불가능하다고 판단하지 못하지만,
// 로직상 절대 None이 될 수 없는 경우
let last = self.blocks.last().expect("Blockchain must have at least one block");
```

### 쓰지 말아야 하는 상황

**외부 입력, 네트워크, 파일 등 예상 가능한 에러**

```rust
// 나쁜 코드 — 외부 입력을 panic으로 처리
fn parse_block_height(s: &str) -> u64 {
    s.parse::<u64>().unwrap()  // 잘못된 입력이면 panic!
}

// 좋은 코드 — Result로 처리
fn parse_block_height(s: &str) -> Result<u64, std::num::ParseIntError> {
    s.parse::<u64>()
}
```

---

## 블록체인에서 panic!의 위험성

### 스마트 컨트랙트에서의 panic

Solana 온체인 프로그램에서 `panic!`이 발생하면:

1. 트랜잭션이 즉시 실패
2. 해당 트랜잭션의 상태 변경이 롤백됨
3. 수수료는 차감됨 (가스는 소모됨)
4. 온체인 로그에 에러 메시지가 남음

```rust
// Solana 프로그램에서 나쁜 패턴
pub fn process_transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
    let balance = ctx.accounts.source.amount;
    // 잔액 부족 시 panic! — 잘못된 접근
    assert!(balance >= amount, "Insufficient balance");
    // ...
}

// 좋은 패턴 — 에러를 반환
pub fn process_transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
    let balance = ctx.accounts.source.amount;
    if balance < amount {
        return Err(ErrorCode::InsufficientFunds.into());
    }
    // ...
    Ok(())
}
```

### 서버 프로그램에서의 panic

Tokio 비동기 런타임에서 태스크 내부의 `panic!`은:
- 해당 태스크만 종료 (프로세스 전체가 죽지 않음)
- `JoinHandle`에서 에러로 처리 가능
- 하지만 예기치 않은 상태 불일치를 만들 수 있음

```rust
// 프로덕션 서버에서 — panic을 잡아서 처리
use std::panic;

let result = panic::catch_unwind(|| {
    // panic이 날 수 있는 코드
    risky_operation()
});

match result {
    Ok(val) => println!("Success: {:?}", val),
    Err(_)  => eprintln!("Caught a panic!"),
}
```

### Cargo.toml에서 panic 동작 설정

```toml
[profile.release]
# 릴리스 빌드에서 panic 시 즉시 abort (스택 unwinding 없음)
# 바이너리 크기 감소, 더 빠름
# 스마트 컨트랙트에서 선호
panic = "abort"

[profile.dev]
# 개발 빌드에서는 unwind (기본값) — 에러 메시지 풍부
panic = "unwind"
```

---

## RUST_BACKTRACE 환경변수

panic 발생 시 스택 트레이스를 보려면:

```bash
RUST_BACKTRACE=1 cargo run
# 또는 전체 트레이스
RUST_BACKTRACE=full cargo run
```

출력 예시:
```
thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 10', src/main.rs:3:20
stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: core::slice::index_failed
   3: my_project::main
             at ./src/main.rs:3:20
```

---

## todo!, unimplemented!, unreachable! 비교

| 매크로 | 의미 | 사용 시점 |
|--------|------|----------|
| `todo!()` | 아직 구현하지 않은 코드 | 개발 중, 나중에 구현 예정 |
| `unimplemented!()` | 의도적으로 구현하지 않음 | 트레이트 메서드 중 일부만 구현 |
| `unreachable!()` | 도달할 수 없는 코드 | 로직상 불가능한 분기 |

```rust
enum Direction { North, South, East, West }

fn turn_left(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::West,
        Direction::West  => Direction::South,
        Direction::South => Direction::East,
        Direction::East  => Direction::North,
    }
}

fn handle_special_only(dir: Direction) {
    match dir {
        Direction::North => println!("Special north handling"),
        _ => unreachable!("Only North should reach here"),
    }
}
```

---

## 요약

- `panic!`: 복구 불가능한 에러 — 프로그램 즉시 종료
- 써야 할 때: 버그, 불변식 위반, 테스트, todo/unimplemented
- 쓰지 말아야 할 때: 외부 입력, 네트워크, 파일 등 예상 가능한 에러
- 블록체인 스마트 컨트랙트에서 `panic!`은 트랜잭션 실패 + 가스 소모
- 프로덕션 코드에서는 `Result<T, E>`를 사용

다음 챕터에서 `Result<T, E>`로 에러를 우아하게 처리하는 방법을 배웁니다.
