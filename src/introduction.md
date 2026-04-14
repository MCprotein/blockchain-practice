# 소개: Rust로 배우는 블록체인 개발

## 이 가이드북에 대하여

이 책은 Node.js/TypeScript 백엔드 개발자가 Rust를 배우고, 궁극적으로 블록체인 스마트 컨트랙트와 온체인 프로그램을 직접 작성할 수 있도록 안내하는 실전 교재입니다.

당신이 이미 알고 있는 것들—비동기 프로그래밍, 타입 시스템, 모듈화, 의존성 관리—을 출발점으로 삼아, Rust와 블록체인이 그것들을 어떻게 다르게 접근하는지 설명합니다. **프로그래밍 경험은 있다고 가정하지만, Rust와 블록체인은 처음이라고 가정합니다.** 따라서 트랜잭션, 블록, 체인, 소유권, 빌림 같은 기본 단어도 처음 등장할 때 뜻을 짚고 넘어갑니다.

코드가 먼저 나오고 설명이 뒤따르는 방식으로 읽으면 초반에 쉽게 막힙니다. 이 책에서는 긴 예제를 보기 전에 “이 코드가 어떤 데이터를 다루는가”, “처음 보는 문법은 지금 완벽히 이해해야 하는가, 뒤에서 다시 배울 문법인가”를 먼저 표시합니다.

## 대상 독자

- Node.js 백엔드 개발 경력 3년 이상
- TypeScript, NestJS, 디자인 패턴에 익숙한 개발자
- Rust를 처음 배우거나, 한번 공부해봤지만 소유권과 빌림에서 막힌 개발자
- 블록체인 용어는 들어봤지만 트랜잭션, 블록, 합의, 가스가 정확히 무엇인지 아직 불명확한 개발자
- Ethereum Solidity, Solana Anchor, 또는 기타 블록체인 환경의 스마트 컨트랙트를 작성하고 싶은 개발자

## 먼저 읽을 것

본격적인 Rust 설치와 블록체인 설명으로 들어가기 전에 [먼저 보는 용어와 코드 읽기 지도](./ch00-00-vocabulary-and-code-map.md)를 읽으세요. 이 장은 다음 두 가지를 미리 정리합니다.

- 트랜잭션, 블록, 체인, 노드, 지갑, 가스 같은 블록체인 기본 용어
- `fn`, `let mut`, `struct`, `impl`, `Vec<T>`, `Result<T, E>`, `?` 같은 Rust 코드 읽기 기호

본문을 읽다가 모르는 단어가 나오면 그 장으로 돌아가 확인하면 됩니다.

## 이 책의 구성

이 책은 Rust를 먼저 끝내고 나서 블록체인으로 넘어가는 방식이 아닙니다. 매주 Rust 문법과 블록체인 개념을 번갈아 배치합니다. 이유는 단순합니다. Rust만 연속으로 공부하면 소유권과 타입 문법에서 쉽게 지치고, 블록체인 이론만 연속으로 공부하면 실제 코드 감각이 늦게 붙습니다.

따라서 각 주차는 다음 리듬을 따릅니다.

```text
Rust 문법을 하나 배운다
        ↓
그 문법이 필요한 블록체인 개념을 본다
        ↓
작은 코드 예제로 연결한다
        ↓
주차 말에 미니프로젝트로 묶는다
```

```text
1주차: Rust 기초 + 블록체인 첫걸음
  - Rust 설치, Cargo, 기본 코드 읽기
  - 블록체인 기본 용어, 해시, 블록/체인 구조
  - 소유권, 참조, 구조체, 열거형
  - 미니 블록체인 구현

2주차: Rust 심화 + Ethereum/Solidity
  - Result, ?, 트레이트, 제네릭
  - 이더리움 계정/트랜잭션, EVM, 가스
  - Solidity와 Foundry
  - Token Vault 구현

3주차: 비동기 Rust + Solana + 컨트랙트 심화
  - 컬렉션, 이터레이터, async/Tokio
  - 컨트랙트 보안과 업그레이드
  - Solana 계정 모델과 Anchor
  - Solana 포인트 시스템 구현

4주차: 실무 통합 + Platform 프로젝트
  - Rust에서 Ethereum 연동(Alloy)
  - 프라이빗 체인과 Besu
  - Mini Trace 서비스
  - Platform 프로젝트 코드 리딩
```

