# 19-2: 트랜잭션 서명과 전송

## 왜 서명이 필요한가

블록체인에서 상태를 변경하는 모든 행위는 트랜잭션이다. 트랜잭션은 반드시 개인키로 서명되어야 한다. 서명이 없으면 누구나 다른 사람의 이름으로 트랜잭션을 보낼 수 있기 때문이다.

```text
서명 과정:
1. 트랜잭션 데이터 구성 (to, value, data, gas, nonce 등)
2. 데이터를 해시 (keccak256)
3. 개인키로 해시에 ECDSA 서명
4. 서명된 트랜잭션을 노드에 전송
5. 노드가 서명 검증 후 블록에 포함
```

Node.js에서는 보통 `ethers.Wallet`으로 서명을 처리한다. Alloy에서는 `PrivateKeySigner`와 `EthereumWallet`을 사용한다.

## 로컬 서명자 (PrivateKeySigner)

`PrivateKeySigner`는 개인키를 메모리에 보관하는 가장 간단한 서명 방식이다. 프로덕션에서는 HSM이나 KMS를 사용해야 하지만, 개발과 학습에는 이것으로 충분하다.

```rust,ignore
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::primitives::Address;

fn create_signer_examples() -> eyre::Result<()> {
    // 방법 1: 16진수 개인키 문자열로 생성
    let signer: PrivateKeySigner = 
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse()?;
    
    // 방법 2: 랜덤 키 생성 (테스트용)
    let random_signer = PrivateKeySigner::random();
    
    // 방법 3: 환경변수에서 읽기 (권장)
    let private_key = std::env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY 환경변수가 없습니다");
    let signer_from_env: PrivateKeySigner = private_key.parse()?;
    
    // 서명자의 주소 확인
    println!("서명자 주소: {}", signer.address());
    
    // 체인 ID 설정 (EIP-155 replay protection)
    let signer_with_chain = signer.with_chain_id(Some(1337)); // 로컬 개발용 체인 ID
    
    Ok(())
}
```

> **보안 주의**: 절대로 개인키를 소스코드에 하드코딩하지 말 것. 환경변수나 시크릿 관리 서비스를 사용하라.

### 서명자가 있는 Provider 생성

트랜잭션을 전송하려면 서명자를 Provider에 연결해야 한다.

```rust,ignore
use alloy::{
    network::EthereumWallet,
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};

async fn create_wallet_provider() -> eyre::Result<()> {
    let private_key = std::env::var("PRIVATE_KEY")?;
    let signer: PrivateKeySigner = private_key.parse()?;
    
    // EthereumWallet은 여러 서명자를 관리할 수 있음
    let wallet = EthereumWallet::from(signer);
    
    let provider = ProviderBuilder::new()
        .with_recommended_fillers() // nonce, gas 자동 처리
        .wallet(wallet)
        .on_http("http://localhost:8545".parse()?);
    
    Ok(())
}
```

`with_recommended_fillers()`는 다음 3가지 Filler를 포함한다:
- **NonceFiller**: 트랜잭션 nonce를 자동으로 관리
- **GasFiller**: `eth_estimateGas`로 가스 한도 자동 추정
- **ChainIdFiller**: 체인 ID를 자동으로 설정

## 트랜잭션 구성

### TransactionRequest

```rust,ignore
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
    rpc::types::TransactionRequest,
};

fn build_transaction() -> eyre::Result<TransactionRequest> {
    let to: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse()?;
    
    let tx = TransactionRequest::default()
        .with_to(to)
        .with_value(U256::from(1_000_000_000_000_000u128)) // 0.001 ETH
        .with_gas_limit(21_000) // 단순 ETH 전송의 기본 가스
        .with_max_fee_per_gas(20_000_000_000u128) // 20 Gwei
        .with_max_priority_fee_per_gas(1_000_000_000u128); // 1 Gwei
    
    Ok(tx)
}
```

