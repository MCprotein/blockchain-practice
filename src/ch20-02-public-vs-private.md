# 20-2: 퍼블릭 vs 프라이빗 - 심층 비교와 설계 결정

## 10가지 기준으로 상세 비교

| # | 기준 | 퍼블릭 체인 (이더리움) | 프라이빗 체인 (Besu IBFT) | 비고 |
|---|------|----------------------|--------------------------|------|
| 1 | **접근 권한** | 누구나 읽기/쓰기 가능 | 허가된 주소/노드만 | Besu 계정 allowlist |
| 2 | **합의 방식** | PoS (검증자 ~100만 명) | IBFT 2.0 (검증자 4-21개) | BFT vs 경제적 보안 |
| 3 | **처리 속도** | 15-30 TPS, 12초/블록 | 수백~수천 TPS, 1-2초/블록 | 검증자가 적어 빠름 |
| 4 | **트랜잭션 비용** | ETH 가스비 (수천~수만 원) | 0 (min-gas-price=0) | 기업 운영 비용 절감 |
| 5 | **데이터 프라이버시** | 완전 공개 (누구나 조회) | 제한적 (허가된 노드만) | Tessera로 더 강화 가능 |
| 6 | **탈중앙화 정도** | 매우 높음 | 낮음 (운영사 통제) | 신뢰 모델이 다름 |
| 7 | **검열 저항성** | 매우 높음 | 없음 (운영사가 제어) | 트레이드오프 |
| 8 | **확정성** | ~13분 (2 에포크) | 즉시 (IBFT 특성) | 비즈니스 UX에 중요 |
| 9 | **스마트 컨트랙트** | Solidity, Vyper | Solidity (EVM 호환) | 코드 재사용 가능 |
| 10 | **운영 복잡도** | 노드 직접 운영 불필요 | 자체 노드 운영 필요 | DevOps 부담 증가 |

### 각 기준의 의미

**접근 권한**: 퍼블릭 체인에서는 개인키만 있으면 누구나 트랜잭션을 보낼 수 있다. 프라이빗 체인에서는 운영사가 승인한 주소만 가능하다. platform에서는 iksan-api 서비스 계정만 TraceRecord 컨트랙트에 데이터를 쓸 수 있다.

**합의 방식**: 이더리움 PoS는 경제적 인센티브(슬래싱)로 검증자를 정직하게 만든다. IBFT는 신원이 알려진 소수의 검증자가 BFT 프로토콜로 합의한다. 후자는 법적 책임이 있는 기업 환경에서 충분히 안전하다.

**확정성**: 이더리움 메인넷에서 트랜잭션이 "안전"하려면 12번 이상 블록이 쌓이기를 기다리는 게 일반적이다 (약 2.5분). IBFT에서는 영수증이 오면 즉시 확정이다. 사용자 경험 측면에서 큰 차이다.

## "PostgreSQL에 저장하면 되는데 왜 블록체인?"

이 질문은 블록체인 프로젝트에서 가장 자주 듣는 비판이다. 솔직하게 답해보자.

### 데이터베이스의 한계

```
일반 PostgreSQL:
  - 데이터 저장: O
  - 빠른 조회: O
  - ACID 트랜잭션: O
  - 관리자가 데이터 수정 가능: O ← 이것이 문제
  - 감사 로그 위변조 가능: O ← 이것이 문제
  - 독립적 제3자 검증: X
```

데이터베이스 관리자(DBA)는 데이터를 수정할 수 있다. 백업을 교체할 수 있다. 로그를 삭제할 수 있다. 이것이 의도적인 설계이지만, 신뢰 문제를 만든다.

실제 사례: 식품 안전 사고가 발생했을 때, 회사 측이 내부 데이터를 수정했다는 의혹이 제기되었다. PostgreSQL 기반이라면 이 의혹을 완전히 반박하기 어렵다.

### 블록체인의 추가 가치

```
블록체인 (Besu):
  - 데이터 저장: O (하지만 느리고 비쌈 → 해시만 저장)
  - 빠른 조회: X (느림 → DB 병행)
  - 불변성: O (블록 추가 후 변경 불가)
  - 독립 검증: O (감사자가 자체 노드로 확인 가능)
  - 시간 증명: O (타임스탬프 조작 불가)
  - 참여자 합의 감사: O (누가, 언제 기록했는지 추적 가능)
```

블록체인은 "누가 이 데이터를 언제 기록했는가"를 독립적으로 검증 가능하게 만든다.

### platform의 선택: 데이터는 DB에, 무결성 증명은 체인에

platform은 두 가지를 조합한다:

```
PostgreSQL (빠른 읽기/쓰기):
  - 실제 이벤트 데이터 (농산물 수확 정보, 유통 경로 등)
  - 전체 이력 조회
  - 복잡한 쿼리 (JOIN, 필터, 정렬)

Besu 블록체인 (불변 증명):
  - 각 이벤트의 keccak256 해시
  - 타임스탬프
  - 기록자 주소
  - 트랜잭션 해시
```

