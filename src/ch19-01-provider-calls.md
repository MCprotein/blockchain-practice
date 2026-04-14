# 19-1: Provider와 읽기 호출

## Provider란 무엇인가

Provider는 Ethereum 노드와의 연결을 추상화한 인터페이스다. 블록체인에서 데이터를 읽거나 트랜잭션을 제출할 때 사용한다. Node.js의 `ethers.provider`나 `viem`의 `publicClient`에 해당한다.

```text
Rust 코드 → Provider → JSON-RPC → Ethereum 노드 (Besu/Geth/etc.)
```

Provider는 두 가지 역할을 한다:
1. **읽기 (Read)**: 잔액 조회, 블록 정보, 트랜잭션 조회, view 함수 호출
2. **쓰기 (Write)**: 트랜잭션 전송 (서명이 필요하므로 다음 장에서 다룸)

## Provider 생성 방법

### HTTP Provider

가장 기본적인 방법이다. 대부분의 RPC 엔드포인트는 HTTP를 지원한다.

```rust,ignore
use alloy::providers::{Provider, ProviderBuilder};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // 로컬 개발 환경 (Anvil 또는 Hardhat)
    let provider = ProviderBuilder::new()
        .on_http("http://localhost:8545".parse()?);
    
    // Besu 프라이빗 체인
    let provider = ProviderBuilder::new()
        .on_http("http://besu-node:8545".parse()?);
    
    // Infura (이더리움 메인넷)
    let provider = ProviderBuilder::new()
        .on_http("https://mainnet.infura.io/v3/YOUR_KEY".parse()?);
    
    Ok(())
}
```

HTTP Provider는 단방향 요청/응답이다. 이벤트 구독이 필요하면 WebSocket을 사용해야 한다.

### WebSocket Provider

실시간 이벤트 구독(블록 생성, 로그 등)에 필요하다.

```rust,ignore
use alloy::providers::{Provider, ProviderBuilder};
use alloy::transports::ws::WsConnect;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ws = WsConnect::new("ws://localhost:8546");
    
    let provider = ProviderBuilder::new()
        .on_ws(ws)
        .await?;
    
    // 새 블록 구독
    let subscription = provider.subscribe_blocks().await?;
    let mut stream = subscription.into_stream();
    
    while let Some(block) = stream.next().await {
        println!("새 블록: {}", block.number);
    }
    
    Ok(())
}
```

### ProviderBuilder 패턴

`ProviderBuilder`는 Provider를 단계적으로 구성하는 빌더 패턴을 제공한다.

```rust,ignore
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;

// 읽기 전용 Provider (서명자 없음)
let read_provider = ProviderBuilder::new()
    .on_http("http://localhost:8545".parse()?);

// 서명 가능한 Provider (트랜잭션 전송용)
let signer: PrivateKeySigner = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".parse()?;
let wallet_provider = ProviderBuilder::new()
    .with_recommended_fillers()  // nonce, gas price 자동 관리
    .wallet(EthereumWallet::from(signer))
    .on_http("http://localhost:8545".parse()?);
```

`with_recommended_fillers()`는 다음을 자동으로 처리한다:
- **Nonce 관리**: 트랜잭션 nonce 자동 증가
- **Gas 추정**: 가스 한도 자동 추정
- **Gas 가격**: 현재 네트워크 가스 가격 자동 설정

## 읽기 호출 예제

### 잔액 조회

```rust,ignore
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;

async fn get_balance_example() -> eyre::Result<()> {
    let provider = ProviderBuilder::new()
        .on_http("http://localhost:8545".parse()?);
    
    let address: Address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".parse()?;
    
    // wei 단위로 반환됨
    let balance = provider.get_balance(address).await?;
    
    // ETH로 변환 (1 ETH = 10^18 wei)
    let balance_eth = balance.to_string();
    println!("잔액 (wei): {}", balance);
    
    // 더 읽기 좋게 표시
    let divisor = alloy::primitives::U256::from(1_000_000_000_000_000_000u128);
    let eth_part = balance / divisor;
    let wei_part = balance % divisor;
    println!("잔액: {}.{:018} ETH", eth_part, wei_part);
    
    Ok(())
}
```

### 블록 정보 조회

```rust,ignore
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::BlockNumberOrTag;

async fn get_block_example() -> eyre::Result<()> {
    let provider = ProviderBuilder::new()
        .on_http("http://localhost:8545".parse()?);
    
    // 최신 블록 번호 조회
    let block_number = provider.get_block_number().await?;
    println!("현재 블록: {}", block_number);
    
    // 특정 블록 정보 조회 (트랜잭션 해시만 포함)
    let block = provider
        .get_block_by_number(BlockNumberOrTag::Latest, false)
        .await?
        .expect("블록을 찾을 수 없음");
    
    println!("블록 해시: {:?}", block.header.hash);
    println!("타임스탬프: {}", block.header.timestamp);
    println!("가스 사용량: {}", block.header.gas_used);
    println!("트랜잭션 수: {}", block.transactions.len());
    
    // 특정 번호의 블록 (트랜잭션 상세 포함)
    let block_with_txs = provider
        .get_block_by_number(BlockNumberOrTag::Number(12345), true)
        .await?;
    
    if let Some(block) = block_with_txs {
        println!("블록 {} 트랜잭션:", block.header.number);
        for tx in block.transactions.into_transactions() {
            println!("  TX: {:?}", tx.hash);
        }
    }
    
    Ok(())
}
```

