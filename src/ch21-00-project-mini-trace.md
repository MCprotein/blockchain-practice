# 21장: 미니프로젝트 - Platform 스타일 트레이서빌리티 서비스

## 프로젝트 개요

이 장에서는 platform 프로젝트의 핵심 패턴을 축소하여 직접 구현한다. 지금까지 배운 모든 것 - Rust, Axum, SQLx, Alloy, Solidity - 을 하나의 동작하는 서비스로 통합한다.

**서비스 이름**: `mini-trace`

**핵심 기능**:
- 이벤트(trace event)를 생성하면 PostgreSQL에 저장하고, 해시를 블록체인에 기록
- 이벤트를 조회하면 DB 데이터와 함께 온체인 검증 결과를 반환
- 별도 검증 엔드포인트로 DB와 체인 간 데이터 무결성을 확인

## 전체 요구사항

### 기능 요구사항

1. **TraceRecord.sol**: 데이터 해시를 기록하고 검증하는 Solidity 컨트랙트
2. **Rust 백엔드 (Axum)**:
   - `POST /events`: 이벤트 생성 → keccak256 해시 계산 → DB 저장 → 체인 기록
   - `GET /events/:id`: 이벤트 조회 + 온체인 해시 비교 결과 포함
   - `GET /events/:id/verify`: DB 해시 vs 온체인 해시 비교, 무결성 검증
3. **SQLite**: 개발 편의를 위해 SQLite 사용 (platform은 PostgreSQL)
4. **Alloy**: TraceRecord 컨트랙트와 상호작용

### 비기능 요구사항

- 에러 처리: 블록체인 실패가 API 실패로 이어지지 않도록 (가용성 우선)
- 로깅: `tracing` 크레이트로 구조화된 로그
- 환경변수: 설정을 코드에서 분리

## 프로젝트 구조

```text
mini-trace/
├── Cargo.toml
├── .env
├── contracts/
│   └── src/
│       └── TraceRecord.sol
├── abi/
│   └── TraceRecord.json        ← Foundry로 컴파일한 ABI
├── migrations/
│   └── 001_create_events.sql
└── src/
    ├── main.rs                 ← Axum 서버 설정
    ├── config.rs               ← 환경변수 설정
    ├── errors.rs               ← 에러 타입
    ├── models.rs               ← 데이터 모델
    ├── routes.rs               ← 라우트 정의
    └── services/
        ├── mod.rs
        ├── trace.rs            ← 비즈니스 로직
        └── blockchain.rs       ← Alloy 연동
```

## Solidity 컨트랙트