나중에 누군가가 "이 데이터가 조작되지 않았음을 증명하라"고 하면:
1. DB에서 원본 데이터를 가져옴
2. 같은 방법으로 해시를 계산
3. 체인에서 해당 해시를 조회
4. 일치하면 → 데이터가 기록 시점 이후 변조되지 않았음을 증명

이것이 핵심 가치다. 데이터 자체를 체인에 올리는 것이 아니라, 데이터의 지문(해시)을 올리는 것이다.

## 하이브리드 아키텍처 패턴

### 패턴 1: 오프체인 데이터 + 온체인 해시

platform이 사용하는 패턴이다.

```
┌─────────────────────────────────────────────────────────┐
│                    iksan-api 서비스                       │
│                                                         │
│  1. 이벤트 생성                                          │
│     event = { id, type, data, timestamp }              │
│                                                         │
│  2. 해시 계산                                            │
│     hash = keccak256(json(event))                      │
│                                                         │
│  3. DB 저장                                             │
│     INSERT INTO trace_events (id, data, hash, ...)     │
│                                                         │
│  4. 블록체인 기록                                        │
│     TraceRecord.recordHash(event_id, hash)             │
│                                                         │
│  5. TX 해시 DB에 저장                                    │
│     UPDATE trace_events SET tx_hash = ...              │
└─────────────────┬───────────────────────────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
   ┌────▼────┐         ┌────▼────┐
   │PostgreSQL│         │  Besu   │
   │         │         │         │
   │이벤트 데이터│       │  해시값  │
   │(수백 KB) │         │(32바이트)│
   └─────────┘         └─────────┘
```

**왜 모든 데이터를 체인에 올리지 않는가?**
- 체인 저장 비용: 1바이트 = 680 gas (이더리움 메인넷 기준 매우 비쌈)
- 프라이빗 체인이라도 블록 크기 제한 존재
- 조회 속도: 체인 조회보다 DB 조회가 100배 이상 빠름
- 프라이버시: 민감 데이터는 DB 접근 제어로 보호

32바이트 해시는 사실상 비용이 없으면서, 원본 데이터의 무결성을 보증한다.

### 패턴 2: IPFS + 블록체인

대용량 파일(문서, 이미지)의 경우:

```
파일 → IPFS 업로드 → CID(해시) 획득 → CID를 체인에 기록
```

IPFS CID 자체가 내용의 해시이므로, CID만 체인에 기록해도 파일 변조를 탐지할 수 있다.

platform은 현재 이 패턴을 사용하지 않지만, 인증서 파일이나 검사 보고서를 저장할 때 확장 가능하다.

### 패턴 3: 이벤트 소싱 + 블록체인

```
모든 상태 변경을 이벤트로 기록 → 이벤트를 블록체인에 저장
현재 상태 = 이벤트들의 순차 적용
```

이 패턴은 구현이 복잡하지만, 완전한 감사 추적이 가능하다. Hyperledger Fabric 같은 플랫폼에서 주로 사용한다.

## platform의 정확한 패턴

```rust
// services/trace.rs의 핵심 로직 (의사코드)

pub async fn create_trace_event(
    db: &PgPool,
    blockchain: &BlockchainService,
    input: CreateEventInput,
) -> Result<TraceEvent> {
    // 1. 이벤트 데이터 구성
    let event = TraceEvent {
        id: Uuid::new_v4(),
        event_type: input.event_type,
        payload: input.payload,
        created_at: Utc::now(),
        ..Default::default()
    };
    
    // 2. 해시 계산 (keccak256)
    let event_json = serde_json::to_string(&event)?;
    let hash = keccak256(event_json.as_bytes());
    
    // 3. DB에 저장 (빠른 쓰기)
    let saved = sqlx::query_as!(
        TraceEvent,
        "INSERT INTO trace_events (id, event_type, payload, data_hash, created_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *",
        event.id,
        event.event_type,
        event.payload,
        hash.as_slice(),
        event.created_at,
    )
    .fetch_one(db)
    .await?;
    
    // 4. 블록체인에 해시 기록 (비동기 - 실패해도 재시도)
    match blockchain.record_hash(&event.id.to_string(), hash).await {
        Ok(tx_hash) => {
            // 5. TX 해시 DB에 저장
            sqlx::query!(
                "UPDATE trace_events SET tx_hash = $1 WHERE id = $2",
                tx_hash.as_slice(),
                event.id
            )
            .execute(db)
            .await?;
        }
        Err(e) => {
            // 블록체인 기록 실패는 별도 큐에서 재시도
            tracing::error!("블록체인 기록 실패: {}", e);
            // 이벤트 자체는 이미 DB에 저장됨 (가용성 우선)
        }
    }
    
    Ok(saved)
}
```

중요한 설계 결정: **블록체인 실패가 이벤트 생성을 막지 않는다**. 가용성 우선이다. 실패한 블록체인 기록은 나중에 재시도한다.

## 언제 블록체인을 쓰는가 / 쓰지 않는가

### 의사결정 프레임워크

블록체인이 적합한 경우:

