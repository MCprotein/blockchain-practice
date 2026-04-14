# 19장: Alloy - Rust에서 Ethereum과 상호작용하기

## Alloy란 무엇인가

Alloy는 Rust에서 Ethereum 블록체인과 상호작용하기 위한 최신 라이브러리 모음이다. `ethers-rs`의 후속 프로젝트로, Foundry 팀(Paradigm)이 주도하여 개발하고 있다. 2023년 말부터 본격적으로 개발되었으며, 2024년에 안정 버전이 출시되었다.

Alloy를 한 문장으로 정의하면: **Rust 생태계에서 Ethereum JSON-RPC를 다루는 표준 방법**이다.

Node.js 개발자에게 익숙한 `ethers.js`나 `viem`과 유사한 역할을 한다. 다만 Rust의 타입 시스템과 비동기 런타임을 완전히 활용한다는 점이 다르다.

```
ethers.js (Node.js)  ↔  Alloy (Rust)
viem (TypeScript)    ↔  Alloy (Rust)
web3.py (Python)     ↔  Alloy (Rust)
```

## ethers-rs에서 Alloy로의 전환 배경

### ethers-rs의 한계

`ethers-rs`는 2020년부터 개발된 Rust용 Ethereum 라이브러리였다. 하지만 몇 가지 구조적 문제가 있었다:

1. **Provider 구조의 복잡성**: 미들웨어 패턴이 중첩되어 타입이 복잡해졌다
2. **ABI 처리 방식**: `ethabi` 크레이트에 의존하여 타입 안전성이 부족했다
3. **유지보수 중단**: 2023년부터 ethers-rs는 더 이상 활발히 유지되지 않는다

공식 저장소에는 이런 메시지가 있다:

> "ethers-rs is in maintenance mode. New projects should use Alloy."

### Alloy의 설계 철학

Alloy는 처음부터 다시 설계되었다:

- **모듈성**: 필요한 크레이트만 선택적으로 사용
- **타입 안전성**: `sol!` 매크로로 컴파일 타임에 ABI 검증
- **성능**: 불필요한 직렬화/역직렬화 최소화
- **인체공학**: Provider 빌더 패턴으로 간결한 설정

### ethers-rs vs Alloy 코드 비교

```rust
// ethers-rs 방식 (구식)
use ethers::providers::{Http, Provider, Middleware};
use ethers::types::Address;

let provider = Provider::<Http>::try_from("http://localhost:8545")?;
let balance = provider.get_balance(address, None).await?;

// Alloy 방식 (현대적)
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;

let provider = ProviderBuilder::new().on_http("http://localhost:8545".parse()?);
let balance = provider.get_balance(address).await?;
```

Alloy는 코드가 더 간결하고 타입이 명확하다.

## Alloy의 구성 요소

Alloy는 단일 거대 라이브러리가 아니라 목적별로 분리된 크레이트들의 모음이다. 각 크레이트를 이해하는 것이 중요하다.

### alloy-primitives

Ethereum에서 사용하는 기본 타입들을 정의한다.

```rust
use alloy::primitives::{
    Address,    // 20바이트 Ethereum 주소
    U256,       // 256비트 부호 없는 정수 (토큰 잔액 등)
    B256,       // 32바이트 해시값 (트랜잭션 해시 등)
    Bytes,      // 동적 크기 바이트 배열
    keccak256,  // Keccak-256 해시 함수
    FixedBytes, // 고정 크기 바이트 배열
};
```

`U256`은 Ethereum에서 모든 숫자를 표현하는 데 사용된다. Solidity의 `uint256`에 대응한다.

```rust
// U256 사용 예시
let one_ether = U256::from(1_000_000_000_000_000_000u128); // 1 ETH = 10^18 wei
let amount: U256 = "1000000000000000000".parse().unwrap();

// Address 사용 예시
let addr: Address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".parse().unwrap();
```

### alloy-provider

블록체인 노드와의 연결을 관리한다. HTTP, WebSocket, IPC 등 다양한 전송 방식을 지원한다.

```rust
use alloy::providers::{Provider, ProviderBuilder};
use alloy::network::Ethereum;
use alloy::transports::http::Http;

// HTTP Provider
let provider = ProviderBuilder::new()
    .on_http("http://localhost:8545".parse()?);

// WebSocket Provider
let provider = ProviderBuilder::new()
    .on_ws(alloy::transports::ws::WsConnect::new("ws://localhost:8546"))
    .await?;
```

Provider는 블록체인 상태 조회(읽기)에 사용된다.

### alloy-signer

개인키를 관리하고 트랜잭션에 서명한다.