### contracts/src/TraceRecord.sol

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title TraceRecord
/// @notice 식품 공급망 이벤트의 데이터 해시를 불변 기록으로 저장
/// @dev platform의 TraceRecord.sol을 단순화한 버전
contract TraceRecord {
    // 레코드 구조체
    struct Record {
        bytes32 dataHash;      // 이벤트 데이터의 keccak256 해시
        uint256 timestamp;     // 기록된 블록 타임스탬프
        address recorder;      // 기록한 주소
        bool exists;           // 레코드 존재 여부 (초기값 false)
    }
    
    // 컨트랙트 소유자
    address public owner;
    
    // 이벤트 ID → 레코드 매핑
    mapping(string => Record) private records;
    
    // 모든 이벤트 ID 목록
    string[] public eventIds;
    
    // 이벤트
    event HashRecorded(
        string indexed eventId,
        bytes32 dataHash,
        address indexed recorder,
        uint256 timestamp
    );
    
    event HashUpdated(
        string indexed eventId,
        bytes32 oldHash,
        bytes32 newHash,
        uint256 timestamp
    );
    
    // 에러
    error NotOwner(address caller);
    error RecordNotFound(string eventId);
    error EmptyEventId();
    error EmptyHash();
    
    // 수정자
    modifier onlyOwner() {
        if (msg.sender != owner) revert NotOwner(msg.sender);
        _;
    }
    
    constructor() {
        owner = msg.sender;
    }
    
    /// @notice 이벤트 해시를 블록체인에 기록
    /// @param eventId 이벤트 고유 ID (UUID 문자열)
    /// @param dataHash 이벤트 데이터의 keccak256 해시
    function recordHash(
        string calldata eventId,
        bytes32 dataHash
    ) external onlyOwner {
        if (bytes(eventId).length == 0) revert EmptyEventId();
        if (dataHash == bytes32(0)) revert EmptyHash();
        
        bool isNew = !records[eventId].exists;
        bytes32 oldHash = records[eventId].dataHash;
        
        records[eventId] = Record({
            dataHash: dataHash,
            timestamp: block.timestamp,
            recorder: msg.sender,
            exists: true
        });
        
        if (isNew) {
            eventIds.push(eventId);
            emit HashRecorded(eventId, dataHash, msg.sender, block.timestamp);
        } else {
            emit HashUpdated(eventId, oldHash, dataHash, block.timestamp);
        }
    }
    
    /// @notice 이벤트 해시 조회
    /// @param eventId 이벤트 고유 ID
    /// @return 레코드 구조체 (exists=false면 기록 없음)
    function getRecord(string calldata eventId)
        external
        view
        returns (Record memory)
    {
        return records[eventId];
    }
    
    /// @notice 제공된 해시가 기록된 해시와 일치하는지 검증
    /// @param eventId 이벤트 고유 ID
    /// @param dataHash 검증할 해시
    /// @return true면 일치 (무결성 확인), false면 불일치 또는 미기록
    function verifyHash(
        string calldata eventId,
        bytes32 dataHash
    ) external view returns (bool) {
        Record memory record = records[eventId];
        if (!record.exists) return false;
        return record.dataHash == dataHash;
    }
    
    /// @notice 전체 이벤트 수 반환
    function getEventCount() external view returns (uint256) {
        return eventIds.length;
    }
    
    /// @notice 소유권 이전
    function transferOwnership(address newOwner) external onlyOwner {
        owner = newOwner;
    }
}
```

### 컨트랙트 컴파일 (Foundry)

```bash
# Foundry 설치 (이미 설치되어 있다면 건너뜀)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# 컨트랙트 컴파일
cd mini-trace/contracts
forge build

# ABI 파일 추출
cp out/TraceRecord.sol/TraceRecord.json ../abi/

# 로컬 Anvil에 배포
anvil &  # 별도 터미널에서

forge create \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  src/TraceRecord.sol:TraceRecord
```

## Cargo.toml

```toml
[package]
name = "mini-trace"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "mini-trace"
path = "src/main.rs"

[dependencies]
# 웹 프레임워크
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# 블록체인
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
] }

# 데이터베이스
sqlx = { version = "0.8", features = [
    "sqlite",
    "runtime-tokio-native-tls",
    "uuid",
    "chrono",
] }

# 직렬화
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 에러 처리
anyhow = "1"
thiserror = "2"

# 로깅
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# 유틸리티
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
hex = "0.4"
sha3 = "0.10"  # keccak256 for data hashing
dotenvy = "0.15"
```

## 데이터베이스 마이그레이션

### migrations/001_create_events.sql

```sql
CREATE TABLE IF NOT EXISTS trace_events (
    id          TEXT PRIMARY KEY,           -- UUID
    event_type  TEXT NOT NULL,              -- "harvest", "transport", "inspection" 등
    payload     TEXT NOT NULL,              -- JSON 데이터
    data_hash   TEXT NOT NULL,              -- keccak256 해시 (hex 문자열)
    tx_hash     TEXT,                       -- 블록체인 TX 해시 (기록 후 채움)
    block_number INTEGER,                   -- 기록된 블록 번호
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    recorded_at TEXT                        -- 체인에 기록된 시각
);

