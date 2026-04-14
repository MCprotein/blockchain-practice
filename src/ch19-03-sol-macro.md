# 19-3: sol! 매크로 - Solidity ABI를 Rust 타입으로

## sol! 매크로란

`sol!`은 Alloy가 제공하는 절차적 매크로(procedural macro)다. Solidity 코드나 ABI JSON을 파싱하여 컴파일 타임에 대응하는 Rust 타입을 자동 생성한다.

이것이 왜 강력한가? 런타임에 ABI를 해석하는 대신, 컴파일 타임에 타입을 확인한다. 잘못된 함수 인자를 전달하면 컴파일 에러가 발생한다. Node.js의 `ethers.js`는 런타임까지 이런 오류를 발견하지 못한다.

```text
ethers.js 방식:
  contract.transfer(address, amount)  ← 런타임에 ABI 인코딩, 타입 체크 없음

sol! 매크로 방식:
  contract.transfer(address, amount)  ← 컴파일 타임에 타입 검증
  // address 자리에 U256을 넣으면 컴파일 에러!
```

## 인라인 Solidity 코드에서 타입 생성

가장 일반적인 방법이다. Solidity 인터페이스나 컨트랙트 코드를 그대로 Rust 파일에 작성한다.

### 기본 사용법

```rust,ignore
use alloy::sol;

// 단순 인터페이스
sol! {
    interface ICounter {
        function count() external view returns (uint256);
        function increment() external;
        function decrement() external;
        function reset(uint256 value) external;
    }
}

// 컨트랙트 (상태 변수 포함)
sol! {
    contract Counter {
        uint256 public count;
        address public owner;
        
        constructor(address _owner);
        
        function increment() external;
        function getCount() external view returns (uint256);
        
        event Incremented(address indexed by, uint256 newCount);
        error NotOwner(address caller);
    }
}
```

### 생성되는 타입들

`sol!` 매크로는 다음 Rust 타입들을 자동 생성한다:

```rust,ignore
// sol! { contract Counter { ... } } 가 생성하는 것들:

// 1. 함수 호출 타입 (각 함수마다)
Counter::incrementCall  // 인자 없음
Counter::getCountCall   // 인자 없음
Counter::resetCall { value: U256 }  // 인자 있음

// 2. 함수 반환 타입
Counter::getCountReturn { _0: U256 }

// 3. 이벤트 타입
Counter::Incremented { by: Address, newCount: U256 }

// 4. 에러 타입
Counter::NotOwner { caller: Address }

// 5. 컨트랙트 인스턴스 (with #[sol(rpc)] 속성 시)
Counter::new(address, provider)  // ContractInstance 반환
```

### #[sol(rpc)] 속성

RPC 호출을 위한 메서드를 생성하려면 `#[sol(rpc)]` 속성이 필요하다:

```rust,ignore
use alloy::sol;

sol! {
    #[sol(rpc)]  // 이 속성이 있어야 .call(), .send() 메서드가 생성됨
    contract Counter {
        uint256 public count;
        
        function increment() external;
        function getCount() external view returns (uint256);
        
        event Incremented(address indexed by, uint256 newCount);
    }
}

// #[sol(rpc)]가 있으면:
let contract = Counter::new(address, &provider);
let count = contract.getCount().call().await?;  // 가능
let tx = contract.increment().send().await?;    // 가능

// #[sol(rpc)]가 없으면 ABI 인코딩만 가능:
let call_data = Counter::incrementCall {}.abi_encode();  // 가능
// contract.increment().call()  // 불가능
```

### 복잡한 타입 처리

Solidity의 구조체, 배열, 매핑을 처리하는 방법:

```rust,ignore
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract TraceRecord {
        struct Record {
            bytes32 dataHash;
            uint256 timestamp;
            address recorder;
            bool verified;
        }
        
        mapping(string => Record) public records;
        string[] public recordIds;
        
        function addRecord(string calldata id, bytes32 hash) external;
        function getRecord(string calldata id) external view returns (Record memory);
        function verifyRecord(string calldata id, bytes32 hash) external view returns (bool);
        function getAllIds() external view returns (string[] memory);
        
        event RecordAdded(
            string indexed id,
            bytes32 hash,
            address indexed recorder,
            uint256 timestamp
        );
        
        error RecordNotFound(string id);
        error DuplicateRecord(string id);
    }
}

// 생성된 타입 사용
async fn use_trace_record(
    contract: &TraceRecord::TraceRecordInstance<_, _>,
) -> eyre::Result<()> {
    // 구조체 반환값 처리
    let record = contract.getRecord("batch-001".to_string()).call().await?;
    
    // Record 구조체 필드 접근
    println!("해시: {:?}", record._0.dataHash);
    println!("타임스탬프: {}", record._0.timestamp);
    println!("기록자: {}", record._0.recorder);
    println!("검증됨: {}", record._0.verified);
    
    Ok(())
}
```

