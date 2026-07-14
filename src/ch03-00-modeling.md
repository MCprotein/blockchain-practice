# 3. 블록체인 데이터를 타입으로 모델링하기

블록체인 프로그램은 결국 상태와 허용된 상태 전이를 다룬다. Rust에서는 구조체(struct), 열거형(enum), 패턴 매칭(match), 트레이트(trait)를 조합해 잘못된 상태를 표현하기 어렵게 만든다.

## 구조체와 구현 블록

구조체는 이름이 있는 필드들을 묶어 하나의 구체 데이터 타입을 만든다. 메서드와 생성 함수는 `impl` 블록에 둔다.

```rust
#[derive(Debug, PartialEq)]
struct Transfer {
    from: String,
    to: String,
    amount: u128,
}

impl Transfer {
    fn new(from: &str, to: &str, amount: u128) -> Result<Self, String> {
        if amount == 0 {
            return Err("수량은 0보다 커야 합니다".into());
        }
        Ok(Self {
            from: from.to_owned(),
            to: to.to_owned(),
            amount,
        })
    }
}

fn main() {
    let tx = Transfer::new("alice", "bob", 10).unwrap();
    assert_eq!(tx.amount, 10);
}
```

생성 함수를 `Result`로 만들면 유효하지 않은 `Transfer`가 만들어지는 경로를 줄일 수 있다. 모든 필드가 `pub`인 구조체를 여기저기서 직접 조립하는 것보다 불변식(invariant)을 지키기 쉽다.

## 열거형은 상태를 제한한다

열거형은 가능한 상태의 집합을 정의한다. 각 variant는 이름만 가질 수도 있고 그 상태에 필요한 데이터를 함께 가질 수도 있다.

```rust
#[derive(Debug, PartialEq)]
enum TxState {
    Pending,
    Confirmed { block_number: u64 },
    Failed(String),
}

fn label(state: &TxState) -> String {
    match state {
        TxState::Pending => "대기".into(),
        TxState::Confirmed { block_number } => format!("블록 {block_number}에서 확정"),
        TxState::Failed(reason) => format!("실패: {reason}"),
    }
}

fn main() {
    let state = TxState::Confirmed { block_number: 42 };
    assert_eq!(label(&state), "블록 42에서 확정");
}
```

새 상태를 enum에 추가하면 빠뜨린 `match` 분기를 컴파일러가 찾아준다. 문자열 상태값과 `if`를 늘리는 방식보다 변경 지점을 추적하기 쉽다.

## 컬렉션과 반복자

체인은 블록의 순서가 중요하므로 `Vec<Block>`이 잘 맞는다. 주소별 잔액처럼 키 조회가 중심이면 `HashMap<Address, Balance>`를 쓴다.

```rust
use std::collections::HashMap;

fn apply_transfer(
    balances: &mut HashMap<String, u128>,
    from: &str,
    to: &str,
    amount: u128,
) -> Result<(), String> {
    if amount == 0 {
        return Err("수량은 0보다 커야 합니다".into());
    }
    if from == to {
        return Err("송신자와 수신자가 같습니다".into());
    }

    let from_balance = balances.get(from).copied().unwrap_or(0);
    if from_balance < amount {
        return Err("잔액이 부족합니다".into());
    }

    let to_balance = balances.get(to).copied().unwrap_or(0);
    let next_to = to_balance.checked_add(amount).ok_or("잔액 범위를 초과했습니다")?;
    balances.insert(from.into(), from_balance - amount);
    balances.insert(to.into(), next_to);
    Ok(())
}

fn main() {
    let mut balances = HashMap::from([("alice".into(), 10)]);
    apply_transfer(&mut balances, "alice", "bob", 4).unwrap();
    assert_eq!(balances["alice"], 6);
    assert_eq!(balances["bob"], 4);
    assert!(apply_transfer(&mut balances, "alice", "alice", 1).is_err());
}
```

이 예제는 학습용 인메모리 상태 전이다. 실제 분산 원장은 트랜잭션 순서, 서명, 수수료, 합의, 영속화까지 처리해야 한다.

반복자(iterator)는 배열을 새로 만들지 않고 변환 단계를 연결한다.

```rust
fn main() {
    let fees = [21_u64, 5, 30, 8];
    let expensive: Vec<u64> = fees
        .iter()
        .copied()
        .filter(|fee| *fee >= 20)
        .collect();
    assert_eq!(expensive, vec![21, 30]);
}
```

Rust 반복자는 지연 평가된다. `filter`까지는 변환 계획만 만들고 `collect()`처럼 반복자를 소비하는 연산이 나올 때 실제로 순회한다.

## 트레이트로 동작을 분리한다

트레이트는 여러 타입이 공유할 동작 계약이다. 메서드 시그니처를 선언하고 각 타입이 그 동작을 구현한다.

```rust
trait Validate {
    fn is_valid(&self) -> bool;
}

struct Block {
    hash: String,
}

impl Validate for Block {
    fn is_valid(&self) -> bool {
        self.hash.len() == 64 && self.hash.bytes().all(|b| b.is_ascii_hexdigit())
    }
}

fn accept(value: &impl Validate) -> bool {
    value.is_valid()
}

fn main() {
    let block = Block { hash: "a".repeat(64) };
    assert!(accept(&block));
}
```

트레이트를 무조건 만드는 것은 좋지 않다. 구현이 하나뿐이고 대체 가능성이 없다면 구체 타입으로 시작하는 편이 단순하다. RPC 공급자, 저장소, 서명기처럼 구현을 바꾸거나 테스트 대역이 필요한 경계에서 트레이트가 특히 유용하다.

## 동시성은 나중에 붙인다

Rust 언어는 `async` 실행기(runtime)를 고정하지 않는다. Tokio 같은 런타임은 RPC 클라이언트나 인덱서에서 중요하지만 온체인 프로그램은 체인의 결정적 실행 환경에서 동작한다. 비동기 Rust는 오프체인 네트워크 코드를 만들 때 따로 배워도 된다.

## 여기서 확인할 것

`TxState`에 `Replaced { by_hash: String }`를 추가해 보자. `label`을 고치지 않으면 어떤 컴파일 오류가 생기는지 확인하면 enum과 `match`의 장점이 바로 보인다.