CREATE INDEX IF NOT EXISTS idx_trace_events_event_type ON trace_events(event_type);
CREATE INDEX IF NOT EXISTS idx_trace_events_created_at ON trace_events(created_at);
```

## Rust 코드

### src/config.rs

```rust,ignore
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub rpc_url: String,
    pub private_key: String,
    pub contract_address: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        Ok(Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:mini-trace.db".to_string()),
            rpc_url: std::env::var("RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8545".to_string()),
            private_key: std::env::var("PRIVATE_KEY")
                .unwrap_or_else(|_| {
                    // Anvil 기본 개인키 (절대 프로덕션에 사용 금지)
                    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                        .to_string()
                }),
            contract_address: std::env::var("CONTRACT_ADDRESS")
                .unwrap_or_else(|_| "0x5FbDB2315678afecb367f032d93F642f64180aa3".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
        })
    }
}
```

### src/errors.rs

```rust,ignore
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("이벤트를 찾을 수 없음: {0}")]
    NotFound(String),
    
    #[error("데이터베이스 오류: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("블록체인 오류: {0}")]
    Blockchain(String),
    
    #[error("입력 오류: {0}")]
    BadRequest(String),
    
    #[error("내부 서버 오류: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Database(e) => {
                tracing::error!("DB 오류: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "데이터베이스 오류".to_string())
            }
            AppError::Blockchain(msg) => {
                tracing::warn!("블록체인 오류: {}", msg);
                (StatusCode::SERVICE_UNAVAILABLE, msg.clone())
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(e) => {
                tracing::error!("내부 오류: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "내부 서버 오류".to_string())
            }
        };
        
        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));
        
        (status, body).into_response()
    }
}
```

### src/models.rs

```rust,ignore
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DB에 저장되는 이벤트
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TraceEvent {
    pub id: String,
    pub event_type: String,
    pub payload: String,          // JSON 문자열
    pub data_hash: String,        // keccak256 hex
    pub tx_hash: Option<String>,  // 블록체인 TX 해시
    pub block_number: Option<i64>,
    pub created_at: String,
    pub recorded_at: Option<String>,
}

/// 이벤트 생성 요청
#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// 이벤트 응답 (온체인 검증 결과 포함)
#[derive(Debug, Serialize)]
pub struct TraceEventResponse {
    pub id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub data_hash: String,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub created_at: String,
    pub blockchain_status: BlockchainStatus,
}

/// 온체인 검증 상태
#[derive(Debug, Serialize)]
pub struct BlockchainStatus {
    pub recorded: bool,           // 체인에 기록됐는가
    pub hash_matches: Option<bool>, // 해시가 일치하는가
    pub on_chain_timestamp: Option<u64>,
    pub recorder_address: Option<String>,
}

/// 검증 응답
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub event_id: String,
    pub db_hash: String,
    pub on_chain_hash: Option<String>,
    pub is_valid: bool,
    pub message: String,
}
```

### src/services/blockchain.rs

```rust,ignore
use alloy::{
    network::EthereumWallet,
    primitives::{Address, FixedBytes},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
};
use anyhow::Result;
use std::sync::Arc;

// TraceRecord 컨트랙트 ABI 정의
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
        function getEventCount() external view returns (uint256);
        
        event HashRecorded(
            string indexed eventId,
            bytes32 dataHash,
            address indexed recorder,
            uint256 timestamp
        );
        
        error NotOwner(address caller);
        error RecordNotFound(string eventId);
    }
}

pub struct OnChainRecord {
    pub data_hash: [u8; 32],
    pub timestamp: u64,
    pub recorder: String,
    pub exists: bool,
}

pub struct BlockchainService {
    contract_address: Address,
    rpc_url: String,
    private_key: String,
}

impl BlockchainService {
    pub fn new(rpc_url: String, private_key: String, contract_address: String) -> Result<Self> {
        let address: Address = contract_address.parse()
            .map_err(|e| anyhow::anyhow!("컨트랙트 주소 파싱 실패: {}", e))?;
        
        Ok(Self {
            contract_address: address,
            rpc_url,
            private_key,
        })
    }
    
    // 읽기 전용 Provider 생성
    fn read_provider(&self) -> Result<impl Provider> {
        let provider = ProviderBuilder::new()
            .on_http(self.rpc_url.parse()
                .map_err(|e| anyhow::anyhow!("RPC URL 파싱 실패: {}", e))?);
        Ok(provider)
    }
    