```
✅ 블록체인 사용 권장:
  1. 여러 신뢰하지 않는 당사자 간 데이터 공유
     예: 경쟁 관계에 있는 공급업체들이 재고 데이터 공유
  
  2. 독립적 감사가 필요한 감사 추적
     예: 식품 안전 이력 추적 (platform의 케이스)
  
  3. 중개자 제거로 비용 절감
     예: 무역 금융 (LC 처리)
  
  4. 토큰화가 핵심인 경우
     예: RWA(실물자산 토큰화), NFT
  
  5. 탈중앙화 자율 조직(DAO) 거버넌스
```

```
❌ 블록체인 불필요:
  1. 단일 조직 내부 데이터 관리
     → 그냥 PostgreSQL + 감사 로그 테이블
  
  2. 빠른 읽기/쓰기가 필요한 트랜잭션 데이터
     → Redis, PostgreSQL
  
  3. 대용량 파일 저장
     → S3, GCS (블록체인은 저장 비용이 극도로 높음)
  
  4. 개인정보를 포함한 데이터 (GDPR 삭제권)
     → 체인에 올리면 삭제 불가
  
  5. 빠른 프로토타이핑/MVP
     → 블록체인 통합은 복잡도를 크게 높임
```

### 3가지 핵심 질문

```
1. "제3자가 이 데이터를 독립적으로 검증해야 하는가?"
   NO → 블록체인 불필요
   YES → 다음 질문으로

2. "여러 신뢰하지 않는 당사자가 같은 데이터를 공유하는가?"
   NO → 감사 로그 DB로 충분
   YES → 다음 질문으로

3. "데이터의 불변성이 비즈니스 핵심 가치인가?"
   NO → 블록체인 과잉설계
   YES → 블록체인 사용 검토
```

platform은 이 세 질문 모두에 YES다:
- 식품 안전 감사 기관이 독립적으로 이력을 검증해야 함
- 농업인, 유통업자, 소매업자가 같은 이력을 공유
- "이 데이터가 변조되지 않았음"이 핵심 서비스 가치

### 흔한 오용 사례

```
오용 1: "내부 ERP를 블록체인으로 만들자"
  → 단일 회사 데이터, 수정 가능해야 함, 빠른 응답 필요
  → 그냥 ERP 사용

오용 2: "공급망 데이터를 모두 이더리움에 올리자"
  → 가스비 감당 불가, 민감 데이터 공개, 느림
  → 해시만 체인에, 데이터는 DB에

오용 3: "투표 시스템을 퍼블릭 체인으로"
  → 개인 투표 내용 공개 (익명성 훼손)
  → ZK-proof나 프라이빗 체인 고려

오용 4: "실시간 게임 상태를 블록체인에"
  → 블록체인은 초당 수십~수백 트랜잭션만 처리
  → 결과물만 체인에, 게임 로직은 서버에
```

## 실제 비교: 같은 기능을 두 방식으로 구현

### 방식 A: PostgreSQL만 사용

```sql
-- 감사 테이블
CREATE TABLE trace_events (
    id UUID PRIMARY KEY,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    created_by VARCHAR(42) NOT NULL
);

-- 변경 이력
CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    table_name VARCHAR(50),
    row_id UUID,
    operation VARCHAR(10),
    old_data JSONB,
    new_data JSONB,
    changed_at TIMESTAMPTZ DEFAULT NOW(),
    changed_by VARCHAR(100)
);
```

문제: DBA가 `audit_log`를 삭제할 수 있다. 회사 내부자가 데이터와 로그를 모두 조작할 수 있다.

### 방식 B: DB + 블록체인 해시 (platform 방식)

```sql
CREATE TABLE trace_events (
    id UUID PRIMARY KEY,
    data JSONB NOT NULL,
    data_hash BYTEA NOT NULL,      -- keccak256(data)
    tx_hash BYTEA,                 -- 블록체인 TX 해시
    block_number BIGINT,           -- 기록된 블록 번호
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

```solidity
// 블록체인에는 해시만 저장
mapping(string => bytes32) public recordHashes;

function recordHash(string calldata id, bytes32 hash) external {
    recordHashes[id] = hash;
    emit HashRecorded(id, hash, block.timestamp);
}
```

이제 DBA가 DB의 `data`를 수정해도:
1. `data_hash`도 수정해야 하고
2. 블록체인의 해시도 바꿔야 하는데
3. 블록체인 데이터는 수정 불가

→ 조작이 즉시 탐지된다.

## 요약

핵심 인사이트:

1. **블록체인 = 신뢰 기계**: 신뢰가 필요한 곳에만 사용
2. **하이브리드가 정답**: 데이터는 DB, 증명은 체인
3. **해시 패턴**: 32바이트 해시로 무한 크기 데이터를 증명
4. **즉시 확정성**: IBFT로 사용자 경험 개선
5. **비용 0**: 프라이빗 체인에서 가스 제거

"블록체인이 필요한가?"보다 "누가 누구를 신뢰하는가?"를 먼저 물어보라. 신뢰 문제가 없으면 데이터베이스로 충분하다.

다음 장(21장)에서는 이 모든 개념을 합쳐 미니 트레이서빌리티 서비스를 직접 구축한다.
