# 1.1 설치와 개발 환경 구성

## rustup: Rust 버전 관리자

rustup은 Node.js의 `nvm`과 동일한 역할을 합니다. Rust 컴파일러(`rustc`)와 패키지 관리자(`cargo`)를 설치하고 버전을 관리합니다.

### macOS / Linux 설치

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

설치 스크립트가 실행되면 다음 메뉴가 나타납니다:

```text
Current installation options:

   default host triple: x86_64-apple-darwin
     default toolchain: stable (default)
               profile: default
  modify PATH variable: yes

1) Proceed with installation (default)
2) Customize installation
3) Cancel installation
```

그냥 `1`을 누르고 Enter. 설치가 완료되면:

```bash
# 현재 쉘에 환경변수 적용 (설치 직후 한 번만)
source "$HOME/.cargo/env"

# 또는 터미널을 재시작
```

### Windows 설치

1. https://rustup.rs 접속
2. `rustup-init.exe` 다운로드 및 실행
3. Visual Studio C++ Build Tools가 없으면 설치 요청이 뜸 → 설치

> **Note**: Windows에서는 WSL2 (Windows Subsystem for Linux) 환경에서 개발하는 것을 강력히 권장합니다. Solana CLI 등 일부 도구가 WSL2에서만 정상 작동합니다.

### 설치 확인

```bash
rustc --version
# rustc 1.75.0 (82e1608df 2023-12-21)

cargo --version
# cargo 1.75.0 (1d8b05cdd 2023-11-20)

rustup --version
# rustup 1.26.0 (5af9b9484 2023-04-05)
```

### 구성 요소 설명

rustup이 설치하는 것들:

- **rustc**: Rust 컴파일러 (tsc에 해당)
- **cargo**: 빌드 시스템 + 패키지 관리자 (npm + webpack 합친 것)
- **rustup**: 버전 관리자 (nvm에 해당)
- **std**: 표준 라이브러리

### 추가 컴포넌트 설치

```bash
# rustfmt: 코드 포매터 (prettier에 해당)
rustup component add rustfmt

# clippy: 린터 (eslint에 해당)
rustup component add clippy

# rust-src: rust-analyzer가 표준 라이브러리 소스를 볼 수 있게
rustup component add rust-src
```

### 릴리스 채널

Rust는 세 가지 채널이 있습니다:

```bash
# stable: 프로덕션용 (기본값)
rustup toolchain install stable

# beta: 다음 stable의 RC 버전
rustup toolchain install beta

# nightly: 최신 기능 (일부 실험적 기능 필요시)
rustup toolchain install nightly

# 특정 버전 설치
rustup toolchain install 1.70.0

# 현재 사용 채널 확인
rustup show
```

블록체인 개발(특히 Solana Anchor)에서는 특정 nightly 버전이 필요한 경우가 있습니다. Anchor 프로젝트는 `rust-toolchain.toml` 파일로 버전을 고정합니다.

---

## VS Code 설정

### 필수 확장 설치

**방법 1: VS Code 내에서**
- `Cmd+Shift+X` (맥) 또는 `Ctrl+Shift+X` (윈도우)
- `rust-analyzer` 검색 → 설치

**방법 2: 커맨드라인**
```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates
code --install-extension vadimcn.vscode-lldb  # 디버거
```

### rust-analyzer 설정

`~/.config/Code/User/settings.json` (또는 VS Code의 settings.json):

```json
{
  // 저장 시 clippy로 검사 (rustfmt만 하는 기본값보다 더 많은 경고)
  "rust-analyzer.check.command": "clippy",

  // 인라인 타입 힌트 표시 (매우 유용!)
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints.enable": true,

  // 저장 시 자동 포맷
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },

  // 파일 저장 시 자동 import 정리
  "rust-analyzer.imports.granularity.group": "module"
}
```

### rust-analyzer가 제공하는 기능

- **자동완성**: 메서드, 필드, 트레이트 메서드 자동완성
- **타입 힌트**: 변수 옆에 추론된 타입을 표시
- **Go to Definition**: `F12` 또는 `Cmd+클릭`
- **Find References**: `Shift+F12`
- **Rename Symbol**: `F2`
- **코드 액션**: 💡 아이콘 클릭으로 빠른 수정
- **에러 표시**: 컴파일 에러를 실시간으로 에디터에 표시