## 현실적인 기대치

### 1달 안에 가능한 것

- Rust 문법과 소유권 시스템 이해
- 표준 라이브러리 주요 타입 활용 (Vec, HashMap, String, Result, Option)
- 간단한 CLI 툴 작성
- 기본적인 비동기 프로그램 작성 (Tokio 사용)
- SHA-256 해싱, 간단한 P2P 로직 등 블록체인 기초 개념 구현
- Rust로 작성된 오픈소스 코드를 읽고 이해

### 1달 안에 어려운 것

- Solana 온체인 프로그램(Anchor) 실전 배포
- 복잡한 수명(lifetime) 어노테이션이 있는 코드 작성
- 고급 매크로(proc-macro) 작성
- 멀티스레드 성능 최적화
- 실제 프로덕션 블록체인 노드 개발

**현실적인 조언**: Rust는 배움의 곡선이 가파릅니다. 특히 소유권과 빌림 검사기(borrow checker)는 처음에 매우 답답하게 느껴집니다. 이건 당신이 나쁜 개발자여서가 아닙니다—Rust를 만든 사람들도 배울 때 똑같이 느꼈습니다. 빌림 검사기와 싸우지 말고, 그것이 왜 그렇게 동작하는지 이해하려고 노력하세요.

## 4주 학습 일정표

### 1주차: Rust 기초 + 블록체인 첫걸음

| 날짜 | 내용 | 목표 |
|------|------|------|
| 1일 | 용어 지도, 환경설정, Hello Cargo | Rust 코드와 블록체인 용어를 읽을 준비 |
| 2일 | 블록체인이란 무엇인가, 해시 함수 | 트랜잭션/블록/체인/해시의 관계 이해 |
| 3일 | 소유권 규칙, Move vs Copy | Rust가 값을 옮기고 해제하는 방식 이해 |
| 4일 | 참조, 빌림, 슬라이스 | 데이터를 복사하지 않고 읽고 수정하는 패턴 익히기 |
| 5일 | 구조체, 열거형, 패턴 매칭 | `Transaction`, `Block`, `Blockchain`을 표현할 타입 준비 |
| 6일 | 블록과 체인 구조, 합의 알고리즘 | Rust 타입으로 표현할 블록체인 구조 이해 |
| 7일 | 미니 블록체인 프로젝트 | 해시, 블록 연결, 검증 흐름을 코드로 묶기 |

**1주차 프로젝트**: Rust로 미니 블록체인 구현

### 2주차: Rust 심화 + 이더리움/Solidity

| 날짜 | 내용 | 목표 |
|------|------|------|
| 8일 | `panic!`, `Result<T, E>`, `?` | 실패를 타입으로 다루는 Rust 방식 이해 |
| 9일 | 이더리움 계정과 트랜잭션 | EOA/CA, nonce, 수수료 모델 이해 |
| 10일 | EVM, 가스, 스마트 컨트랙트 개요 | 온체인 실행 비용과 컨트랙트 실행 흐름 이해 |
| 11일 | 제네릭과 트레이트 | 공통 동작을 타입 안전하게 추상화 |
| 12일 | Solidity 타입, 함수, modifier | Rust/TypeScript와 비교하며 Solidity 문법 익히기 |
| 13일 | Foundry, ERC-20, ERC-721 | 테스트 가능한 스마트 컨트랙트 개발 흐름 익히기 |
| 14일 | Token Vault 프로젝트 | 에러 처리, 권한, 토큰 전송을 프로젝트로 묶기 |

**2주차 프로젝트**: Token Vault

### 3주차: 비동기 Rust + Solana + 컨트랙트 심화

| 날짜 | 내용 | 목표 |
|------|------|------|
| 15일 | `Vec<T>`, `String`, `HashMap`, 이터레이터 | 트랜잭션/계정 목록을 다루는 자료구조 감각 익히기 |
| 16일 | 컨트랙트 상속, 프록시, 보안 | Solidity 실무 위험 요소 이해 |
| 17일 | 클로저, async/await, Future | 비동기 흐름을 Rust 타입으로 이해 |
| 18일 | Tokio, channel, 공유 상태 | 노드/인덱서/백엔드식 동시 처리 패턴 익히기 |
| 19일 | Solana 아키텍처와 계정 모델 | 이더리움과 다른 상태 모델 이해 |
| 20일 | Solana 프로그램, PDA, CPI, Anchor | Anchor 코드 구조와 계정 검증 이해 |
| 21일 | Solana 포인트 시스템 프로젝트 | Anchor 프로그램과 TypeScript 테스트 연결 |

