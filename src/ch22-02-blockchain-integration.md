# 22-2: 블록체인 연동 흐름 상세

## 전체 흐름 개요

platform에서 블록체인 연동은 iksan-api 서비스가 담당한다. 이벤트 하나가 생성되면 다음 흐름이 실행된다:

```text
클라이언트
    │
    │ POST /events { crop_id, event_type, payload }
    ▼
[iksan-api: routes/event.rs]
    │
    │ create_event(req)
    ▼
[iksan-api: services/event.rs]
    │
    ├─ 1. payload → keccak256 해시 계산
    ├─ 2. DB 저장 (trace_events 테이블)
    │
    │ record_hash(event_id, hash)
    ▼
[iksan-api: services/blockchain.rs]
    │
    ├─ 3. TraceRecord.recordHash() 트랜잭션 전송
    ├─ 4. 영수증 대기
    ├─ 5. TX 해시 DB 업데이트 (blockchain_records 테이블)
    │
    ▼
클라이언트 응답 (event + tx_hash)
```

## 단계별 상세 코드

### 1단계: 이벤트 생성 → 해시 계산

```rust,ignore
// apps/iksan-api/src/services/event.rs

use sha3::{Digest, Keccak256};
use alloy::primitives::keccak256;

pub async fn create_event(
    db: &PgPool,
    blockchain: &BlockchainService,
    req: CreateEventRequest,
) -> Result<TraceEventResponse, AppError> {
    // 이벤트 ID 생성
    let event_id = Uuid::new_v4();
    
    // payload를 정규화된 JSON으로 직렬화
    // 키 정렬로 동일 데이터는 항상 같은 해시 보장
    let payload_normalized = {
        let mut map: std::collections::BTreeMap<String, serde_json::Value> =
            serde_json::from_value(req.payload.clone())
                .map_err(|_| AppError::BadRequest("payload는 객체여야 합니다".into()))?;
        serde_json::to_string(&map)?
    };
    
    // 해시 입력: event_id + event_type + normalized_payload
    let hash_input = format!(
        "{}{}{}",
        event_id,
        req.event_type,
        payload_normalized
    );
    
    // keccak256 계산 (Alloy primitives 사용)
    use alloy::primitives::keccak256 as alloy_keccak256;
    let hash_bytes = alloy_keccak256(hash_input.as_bytes());
    let data_hash = format!("0x{}", hex::encode(hash_bytes.as_slice()));
    
    tracing::debug!(
        event_id = %event_id,
        event_type = %req.event_type,
        data_hash = %data_hash,
        "해시 계산 완료"
    );
    
    sqlx::query!(
        "INSERT INTO trace_events (id, payload, data_hash) VALUES ($1, $2, $3)",
        event_id,
        payload,
        data_hash
    )
    .execute(&self.db)
    .await?;
```

해시 입력에 `event_id`를 포함하는 이유: 동일한 payload라도 다른 이벤트라면 다른 해시를 가져야 한다. `event_id`(UUID v4)는 항상 유니크하므로 해시 충돌을 방지한다.

### 2단계: DB 저장

```rust,ignore
    // 트랜잭션으로 원자적 저장
    let mut tx = db.begin().await.map_err(AppError::Database)?;
    
    // trace_events 삽입
    let event = sqlx::query_as!(
        TraceEvent,
        r#"
        INSERT INTO trace_events
            (id, crop_id, farmer_id, event_type, payload, data_hash, created_at)
        VALUES
            ($1, $2, $3, $4, $5::jsonb, $6, NOW())
        RETURNING *
        "#,
        event_id,
        req.crop_id,
        req.farmer_id,
        req.event_type,
        serde_json::to_string(&req.payload)?,
        data_hash,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::Database)?;
    
    tx.commit().await.map_err(AppError::Database)?;
    
    tracing::info!(event_id = %event_id, "이벤트 DB 저장 완료");
```

### 3단계: 블록체인에 해시 기록