### 트랜잭션 조회

```rust,ignore
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::B256;

async fn get_transaction_example() -> eyre::Result<()> {
    let provider = ProviderBuilder::new()
        .on_http("http://localhost:8545".parse()?);
    
    let tx_hash: B256 = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060"
        .parse()?;
    
    // 트랜잭션 정보 조회
    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .expect("트랜잭션 없음");
    
    println!("From: {:?}", tx.from);
    println!("To: {:?}", tx.to);
    println!("Value: {}", tx.value);
    println!("Gas: {}", tx.gas);
    println!("Nonce: {}", tx.nonce);
    
    // 트랜잭션 영수증 (트랜잭션이 채굴된 후에만 존재)
    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await?;
    
    match receipt {
        Some(r) => {
            println!("상태: {}", if r.status() { "성공" } else { "실패" });
            println!("가스 사용량: {}", r.gas_used);
            println!("블록 번호: {:?}", r.block_number);
        }
        None => println!("아직 채굴되지 않음"),
    }
    
    Ok(())
}
```

## 컨트랙트 읽기 호출 (view 함수)

`view` 함수는 블록체인 상태를 변경하지 않는 읽기 전용 함수다. 가스가 들지 않고, 트랜잭션이 아닌 `eth_call` RPC로 처리된다.

```rust,ignore
use alloy::sol;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;

// ERC-20 인터페이스 정의
sol! {
    interface IERC20 {
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
    }
}

async fn read_contract_example() -> eyre::Result<()> {
    let provider = ProviderBuilder::new()
        .on_http("http://localhost:8545".parse()?);
    
    // USDC 컨트랙트 주소 (이더리움 메인넷)
    let contract_address: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    
    // 컨트랙트 인스턴스 생성
    let contract = IERC20::new(contract_address, &provider);
    
    // view 함수 호출 - .call()을 사용
    let name = contract.name().call().await?;
    let symbol = contract.symbol().call().await?;
    let decimals = contract.decimals().call().await?;
    let total_supply = contract.totalSupply().call().await?;
    
    println!("이름: {}", name._0);
    println!("심볼: {}", symbol._0);
    println!("소수점: {}", decimals._0);
    println!("총 공급량: {}", total_supply._0);
    
    // 특정 주소의 잔액 조회
    let holder: Address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".parse()?;
    let balance = contract.balanceOf(holder).call().await?;
    println!("잔액: {} (최소 단위)", balance._0);
    
    Ok(())
}
```

`sol!` 매크로가 자동으로 `IERC20::balanceOfCall` 같은 타입을 생성한다. 반환값은 구조체이며, `._0`, `._1` 등으로 접근한다.

## 전체 코드 예제 (async main + 에러 처리)

실제 프로젝트에서 사용할 수 있는 완전한 예제다:

```rust,ignore
use alloy::{
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::BlockNumberOrTag,
    sol,
};
use eyre::Result;

// ERC-20 ABI 정의
sol! {
    #[sol(rpc)]
    interface IERC20 {
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function balanceOf(address account) external view returns (uint256);
        
        event Transfer(address indexed from, address indexed to, uint256 value);
    }
}

struct BlockchainClient {
    provider: alloy::providers::RootProvider<alloy::transports::http::Http<reqwest::Client>>,
}

impl BlockchainClient {
    fn new(rpc_url: &str) -> Result<Self> {
        let provider = ProviderBuilder::new()
            .on_http(rpc_url.parse()?);
        Ok(Self { provider })
    }
    
    async fn get_chain_info(&self) -> Result<()> {
        let chain_id = self.provider.get_chain_id().await?;
        let block_number = self.provider.get_block_number().await?;
        
        println!("체인 ID: {}", chain_id);
        println!("현재 블록: {}", block_number);
        Ok(())
    }
    
    async fn get_eth_balance(&self, address: Address) -> Result<U256> {
        let balance = self.provider.get_balance(address).await?;
        Ok(balance)
    }
    
    async fn get_token_info(&self, token_address: Address) -> Result<()> {
        let token = IERC20::new(token_address, &self.provider);
        
        let name = token.name().call().await
            .map_err(|e| eyre::eyre!("name() 호출 실패: {}", e))?;
        
        let symbol = token.symbol().call().await
            .map_err(|e| eyre::eyre!("symbol() 호출 실패: {}", e))?;
        
        let decimals = token.decimals().call().await
            .map_err(|e| eyre::eyre!("decimals() 호출 실패: {}", e))?;
        
        println!("토큰: {} ({})", name._0, symbol._0);
        println!("소수점: {}", decimals._0);
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // tracing으로 로그 초기화
    tracing_subscriber::fmt::init();
    
    let client = BlockchainClient::new("http://localhost:8545")?;
    
    // 체인 정보
    client.get_chain_info().await?;
    
    // ETH 잔액 조회
    let address: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse()?;
    let balance = client.get_eth_balance(address).await?;
    println!("ETH 잔액: {} wei", balance);
    
    // 에러 처리 예시 - 잘못된 주소
    match client.get_eth_balance(Address::ZERO).await {
        Ok(balance) => println!("Zero 주소 잔액: {}", balance),
        Err(e) => eprintln!("에러: {}", e),
    }
    
    Ok(())
}
```