```rust
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;

// 개인키로 서명자 생성
let signer: PrivateKeySigner = "0xac0974...".parse()?;

// 서명자의 주소 확인
println!("Address: {}", signer.address());
```

### alloy-contract

스마트 컨트랙트와의 상호작용을 추상화한다. `sol!` 매크로로 생성된 타입과 함께 사용한다.

```rust
use alloy::contract::ContractInstance;
use alloy::dyn_abi::DynSolValue;

// 동적 ABI로 컨트랙트 호출
let contract = ContractInstance::new(address, provider, interface);
let result = contract.function("transfer", &[to_value, amount_value])?.call().await?;
```

### alloy-sol-types와 sol! 매크로

가장 강력한 기능이다. Solidity 코드나 ABI를 Rust 타입으로 컴파일 타임에 변환한다.

```rust
use alloy::sol;

// Solidity 코드를 직접 인라인으로 작성
sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        event Transfer(address indexed from, address indexed to, uint256 value);
    }
}

// 이제 IERC20::balanceOfCall, IERC20::transferCall 등의 타입을 사용 가능
```

### alloy-network

여러 Ethereum 호환 네트워크를 추상화한다. Ethereum 메인넷, Besu 프라이빗 체인, Optimism 등을 동일한 인터페이스로 다룰 수 있다.

## Cargo.toml 설정 방법

Alloy는 feature flag로 필요한 기능만 선택할 수 있다. 이는 컴파일 시간과 바이너리 크기를 줄이는 데 도움이 된다.

### 기본 설정

```toml
[dependencies]
# Alloy 전체 패키지 (개발/학습용으로 편리)
alloy = { version = "0.9", features = ["full"] }

# 비동기 런타임
tokio = { version = "1", features = ["full"] }

# 에러 처리
eyre = "0.6"
anyhow = "1"
```

### 프로덕션 설정 (필요한 기능만)

```toml
[dependencies]
alloy = { version = "0.9", features = [
    # 코어 기능
    "providers",          # Provider, ProviderBuilder
    "provider-http",      # HTTP 전송
    "provider-ws",        # WebSocket 전송 (이벤트 구독 필요 시)
    
    # 컨트랙트 상호작용
    "contract",           # ContractInstance
    "sol-types",          # sol! 매크로, ABI 인코딩
    "json-abi",           # JSON ABI 파일 파싱
    
    # 서명
    "signers",            # Signer 트레이트
    "signer-local",       # PrivateKeySigner
    
    # 네트워크
    "network",            # Ethereum 네트워크 추상화
    "rpc-types",          # RPC 요청/응답 타입
    
    # 추가 유틸리티
    "consensus",          # 블록, 트랜잭션 타입
] }

tokio = { version = "1", features = ["full"] }
eyre = "0.6"
```

### platform 프로젝트의 실제 Cargo.toml 패턴

platform의 iksan-api 서비스는 다음과 같이 설정한다:

```toml
[package]
name = "iksan-api"
version = "0.1.0"
edition = "2021"

[dependencies]
# Alloy - 블록체인 상호작용
alloy = { version = "0.9", features = [
    "providers",
    "provider-http",
    "contract",
    "sol-types",
    "json-abi",
    "signers",
    "signer-local",
    "network",
    "rpc-types",
    "consensus",
    "node-bindings",   # 테스트용 Anvil 실행
] }

# 웹 프레임워크
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# 데이터베이스
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-native-tls", "uuid", "chrono"] }

# 직렬화
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 에러 처리
anyhow = "1"
thiserror = "2"

# 로깅
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 유틸리티
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
hex = "0.4"
sha2 = "0.10"
```

## Alloy 버전과 호환성

Alloy는 빠르게 발전하고 있다. 버전 간 API 변경이 있을 수 있으므로 주의가 필요하다.

| 버전 | 상태 | 주요 변경사항 |
|------|------|--------------|
| 0.1 | EOL | 최초 안정 버전 |
| 0.3 | EOL | Provider API 개선 |
| 0.6 | 유지보수 | sol! 매크로 안정화 |
| 0.9 | 현재 권장 | 네트워크 추상화 개선 |

항상 `Cargo.lock`을 커밋하여 팀 간 버전 일관성을 유지하라.

## 요약

Alloy는 Rust에서 Ethereum과 상호작용하는 현대적인 표준이다:

- **alloy-primitives**: 기본 타입 (Address, U256, B256)
- **alloy-provider**: 노드 연결과 RPC 호출
- **alloy-signer**: 개인키 관리와 트랜잭션 서명
- **alloy-contract**: 스마트 컨트랙트 상호작용
- **alloy-sol-types**: Solidity ABI를 Rust 타입으로 변환

다음 장에서는 Provider를 사용하여 실제로 블록체인 데이터를 읽어보겠다.
