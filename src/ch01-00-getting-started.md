# 1장: Rust 시작하기

## Rust란 무엇인가?

Rust는 Mozilla Research에서 시작해 현재 Rust Foundation이 관리하는 시스템 프로그래밍 언어입니다. 2015년에 1.0이 출시되었고, 2016년부터 매년 Stack Overflow 설문에서 "가장 사랑받는 언어" 1위를 차지하고 있습니다 (2024년까지 9년 연속).

**핵심 특징 세 가지:**

1. **메모리 안전성** — 가비지 컬렉터 없이도 메모리 오류(null dereference, use-after-free, buffer overflow)를 컴파일 타임에 방지
2. **제로 비용 추상화** — 고수준 추상화를 써도 런타임 오버헤드가 없음 (C/C++ 수준 성능)
3. **두려움 없는 동시성** — 데이터 레이스를 컴파일 타임에 방지

## 왜 블록체인에서 Rust를 쓰는가?

### 성능이 곧 돈이다

블록체인 노드는 초당 수천~수만 건의 트랜잭션을 처리해야 합니다. 가비지 컬렉터가 있는 언어(Go, Java, Node.js)는 GC 일시 중지(stop-the-world pause)가 발생하는데, 블록체인 컨텍스트에서 이는 예측 불가능한 레이턴시를 만듭니다.

Rust는 GC가 없으므로 예측 가능한 성능을 보장합니다.

### 보안이 치명적이다

스마트 컨트랙트의 버그는 돌이킬 수 없는 금전적 손실로 이어집니다. 2016년 The DAO 해킹($60M), 2022년 Wormhole 해킹($320M) 등 역사적 사례들은 대부분 메모리 오류나 로직 버그에서 비롯되었습니다.

Rust의 타입 시스템과 빌림 검사기는 이런 종류의 버그를 원천 차단합니다.

### 주요 블록체인 프로젝트들의 선택

| 프로젝트 | Rust 사용 이유 |
|---------|--------------|
| **Solana** | 온체인 프로그램(스마트 컨트랙트) 전체가 Rust로 작성 |
| **Near Protocol** | 스마트 컨트랙트를 Rust로 작성, AssemblyScript도 지원 |
| **Polkadot / Substrate** | 런타임 전체가 Rust |
| **Ethereum (클라이언트)** | Reth(새 실행 클라이언트), Lighthouse(합의 클라이언트) |
| **Bitcoin (클라이언트)** | Bitcoin Dev Kit (BDK) |
| **Aptos / Sui** | Move VM을 Rust로 구현 |

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

```
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