---

## Cargo 기본 명령어

### 새 프로젝트 생성

```bash
# 바이너리 프로젝트 (실행 가능한 프로그램)
cargo new my-project
# 또는
cargo new my-project --bin

# 라이브러리 프로젝트 (다른 프로젝트에서 가져다 쓰는 크레이트)
cargo new my-lib --lib

# 현재 디렉토리를 프로젝트로 초기화
cargo init
cargo init --lib
```

생성된 구조:

```text
my-project/
├── Cargo.toml          # 프로젝트 설정 (package.json)
├── Cargo.lock          # 잠금 파일 (package-lock.json)
└── src/
    └── main.rs         # 진입점 (index.ts / main.ts)
```

라이브러리의 경우 `src/lib.rs`가 생성됩니다.

### 주요 명령어 비교

| npm/Node.js | Cargo | 설명 |
|-------------|-------|------|
| `npm install` | `cargo build` | 의존성 다운로드 + 빌드 |
| `npm run build` | `cargo build --release` | 최적화 빌드 |
| `npm start` | `cargo run` | 빌드 후 실행 |
| `npm test` | `cargo test` | 테스트 실행 |
| `npm install <pkg>` | `cargo add <crate>` | 의존성 추가 |
| `npx eslint .` | `cargo clippy` | 린트 검사 |
| `npx prettier --write .` | `cargo fmt` | 코드 포맷 |
| `npm run build -- --watch` | `cargo watch -x run` | 변경 감지 + 재실행 |

### 자주 쓰는 cargo 명령어

```bash
# 빌드 (개발용, 빠른 컴파일, 디버그 심볼 포함)
cargo build
# 결과물: target/debug/my-project

# 빌드 (배포용, 최적화, 느린 컴파일)
cargo build --release
# 결과물: target/release/my-project

# 빌드 + 실행
cargo run

# 빌드 + 실행 (릴리스 모드)
cargo run --release

# 실행 인자 전달
cargo run -- --port 8080 --verbose

# 컴파일만 확인 (실행 파일 생성 안 함, 빠름)
cargo check

# 테스트 실행
cargo test

# 특정 테스트만 실행
cargo test test_block_hash

# 문서 생성 및 브라우저에서 열기
cargo doc --open

# 의존성 추가
cargo add serde
cargo add serde --features derive
cargo add tokio --features full

# 의존성 제거
cargo remove serde

# 사용하지 않는 의존성 확인
cargo +nightly udeps  # cargo-udeps 설치 필요

# 린트
cargo clippy
cargo clippy -- -D warnings  # 경고를 에러로 처리

# 포맷
cargo fmt
cargo fmt --check  # CI에서 포맷 검사용

# 패키지 정보 확인
cargo metadata
```

### cargo watch 설정 (파일 변경 감지)

```bash
# cargo-watch 설치
cargo install cargo-watch

# 파일 변경시 자동으로 cargo run
cargo watch -x run

# 파일 변경시 cargo check (더 빠름)
cargo watch -x check

# 파일 변경시 테스트 실행
cargo watch -x test
```

---

## 첫 번째 프로젝트 생성 및 실행

### 프로젝트 생성

```bash
cargo new hello-blockchain
cd hello-blockchain
```

### src/main.rs 살펴보기

```rust
fn main() {
    println!("Hello, world!");
}
```

처음 보는 Rust 코드이므로 한 줄씩 읽어봅시다.

| 코드 | 뜻 |
|------|----|
| `fn` | 함수를 정의한다는 키워드 |
| `main` | 실행 파일이 시작되는 함수 이름 |
| `{ ... }` | 함수 본문 |
| `println!` | 콘솔에 한 줄 출력하는 매크로 |
| `"Hello, world!"` | 문자열 리터럴 |
| `;` | 이 문장이 여기서 끝난다는 표시 |

TypeScript의 `console.log`에 해당하는 것이 `println!`입니다. 뒤에 `!`가 붙으면 매크로입니다. 매크로는 컴파일 시점에 코드를 만들어내는 Rust 기능인데, 지금은 그냥 "특별한 함수처럼 호출하는 출력 도구"라고 생각해도 됩니다.

### 실행

```bash
cargo run
```

출력:
```text
   Compiling hello-blockchain v0.1.0 (/Users/you/hello-blockchain)
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
     Running `target/debug/hello-blockchain`
Hello, world!
```