## JSON ABI 파일에서 타입 생성

컨트랙트 ABI를 JSON 파일로 가지고 있을 때 사용한다. Foundry로 컴파일하면 `out/` 디렉토리에 JSON이 생성된다.

### ABI JSON 파일 구조

Foundry가 생성하는 ABI JSON 파일 (`out/TraceRecord.sol/TraceRecord.json`):

```json
{
  "abi": [
    {
      "type": "function",
      "name": "addRecord",
      "inputs": [
        {"name": "id", "type": "string"},
        {"name": "hash", "type": "bytes32"}
      ],
      "outputs": [],
      "stateMutability": "nonpayable"
    },
    {
      "type": "event",
      "name": "RecordAdded",
      "inputs": [
        {"name": "id", "type": "string", "indexed": true},
        {"name": "hash", "type": "bytes32", "indexed": false}
      ]
    }
  ],
  "bytecode": {
    "object": "0x608060..."
  }
}
```

### JSON ABI 파일에서 로드

```rust,ignore
use alloy::sol;

// 방법 1: ABI만 있는 JSON
sol!(
    #[sol(rpc)]
    TraceRecord,
    "abi/TraceRecord.json"  // ABI 배열만 있는 파일
);

// 방법 2: Foundry 출력 파일 (ABI + bytecode 포함)
sol!(
    #[sol(rpc)]
    TraceRecord,
    "out/TraceRecord.sol/TraceRecord.json"
);

// 방법 3: 환경에 따라 경로 지정
sol!(
    #[sol(rpc)]
    MyContract,
    concat!(env!("CARGO_MANIFEST_DIR"), "/abi/MyContract.json")
);
```

### Cargo.toml에 ABI 파일 경로 설정

```toml
# build.rs가 ABI 파일 변경 시 재컴파일하도록
[package.metadata]
abi-dir = "abi/"

# 또는 build.rs 작성
```

```rust,ignore
// build.rs
fn main() {
    // ABI 파일이 바뀌면 재컴파일
    println!("cargo:rerun-if-changed=abi/");
}
```

## 생성된 타입으로 컨트랙트 호출

실제로 생성된 타입을 어떻게 활용하는지 보여주는 상세 예제:

```rust,ignore
use alloy::{
    primitives::{Address, FixedBytes, U256},
    sol,
    sol_types::SolEvent,
};

sol! {
    #[sol(rpc)]
    contract TraceRecord {
        struct Record {
            bytes32 dataHash;
            uint256 timestamp;
            address recorder;
        }
        
        function addRecord(string calldata id, bytes32 hash) external returns (uint256 recordIndex);
        function getRecord(string calldata id) external view returns (Record memory);
        function recordExists(string calldata id) external view returns (bool);
        
        event RecordAdded(string indexed id, bytes32 hash, uint256 timestamp);
        error RecordAlreadyExists(string id);
    }
}

// ABI 인코딩 직접 사용 (Provider 없이)
fn encode_call_data() {
    // 함수 호출 데이터 인코딩
    let call = TraceRecord::addRecordCall {
        id: "batch-001".to_string(),
        hash: FixedBytes::from([0x42u8; 32]),
    };
    
    use alloy::sol_types::SolCall;
    let encoded: Vec<u8> = call.abi_encode();
    println!("인코딩된 calldata: 0x{}", hex::encode(&encoded));
    
    // 반환값 디코딩
    let return_data: Vec<u8> = vec![/* raw bytes from node */];
    // let decoded = TraceRecord::addRecordReturn::abi_decode(&return_data, true)?;
}

// 이벤트 로그 파싱
fn parse_event_log(log: &alloy::rpc::types::Log) -> Option<TraceRecord::RecordAdded> {
    use alloy::sol_types::SolEvent;
    TraceRecord::RecordAdded::decode_log(log.inner.as_ref(), true).ok()
}
```

