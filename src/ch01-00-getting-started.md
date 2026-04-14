# 1장: Rust 시작하기

## Rust란 무엇인가?

Rust는 Mozilla Research에서 시작해 현재 Rust Foundation이 관리하는 시스템 프로그래밍 언어입니다. 2015년에 1.0이 출시되었고, 2016년부터 매년 Stack Overflow 설문에서 "가장 사랑받는 언어" 1위를 차지하고 있습니다 (2024년까지 9년 연속).

처음 읽을 때는 Rust를 “Node.js보다 빠른 언어” 정도로만 이해하면 부족합니다. Rust의 핵심 목표는 **빠른 실행**, **메모리 안전성**, **동시성 안전성**을 동시에 얻는 것입니다. 이 세 가지는 블록체인처럼 돈과 합의가 걸린 시스템에서 특히 중요합니다.

**핵심 특징 세 가지:**

1. **메모리 안전성** — 가비지 컬렉터 없이도 메모리 오류(null dereference, use-after-free, buffer overflow)를 컴파일 타임에 방지
2. **제로 비용 추상화** — 고수준 추상화를 써도 런타임 오버헤드가 없음 (C/C++ 수준 성능)
3. **두려움 없는 동시성** — 데이터 레이스를 컴파일 타임에 방지

### 이 장에서 먼저 알아둘 블록체인 단어

이 장은 Rust 소개 장이지만 블록체인 예시가 함께 나옵니다. 다음 단어만 먼저 잡고 넘어가세요.

| 단어 | 지금은 이렇게 이해하세요 |
|------|--------------------------|
| 트랜잭션(Transaction) | 체인 상태를 바꾸기 위해 네트워크에 제출하는 서명된 요청 |
| 노드(Node) | 블록체인 네트워크에 참여해 트랜잭션과 블록을 검증하는 프로그램 |
| 검증자(Validator) | 새 블록을 제안하거나 검증하는 역할을 맡은 노드 |
| 합의(Consensus) | 여러 노드가 같은 블록을 정식 기록으로 인정하는 규칙 |
| 온체인 프로그램(On-chain Program) | Solana에서 블록체인 위에 배포되어 실행되는 프로그램 |

자세한 설명은 [먼저 보는 용어와 코드 읽기 지도](./ch00-00-vocabulary-and-code-map.md)와 9장에서 다시 다룹니다.

## 왜 블록체인에서 Rust를 쓰는가?

### 성능이 곧 돈이다

블록체인 노드는 초당 수천~수만 건의 트랜잭션을 처리해야 합니다. 여기서 트랜잭션은 HTTP 요청처럼 “무언가를 처리해 달라”는 요청이지만, 개인 키 서명과 네트워크 검증을 거쳐 블록에 포함되어야 한다는 점이 다릅니다.

가비지 컬렉터가 있는 언어(Go, Java, Node.js)는 GC 일시 중지(stop-the-world pause)가 발생할 수 있는데, 블록체인 컨텍스트에서 이는 예측 불가능한 레이턴시를 만듭니다.

Rust는 GC가 없으므로 예측 가능한 성능을 보장합니다.

### 보안이 치명적이다

스마트 컨트랙트의 버그는 돌이킬 수 없는 금전적 손실로 이어집니다. 2016년 The DAO 해킹($60M), 2022년 Wormhole 해킹($320M) 등 역사적 사례들은 대부분 메모리 오류나 로직 버그에서 비롯되었습니다.

Rust의 타입 시스템과 빌림 검사기는 이런 종류의 버그를 원천 차단합니다.

### 주요 블록체인 프로젝트들의 선택

| 프로젝트 | Rust를 선택한 이유 |
|---------|------------------|
| **Solana** | 초당 수만 건의 트랜잭션을 병렬 처리(Sealevel)하는 검증자 코드에 GC 일시정지가 허용되지 않음. 합의 임계 경로(PoH 해시 체인)의 예측 가능한 레이턴시가 필수. 스마트 컨트랙트(Program)를 WASM이 아닌 네이티브 BPF 바이트코드로 컴파일해 최대 성능 확보 |
| **Near Protocol** | 스마트 컨트랙트를 Rust → WASM으로 컴파일하여 샌드박스 실행. 금융 코드에서 정수 오버플로우·버퍼 오버플로우를 컴파일 타임에 방지. 결정론적 WASM 실행으로 모든 노드가 동일한 결과를 보장 |
| **Polkadot / Substrate** | 런타임(팔렛)을 Rust → WASM으로 컴파일하여 네트워크를 중단하지 않고 온체인 업그레이드(forkless upgrade) 가능. GC 없는 결정론적 실행으로 블록 생산 중 레이턴시 스파이크 방지. 강력한 타입 시스템으로 크로스체인 메시지(XCM) 포맷 오류를 런타임 전에 차단 |
| **Reth (Ethereum 실행 클라이언트)** | Go 기반 Geth와 성능으로 경쟁하면서 합의 임계 코드의 메모리 안전성 확보. tokio 비동기 런타임으로 수천 개의 P2P 피어 연결을 효율적으로 처리. 병렬 블록 처리(parallel block execution)를 데이터 레이스 없이 구현 |
| **Lighthouse (Ethereum 합의 클라이언트)** | PoS 검증자 서명과 슬래싱 방지 로직에서 메모리 오류는 스테이킹 자산 손실로 이어짐. Rust의 빌림 검사기가 이중 서명(double-vote) 버그 유발 가능한 공유 가변 상태를 원천 차단 |
| **Bitcoin Dev Kit (BDK)** | 개인키·UTXO 처리 코드에서 use-after-free·버퍼 오버플로우는 자산 도난으로 직결. 외부 라이브러리로 임베딩될 때 GC 없이 예측 가능한 메모리 사용량 유지 |
| **Aptos / Sui** | Move VM 자체를 Rust로 구현하여 VM 인터프리터 버그가 체인 자산에 영향을 미치지 않도록 안전성 확보. Move의 자원(Resource) 타입 시스템과 Rust의 소유권 모델이 개념적으로 일치하여 VM 구현이 자연스러움 |

