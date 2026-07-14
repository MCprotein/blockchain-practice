# 2. 소유권과 실패 처리

Rust가 블록체인 개발에 자주 쓰이는 이유를 “빠르고 안전해서”라고만 말하면 핵심을 놓친다. 더 구체적으로는 메모리의 소유 관계와 실패 경로가 타입에 드러난다. 트랜잭션 파서, 서명 검증기, 노드처럼 입력을 함부로 믿을 수 없는 프로그램에서 이 성질이 도움이 된다.

## 값에는 소유자가 하나 있다

힙에 저장되는 `String`을 다른 변수에 대입하면 소유권이 이동(move)한다.

```rust,compile_fail
fn main() {
    let sender = String::from("alice");
    let moved = sender;
    println!("{sender} -> {moved}");
}
```

`String`은 힙 버퍼의 주소, 길이, 용량을 가진다. 대입 뒤 두 변수가 같은 버퍼의 해제 책임을 함께 가지면 이중 해제가 생길 수 있다. Rust는 책임을 `moved`로 옮기고 원래 변수의 사용을 컴파일 단계에서 막는다.

Rust에서 정말 복제본이 필요하면 `clone()`을 명시한다. 비용이 있다는 사실이 코드에 보인다.

```rust
fn main() {
    let sender = String::from("alice");
    let copied = sender.clone();
    assert_eq!(sender, copied);
}
```

## 빌림과 참조

값을 넘기지 않고 잠시 읽으려면 참조(reference)를 빌려준다.

```rust
fn has_prefix(hash: &str, prefix: &str) -> bool {
    hash.starts_with(prefix)
}

fn main() {
    let hash = String::from("0000ab12");
    assert!(has_prefix(&hash, "0000"));
    assert_eq!(hash.len(), 8); // 소유권이 남아 있다
}
```

수정하려면 가변 참조 `&mut T`가 필요하다. 한 시점에 여러 가변 참조를 허용하지 않는 규칙은 데이터 경쟁을 컴파일 단계에서 막는다.

```rust
fn confirm(count: &mut u32) {
    *count += 1;
}

fn main() {
    let mut confirmations = 0;
    confirm(&mut confirmations);
    assert_eq!(confirmations, 1);
}
```

실전에서 타입을 고르는 간단한 기준은 이렇다.

| 의도 | 흔한 타입 |
|---|---|
| 읽기만 한다 | `&T`, `&str`, `&[u8]` |
| 값을 수정한다 | `&mut T` |
| 함수가 값을 보관한다 | `T`, `String`, `Vec<u8>` |
| 복사 비용이 작은 정수·불리언 | 값 자체 |

## 슬라이스

슬라이스(slice)는 컬렉션 전체를 소유하지 않고 연속된 일부를 바라보는 참조다. `&str`은 유효한 UTF-8 문자열 슬라이스이고 `&[u8]`은 바이트 슬라이스다.

```rust
fn first_four(bytes: &[u8]) -> Option<&[u8]> {
    bytes.get(..4)
}

fn main() {
    let signature = [1, 2, 3, 4, 5, 6];
    assert_eq!(first_four(&signature), Some(&[1, 2, 3, 4][..]));
    assert_eq!(first_four(&[1, 2]), None);
}
```

인덱스를 바로 쓰는 `&bytes[..4]`는 입력이 짧으면 panic을 낸다. 외부 입력을 처리할 때는 `get`처럼 실패를 값으로 돌려주는 API가 안전하다.

## `Option`과 `Result`

값이 없을 수 있으면 `Option<T>`, 작업이 실패할 수 있으면 `Result<T, E>`를 쓴다.

```rust
fn parse_amount(raw: &str) -> Result<u128, String> {
    let amount = raw
        .parse::<u128>()
        .map_err(|_| format!("잘못된 수량: {raw}"))?;

    if amount == 0 {
        return Err("수량은 0보다 커야 합니다".into());
    }
    Ok(amount)
}

fn main() {
    assert_eq!(parse_amount("25"), Ok(25));
    assert!(parse_amount("0").is_err());
    assert!(parse_amount("1.5").is_err());
}
```

`?`는 `Result`를 분기하는 축약 문법이다. `Err`면 현재 함수에서 즉시 반환하고 `Ok`면 안의 값을 꺼내 다음 줄을 실행한다. 실패 가능성은 함수의 `Result<성공값, 오류>` 반환 타입에 드러난다.

`unwrap()`은 실패 시 panic을 낸다. 테스트나 “반드시 존재한다”는 내부 불변식을 확인할 때는 쓸 수 있지만 RPC 응답·사용자 입력·온체인 데이터처럼 통제할 수 없는 값에는 피한다.

## 수명 표기는 언제 필요한가

대부분의 참조 수명(lifetime)은 컴파일러가 추론한다. 반환 참조가 어느 입력에서 왔는지 모호할 때만 관계를 적는다.

```rust
fn longer<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() { left } else { right }
}

fn main() {
    assert_eq!(longer("block", "tx"), "block");
}
```

`'a`는 객체가 얼마나 오래 사는지 숫자로 정하는 문법이 아니다. 두 입력 참조와 반환 참조가 같은 유효 범위 안에 있어야 한다는 관계를 표시한다. Anchor 코드의 `Account<'info, T>`에서도 instruction 동안 유효한 계정 참조 관계를 같은 표기로 읽는다.

## 여기서 확인할 것

`parse_amount`가 빈 문자열, 음수 문자열, `u128` 범위를 넘는 숫자를 받았을 때 어떤 경로로 실패하는지 확인해 보자. panic 없이 모두 `Err`가 되어야 한다.