```rust,ignore
    // 블록체인 기록은 DB 커밋 이후 별도로 실행
    // 실패해도 DB 데이터는 보존됨 (가용성 우선 설계)
    
    let hash_bytes: [u8; 32] = {
        let decoded = hex::decode(data_hash.trim_start_matches("0x"))
            .map_err(|e| AppError::Internal(e.into()))?;
        decoded.try_into()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("해시 크기 오류")))?
    };
    
    match blockchain.record_hash(&event_id.to_string(), hash_bytes).await {
        Ok(BlockchainReceipt { tx_hash, block_number }) => {
            // blockchain_records 테이블에 기록
            sqlx::query!(
                r#"
                INSERT INTO blockchain_records
                    (event_id, tx_hash, block_number, status, recorded_at)
                VALUES
                    ($1, $2, $3, 'confirmed', NOW())
                "#,
                event_id,
                tx_hash,
                block_number as i64,
            )
            .execute(db)
            .await
            .map_err(AppError::Database)?;
            
            tracing::info!(
                event_id = %event_id,
                tx_hash = %tx_hash,
                block_number = block_number,
                "블록체인 기록 완료"
            );
            
            Ok(build_response(event, Some(tx_hash), Some(block_number)))
        }
        Err(e) => {
            // 블록체인 실패: 재시도 큐에 추가
            tracing::warn!(
                event_id = %event_id,
                error = %e,
                "블록체인 기록 실패, 재시도 예정"
            );
            
            // pending 상태로 blockchain_records 삽입
            sqlx::query!(
                r#"
                INSERT INTO blockchain_records
                    (event_id, tx_hash, status)
                VALUES
                    ($1, '', 'pending')
                "#,
                event_id,
            )
            .execute(db)
            .await
            .ok(); // 이것도 실패하면 무시
            
            // API는 성공 응답 반환 (블록체인 없이도 사용 가능)
            Ok(build_response(event, None, None))
        }
    }
```

### 4단계: Alloy로 컨트랙트 호출 (blockchain.rs)

