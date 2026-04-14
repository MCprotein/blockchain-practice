# 1.2 Cargo와 프로젝트 구조

## Cargo.toml 구조

`Cargo.toml`은 Node.js의 `package.json`에 해당합니다. TOML(Tom's Obvious, Minimal Language) 형식으로 작성됩니다.

### 기본 구조

```toml
[package]
name = "my-blockchain"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A simple blockchain implementation in Rust"
license = "MIT"
repository = "https://github.com/you/my-blockchain"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
hex = "0.4"
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
# 테스트에서만 사용하는 의존성
assert_eq = "1.0"

[build-dependencies]
# 빌드 스크립트(build.rs)에서 사용하는 의존성

[[bin]]
name = "blockchain"
path = "src/main.rs"

[[bin]]
name = "miner"
path = "src/bin/miner.rs"

[lib]
name = "blockchain_lib"
path = "src/lib.rs"

[profile.dev]
opt-level = 0      # 최적화 없음 (빠른 컴파일)
debug = true       # 디버그 심볼 포함

[profile.release]
opt-level = 3      # 최대 최적화
debug = false
lto = true         # Link Time Optimization
codegen-units = 1  # 단일 코드 생성 단위 (더 나은 최적화)
panic = "abort"    # panic시 abort (블록체인에서 중요)

[features]
default = []
experimental = ["some-experimental-crate"]
```

### package.json과 비교

```json
// package.json (Node.js)
{
  "name": "my-blockchain",
  "version": "0.1.0",
  "description": "A simple blockchain implementation",
  "main": "dist/index.js",
  "scripts": {
    "build": "tsc",
    "start": "node dist/index.js",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.0",
    "axios": "^1.0.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "@types/express": "^4.17.0"
  }
}
```

```toml
# Cargo.toml (Rust)
[package]
name = "my-blockchain"
version = "0.1.0"
edition = "2021"

# "scripts"에 해당하는 것: cargo run, cargo test, cargo build
# main/entry point: src/main.rs (기본값)

[dependencies]
# "dependencies"에 해당
axum = "0.7"    # express 대신
reqwest = { version = "0.11", features = ["json"] }  # axios 대신

[dev-dependencies]
# "devDependencies"에 해당
```

**주요 차이점:**

| package.json | Cargo.toml |
|-------------|-----------|
| `"^4.18.0"` (캐럿 범위) | `"4"` (메이저 버전 호환) |
| `scripts` 섹션 있음 | 스크립트 없음 (cargo 명령어 사용) |
| `devDependencies` | `[dev-dependencies]` |
| 없음 | `[profile.dev]`, `[profile.release]` |
| 없음 | `features` (조건부 컴파일) |

---

## 의존성 관리

### 크레이트(Crate)란?

Rust의 패키지 단위를 **크레이트(crate)**라고 합니다. npm의 패키지와 같습니다. https://crates.io 에서 검색할 수 있습니다.

### 의존성 추가 방법

**방법 1: cargo add 명령어 (권장)**

```bash
# 최신 버전 추가
cargo add serde

# features 포함
cargo add serde --features derive
cargo add tokio --features full

# 특정 버전 지정
cargo add serde@1.0.195

# dev-dependency로 추가
cargo add --dev pretty_assertions

# 여러 개 동시에
cargo add sha2 hex chrono
```

**방법 2: Cargo.toml 직접 수정**

```toml
[dependencies]
# 버전 문자열만
serde_json = "1.0"

# 상세 설정
serde = { version = "1.0", features = ["derive"] }

# git 저장소에서 직접
my-crate = { git = "https://github.com/user/my-crate" }
my-crate = { git = "https://github.com/user/my-crate", branch = "main" }
my-crate = { git = "https://github.com/user/my-crate", rev = "abc1234" }

# 로컬 경로
my-local-crate = { path = "../my-local-crate" }

# 특정 조건에서만 포함
[target.'cfg(unix)'.dependencies]
nix = "0.27"
```

### 버전 지정 방식

```toml
[dependencies]
# 정확한 버전
exact = "=1.0.0"

# 1.x.x (하위 호환, npm의 ^ 와 동일)
compatible = "1"
compatible2 = "1.0"
compatible3 = "^1.0.0"

# 패치 버전만 (1.2.x)
patch = "~1.2"
patch2 = "~1.2.0"

# 모든 버전 (권장하지 않음)
any = "*"

# 범위
range = ">=1.0, <2.0"
```

> **팁**: crates.io에서 크레이트 이름을 검색하면 "Install" 섹션에 Cargo.toml에 추가할 줄이 나옵니다.

### Cargo.lock

`Cargo.lock`은 npm의 `package-lock.json`과 같습니다. 모든 의존성의 정확한 버전을 기록합니다.

**중요한 규칙:**
- **바이너리 프로젝트** (main.rs가 있는 프로젝트): Cargo.lock을 **반드시 커밋**
- **라이브러리 프로젝트** (lib.rs만 있는 프로젝트): Cargo.lock을 **.gitignore에 추가** (사용자가 자신의 버전으로 결정하게)

```bash
# 의존성 업데이트
cargo update           # 모든 의존성 업데이트 (semver 범위 내)
cargo update serde     # 특정 크레이트만 업데이트
```

---

## 블록체인 프로젝트 Cargo.toml 예시

실제 이 책에서 만들 미니 블록체인의 Cargo.toml:

```toml
[package]
name = "mini-blockchain"
version = "0.1.0"
edition = "2021"

[dependencies]
# SHA-256 해싱
sha2 = "0.10"

# 바이트 배열 <-> 16진수 문자열 변환
hex = "0.4"

# 직렬화/역직렬화 (TypeScript의 JSON.stringify/parse와 유사)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 시간 처리
chrono = { version = "0.4", features = ["serde"] }

# 에러 처리 (커스텀 에러 타입 쉽게 만들기)
thiserror = "1.0"

[dev-dependencies]
# 테스트용 assertion 강화
pretty_assertions = "1.4"
```

---

## 빌드 시스템 이해

### 컴파일 과정

```text
src/main.rs
    ↓
  rustc (컴파일러)
    ↓
target/debug/mini-blockchain  (실행 파일)
```

TypeScript와 비교:

```text
src/main.ts
    ↓
  tsc (TypeScript 컴파일러)
    ↓
dist/main.js  (JavaScript)
    ↓
  node  (런타임)
    ↓
실행
```

Rust는 네이티브 바이너리로 컴파일됩니다. 별도의 런타임이 필요 없습니다.

### 개발 빌드 vs 릴리스 빌드

```bash
# 개발 빌드: 빠른 컴파일, 느린 실행, 디버그 정보 포함
cargo build
# → target/debug/mini-blockchain (수십 MB)

# 릴리스 빌드: 느린 컴파일, 빠른 실행, 최적화
cargo build --release
# → target/release/mini-blockchain (수 MB, 훨씬 빠름)
```

**성능 차이**: 릴리스 빌드는 개발 빌드보다 10배~100배 빠를 수 있습니다. 블록체인의 Proof of Work 같은 연산 집약적 작업은 반드시 `--release`로 테스트해야 합니다.

### cargo check

```bash
cargo check
```

실행 파일을 생성하지 않고 컴파일 에러만 확인합니다. `cargo build`보다 훨씬 빠르므로, TDD나 빠른 피드백 루프에서 유용합니다.

IDE(rust-analyzer)는 내부적으로 `cargo check`를 지속적으로 실행하여 실시간 에러를 표시합니다.

---

## 워크스페이스

여러 관련 크레이트를 하나의 저장소에서 관리할 때 사용합니다. Node.js의 monorepo(npm workspaces, Turborepo)와 유사합니다.

```text
blockchain-workspace/
├── Cargo.toml          # 워크스페이스 루트
├── core/               # 핵심 블록체인 로직 (라이브러리)
│   ├── Cargo.toml
│   └── src/lib.rs
├── node/               # 노드 실행 프로그램
│   ├── Cargo.toml
│   └── src/main.rs
└── cli/                # CLI 도구
    ├── Cargo.toml
    └── src/main.rs
```

루트 `Cargo.toml`:

```toml
[workspace]
members = [
    "core",
    "node",
    "cli",
]
resolver = "2"  # 의존성 해석 버전 (2021 edition에서 권장)
```

멤버 크레이트에서 다른 멤버 참조:

```toml
# node/Cargo.toml
[dependencies]
core = { path = "../core" }
```

---

## 모듈 시스템 기초

Rust의 모듈 시스템은 Node.js의 `import/export`와 다릅니다. 파일 이름이 모듈 이름이 됩니다.

처음에는 다음 대응만 기억하세요.

| Node.js/TypeScript | Rust |
|--------------------|------|
| `import { sha256 } from "./crypto"` | `mod crypto; use crypto::sha256;` |
| `export function sha256(...)` | `pub fn sha256(...)` |
| `export class Block` | `pub struct Block` + `impl Block` |
| `models/index.ts`에서 re-export | `models/mod.rs`에서 `pub use` |

Rust는 파일을 만들었다고 자동으로 import하지 않습니다. `mod crypto;`처럼 “이 파일을 모듈 트리에 포함한다”고 선언해야 합니다.

### 기본 모듈 선언

```rust,ignore
// src/main.rs
mod crypto;    // src/crypto.rs 또는 src/crypto/mod.rs 를 모듈로 선언
mod models;    // src/models.rs 또는 src/models/mod.rs

use crypto::sha256;
use models::Block;

fn main() {
    let hash = sha256("hello");
    let block = Block::new();

    println!("hash = {}", hash);
    println!("block #{} = {}", block.index, block.hash);
}
```

```rust,ignore
// src/crypto.rs
use sha2::{Digest, Sha256};

pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
```

```rust
// src/models.rs
pub struct Block {
    pub index: u64,
    pub hash: String,
}

impl Block {
    pub fn new() -> Self {
        Block {
            index: 0,
            hash: String::new(),
        }
    }
}
```

이 예제에서 `pub`은 “다른 모듈에서도 접근 가능”이라는 뜻입니다. `pub`이 없으면 같은 파일 또는 같은 모듈 안에서만 접근할 수 있습니다. TypeScript에서 `export`를 붙이지 않으면 다른 파일에서 import할 수 없는 것과 비슷합니다.

### 하위 디렉토리 모듈

```text
src/
├── main.rs
└── models/
    ├── mod.rs      # models 모듈의 루트
    ├── block.rs    # models::block 서브모듈
    └── tx.rs       # models::tx 서브모듈
```

```rust,ignore
// src/models/mod.rs
pub mod block;
pub mod tx;

// re-export (외부에서 models::Block으로 접근 가능)
pub use block::Block;
pub use tx::Transaction;
```

```rust,ignore
// src/main.rs
mod models;
use models::Block;  // models/mod.rs에서 re-export 했으므로 가능
```

Node.js의 `index.ts`에서 re-export하는 패턴과 동일합니다:

```typescript
// models/index.ts (Node.js)
export { Block } from './block';
export { Transaction } from './tx';
```

---

## 자주 쓰이는 크레이트 목록

블록체인 개발에서 자주 보게 될 크레이트들:

| 크레이트 | 용도 | npm 유사물 |
|---------|------|-----------|
| `serde` | 직렬화/역직렬화 | `class-transformer` |
| `serde_json` | JSON 처리 | `JSON` 내장 |
| `tokio` | 비동기 런타임 | Node.js 자체 |
| `reqwest` | HTTP 클라이언트 | `axios` |
| `axum` | HTTP 서버 | `express` / `fastify` |
| `sha2` | SHA 해시 | `crypto` (내장) |
| `hex` | 16진수 인코딩 | `Buffer.toString('hex')` |
| `thiserror` | 에러 타입 정의 | - |
| `anyhow` | 에러 처리 편의 | - |
| `log` | 로깅 인터페이스 | `winston` (인터페이스) |
| `env_logger` | 로깅 구현체 | `winston` |
| `tracing` | 구조화된 로깅 | `pino` |
| `clap` | CLI 인자 파싱 | `commander` / `yargs` |
| `dotenv` | .env 파일 로드 | `dotenv` |
| `chrono` | 날짜/시간 | `dayjs` / `date-fns` |

---

## 요약

- `Cargo.toml`은 `package.json`과 유사하지만 빌드 프로파일, features 등 더 많은 설정 가능
- `cargo add <크레이트>`로 의존성 추가
- `Cargo.lock`은 바이너리 프로젝트에서 반드시 커밋
- 개발 빌드(`cargo build`)와 릴리스 빌드(`cargo build --release`)의 성능 차이가 큼
- 모듈 시스템은 파일 이름 기반, `mod` 키워드로 선언

다음으로는 블록체인의 기본 개념을 먼저 살펴본 후, Rust의 핵심인 소유권 시스템을 배웁니다.