**3주차 프로젝트**: Solana 토큰/포인트 프로그램

### 4주차: 실무 통합 + Platform 프로젝트

| 날짜 | 내용 | 목표 |
|------|------|------|
| 22일 | Alloy provider와 컨트랙트 호출 | Rust 백엔드에서 이더리움 읽기 |
| 23일 | 트랜잭션 서명과 전송 | Rust에서 온체인 쓰기 수행 |
| 24일 | `sol!` 매크로와 ABI | Solidity ABI를 Rust 타입으로 연결 |
| 25일 | 프라이빗 체인과 Besu | 엔터프라이즈 체인 선택 기준 이해 |
| 26일 | Mini Trace 프로젝트 | DB와 체인 해시 기록을 연결 |
| 27일 | Platform 서비스 아키텍처 | Rust/Axum/NestJS 대응 구조 읽기 |
| 28일 | Platform 블록체인 연동 흐름 | 실무 코드 리딩으로 전체 흐름 정리 |

**4주차 프로젝트**: Mini Trace + Platform 분석

---

## 환경 설정

### 1. Rust 설치 (rustup)

rustup은 Rust의 버전 관리자입니다. Node.js의 nvm과 같은 역할입니다.

**macOS / Linux:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

설치 중 옵션을 물어보면 `1) Proceed with installation (default)` 선택.

설치 후 쉘을 재시작하거나:

```bash
source "$HOME/.cargo/env"
```

**Windows:**

https://rustup.rs 에서 `rustup-init.exe` 다운로드 후 실행.
Visual Studio C++ Build Tools 설치가 필요할 수 있습니다.

**설치 확인:**

```bash
rustc --version    # rustc 1.75.0 (82e1608df 2023-12-21) 같은 출력
cargo --version    # cargo 1.75.0 (1d8b05cdd 2023-11-20) 같은 출력
rustup --version   # rustup 1.26.0 (5af9b9484 2023-04-05) 같은 출력
```

**안정 채널로 업데이트:**

```bash
rustup update stable
```

### 2. Rust 컴포넌트 추가

```bash
# 코드 포매터 (prettier 같은 것)
rustup component add rustfmt

# 린터 (eslint 같은 것)
rustup component add clippy
```

### 3. VS Code 설정

**필수 확장:**

1. **rust-analyzer** (rustlang.rust-analyzer)
   - 자동완성, 타입 힌트, 에러 표시, Go to definition
   - 반드시 설치해야 합니다

2. **Even Better TOML** (tamasfe.even-better-toml)
   - Cargo.toml 파일 하이라이팅

3. **crates** (serayuzgur.crates)
   - Cargo.toml에서 크레이트 버전을 인라인으로 보여줌

**VS Code settings.json 추천 설정:**

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### 4. Foundry 설치 (Ethereum 스마트 컨트랙트 도구)

Foundry는 Ethereum 스마트 컨트랙트 개발 툴킷입니다 (Hardhat/Truffle의 Rust 버전).

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

설치 확인:

```bash
forge --version    # forge 0.2.0 같은 출력
cast --version
anvil --version
```

### 5. Solana CLI 설치 (Solana 프로그램 개발)

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

PATH 추가 (`.zshrc` 또는 `.bashrc`에):

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

설치 확인:

```bash
solana --version    # solana-cli 1.18.x 같은 출력
```

**로컬 테스트 네트워크 설정:**

```bash
solana config set --url localhost
solana-test-validator  # 로컬 validator 실행 (별도 터미널)
```

### 6. Anchor CLI 설치 (Solana 스마트 컨트랙트 프레임워크)

Anchor는 Solana의 NestJS 같은 프레임워크입니다.

```bash
# avm (Anchor Version Manager) 설치
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
anchor --version
```

---

## 첫 번째 Rust 프로그램 확인

모든 환경이 갖춰졌는지 확인하는 간단한 테스트:

```bash
cargo new hello-blockchain
cd hello-blockchain
cargo run
```

출력:
```text
   Compiling hello-blockchain v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
     Running `target/debug/hello-blockchain`
Hello, world!
```

이 출력이 나오면 준비 완료입니다. 다음 챕터로 넘어가세요.
