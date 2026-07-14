# 1. Cargo와 Rust의 기본 문법

Rust 프로젝트를 시작할 때 먼저 익힐 도구는 Cargo다. 의존성 관리, 빌드, 실행, 테스트, 문서 생성을 한 명령 체계로 다룬다.

## 설치와 프로젝트 생성

Rust는 `rustup`으로 설치하는 것이 표준이다.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version
cargo --version
```

프로젝트를 만들고 실행해 보자.

```bash
cargo new hello-chain
cd hello-chain
cargo run
```

기본 구조는 단순하다.

```text
hello-chain/
├── Cargo.toml
└── src/
    └── main.rs
```

`Cargo.toml`은 패키지 메타데이터, Rust edition, 기능 플래그, 의존성을 선언하는 파일이다. 자주 쓰는 명령은 다음 네 개면 충분하다.

| 명령 | 용도 |
|---|---|
| `cargo check` | 바이너리를 만들지 않고 빠르게 타입 검사 |
| `cargo run` | 빌드 후 실행 |
| `cargo test` | 테스트 빌드와 실행 |
| `cargo fmt` | 표준 포매터 적용 |

## 변수와 타입

Rust 변수는 기본적으로 불변(immutable)이다. 값이 바뀌어야 한다면 `mut`를 적는다.

```rust
fn main() {
    let chain_id: u64 = 1;
    let mut confirmations = 0;
    confirmations += 1;

    assert_eq!(chain_id, 1);
    assert_eq!(confirmations, 1);
}
```

불변 바인딩은 같은 이름에 새 값을 대입하거나 그 값을 가변 참조로 빌려줄 수 없다는 뜻이다.

```rust,compile_fail
fn main() {
    let status = String::from("pending");
    status.push_str(" -> confirmed");
}
```

위 예제는 일부러 실패한다. 수정하려면 `let mut status`라고 선언해야 한다.

블록체인 코드에서 자주 보는 기본 타입은 다음과 같다.

- `u64`, `u128`: 음수가 없는 높이, 잔액, 수량
- `bool`: 검증 결과나 플래그
- `String`: 소유한 가변 문자열
- `&str`: 빌려 온 문자열 조각
- `[u8; 32]`: 크기가 고정된 32바이트 해시나 공개키
- `Vec<u8>`: 길이가 달라지는 직렬화 데이터

금액에 `f64`를 쓰지 않는 이유도 중요하다. 부동소수점에는 반올림 오차가 있다. 토큰 수량은 가장 작은 단위를 정수로 저장한다. ETH라면 wei, ERC-20이라면 컨트랙트가 정한 소수 자릿수 기준의 정수다.

## 함수와 표현식

Rust 함수는 인자와 반환 타입을 명시한다. 마지막 표현식에 세미콜론이 없으면 그 값이 반환된다.

```rust
fn fee(gas_used: u128, price_per_gas: u128) -> u128 {
    gas_used * price_per_gas
}

fn main() {
    assert_eq!(fee(21_000, 2), 42_000);
}
```

`gas_used * price_per_gas` 뒤에는 세미콜론이 없다. 세미콜론을 붙이면 값이 아니라 unit 타입 `()`이 되어 선언한 `u128` 반환 타입과 맞지 않는다. 중간에 일찍 끝낼 때는 `return 값;`을 쓴다.

정수 곱셈은 범위를 넘을 수 있다. 디버그 빌드에서는 panic이 나고 릴리스 설정에 따라 동작이 달라질 수 있으므로 자산 계산은 `checked_add`, `checked_mul` 같은 검사를 써야 한다.

```rust
fn checked_fee(gas_used: u128, price_per_gas: u128) -> Option<u128> {
    gas_used.checked_mul(price_per_gas)
}

fn main() {
    assert_eq!(checked_fee(21_000, 2), Some(42_000));
    assert_eq!(checked_fee(u128::MAX, 2), None);
}
```

## 모듈과 크레이트

크레이트(crate)는 Rust의 컴파일 단위다. 실행 파일이면 binary crate, 다른 코드가 가져다 쓰는 라이브러리면 library crate다. 한 Cargo package 안에 여러 binary와 하나의 library crate를 둘 수도 있다.

파일이 커지면 `mod`로 나눈다.

```text
src/
├── main.rs
└── hash.rs
```

```rust,ignore
// src/hash.rs
pub fn short(value: &str) -> String {
    value.chars().take(8).collect()
}
```

```rust,ignore
// src/main.rs
mod hash;

fn main() {
    println!("{}", hash::short("abcdef0123456789"));
}
```

`pub`가 없으면 모듈 밖에서 접근할 수 없다. Rust는 컴파일 시점에 모듈 경로와 가시성을 검사하며 필요한 이름만 경계 밖으로 공개한다.

## 여기서 확인할 것

다음 장으로 넘어가기 전에 `cargo new`로 프로젝트를 하나 만들고 `u128::MAX.checked_add(1)`의 결과를 출력해 보자. `None`이 나오는 이유를 설명할 수 있으면 충분하다.