## 이벤트 필터링과 로그 파싱

이벤트(Event)는 Solidity에서 `emit`으로 발생시키는 로그다. 블록체인에 영구적으로 저장되며 Rust에서 필터링하여 읽을 수 있다.

### 과거 이벤트 조회

```rust,ignore
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::{Filter, FilterBlockOption},
    sol,
    sol_types::SolEvent,
};

sol! {
    #[sol(rpc)]
    contract ERC20Token {
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
}

async fn get_past_transfer_events(
    provider: &impl Provider,
    token_address: Address,
    from_block: u64,
) -> eyre::Result<Vec<ERC20Token::Transfer>> {
    // 이벤트 필터 구성
    let filter = Filter::new()
        .address(token_address)
        .event_signature(ERC20Token::Transfer::SIGNATURE_HASH)
        .from_block(from_block)
        .to_block(alloy::rpc::types::BlockNumberOrTag::Latest);
    
    // 로그 조회
    let logs = provider.get_logs(&filter).await?;
    
    println!("Transfer 이벤트 {}개 발견", logs.len());
    
    // 로그 파싱
    let mut events = Vec::new();
    for log in logs {
        match ERC20Token::Transfer::decode_log(log.inner.as_ref(), true) {
            Ok(transfer) => {
                println!(
                    "Transfer: {} → {} : {}",
                    transfer.from, transfer.to, transfer.value
                );
                events.push(transfer);
            }
            Err(e) => {
                eprintln!("로그 파싱 실패: {}", e);
            }
        }
    }
    
    Ok(events)
}
```

### 특정 주소가 관련된 이벤트만 필터링

```rust,ignore
async fn get_transfers_for_address(
    provider: &impl Provider,
    token_address: Address,
    user_address: Address,
) -> eyre::Result<()> {
    use alloy::primitives::B256;
    
    // indexed 파라미터로 필터링
    // Transfer(address indexed from, address indexed to, uint256 value)
    // from이 user_address인 이벤트만
    
    let topic1: B256 = user_address.into_word(); // from 필드
    
    let filter = Filter::new()
        .address(token_address)
        .event_signature(ERC20Token::Transfer::SIGNATURE_HASH)
        .topic1(topic1); // indexed 첫 번째 파라미터 (from)
    
    let outgoing_logs = provider.get_logs(&filter).await?;
    println!("발신 Transfer: {}개", outgoing_logs.len());
    
    // to가 user_address인 이벤트만
    let topic2: B256 = user_address.into_word(); // to 필드
    
    let filter_incoming = Filter::new()
        .address(token_address)
        .event_signature(ERC20Token::Transfer::SIGNATURE_HASH)
        .topic2(topic2); // indexed 두 번째 파라미터 (to)
    
    let incoming_logs = provider.get_logs(&filter_incoming).await?;
    println!("수신 Transfer: {}개", incoming_logs.len());
    
    Ok(())
}
```

### WebSocket으로 실시간 이벤트 구독

```rust,ignore
use alloy::providers::{Provider, ProviderBuilder};
use alloy::transports::ws::WsConnect;
use futures_util::StreamExt;

async fn subscribe_to_events(token_address: Address) -> eyre::Result<()> {
    let ws = WsConnect::new("ws://localhost:8546");
    let provider = ProviderBuilder::new().on_ws(ws).await?;
    
    let filter = Filter::new()
        .address(token_address)
        .event_signature(ERC20Token::Transfer::SIGNATURE_HASH);
    
    // 실시간 로그 구독
    let subscription = provider.subscribe_logs(&filter).await?;
    let mut stream = subscription.into_stream();
    
    println!("Transfer 이벤트 구독 시작...");
    
    while let Some(log) = stream.next().await {
        if let Ok(transfer) = ERC20Token::Transfer::decode_log(log.inner.as_ref(), true) {
            println!(
                "새 Transfer: {} → {} : {}",
                transfer.from, transfer.to, transfer.value
            );
        }
    }
    
    Ok(())
}
```

## 전체 예제: ERC-20 컨트랙트와 상호작용하는 Rust 클라이언트