트랜잭션 필드:
- **`to`**: 수신 주소 (None이면 컨트랙트 배포)
- **`value`**: 전송할 ETH 양 (wei 단위)
- **`gas_limit`**: 허용할 최대 가스
- **`max_fee_per_gas`**: EIP-1559 최대 가스 가격 (wei/gas)
- **`max_priority_fee_per_gas`**: 검증자에게 주는 팁
- **`input`/`data`**: 컨트랙트 호출 데이터

### 가스 추정

`with_recommended_fillers()`를 사용하면 자동이지만, 직접 추정할 수도 있다:

```rust,ignore
use alloy::providers::Provider;

async fn estimate_gas_example(
    provider: &impl Provider,
    tx: &TransactionRequest,
) -> eyre::Result<u64> {
    let gas_estimate = provider.estimate_gas(tx).await?;
    
    // 안전 마진 20% 추가
    let gas_with_buffer = gas_estimate * 120 / 100;
    
    println!("추정 가스: {}", gas_estimate);
    println!("버퍼 포함: {}", gas_with_buffer);
    
    Ok(gas_with_buffer)
}
```

### Nonce 관리

Nonce는 계정에서 발송한 트랜잭션의 순번이다. 0부터 시작하여 트랜잭션마다 1씩 증가한다. 같은 nonce로 두 트랜잭션을 보내면 하나만 처리된다.

```rust,ignore
use alloy::providers::Provider;
use alloy::primitives::Address;

async fn nonce_management(
    provider: &impl Provider,
    sender: Address,
) -> eyre::Result<u64> {
    // 현재 nonce 조회 (확정된 트랜잭션 기준)
    let nonce = provider.get_transaction_count(sender).await?;
    
    println!("다음 nonce: {}", nonce);
    
    // with_recommended_fillers()를 사용하면 자동으로 관리됨
    // 수동으로 설정할 경우:
    // tx.with_nonce(nonce)
    
    Ok(nonce)
}
```

`with_recommended_fillers()`를 쓰면 nonce 관리가 자동이다. 하지만 빠른 연속 트랜잭션이나 특수한 경우에는 직접 관리가 필요할 수 있다.

## 트랜잭션 전송과 영수증 대기

```rust,ignore
use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};

async fn send_eth_transfer() -> eyre::Result<()> {
    // 서명자 설정
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = EthereumWallet::from(signer);
    
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http("http://localhost:8545".parse()?);
    
    let to: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse()?;
    let value = U256::from(100_000_000_000_000_000u128); // 0.1 ETH
    
    // 트랜잭션 전송
    println!("트랜잭션 전송 중...");
    let tx_hash = provider
        .send_transaction(
            alloy::rpc::types::TransactionRequest::default()
                .with_to(to)
                .with_value(value),
        )
        .await?;
    
    println!("TX 해시: {:?}", tx_hash.tx_hash());
    
    // 영수증 대기 (트랜잭션이 블록에 포함될 때까지)
    println!("영수증 대기 중...");
    let receipt = tx_hash.get_receipt().await?;
    
    println!("블록 번호: {:?}", receipt.block_number);
    println!("가스 사용량: {}", receipt.gas_used);
    println!("상태: {}", if receipt.status() { "성공" } else { "실패" });
    
    Ok(())
}
```

`send_transaction()`은 트랜잭션을 전송하고 `PendingTransactionBuilder`를 반환한다. `.get_receipt()`는 영수증이 올 때까지 폴링한다.

### 영수증 대기 옵션

```rust,ignore
use alloy::providers::PendingTransactionConfig;

// 기본: 1번 확인으로 완료 처리
let receipt = pending_tx.get_receipt().await?;

// 커스텀: 3번 블록 확인 후 완료
let receipt = pending_tx
    .with_required_confirmations(3)
    .with_timeout(Some(std::time::Duration::from_secs(60)))
    .get_receipt()
    .await?;
```

## 컨트랙트 쓰기 호출 (상태 변경 함수)

