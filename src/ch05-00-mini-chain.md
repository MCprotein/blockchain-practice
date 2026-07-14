# 5. 해시로 연결한 미니 체인 실습

지금까지 배운 Rust를 작은 프로그램으로 묶어 보자. 이 실습은 블록마다 이전 블록의 해시를 저장하고 데이터가 바뀌면 검증이 실패하는 구조를 만든다.

완성 코드는 `practice/mini-chain`에 있다.

```bash
cargo run -p mini-chain
cargo test -p mini-chain
```

## 이 프로그램이 보여 주는 것

블록에는 다섯 필드가 있다.

```rust,ignore
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub index: u64,
    pub data: String,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}
```

`hash`는 블록 내용을 요약한 32바이트 SHA-256 결과를 16진수 문자열로 표시한 값이다. `previous_hash`가 앞 블록을 가리키므로 중간 데이터를 바꾸면 뒤쪽 연결도 맞지 않게 된다.

```text
Block 0                  Block 1                  Block 2
hash: 00ab... ─────────> previous_hash: 00ab...
                          hash: 001c... ─────────> previous_hash: 001c...
                                                    hash: 007f...
```

## 해시 입력을 모호하지 않게 만들기

다음처럼 문자열만 이어 붙이면 경계가 모호해진다.

```text
"1" + "23"  ==  "12" + "3"
```

실습 코드는 정수를 고정 길이 바이트로 바꾸고 가변 길이 데이터 앞에는 길이를 넣는다.

```rust,ignore
fn calculate_hash(index: u64, data: &str, previous_hash: &str, nonce: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(index.to_be_bytes());
    hasher.update((data.len() as u64).to_be_bytes());
    hasher.update(data.as_bytes());
    hasher.update(previous_hash.as_bytes());
    hasher.update(nonce.to_be_bytes());
    hex::encode(hasher.finalize())
}
```

실제 프로토콜은 합의된 직렬화 형식을 정확히 정의한다. JSON 객체를 임의로 문자열화해 해시하면 필드 순서나 숫자 표현 차이로 노드마다 결과가 달라질 수 있다.

## 채굴 루프의 의미

`Block::mine`은 해시 앞에 정해진 수의 `0`이 나올 때까지 `nonce`를 바꾼다.

```rust,ignore
let target = "0".repeat(difficulty);
let mut nonce = 0;

loop {
    let hash = calculate_hash(index, &data, &previous_hash, nonce);
    if hash.starts_with(&target) {
        return Block { index, data, previous_hash, nonce, hash };
    }
    nonce = nonce.checked_add(1).expect("nonce 범위를 초과했습니다");
}
```

`nonce`가 바뀔 때마다 새로운 후보 해시가 나온다. SHA-256 결과를 미리 예측할 수 없으므로 조건을 만족할 때까지 반복한다. 16진수 앞자리 `0`을 하나 더 요구하면 학습용 모델의 평균 시도 횟수는 약 16배 늘어난다.

이것은 작업 증명(Proof of Work)의 모양만 보여 주는 축소판이다. 난이도 조정, 네트워크 전파, 경쟁 체인 선택, 보상, 타임스탬프 규칙은 구현하지 않았다. Ethereum도 현재는 지분 증명(Proof of Stake)을 쓰므로 이 코드를 Ethereum 합의의 구현으로 보면 안 된다.

## 검증은 무엇을 확인하나

체인 검증은 제네시스 블록의 고정 값과 각 블록의 네 조건을 본다.

1. 인덱스가 하나씩 증가한다.
2. `previous_hash`가 직전 블록의 `hash`와 같다.
3. 저장된 `hash`를 다시 계산해도 같은 값이다.
4. 해시가 학습용 난이도 조건을 만족한다.

```rust,ignore
self.blocks.windows(2).all(|pair| {
    let previous = &pair[0];
    let current = &pair[1];
    current.index == previous.index + 1
        && current.previous_hash == previous.hash
        && current.has_valid_hash(self.difficulty)
})
```

테스트는 정상 경로뿐 아니라 `index`, 데이터, `previous_hash`, `nonce`, 저장된 `hash` 조작이 모두 거부되는지 확인한다.

```rust,ignore
#[test]
fn 과거_데이터를_바꾸면_검증에_실패한다() {
    let mut chain = Blockchain::new(2);
    chain.add_block("alice -> bob: 10");
    chain.blocks[1].data = "alice -> bob: 1000".into();

    assert!(!chain.is_valid());
}
```

## 이 미니 체인이 보장하지 못하는 것

해시 연결만으로 블록체인이 되지는 않는다. 이 프로그램에는 다음이 없다.

- 트랜잭션 서명과 발신자 인증
- 이중 지불 방지와 잔액 상태
- 여러 노드 사이의 합의
- 디스크 영속화와 네트워크 통신
- 포크 선택과 최종성
- Sybil 공격을 막는 경제적 비용

여기서는 “데이터 변경 감지”와 “올바른 기록의 결정”을 구분해야 한다. 전자는 해시 연결로 일부 해결하지만 후자는 합의 프로토콜이 필요하다.

## 실습 과제

`cargo test -p mini-chain`을 실행한 뒤 각 조작 테스트가 `is_valid()`의 어느 조건에서 실패하는지 한 줄씩 적어 보자.