```rust,ignore
use alloy::{
    network::EthereumWallet,
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::Filter,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolEvent,
};
use eyre::Result;

// 전체 ERC-20 ABI 정의
sol! {
    #[sol(rpc)]
    contract ERC20 {
        // 상태 변수 (자동으로 getter 생성됨)
        string public name;
        string public symbol;
        uint8 public decimals;
        uint256 public totalSupply;
        
        // 조회 함수
        function balanceOf(address account) external view returns (uint256);
        function allowance(address owner, address spender) external view returns (uint256);
        
        // 상태 변경 함수
        function transfer(address to, uint256 amount) external returns (bool);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
        
        // 이벤트
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);
        
        // 에러
        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
    }
}

pub struct Erc20Client {
    contract: ERC20::ERC20Instance<
        alloy::transports::http::Http<reqwest::Client>,
        alloy::providers::fillers::FillProvider<
            alloy::providers::fillers::JoinFill<
                alloy::providers::Identity,
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::GasFiller,
                    alloy::providers::fillers::JoinFill<
                        alloy::providers::fillers::BlobGasFiller,
                        alloy::providers::fillers::JoinFill<
                            alloy::providers::fillers::NonceFiller,
                            alloy::providers::fillers::ChainIdFiller,
                        >,
                    >,
                >,
            >,
            alloy::providers::fillers::WalletFiller<EthereumWallet>,
            alloy::network::Ethereum,
        >,
        alloy::network::Ethereum,
    >,
    token_address: Address,
}

// 타입이 너무 복잡할 때는 Box<dyn Provider>나 Arc<dyn Provider>를 사용하는 것도 방법
// 실제 코드에서는 제네릭으로 처리하는 경우가 많음

/// 더 실용적인 구조 - 제네릭 사용
pub struct SimpleErc20Client<P: Provider> {
    provider: P,
    token_address: Address,
}

impl<P: Provider + Clone> SimpleErc20Client<P> {
    pub fn new(provider: P, token_address: Address) -> Self {
        Self { provider, token_address }
    }
    
    fn contract(&self) -> ERC20::ERC20Instance<P::Transport, &P, alloy::network::Ethereum>
    where
        P::Transport: Clone,
    {
        ERC20::new(self.token_address, &self.provider)
    }
    
    /// 토큰 기본 정보 조회
    pub async fn get_info(&self) -> Result<TokenInfo> where P::Transport: Clone {
        let contract = self.contract();
        
        let name = contract.name().call().await?.name;
        let symbol = contract.symbol().call().await?.symbol;
        let decimals = contract.decimals().call().await?.decimals;
        let total_supply = contract.totalSupply().call().await?.totalSupply;
        
        Ok(TokenInfo { name, symbol, decimals, total_supply })
    }
    
    /// 잔액 조회
    pub async fn balance_of(&self, account: Address) -> Result<U256>
    where P::Transport: Clone {
        let result = self.contract().balanceOf(account).call().await?;
        Ok(result._0)
    }
    
    /// 전송
    pub async fn transfer(&self, to: Address, amount: U256) -> Result<alloy::primitives::B256>
    where P::Transport: Clone {
        let pending = self.contract()
            .transfer(to, amount)
            .send()
            .await?;
        
        let tx_hash = *pending.tx_hash();
        let receipt = pending.get_receipt().await?;
        
        if !receipt.status() {
            return Err(eyre::eyre!("전송 실패 (revert)"));
        }
        
        Ok(tx_hash)
    }
    
    /// 과거 Transfer 이벤트 조회
    pub async fn get_transfer_history(
        &self,
        from_block: u64,
    ) -> Result<Vec<TransferEvent>> where P::Transport: Clone {
        let filter = Filter::new()
            .address(self.token_address)
            .event_signature(ERC20::Transfer::SIGNATURE_HASH)
            .from_block(from_block);
        
        let logs = self.provider.get_logs(&filter).await?;
        
        let mut transfers = Vec::new();
        for log in logs {
            if let Ok(event) = ERC20::Transfer::decode_log(log.inner.as_ref(), true) {
                transfers.push(TransferEvent {
                    from: event.from,
                    to: event.to,
                    value: event.value,
                    block_number: log.block_number.unwrap_or(0),
                    tx_hash: log.transaction_hash.unwrap_or_default(),
                });
            }
        }
        
        Ok(transfers)
    }
}

#[derive(Debug)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: U256,
}

#[derive(Debug)]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub block_number: u64,
    pub tx_hash: alloy::primitives::B256,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    // 서명자 설정 (쓰기 작업용)
    let private_key = std::env::var("PRIVATE_KEY")
        .unwrap_or_else(|_| {
            // 개발용 Anvil 기본 키
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
        });
    
    let signer: PrivateKeySigner = private_key.parse()?;
    let my_address = signer.address();
    let wallet = EthereumWallet::from(signer);
    
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http("http://localhost:8545".parse()?);
    
    // 토큰 주소 (로컬 Anvil에 배포된 테스트 토큰 가정)
    let token_address: Address = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512".parse()?;
    
    let client = SimpleErc20Client::new(provider, token_address);
    
    // 1. 토큰 정보 조회
    match client.get_info().await {
        Ok(info) => {
            println!("=== 토큰 정보 ===");
            println!("이름: {}", info.name);
            println!("심볼: {}", info.symbol);
            println!("소수점: {}", info.decimals);
            println!("총 공급량: {}", info.total_supply);
        }
        Err(e) => eprintln!("토큰 정보 조회 실패: {}", e),
    }
    
    // 2. 잔액 조회
    match client.balance_of(my_address).await {
        Ok(balance) => println!("\n내 잔액: {}", balance),
        Err(e) => eprintln!("잔액 조회 실패: {}", e),
    }
    
    // 3. 전송
    let recipient: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse()?;
    let amount = U256::from(1_000u64);
    
    println!("\n{} 토큰 전송 중...", amount);
    match client.transfer(recipient, amount).await {
        Ok(tx_hash) => println!("전송 성공: {:?}", tx_hash),
        Err(e) => eprintln!("전송 실패: {}", e),
    }
    
    // 4. 이벤트 이력 조회
    println!("\n=== Transfer 이벤트 이력 ===");
    match client.get_transfer_history(0).await {
        Ok(events) => {
            for event in events {
                println!(
                    "블록 {}: {} → {} : {}",
                    event.block_number, event.from, event.to, event.value
                );
            }
        }
        Err(e) => eprintln!("이벤트 조회 실패: {}", e),
    }
    
    Ok(())
}
```