```rust,ignore
// apps/iksan-api/src/services/blockchain.rs

use alloy::{
    network::EthereumWallet,
    primitives::{Address, FixedBytes},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
};

sol! {
    #[sol(rpc)]
    contract TraceRecord {
        struct Record {
            bytes32 dataHash;
            uint256 timestamp;
            address recorder;
            bool exists;
        }
        
        function recordHash(string calldata eventId, bytes32 dataHash) external;
        function getRecord(string calldata eventId) external view returns (Record memory);
        function verifyHash(string calldata eventId, bytes32 dataHash) external view returns (bool);
        
        event HashRecorded(
            string indexed eventId,
            bytes32 dataHash,
            address indexed recorder,
            uint256 timestamp
        );
        
        error NotOwner(address caller);
    }
}

pub struct BlockchainReceipt {
    pub tx_hash: String,
    pub block_number: u64,
}

pub struct BlockchainService {
    rpc_url: String,
    private_key: String,
    contract_address: Address,
}

impl BlockchainService {
    pub async fn new(
        rpc_url: String,
        private_key: String,
        contract_address_str: String,
    ) -> anyhow::Result<Self> {
        let contract_address: Address = contract_address_str.parse()
            .map_err(|e| anyhow::anyhow!("컨트랙트 주소 파싱 실패: {}", e))?;
        
        // 연결 테스트
        let provider = ProviderBuilder::new()
            .on_http(rpc_url.parse()
                .map_err(|e| anyhow::anyhow!("RPC URL 오류: {}", e))?);
        
        let chain_id = provider.get_chain_id().await
            .map_err(|e| anyhow::anyhow!("블록체인 연결 실패: {}", e))?;
        
        tracing::info!(chain_id = chain_id, "블록체인 연결 확인");
        
        Ok(Self { rpc_url, private_key, contract_address })
    }
    
    /// 트랜잭션 서명 Provider 생성 (매 호출마다 새로 생성 - 단순성 우선)
    async fn signed_provider(
        &self,
    ) -> anyhow::Result<impl Provider> {
        let signer: PrivateKeySigner = self.private_key.parse()
            .map_err(|e| anyhow::anyhow!("개인키 파싱 실패: {}", e))?;
        
        let wallet = EthereumWallet::from(signer);
        
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(self.rpc_url.parse()
                .map_err(|e| anyhow::anyhow!("RPC URL 오류: {}", e))?);
        
        Ok(provider)
    }
    
    /// 해시를 블록체인에 기록
    pub async fn record_hash(
        &self,
        event_id: &str,
        data_hash: [u8; 32],
    ) -> anyhow::Result<BlockchainReceipt> {
        let provider = self.signed_provider().await?;
        let contract = TraceRecord::new(self.contract_address, &provider);
        
        let hash_bytes: FixedBytes<32> = FixedBytes::from(data_hash);
        
        tracing::debug!(
            event_id = event_id,
            hash = %hex::encode(&data_hash),
            "recordHash 트랜잭션 전송"
        );
        
        let pending = contract
            .recordHash(event_id.to_string(), hash_bytes)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("트랜잭션 전송 실패: {}", e))?;
        
        let tx_hash = format!("{:?}", pending.tx_hash());
        
        // 영수증 대기 (최대 60초)
        let receipt = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            pending.get_receipt(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("트랜잭션 타임아웃 (60초)"))?
        .map_err(|e| anyhow::anyhow!("영수증 대기 실패: {}", e))?;
        
        if !receipt.status() {
            return Err(anyhow::anyhow!(
                "트랜잭션 revert됨: {}",
                tx_hash
            ));
        }
        
        let block_number = receipt.block_number.unwrap_or(0);
        
        tracing::info!(
            event_id = event_id,
            tx_hash = %tx_hash,
            block_number = block_number,
            gas_used = receipt.gas_used,
            "recordHash 완료"
        );
        
        Ok(BlockchainReceipt {
            tx_hash,
            block_number,
        })
    }
    
    /// 온체인 레코드 조회
    pub async fn get_record(
        &self,
        event_id: &str,
    ) -> anyhow::Result<Option<OnChainRecord>> {
        let provider = ProviderBuilder::new()
            .on_http(self.rpc_url.parse()?);
        
        let contract = TraceRecord::new(self.contract_address, &provider);
        
        let result = contract
            .getRecord(event_id.to_string())
            .call()
            .await
            .map_err(|e| anyhow::anyhow!("getRecord 실패: {}", e))?;
        
        let record = result._0;
        
        if !record.exists {
            return Ok(None);
        }
        
        Ok(Some(OnChainRecord {
            data_hash: *record.dataHash,
            timestamp: record.timestamp.to::<u64>(),
            recorder: format!("{}", record.recorder),
        }))
    }
    
    /// 해시 검증
    pub async fn verify_hash(
        &self,
        event_id: &str,
        expected_hash: [u8; 32],
    ) -> anyhow::Result<bool> {
        let provider = ProviderBuilder::new()
            .on_http(self.rpc_url.parse()?);
        
        let contract = TraceRecord::new(self.contract_address, &provider);
        let hash_bytes: FixedBytes<32> = FixedBytes::from(expected_hash);
        
        let result = contract
            .verifyHash(event_id.to_string(), hash_bytes)
            .call()
            .await
            .map_err(|e| anyhow::anyhow!("verifyHash 실패: {}", e))?;
        
        Ok(result._0)
    }
}
```

## DID 컨트랙트 - 분산 신원 관리

platform은 DID(Decentralized Identifier)를 사용하여 농업인의 신원을 블록체인으로 관리한다. DID는 W3C 표준으로, 중앙 기관 없이 신원을 검증할 수 있다.

### DID 형식

```text
did:ethr:0x742d35Cc6634C0532925a3b844Bc454e4438f44e
│    │     └─ Ethereum 주소
│    └─ DID 메서드 (ethr = Ethereum 기반)
└─ DID 스킴
```

### DID 컨트랙트 (단순화)

