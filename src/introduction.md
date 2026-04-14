# 소개: Rust로 배우는 블록체인 개발

## 이 가이드북에 대하여

이 책은 Node.js/TypeScript 백엔드 개발자가 Rust를 배우고, 궁극적으로 블록체인 스마트 컨트랙트와 온체인 프로그램을 직접 작성할 수 있도록 안내하는 실전 교재입니다.

당신이 이미 알고 있는 것들—비동기 프로그래밍, 타입 시스템, 모듈화, 의존성 관리—을 출발점으로 삼아, Rust가 그것들을 어떻게 다르게 접근하는지 설명합니다. "완전 초보" 대상 책이 아닙니다. 프로그래밍을 이미 잘 하는 사람이 새로운 언어의 철학을 빠르게 흡수하도록 돕는 책입니다.

## 대상 독자

- Node.js 백엔드 개발 경력 3년 이상
- TypeScript, NestJS, 디자인 패턴에 익숙한 개발자
- Rust를 한번 공부해본 적 있으나 기억이 흐릿한 개발자
- Ethereum Solidity, Solana Anchor, 또는 기타 블록체인 환경의 스마트 컨트랙트를 작성하고 싶은 개발자

## 이 책의 구성

```
1부: Rust 기초 (1~2주)
  - 설치와 환경설정
  - 소유권 시스템 (Rust의 핵심)
  - 구조체, 열거형, 패턴 매칭

2부: Rust 중급 (2~3주)
  - 에러 처리
  - 제네릭과 트레이트
  - 컬렉션과 이터레이터
  - 클로저

3부: 비동기 Rust (3~4주)
  - async/await
  - Tokio 런타임
  - 공유 상태와 동시성

4부: 블록체인 프로젝트 (4주+)
  - Rust로 미니 블록체인 구현
  - (향후) Solana Anchor 프로그래밍
  - (향후) EVM 스마트 컨트랙트 (Foundry)
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

### 1주차: Rust 기초와 소유권

| 날짜 | 내용 | 목표 |
|------|------|------|
| 1일 | 환경설정, Hello World, Cargo | 툴체인 완전 설치, 첫 프로젝트 실행 |
| 2일 | 기본 타입, 변수, 함수, 제어 흐름 | 간단한 계산기 프로그램 작성 |
| 3일 | 소유권 규칙, Move vs Copy | 소유권 컴파일 에러 직접 경험하고 고치기 |
| 4일 | 참조(&T, &mut T), 빌림 규칙 | 함수에 참조 전달하는 패턴 익히기 |
| 5~7일 | 슬라이스, 구조체, 열거형 | 간단한 학생 관리 프로그램 작성 |

**1주차 프로젝트**: 커맨드라인 주소록 (이름, 이메일, 전화번호 저장/조회)

### 2주차: 에러 처리와 트레이트

| 날짜 | 내용 | 목표 |
|------|------|------|
| 8일 | `Result<T,E>`, `Option<T>`, panic! | 에러를 반환하는 함수 작성 |
| 9일 | ? 연산자, 커스텀 에러 타입 | thiserror 크레이트 사용 |
| 10일 | 제네릭 타입과 함수 | 재사용 가능한 컨테이너 작성 |
| 11일 | 트레이트 정의와 구현 | Display, Debug, Clone 직접 구현 |
| 12일 | 트레이트 바운드, derive | 트레이트 바운드로 제네릭 제한 |
| 13~14일 | 수명 어노테이션 | 수명이 필요한 상황 이해 |

**2주차 프로젝트**: 파일 파서 (CSV 읽기, 에러 처리 포함)

### 3주차: 컬렉션, 클로저, 비동기

| 날짜 | 내용 | 목표 |
|------|------|------|
| 15일 | `Vec<T>`, String, HashMap 심화 | JS Array/Map과 비교하며 익히기 |
| 16일 | 클로저, Fn/FnMut/FnOnce | 클로저를 함수에 전달하는 패턴 |
| 17일 | Iterator 트레이트, 어댑터 | map/filter/collect 체인 작성 |
| 18일 | async fn, Future, .await | 비동기 함수 작성 |
| 19일 | Tokio 기초 (spawn, channel) | Tokio로 동시 작업 실행 |
| 20일 | Arc, Mutex, 공유 상태 | 스레드 간 안전한 데이터 공유 |
| 21일 | reqwest로 HTTP 요청 | 외부 API 호출하는 비동기 클라이언트 |

**3주차 프로젝트**: 비동기 암호화폐 가격 조회 CLI

### 4주차: 블록체인 프로젝트

| 날짜 | 내용 | 목표 |
|------|------|------|
| 22~23일 | Block 구조체, SHA-256 해싱 | 블록 직렬화/해싱 구현 |
| 24~25일 | Blockchain 구조체, 체인 검증 | 제네시스 블록, 체인 무결성 검사 |
| 26~27일 | Proof of Work 마이닝 | 난이도 조절, nonce 탐색 |
| 28일 | 전체 코드 리뷰, 리팩토링 | 에러 처리 개선, 테스트 추가 |

**4주차 프로젝트**: 완전한 미니 블록체인 구현

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
```
   Compiling hello-blockchain v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
     Running `target/debug/hello-blockchain`
Hello, world!
```

이 출력이 나오면 준비 완료입니다. 다음 챕터로 넘어가세요.