`sol!` 매크로로 정의된 컨트랙트의 상태 변경 함수를 호출하는 방법이다.

```rust,ignore
use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};

// 상태 변경 함수가 있는 컨트랙트 정의
sol! {
    #[sol(rpc)]
    contract SimpleStorage {
        uint256 public value;
        
        function setValue(uint256 newValue) external;
        function increment() external;
        
        event ValueChanged(uint256 indexed oldValue, uint256 indexed newValue);
    }
}

async fn write_contract_example() -> eyre::Result<()> {
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = EthereumWallet::from(signer);
    
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http("http://localhost:8545".parse()?);
    
    let contract_address: Address = "0x5FbDB2315678afecb367f032d93F642f64180aa3".parse()?;
    let contract = SimpleStorage::new(contract_address, &provider);
    
    // 읽기 호출 (view 함수) - .call()
    let current_value = contract.value().call().await?;
    println!("현재 값: {}", current_value._0);
    
    // 쓰기 호출 (상태 변경) - .send()
    let new_value = U256::from(42u64);
    
    println!("값 설정 중: {}", new_value);
    let pending_tx = contract.setValue(new_value).send().await?;
    
    println!("TX 해시: {:?}", pending_tx.tx_hash());
    
    // 영수증 대기
    let receipt = pending_tx.get_receipt().await?;
    println!("트랜잭션 완료. 블록: {:?}", receipt.block_number);
    
    // 이벤트 로그 파싱
    for log in &receipt.inner.logs() {
        if let Ok(event) = SimpleStorage::ValueChanged::decode_log(log.inner.as_ref(), true) {
            println!("이벤트: {} → {}", event.oldValue, event.newValue);
        }
    }
    
    // 변경된 값 확인
    let updated_value = contract.value().call().await?;
    println!("업데이트된 값: {}", updated_value._0);
    
    Ok(())
}
```

핵심 차이:
- **`.call()`**: view/pure 함수, 가스 없음, 즉시 결과 반환
- **`.send()`**: 상태 변경 함수, 가스 필요, PendingTransaction 반환

## 전체 코드 예제: ERC-20 전송

실제 ERC-20 토큰 전송을 포함한 완전한 예제다:

```rust,ignore
use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
};
use eyre::Result;

sol! {
    #[sol(rpc)]
    contract ERC20 {
        string public name;
        string public symbol;
        uint8 public decimals;
        
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
}

async fn transfer_erc20(
    token_address: Address,
    recipient: Address,
    amount: U256,
) -> Result<()> {
    // 서명자 설정
    let private_key = std::env::var("PRIVATE_KEY")
        .map_err(|_| eyre::eyre!("PRIVATE_KEY 환경변수 없음"))?;
    
    let signer: PrivateKeySigner = private_key
        .parse()
        .map_err(|e| eyre::eyre!("개인키 파싱 실패: {}", e))?;
    
    let sender_address = signer.address();
    let wallet = EthereumWallet::from(signer);
    
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http("http://localhost:8545".parse()?);
    
    let token = ERC20::new(token_address, &provider);
    
    // 전송 전 잔액 확인
    let sender_balance = token.balanceOf(sender_address).call().await?;
    println!("내 잔액: {}", sender_balance._0);
    
    if sender_balance._0 < amount {
        return Err(eyre::eyre!("잔액 부족: {} < {}", sender_balance._0, amount));
    }
    
    // 전송
    println!("{} 토큰을 {} 에게 전송 중...", amount, recipient);
    
    let pending = token
        .transfer(recipient, amount)
        .send()
        .await
        .map_err(|e| eyre::eyre!("전송 실패: {}", e))?;
    
    let tx_hash = *pending.tx_hash();
    println!("TX 해시: {:?}", tx_hash);
    
    // 영수증 대기
    let receipt = pending
        .get_receipt()
        .await
        .map_err(|e| eyre::eyre!("영수증 대기 실패: {}", e))?;
    
    if !receipt.status() {
        return Err(eyre::eyre!("트랜잭션 실패 (reverted)"));
    }
    
    println!("전송 성공!");
    println!("블록: {:?}", receipt.block_number);
    println!("가스 사용: {}", receipt.gas_used);
    
    // Transfer 이벤트 파싱
    for log in receipt.inner.logs() {
        if let Ok(transfer) = ERC20::Transfer::decode_log(log.inner.as_ref(), true) {
            println!(
                "이벤트 Transfer: {} → {} : {}",
                transfer.from, transfer.to, transfer.value
            );
        }
    }
    
    // 전송 후 잔액 확인
    let new_balance = token.balanceOf(sender_address).call().await?;
    println!("전송 후 내 잔액: {}", new_balance._0);
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let token_address: Address = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512".parse()?;
    let recipient: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse()?;
    let amount = U256::from(1_000_000u64); // 1 토큰 (decimals=6 가정)
    
    transfer_erc20(token_address, recipient, amount).await?;
    
    Ok(())
}
```