```solidity
// apps/account/contracts/src/DID.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract DIDRegistry {
    struct DIDDocument {
        string did;
        address owner;
        bytes publicKey;
        string[] serviceEndpoints;
        uint256 createdAt;
        bool active;
    }
    
    mapping(string => DIDDocument) public documents;
    mapping(address => string) public addressToDid;
    
    event DIDRegistered(string indexed did, address indexed owner);
    event DIDDeactivated(string indexed did);
    
    function register(
        string calldata did,
        bytes calldata publicKey,
        string[] calldata serviceEndpoints
    ) external {
        require(bytes(documents[did].did).length == 0, "DID already exists");
        require(bytes(addressToDid[msg.sender]).length == 0, "Address already has DID");
        
        documents[did] = DIDDocument({
            did: did,
            owner: msg.sender,
            publicKey: publicKey,
            serviceEndpoints: serviceEndpoints,
            createdAt: block.timestamp,
            active: true
        });
        
        addressToDid[msg.sender] = did;
        
        emit DIDRegistered(did, msg.sender);
    }
    
    function resolve(string calldata did)
        external
        view
        returns (DIDDocument memory)
    {
        require(documents[did].active, "DID not found or deactivated");
        return documents[did];
    }
    
    function deactivate(string calldata did) external {
        require(documents[did].owner == msg.sender, "Not DID owner");
        documents[did].active = false;
        emit DIDDeactivated(did);
    }
}
```

### Rust에서 DID 등록

```rust,ignore
// apps/account/src/services/did.rs

sol! {
    #[sol(rpc)]
    contract DIDRegistry {
        struct DIDDocument {
            string did;
            address owner;
            bytes publicKey;
            string[] serviceEndpoints;
            uint256 createdAt;
            bool active;
        }
        
        function register(
            string calldata did,
            bytes calldata publicKey,
            string[] calldata serviceEndpoints
        ) external;
        
        function resolve(string calldata did)
            external view returns (DIDDocument memory);
        
        event DIDRegistered(string indexed did, address indexed owner);
    }
}

pub async fn register_did(
    blockchain: &BlockchainService,
    account_address: Address,
    public_key: Vec<u8>,
) -> anyhow::Result<String> {
    // DID 생성: did:ethr:{address}
    let did = format!("did:ethr:{}", account_address);
    
    // 서비스 엔드포인트 (플랫폼 API)
    let endpoints = vec![
        "https://platform.example.com/api/v1".to_string()
    ];
    
    // 블록체인에 등록
    let tx_hash = blockchain
        .register_did(&did, public_key, endpoints)
        .await?;
    
    tracing::info!(did = %did, tx_hash = %tx_hash, "DID 등록 완료");
    
    Ok(did)
}
```

## UUPS 프록시로 컨트랙트 업그레이드

platform의 TraceRecord와 DID 컨트랙트는 UUPS(Universal Upgradeable Proxy Standard) 패턴을 사용한다. 이를 통해 컨트랙트 로직을 업그레이드해도 기존 데이터와 주소가 유지된다.

### UUPS 패턴의 구조

```text
사용자/Rust 코드 → 프록시 컨트랙트 (주소 불변)
                        │
                        │ delegatecall
                        ▼
                   구현 컨트랙트 (로직, 교체 가능)
                   (데이터는 프록시에 저장됨)
```

### OpenZeppelin UUPS 사용

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

contract TraceRecordV1 is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    mapping(string => bytes32) private _hashes;
    mapping(string => uint256) private _timestamps;
    
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }
    
    function initialize(address initialOwner) public initializer {
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
    }
    
    function recordHash(string calldata eventId, bytes32 hash) external onlyOwner {
        _hashes[eventId] = hash;
        _timestamps[eventId] = block.timestamp;
    }
    
    function getHash(string calldata eventId) external view returns (bytes32) {
        return _hashes[eventId];
    }
    
    // 업그레이드 권한 - 오너만 가능
    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}
}

// V2 - 새 기능 추가
contract TraceRecordV2 is TraceRecordV1 {
    // 기존 storage layout 유지 필수!
    mapping(string => address) private _recorders; // 새 storage 추가
    
    function recordHashWithRecorder(
        string calldata eventId,
        bytes32 hash
    ) external onlyOwner {
        _hashes[eventId] = hash;
        _timestamps[eventId] = block.timestamp;
        _recorders[eventId] = msg.sender;
    }
}
```

### Rust에서 프록시 배포

```rust,ignore
// apps/account/src/foundry/contract.rs