    /// 해시를 블록체인에 기록
    pub async fn record_hash(
        &self,
        event_id: &str,
        data_hash: [u8; 32],
    ) -> Result<String> {
        let signer: PrivateKeySigner = self.private_key.parse()
            .map_err(|e| anyhow::anyhow!("개인키 파싱 실패: {}", e))?;
        
        let wallet = EthereumWallet::from(signer);
        
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(self.rpc_url.parse()
                .map_err(|e| anyhow::anyhow!("RPC URL 파싱 실패: {}", e))?);
        
        let contract = TraceRecord::new(self.contract_address, &provider);
        
        let hash_bytes: FixedBytes<32> = FixedBytes::from(data_hash);
        
        tracing::info!("블록체인에 해시 기록: event_id={}", event_id);
        
        let pending = contract
            .recordHash(event_id.to_string(), hash_bytes)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("트랜잭션 전송 실패: {}", e))?;
        
        let tx_hash = format!("{:?}", pending.tx_hash());
        
        let receipt = pending.get_receipt().await
            .map_err(|e| anyhow::anyhow!("영수증 대기 실패: {}", e))?;
        
        if !receipt.status() {
            return Err(anyhow::anyhow!("컨트랙트 호출이 revert됨"));
        }
        
        tracing::info!("해시 기록 완료: tx_hash={}", tx_hash);
        
        Ok(tx_hash)
    }
    
    /// 온체인 레코드 조회
    pub async fn get_record(&self, event_id: &str) -> Result<OnChainRecord> {
        let provider = self.read_provider()?;
        let contract = TraceRecord::new(self.contract_address, &provider);
        
        let result = contract.getRecord(event_id.to_string()).call().await
            .map_err(|e| anyhow::anyhow!("getRecord 호출 실패: {}", e))?;
        
        let record = result._0;
        
        Ok(OnChainRecord {
            data_hash: *record.dataHash,
            timestamp: record.timestamp.to::<u64>(),
            recorder: format!("{}", record.recorder),
            exists: record.exists,
        })
    }
    
    /// 해시 검증
    pub async fn verify_hash(
        &self,
        event_id: &str,
        data_hash: [u8; 32],
    ) -> Result<bool> {
        let provider = self.read_provider()?;
        let contract = TraceRecord::new(self.contract_address, &provider);
        
        let hash_bytes: FixedBytes<32> = FixedBytes::from(data_hash);
        
        let result = contract
            .verifyHash(event_id.to_string(), hash_bytes)
            .call()
            .await
            .map_err(|e| anyhow::anyhow!("verifyHash 호출 실패: {}", e))?;
        
        Ok(result._0)
    }
}
```

### src/services/trace.rs

```rust,ignore
use crate::{
    errors::AppError,
    models::{
        BlockchainStatus, CreateEventRequest, TraceEvent, TraceEventResponse, VerifyResponse,
    },
    services::blockchain::BlockchainService,
};
use anyhow::Result;
use sha3::{Digest, Keccak256};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

pub struct TraceService {
    pub db: SqlitePool,
    pub blockchain: Arc<BlockchainService>,
}

impl TraceService {
    pub fn new(db: SqlitePool, blockchain: Arc<BlockchainService>) -> Self {
        Self { db, blockchain }
    }
    