### 코드 수정해보기

`src/main.rs`를 수정합니다:

```rust,ignore
fn main() {
    let name = "Blockchain Developer";
    let year = 2024;

    println!("Hello, {}!", name);
    println!("Welcome to Rust in {}!", year);

    // 기본 연산
    let block_height: u64 = 100;
    let reward: f64 = 6.25;
    println!("Block #{}: reward = {} BTC", block_height, reward);
}
```

여기서 새로 나온 문법은 세 가지입니다.

| 문법 | 의미 | TypeScript 감각 |
|------|------|-----------------|
| `let name = ...` | 불변 변수 선언 | `const name = ...` |
| `let block_height: u64 = 100` | 타입을 직접 적은 변수 선언 | `const blockHeight: number = 100` |
| `{}` | 출력 문자열의 자리표시자 | template literal의 `${value}` |

Rust의 변수는 기본적으로 다시 대입할 수 없습니다. 값을 바꾸려면 뒤에서 볼 `let mut`를 사용해야 합니다. 숫자 타입 `u64`는 부호 없는 64비트 정수입니다. 블록 높이처럼 음수가 될 수 없는 큰 정수에 자주 씁니다.

```bash
cargo run
```

출력:
```text
Hello, Blockchain Developer!
Welcome to Rust in 2024!
Block #100: reward = 6.25 BTC
```

### println! 포맷 문자열

```rust
fn main() {
    let x = 42;
    let pi = 3.14159;

    // 기본 출력
    println!("{}", x);           // 42

    // 디버그 출력 (Debug 트레이트 구현 필요)
    println!("{:?}", x);         // 42
    println!("{:#?}", x);        // 예쁜 출력 (pretty print)

    // 소수점 자리수
    println!("{:.2}", pi);       // 3.14
    println!("{:.4}", pi);       // 3.1416

    // 패딩
    println!("{:10}", x);        // "        42" (우측 정렬, 너비 10)
    println!("{:<10}", x);       // "42        " (좌측 정렬)
    println!("{:0>5}", x);       // "00042" (0으로 채움)

    // 16진수, 2진수, 8진수
    println!("{:x}", x);         // 2a
    println!("{:X}", x);         // 2A
    println!("{:b}", x);         // 101010
    println!("{:o}", x);         // 52

    // 여러 변수
    let (a, b) = (10, 20);
    println!("{a} + {b} = {}", a + b);  // Rust 1.58+: 변수 직접 참조
}
```

### eprintln!: stderr 출력

```rust
fn main() {
    println!("이건 stdout으로");   // 정상 출력
    eprintln!("이건 stderr로");    // 에러/로그 출력
}
```

Node.js의 `console.log`와 `console.error`에 해당합니다.

---

## 디렉토리 구조 이해

실제 프로젝트에서 자주 보게 될 구조:

```text
my-project/
├── Cargo.toml              # 프로젝트 메타데이터, 의존성
├── Cargo.lock              # 버전 잠금 (커밋에 포함시킬 것)
├── src/
│   ├── main.rs             # 바이너리 진입점
│   ├── lib.rs              # 라이브러리 루트 (lib 크레이트)
│   ├── models/
│   │   ├── mod.rs          # 모듈 선언
│   │   ├── block.rs        # Block 구조체
│   │   └── transaction.rs  # Transaction 구조체
│   └── utils/
│       ├── mod.rs
│       └── crypto.rs
├── tests/
│   └── integration_test.rs # 통합 테스트
├── examples/
│   └── basic_usage.rs      # 예제 코드
└── target/                 # 빌드 결과물 (gitignore)
    ├── debug/
    └── release/
```

`.gitignore`에 추가할 것:

```gitignore
/target
```

---

## 요약

- `rustup`: Rust 버전 관리자 (nvm)
- `cargo`: 빌드 + 패키지 관리 (npm + webpack)
- `rustc`: 컴파일러 (직접 쓸 일 별로 없음, cargo가 대신)
- `rust-analyzer`: VS Code 언어 서버 (필수)
- `cargo run` → 빌드 + 실행
- `cargo build --release` → 최적화 빌드
- `cargo test` → 테스트

다음 챕터에서 Cargo.toml 구조와 의존성 관리를 자세히 배웁니다.