pub async fn deploy_upgradeable_trace_record(
    rpc_url: &str,
    deployer_key: &str,
    owner: Address,
) -> anyhow::Result<Address> {
    // 1. 구현 컨트랙트 배포
    let impl_address = deploy_implementation(rpc_url, deployer_key).await?;
    
    // 2. 초기화 데이터 인코딩
    //    initialize(owner) 함수 호출 데이터
    use alloy::sol_types::SolCall;
    let init_data = TraceRecordV1::initializeCall { initialOwner: owner }
        .abi_encode();
    
    // 3. ERC1967Proxy 배포 (구현 주소 + 초기화 데이터)
    let proxy_address = deploy_proxy(
        rpc_url,
        deployer_key,
        impl_address,
        init_data,
    ).await?;
    
    tracing::info!(
        impl_address = %impl_address,
        proxy_address = %proxy_address,
        "UUPS 프록시 배포 완료"
    );
    
    // 4. 실제로 사용하는 것은 proxy_address
    Ok(proxy_address)
}
```

업그레이드 시:
```rust,ignore
pub async fn upgrade_to_v2(
    rpc_url: &str,
    owner_key: &str,
    proxy_address: Address,
    new_impl_address: Address,
) -> anyhow::Result<()> {
    // upgradeToAndCall() 호출
    // 프록시가 새 구현으로 교체됨
    // 저장된 데이터(해시들)는 그대로 유지
    Ok(())
}
```

## 재시도 패턴

블록체인 기록 실패는 흔히 발생한다 (네트워크 일시 장애, 가스 부족 등). platform은 실패한 기록을 나중에 재시도하는 패턴을 사용한다:

```rust,ignore
// apps/iksan-api/src/services/retry.rs

pub struct RetryService {
    db: PgPool,
    blockchain: Arc<BlockchainService>,
}

impl RetryService {
    /// 주기적으로 실행: pending 상태 레코드 재시도
    pub async fn retry_pending_records(&self) -> anyhow::Result<()> {
        // pending 상태인 blockchain_records 조회
        let pending = sqlx::query!(
            r#"
            SELECT br.id, br.event_id, te.data_hash
            FROM blockchain_records br
            JOIN trace_events te ON te.id = br.event_id
            WHERE br.status = 'pending'
            AND br.recorded_at < NOW() - INTERVAL '5 minutes'
            ORDER BY br.recorded_at ASC
            LIMIT 10
            "#,
        )
        .fetch_all(&self.db)
        .await?;
        
        for record in pending {
            let hash_bytes: [u8; 32] = hex::decode(
                record.data_hash.trim_start_matches("0x")
            )
            .unwrap_or_default()
            .try_into()
            .unwrap_or([0u8; 32]);
            
            match self.blockchain.record_hash(&record.event_id.to_string(), hash_bytes).await {
                Ok(receipt) => {
                    sqlx::query!(
                        "UPDATE blockchain_records
                         SET tx_hash = $1, block_number = $2, status = 'confirmed', recorded_at = NOW()
                         WHERE id = $3",
                        receipt.tx_hash,
                        receipt.block_number as i64,
                        record.id,
                    )
                    .execute(&self.db)
                    .await?;
                    
                    tracing::info!(event_id = %record.event_id, "재시도 성공");
                }
                Err(e) => {
                    tracing::warn!(event_id = %record.event_id, error = %e, "재시도 실패");
                }
            }
        }
        
        Ok(())
    }
}

// main.rs에서 백그라운드 태스크로 실행
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5분마다
    loop {
        interval.tick().await;
        if let Err(e) = retry_service.retry_pending_records().await {
            tracing::error!("재시도 실패: {}", e);
        }
    }
});
```

## 검증 흐름

`GET /events/:id/verify` 엔드포인트의 처리 흐름:

```rust,ignore
// apps/iksan-api/src/services/event.rs