## 자주 발생하는 문제

### 1. 반환 타입 접근

```rust,ignore
// sol! 생성 타입의 반환값은 구조체임
let result = contract.balanceOf(addr).call().await?;
// result는 ERC20::balanceOfReturn { _0: U256 }

// 단일 반환값: ._0으로 접근
let balance: U256 = result._0;

// 여러 반환값:
// function getInfo() returns (string name, uint256 value)
// → result.name, result.value
```

### 2. bytes32 처리

```rust,ignore
use alloy::primitives::FixedBytes;

// bytes32는 FixedBytes<32>로 매핑됨
let hash: FixedBytes<32> = FixedBytes::from([0u8; 32]);

// &[u8; 32] 변환
let raw: [u8; 32] = *hash;

// Vec<u8>에서 변환
let data: Vec<u8> = vec![1, 2, 3]; // 반드시 32바이트여야 함
// let hash = FixedBytes::<32>::try_from(data.as_slice())?;
```

### 3. string 처리

```rust,ignore
// Solidity string → Rust String
let result = contract.getName().call().await?;
let name: String = result._0; // 이미 String

// Rust String → Solidity string (함수 인자)
let id = "batch-001".to_string();
contract.getRecord(id).call().await?;
// &str도 자동 변환됨
contract.getRecord("batch-001".to_string()).call().await?;
```

## 요약

`sol!` 매크로의 핵심:

- **인라인 Solidity**: 소스에 직접 ABI 작성, 간단하고 명확
- **JSON ABI 파일**: Foundry 출력과 연동, 빌드 파이프라인에 통합 가능
- **`#[sol(rpc)]`**: RPC 호출 메서드 생성 (없으면 ABI 인코딩만)
- **`.call()`**: view 함수 → 즉시 결과
- **`.send()`**: 상태 변경 함수 → PendingTransaction
- **이벤트**: `Filter` + `decode_log()` 또는 WebSocket 구독

다음 장(20장)부터는 Hyperledger Besu와 프라이빗 체인을 다룬다.