    /// 이벤트 생성: DB 저장 → 해시 계산 → 체인 기록
    pub async fn create_event(
        &self,
        req: CreateEventRequest,
    ) -> Result<TraceEventResponse, AppError> {
        if req.event_type.is_empty() {
            return Err(AppError::BadRequest("event_type이 비어있습니다".to_string()));
        }
        
        let event_id = Uuid::new_v4().to_string();
        let payload_str = serde_json::to_string(&req.payload)
            .map_err(|e| AppError::BadRequest(format!("payload 직렬화 실패: {}", e)))?;
        
        // 해시 계산: keccak256(event_id + event_type + payload)
        let hash_input = format!("{}{}{}", event_id, req.event_type, payload_str);
        let mut hasher = Keccak256::new();
        hasher.update(hash_input.as_bytes());
        let hash_bytes: [u8; 32] = hasher.finalize().into();
        let data_hash_hex = hex::encode(hash_bytes);
        
        // DB에 저장
        sqlx::query!(
            r#"
            INSERT INTO trace_events (id, event_type, payload, data_hash, created_at)
            VALUES (?1, ?2, ?3, ?4, datetime('now'))
            "#,
            event_id,
            req.event_type,
            payload_str,
            data_hash_hex,
        )
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;
        
        tracing::info!("이벤트 DB 저장 완료: id={}", event_id);
        
        // 블록체인에 해시 기록 (실패해도 API는 성공)
        let mut tx_hash = None;
        let mut block_number = None;
        
        match self.blockchain.record_hash(&event_id, hash_bytes).await {
            Ok(hash) => {
                tracing::info!("블록체인 기록 성공: tx_hash={}", hash);
                tx_hash = Some(hash.clone());
                
                // TX 해시 DB에 업데이트
                sqlx::query!(
                    "UPDATE trace_events SET tx_hash = ?1, recorded_at = datetime('now') WHERE id = ?2",
                    hash,
                    event_id,
                )
                .execute(&self.db)
                .await
                .ok(); // 실패해도 무시
            }
            Err(e) => {
                tracing::warn!("블록체인 기록 실패 (나중에 재시도): {}", e);
                // 운영 버전에서는 outbox 테이블이나 메시지 큐에
                // event_id와 data_hash를 저장해 백그라운드 워커가 재시도한다.
            }
        }
        
        Ok(TraceEventResponse {
            id: event_id,
            event_type: req.event_type,
            payload: req.payload,
            data_hash: data_hash_hex,
            tx_hash,
            block_number,
            created_at: chrono::Utc::now().to_rfc3339(),
            blockchain_status: BlockchainStatus {
                recorded: tx_hash.is_some(),
                hash_matches: None,
                on_chain_timestamp: None,
                recorder_address: None,
            },
        })
    }
    
    /// 이벤트 조회 + 온체인 검증
    pub async fn get_event(
        &self,
        event_id: &str,
    ) -> Result<TraceEventResponse, AppError> {
        // DB에서 이벤트 조회
        let event = sqlx::query_as!(
            TraceEvent,
            "SELECT * FROM trace_events WHERE id = ?1",
            event_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("이벤트 없음: {}", event_id)))?;
        
        // 온체인 상태 확인
        let blockchain_status = match self.blockchain.get_record(event_id).await {
            Ok(record) if record.exists => {
                let db_hash_bytes = hex::decode(&event.data_hash)
                    .unwrap_or_default();
                let hash_matches = db_hash_bytes == record.data_hash;
                
                BlockchainStatus {
                    recorded: true,
                    hash_matches: Some(hash_matches),
                    on_chain_timestamp: Some(record.timestamp),
                    recorder_address: Some(record.recorder),
                }
            }
            Ok(_) => BlockchainStatus {
                recorded: false,
                hash_matches: None,
                on_chain_timestamp: None,
                recorder_address: None,
            },
            Err(e) => {
                tracing::warn!("온체인 조회 실패: {}", e);
                BlockchainStatus {
                    recorded: event.tx_hash.is_some(),
                    hash_matches: None,
                    on_chain_timestamp: None,
                    recorder_address: None,
                }
            }
        };
        
        let payload: serde_json::Value = serde_json::from_str(&event.payload)
            .unwrap_or(serde_json::Value::String(event.payload.clone()));
        