## 트랜잭션 에러 처리 패턴

컨트랙트 호출이 revert되는 경우를 처리하는 방법:

```rust,ignore
use alloy::contract::Error as ContractError;
use alloy::transports::RpcError;

async fn robust_contract_call(contract: &ERC20, to: Address, amount: U256) -> Result<()> {
    match contract.transfer(to, amount).send().await {
        Ok(pending) => {
            let receipt = pending.get_receipt().await?;
            if receipt.status() {
                println!("성공");
            } else {
                // 트랜잭션이 실행되었지만 revert됨
                eprintln!("트랜잭션 revert됨. TX: {:?}", receipt.transaction_hash);
                return Err(eyre::eyre!("트랜잭션 revert"));
            }
        }
        Err(e) => {
            // RPC 레벨 에러 (예: 가스 부족, 인코딩 에러)
            eprintln!("트랜잭션 전송 실패: {}", e);
            return Err(eyre::eyre!("전송 실패: {}", e));
        }
    }
    
    Ok(())
}
```

## platform 프로젝트의 트랜잭션 패턴

platform의 iksan-api 서비스에서 블록체인에 해시를 기록하는 실제 패턴:

```rust,ignore
// apps/iksan-api/src/services/blockchain.rs 패턴
pub struct BlockchainService {
    provider: Arc<dyn Provider>,
    contract_address: Address,
    wallet: EthereumWallet,
}

impl BlockchainService {
    pub async fn record_trace_hash(
        &self,
        record_id: &str,
        data_hash: [u8; 32],
    ) -> Result<B256> {
        // TraceRecord 컨트랙트 호출
        let contract = TraceRecord::new(self.contract_address, &self.provider);
        
        let pending = contract
            .recordHash(record_id.to_string(), FixedBytes::from(data_hash))
            .send()
            .await
            .map_err(|e| AppError::BlockchainError(e.to_string()))?;
        
        let tx_hash = *pending.tx_hash();
        
        // 영수증 대기 (비즈니스 로직에 따라 필요 여부 결정)
        let receipt = pending.get_receipt().await
            .map_err(|e| AppError::BlockchainError(e.to_string()))?;
        
        if !receipt.status() {
            return Err(AppError::BlockchainError("컨트랙트 call revert됨".to_string()));
        }
        
        Ok(tx_hash)
    }
}
```

## 요약

트랜잭션 서명과 전송의 핵심:

1. **PrivateKeySigner**: 개인키로 서명자 생성
2. **EthereumWallet**: 서명자를 감싸는 지갑
3. **`with_recommended_fillers()`**: nonce, gas 자동 관리
4. **`.send()`**: 상태 변경 트랜잭션 전송 → `PendingTransaction`
5. **`.get_receipt()`**: 영수증 대기 (트랜잭션 확정 확인)
6. **`.call()`**: view 함수, 트랜잭션 없음

다음 장에서는 `sol!` 매크로를 더 깊이 이해한다.