## ethers.js (Node.js)와의 코드 비교

Node.js 배경의 개발자를 위한 대응 코드 비교표다.

### 잔액 조회

```typescript
// ethers.js (Node.js)
import { ethers } from "ethers";

const provider = new ethers.JsonRpcProvider("http://localhost:8545");
const balance = await provider.getBalance("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
console.log(ethers.formatEther(balance)); // "1.5"
```

```rust,ignore
// Alloy (Rust)
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;

let provider = ProviderBuilder::new().on_http("http://localhost:8545".parse()?);
let address: Address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".parse()?;
let balance = provider.get_balance(address).await?;
// U256로 반환됨, 직접 포맷팅 필요
```

### 블록 조회

```typescript
// ethers.js
const block = await provider.getBlock("latest");
console.log(block.number, block.hash, block.timestamp);
```

```rust,ignore
// Alloy
use alloy::rpc::types::BlockNumberOrTag;

let block = provider
    .get_block_by_number(BlockNumberOrTag::Latest, false)
    .await?
    .unwrap();
println!("{} {:?} {}", block.header.number, block.header.hash, block.header.timestamp);
```

### 컨트랙트 읽기

```typescript
// ethers.js
const abi = ["function balanceOf(address) view returns (uint256)"];
const contract = new ethers.Contract(tokenAddress, abi, provider);
const balance = await contract.balanceOf(userAddress);
```

```rust,ignore
// Alloy - sol! 매크로로 타입 안전한 호출
sol! {
    interface IERC20 {
        function balanceOf(address) external view returns (uint256);
    }
}
let contract = IERC20::new(token_address, &provider);
let result = contract.balanceOf(user_address).call().await?;
let balance = result._0; // U256 타입
```

### 주요 차이점 정리

| 항목 | ethers.js | Alloy |
|------|-----------|-------|
| 비동기 | Promise / async-await | Future / async-await |
| 에러 처리 | try/catch | Result<T, E> |
| 타입 안전성 | 런타임 ABI 디코딩 | 컴파일 타임 타입 검증 |
| 숫자 타입 | BigInt | U256 |
| 주소 타입 | string | Address (20바이트) |
| 컨트랙트 정의 | ABI 배열 | sol! 매크로 |
| 에러 메시지 | JS 예외 | Rust Result |

## 실습: 체인 상태 모니터링

```rust,ignore
use alloy::providers::{Provider, ProviderBuilder};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let provider = ProviderBuilder::new()
        .on_http("http://localhost:8545".parse()?);
    
    println!("블록 모니터링 시작...");
    
    let mut last_block = provider.get_block_number().await?;
    
    loop {
        sleep(Duration::from_secs(2)).await;
        
        let current_block = provider.get_block_number().await?;
        
        if current_block > last_block {
            for block_num in (last_block + 1)..=current_block {
                let block = provider
                    .get_block_by_number(block_num.into(), false)
                    .await?;
                
                if let Some(b) = block {
                    println!(
                        "블록 {}: {} TX, 가스 {}",
                        block_num,
                        b.transactions.len(),
                        b.header.gas_used
                    );
                }
            }
            last_block = current_block;
        }
    }
}
```

## 요약

- **HTTP Provider**: 단순 읽기 호출에 적합, `ProviderBuilder::new().on_http(url)`
- **WebSocket Provider**: 실시간 이벤트 구독에 필요, `.on_ws(ws).await?`
- **`get_balance()`**: ETH 잔액을 wei 단위로 반환
- **`get_block_by_number()`**: 블록 정보 조회
- **`get_transaction_by_hash()`**: 트랜잭션 정보 조회
- **`contract.view_fn().call().await?`**: 컨트랙트 view 함수 호출
- `with_recommended_fillers()`로 nonce, gas를 자동 관리

다음 장에서는 서명자를 추가하여 실제 트랜잭션을 전송하는 방법을 배운다.