        Ok(TraceEventResponse {
            id: event.id,
            event_type: event.event_type,
            payload,
            data_hash: event.data_hash,
            tx_hash: event.tx_hash,
            block_number: event.block_number,
            created_at: event.created_at,
            blockchain_status,
        })
    }
    
    /// 무결성 검증: DB 해시 vs 온체인 해시
    pub async fn verify_event(
        &self,
        event_id: &str,
    ) -> Result<VerifyResponse, AppError> {
        // DB에서 이벤트 조회
        let event = sqlx::query_as!(
            TraceEvent,
            "SELECT * FROM trace_events WHERE id = ?1",
            event_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("이벤트 없음: {}", event_id)))?;
        
        let db_hash = event.data_hash.clone();
        
        // 온체인 해시 조회
        match self.blockchain.get_record(event_id).await {
            Ok(record) if record.exists => {
                let on_chain_hash = hex::encode(record.data_hash);
                let is_valid = db_hash == on_chain_hash;
                
                let message = if is_valid {
                    "데이터 무결성 확인: DB와 체인의 해시가 일치합니다".to_string()
                } else {
                    "경고: DB와 체인의 해시가 불일치합니다. 데이터가 변조되었을 수 있습니다!".to_string()
                };
                
                tracing::info!(
                    event_id = event_id,
                    is_valid = is_valid,
                    "무결성 검증 완료"
                );
                
                Ok(VerifyResponse {
                    event_id: event_id.to_string(),
                    db_hash,
                    on_chain_hash: Some(on_chain_hash),
                    is_valid,
                    message,
                })
            }
            Ok(_) => {
                Ok(VerifyResponse {
                    event_id: event_id.to_string(),
                    db_hash,
                    on_chain_hash: None,
                    is_valid: false,
                    message: "블록체인에 아직 기록되지 않았습니다".to_string(),
                })
            }
            Err(e) => {
                Err(AppError::Blockchain(format!("온체인 조회 실패: {}", e)))
            }
        }
    }
}
```

### src/routes.rs

```rust,ignore
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{
    errors::AppError,
    models::CreateEventRequest,
    services::trace::TraceService,
};

pub type AppState = Arc<TraceService>;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/events", post(create_event))
        .route("/events/:id", get(get_event))
        .route("/events/:id/verify", get(verify_event))
        .with_state(state)
}

/// GET /health
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "mini-trace"
    }))
}

/// POST /events
/// Body: { "event_type": "harvest", "payload": { ... } }
async fn create_event(
    State(service): State<AppState>,
    Json(req): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let event = service.create_event(req).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "data": event
        })),
    ))
}

/// GET /events/:id
async fn get_event(
    State(service): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let event = service.get_event(&id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": event
    })))
}

/// GET /events/:id/verify
async fn verify_event(
    State(service): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = service.verify_event(&id).await?;
    
    let status = if result.is_valid {
        StatusCode::OK
    } else {
        StatusCode::OK // 검증 실패도 200, is_valid로 구분
    };
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": result
    })))
}
```

### src/main.rs

```rust,ignore
mod config;
mod errors;
mod models;
mod routes;
mod services;

use std::sync::Arc;

use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    routes::create_router,
    services::{blockchain::BlockchainService, trace::TraceService},
};