pub async fn verify_event(
    db: &PgPool,
    blockchain: &BlockchainService,
    event_id: &str,
) -> Result<VerifyResult, AppError> {
    // 1. DB에서 이벤트 조회
    let event = sqlx::query!(
        "SELECT id, payload, event_type, data_hash FROM trace_events WHERE id = $1",
        Uuid::parse_str(event_id)?,
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("이벤트 없음: {}", event_id)))?;
    
    let db_hash = event.data_hash.clone();
    
    // 2. DB 데이터로 해시 재계산 (데이터 자체의 무결성 검증)
    let recalculated_hash = {
        let hash_input = format!(
            "{}{}{}",
            event.id,
            event.event_type,
            event.payload
        );
        let bytes = alloy::primitives::keccak256(hash_input.as_bytes());
        format!("0x{}", hex::encode(bytes.as_slice()))
    };
    
    // 3. DB에 저장된 해시와 재계산 해시 비교
    let db_integrity = recalculated_hash == db_hash;
    
    if !db_integrity {
        // 매우 심각한 상황: DB 데이터가 변조됨
        tracing::error!(
            event_id = event_id,
            stored_hash = %db_hash,
            recalculated = %recalculated_hash,
            "DB 데이터 변조 의심!"
        );
    }
    
    // 4. 온체인 해시와 DB 해시 비교
    let hash_bytes: [u8; 32] = hex::decode(db_hash.trim_start_matches("0x"))
        .map_err(|e| AppError::Internal(e.into()))?
        .try_into()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("해시 크기 오류")))?;
    
    match blockchain.verify_hash(event_id, hash_bytes).await {
        Ok(on_chain_matches) => {
            Ok(VerifyResult {
                event_id: event_id.to_string(),
                db_hash,
                db_integrity,           // DB 내부 일관성
                blockchain_verified: on_chain_matches, // DB-체인 일치
                is_fully_valid: db_integrity && on_chain_matches,
                message: match (db_integrity, on_chain_matches) {
                    (true, true) => "완전 무결성 확인".to_string(),
                    (true, false) => "DB는 일관적이나 체인과 불일치 (미기록 또는 변조)".to_string(),
                    (false, _) => "DB 데이터 변조 의심!".to_string(),
                },
            })
        }
        Err(e) => {
            Err(AppError::Blockchain(format!("온체인 검증 실패: {}", e)))
        }
    }
}
```

## 이벤트 인덱싱 (선택적)

이벤트 수가 많아지면 체인에서 과거 이벤트를 효율적으로 조회하기 어렵다. platform에서는 HashRecorded 이벤트를 인덱싱하는 백그라운드 서비스를 운영한다:

```rust,ignore
// apps/iksan-api/src/services/indexer.rs

pub async fn index_blockchain_events(
    provider: &impl Provider,
    db: &PgPool,
    contract_address: Address,
    from_block: u64,
) -> anyhow::Result<()> {
    use alloy::rpc::types::Filter;
    use alloy::sol_types::SolEvent;
    
    let filter = Filter::new()
        .address(contract_address)
        .event_signature(TraceRecord::HashRecorded::SIGNATURE_HASH)
        .from_block(from_block);
    
    let logs = provider.get_logs(&filter).await?;
    
    for log in logs {
        if let Ok(event) = TraceRecord::HashRecorded::decode_log(
            log.inner.as_ref(),
            true,
        ) {
            let event_id = &event.eventId;
            let tx_hash = format!("{:?}", log.transaction_hash.unwrap_or_default());
            let block_number = log.block_number.unwrap_or(0);
            
            // blockchain_records 업데이트
            sqlx::query!(
                r#"
                INSERT INTO blockchain_records (event_id, tx_hash, block_number, status)
                VALUES ($1::uuid, $2, $3, 'confirmed')
                ON CONFLICT (event_id) DO UPDATE
                SET tx_hash = EXCLUDED.tx_hash,
                    block_number = EXCLUDED.block_number,
                    status = 'confirmed'
                "#,
                Uuid::parse_str(event_id).ok(),
                tx_hash,
                block_number as i64,
            )
            .execute(db)
            .await?;
        }
    }
    
    Ok(())
}
```

## 요약

platform의 블록체인 연동 핵심:

1. **해시 계산**: `keccak256(event_id + type + normalized_payload)` - 결정론적
2. **DB 우선**: 이벤트는 항상 DB에 먼저 저장, 블록체인은 이후
3. **비동기 처리**: 블록체인 실패가 API를 막지 않음
4. **재시도**: `pending` 상태 레코드를 주기적으로 재처리
5. **DID**: 농업인 신원을 탈중앙 방식으로 관리
6. **UUPS 프록시**: 컨트랙트 로직 업그레이드 가능, 데이터 유지
7. **검증**: DB 해시 재계산 + 온체인 비교로 이중 검증

다음 장에서는 실제 platform 코드를 읽는 순서와 방법을 안내한다.