특히 **Solana**는 모든 온체인 프로그램을 Rust로 작성합니다. Anchor 프레임워크를 사용하면 보일러플레이트가 줄어들지만, 기본은 여전히 Rust입니다.

## Node.js 개발자 관점에서 보는 Rust

### 유사한 개념들

| Node.js/TypeScript | Rust | 비고 |
|-------------------|------|------|
| `npm` / `package.json` | `cargo` / `Cargo.toml` | 패키지 관리자 + 빌드 툴 |
| `node_modules/` | `~/.cargo/registry` | 의존성 저장소 (글로벌 캐시) |
| `npm install` | `cargo add` 또는 Cargo.toml 수정 | 의존성 추가 |
| `npm run build` | `cargo build` | 빌드 |
| `npm test` | `cargo test` | 테스트 실행 |
| `npx ts-node src/main.ts` | `cargo run` | 실행 |
| `interface` | `trait` | 타입 계약 (차이점 있음) |
| `class` | `struct` + `impl` | 데이터와 메서드 묶음 |
| `enum` (문자열 유니온) | `enum` | 대수적 데이터 타입 (훨씬 강력) |
| `Promise<T>` | `Future<Output = T>` | 비동기 계산 |
| `async/await` | `async fn` + `.await` | 비동기 문법 |
| `try/catch` | `Result<T, E>` | 에러 처리 (패턴이 다름) |
| `null` / `undefined` | `Option<T>` | 없는 값 표현 |

이 표는 “완전히 같다”는 뜻이 아닙니다. 첫 독해용 대응표입니다. 예를 들어 `trait`는 TypeScript `interface`처럼 타입 계약을 표현하지만, 구현 방식과 제네릭 제약에서 차이가 큽니다. 각 개념은 해당 장에서 다시 풀어갑니다.

### 근본적으로 다른 개념들

**1. 가비지 컬렉터 없음**

Node.js는 V8 엔진이 알아서 메모리를 회수합니다. 개발자는 메모리를 신경 쓸 필요가 없었습니다.

Rust는 **소유권(ownership)** 시스템으로 컴파일 타임에 메모리 해제 시점을 결정합니다. 런타임에 아무런 GC 비용이 없습니다.

**2. 예외(exception)가 없음**

Node.js는 `throw`와 `try/catch`를 씁니다. Rust는 `Result<T, E>`를 반환하며 에러를 값으로 처리합니다.

**3. null이 없음**

TypeScript에서 `null`과 `undefined`는 악명 높은 버그의 원천입니다. Rust는 `Option<T>` 타입으로 값이 없는 상황을 명시적으로 표현하며, 처리하지 않으면 컴파일 에러가 납니다.

**4. 암묵적 복사가 없음**

TypeScript에서 객체를 함수에 전달하면 참조가 공유됩니다. Rust에서 값을 전달하면 기본적으로 소유권이 이동(move)합니다. 이게 처음에 가장 어색한 부분입니다.

## Rust 컴파일러와 친해지기

Rust의 컴파일러 `rustc`는 매우 친절합니다. 에러 메시지가 구체적이고, 종종 수정 방법까지 제안합니다.

```text
error[E0382]: borrow of moved value: `s`
 --> src/main.rs:5:20
  |
3 |     let s = String::from("hello");
  |         - move occurs because `s` has type `String`
4 |     takes_ownership(s);
  |                     - value moved here
5 |     println!("{}", s);  // 에러!
  |                    ^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
4 |     takes_ownership(s.clone());
  |                      ++++++++
```

컴파일러가 에러를 설명하고, 해결책까지 제시합니다. 이 에러 메시지를 읽는 능력이 Rust 학습의 핵심입니다.

## 요약

Rust를 배우는 이유:
1. 블록체인의 주요 플랫폼(Solana, Near, Polkadot)이 Rust를 사용
2. 메모리 안전성으로 스마트 컨트랙트 버그 방지
3. GC 없는 예측 가능한 고성능
4. 강력한 타입 시스템으로 런타임 에러를 컴파일 타임에 잡기

다음 챕터에서 실제로 Rust를 설치하고 첫 프로젝트를 만들어봅시다.