#[tokio::main]
async fn main() -> Result<()> {
    // 로깅 초기화
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mini_trace=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("mini-trace 서버 시작 중...");
    
    // 설정 로드
    let config = Config::from_env()?;
    
    // 데이터베이스 연결
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    
    // 마이그레이션 실행
    sqlx::migrate!("./migrations")
        .run(&db)
        .await?;
    
    tracing::info!("데이터베이스 연결 완료");
    
    // 블록체인 서비스 초기화
    let blockchain = Arc::new(
        BlockchainService::new(
            config.rpc_url.clone(),
            config.private_key.clone(),
            config.contract_address.clone(),
        )?
    );
    
    tracing::info!("블록체인 서비스 초기화 완료: contract={}", config.contract_address);
    
    // 트레이스 서비스
    let trace_service = Arc::new(TraceService::new(db, blockchain));
    
    // Axum 라우터
    let app = create_router(trace_service)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());
    
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("서버 시작: http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### src/services/mod.rs

```rust,ignore
pub mod blockchain;
pub mod trace;
```

## .env 파일

```env
DATABASE_URL=sqlite:mini-trace.db
RPC_URL=http://localhost:8545
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
CONTRACT_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3
PORT=3000
RUST_LOG=mini_trace=info,tower_http=debug
```

## 단계별 실행 가이드

### 1단계: 환경 준비

```bash
# Foundry 설치
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Rust 설정 확인
rustc --version  # 1.75+
cargo --version

# SQLx CLI 설치 (마이그레이션용)
cargo install sqlx-cli --features sqlite
```

### 2단계: 프로젝트 생성

```bash
cargo new mini-trace
cd mini-trace
mkdir -p contracts/src abi migrations src/services
```

위의 모든 파일을 해당 위치에 작성한다.

### 3단계: 블록체인 환경 시작

```bash
# 터미널 1: Anvil 시작 (로컬 이더리움 노드)
anvil

# 터미널 2: 컨트랙트 배포
cd mini-trace/contracts
forge init --no-git  # contracts 폴더 초기화

# TraceRecord.sol을 src/에 작성 후
forge build
forge create \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  src/TraceRecord.sol:TraceRecord

# 배포된 컨트랙트 주소를 .env의 CONTRACT_ADDRESS에 설정
```

### 4단계: 데이터베이스 초기화

```bash
cd mini-trace
sqlx database create
sqlx migrate run
```

### 5단계: 서버 실행

```bash
cargo run
# 또는 개발 중 자동 재시작
cargo install cargo-watch
cargo watch -x run
```

### 6단계: API 테스트

```bash
# 1. 헬스체크
curl http://localhost:3000/health

# 2. 이벤트 생성
curl -X POST http://localhost:3000/events \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "harvest",
    "payload": {
      "crop": "사과",
      "quantity_kg": 500,
      "farm_id": "farm-001",
      "location": "경북 안동",
      "quality_grade": "A"
    }
  }'

# 응답 예시:
# {
#   "success": true,
#   "data": {
#     "id": "550e8400-e29b-41d4-a716-446655440000",
#     "event_type": "harvest",
#     "payload": { ... },
#     "data_hash": "0x3f4a...",
#     "tx_hash": "0x8b2c...",
#     "blockchain_status": {
#       "recorded": true,
#       "hash_matches": null,
#       ...
#     }
#   }
# }

# 3. 이벤트 조회 (EVENT_ID는 위 응답의 id)
EVENT_ID="550e8400-e29b-41d4-a716-446655440000"
curl http://localhost:3000/events/$EVENT_ID

# 4. 무결성 검증
curl http://localhost:3000/events/$EVENT_ID/verify

# 응답 예시:
# {
#   "success": true,
#   "data": {
#     "event_id": "550e8400...",
#     "db_hash": "3f4a...",
#     "on_chain_hash": "3f4a...",
#     "is_valid": true,
#     "message": "데이터 무결성 확인: DB와 체인의 해시가 일치합니다"
#   }
# }
```

### 7단계: 데이터 변조 시뮬레이션

```bash
# DB에서 직접 데이터 수정 (변조 시뮬레이션)
sqlite3 mini-trace.db \
  "UPDATE trace_events SET payload = '{\"crop\":\"배추\",\"quantity_kg\":999}' WHERE id='$EVENT_ID'"

# 검증 - 해시 불일치 탐지
curl http://localhost:3000/events/$EVENT_ID/verify
# {
#   "is_valid": false,
#   "message": "경고: DB와 체인의 해시가 불일치합니다. 데이터가 변조되었을 수 있습니다!"
# }
```

이것이 블록체인의 가치다. DB 데이터가 변조되었지만 체인의 해시가 이를 탐지한다.

## 확장 아이디어

이 미니프로젝트를 완성하면 다음 기능을 추가해볼 수 있다:

1. **재시도 큐**: 블록체인 기록 실패 시 Redis/DB 큐로 재시도
2. **이벤트 목록**: `GET /events?type=harvest&page=1` 페이지네이션
3. **배치 기록**: 여러 이벤트를 한 트랜잭션에 기록
4. **WebSocket**: 새 이벤트를 실시간으로 클라이언트에 푸시
5. **인증**: JWT 미들웨어로 API 보호
6. **PostgreSQL 전환**: SQLite에서 PostgreSQL로 (platform 수준)

## 요약

이 장에서 구현한 것:
- Solidity 컨트랙트: 해시 기록/조회/검증
- Axum + SQLx: REST API와 SQLite 연동
- Alloy: 컨트랙트 쓰기/읽기
- 하이브리드 패턴: 데이터는 DB, 해시는 체인
- 무결성 검증: DB vs 체인 해시 비교
- 에러 처리: 블록체인 실패가 API를 막지 않음

platform의 핵심 패턴을 이해했다면 다음 장(22장)에서 실제 platform 코드를 읽어본다.
