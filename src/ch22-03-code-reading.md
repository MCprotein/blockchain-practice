# 22-3: Platform 코드 읽는 법과 인수인계 체크리스트

## 코드를 읽는 순서

처음 프로젝트를 받았을 때 막막하다면 다음 순서로 읽어라. 무작위로 파일을 열지 말고 계층을 따라 내려가는 것이 핵심이다.

### 1단계: 진입점 파악 (main.rs)

모든 Rust 서비스는 `main.rs`에서 시작한다. main.rs를 읽으면 서비스의 전체 구조가 보인다.

```
apps/iksan-api/src/main.rs
```

읽을 때 찾아야 할 것들:
- 어떤 환경변수를 읽는가? → 배포 설정 이해
- 어떤 외부 의존성을 초기화하는가? (DB, 블록체인, HTTP 클라이언트)
- 어떤 라우터를 마운트하는가? → API 엔드포인트 구조

```rust
// main.rs에서 확인할 패턴
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 로깅: 어떤 레벨과 필터를 사용하는가
    tracing_subscriber::...
    
    // 설정: 어떤 환경변수가 필요한가
    let config = Config::from_env()?;
    
    // AppState: 어떤 의존성을 갖는가
    let state = AppState::new(config).await?;
    
    // 라우터: 어떤 경로가 있는가
    let app = create_app(state);
    
    // 서버: 포트와 리슨 설정
    axum::serve(listener, app).await?;
}
```

### 2단계: AppState 이해 (core/app.rs)

```
apps/iksan-api/src/core/app.rs
```

AppState는 서비스의 "신경계"다. 여기서 다음을 파악한다:
- 어떤 서비스 객체들이 있는가
- 의존성 주입 구조는 어떻게 되는가
- 초기화 순서는 어떻게 되는가

NestJS 개발자라면 AppModule의 providers 목록을 읽는 것과 같다.

### 3단계: 블록체인 서비스 (services/blockchain.rs)

```
apps/iksan-api/src/services/blockchain.rs
```

이 파일이 이 교재의 핵심이다. Alloy 사용 패턴, sol! 매크로, 트랜잭션 전송, 영수증 처리가 모두 여기 있다.

읽을 때 집중해야 할 것:
- `sol!` 매크로로 어떤 컨트랙트를 정의했는가
- Provider를 어떻게 생성하는가 (with_recommended_fillers 등)
- 에러 처리를 어떻게 하는가
- 비동기 처리 패턴

### 4단계: 이벤트 서비스 (services/event.rs)

```
apps/iksan-api/src/services/event.rs
```

비즈니스 로직과 블록체인 연동이 결합되는 곳이다.

집중 포인트:
- 해시 계산 로직 (keccak256 입력 구성)
- DB 저장과 블록체인 기록의 순서
- 실패 처리 (블록체인 실패 시 어떻게 하는가)
- Result/에러 전파 패턴

### 5단계: 컨트랙트 (contracts/src/TraceRecord.sol)

```
apps/iksan-api/contracts/src/TraceRecord.sol
```

Rust 코드에서 호출하는 컨트랙트의 실제 구현이다.

집중 포인트:
- 어떤 storage 변수가 있는가 (데이터 구조)
- 어떤 함수가 있는가 (public/external)
- 어떤 이벤트를 emit하는가
- 접근 제어는 어떻게 되는가 (onlyOwner 등)

### 6단계: account 서비스의 컨트랙트 배포 (foundry/contract.rs)

```
apps/account/src/foundry/contract.rs
```

Rust에서 forge를 어떻게 실행하는지, 새 고객의 컨트랙트를 어떻게 자동 배포하는지 볼 수 있다.

### 7단계: 인증 서비스 (services/auth.rs)

```
apps/account/src/services/auth.rs
```

JWT 발급과 검증 로직이다. 다른 서비스의 auth 미들웨어와 연결 지점을 이해할 수 있다.

## 핵심 파일별 주목할 패턴

### apps/iksan-api/src/main.rs

```rust
// 주목 1: tokio::spawn으로 백그라운드 태스크
tokio::spawn(async move {
    retry_service.run_forever().await;
});

// 주목 2: graceful shutdown
tokio::signal::ctrl_c().await?;
tracing::info!("종료 신호 수신, 서버 종료 중...");

// 주목 3: 레이어 순서 (아래서 위로 실행됨)
let app = router
    .layer(TraceLayer::new_for_http())  // 3번째 실행
    .layer(CorsLayer::permissive())      // 2번째 실행
    .layer(CompressionLayer::new());     // 1번째 실행
```

### apps/iksan-api/src/core/app.rs

```rust
// 주목 1: Arc 사용 패턴
// 무거운 객체(DB 풀, HTTP 클라이언트)는 Arc로 감싸 참조 공유
pub blockchain: Arc<BlockchainService>,

// 주목 2: Clone은 Arc 참조만 복사
#[derive(Clone)]  // 실제 데이터 복사 없음
pub struct AppState { ... }

// 주목 3: 의존성 순서
// blockchain → event_service (blockchain을 주입받음)
let blockchain = Arc::new(BlockchainService::new(...).await?);
let event_service = Arc::new(EventService::new(db.clone(), Arc::clone(&blockchain)));
```

### apps/iksan-api/src/services/blockchain.rs

```rust
// 주목 1: sol! 매크로의 #[sol(rpc)] 속성
sol! {
    #[sol(rpc)]  // 이게 있어야 .call(), .send() 가능
    contract TraceRecord { ... }
}

// 주목 2: 타입 변환
// Rust [u8; 32] → Alloy FixedBytes<32>
let hash: FixedBytes<32> = FixedBytes::from(raw_bytes);

// 주목 3: 타임아웃 처리
let receipt = tokio::time::timeout(
    Duration::from_secs(60),
    pending.get_receipt(),
).await??;  // ? 두 번: timeout 에러, 영수증 에러

// 주목 4: tracing structured logging
tracing::info!(
    event_id = event_id,  // key=value 형식
    tx_hash = %tx_hash,   // % = Display trait 사용
    block_number = block_number,
    "recordHash 완료"
);
```

### apps/iksan-api/src/services/event.rs

```rust
// 주목 1: BTreeMap으로 JSON 정규화 (키 정렬)
let sorted: BTreeMap<_, _> = payload.as_object()
    .unwrap()
    .iter()
    .collect();

// 주목 2: 에러 전파 패턴
let event = sqlx::query_as!(...)
    .fetch_one(&mut *tx)  // 트랜잭션 내에서 실행
    .await
    .map_err(AppError::Database)?;  // sqlx::Error → AppError

// 주목 3: match로 블록체인 실패 처리
match blockchain.record_hash(...).await {
    Ok(receipt) => { /* 성공 처리 */ }
    Err(e) => {
        tracing::warn!(...);
        // API는 성공, 블록체인만 나중에 재시도
    }
}
```

### apps/iksan-api/contracts/src/TraceRecord.sol

```solidity
// 주목 1: UUPS 업그레이드 패턴
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

// 주목 2: storage layout 주의 (업그레이드 시)
// 새 변수는 반드시 기존 변수 뒤에 추가
mapping(string => Record) private records;  // slot 0
string[] public eventIds;                    // slot 1
// 업그레이드 후 추가:
// uint256 public newFeature;               // slot 2 (반드시)

// 주목 3: custom error (gas 효율적)
error NotOwner(address caller);  // string error보다 저렴
revert NotOwner(msg.sender);

// 주목 4: indexed event parameter
event HashRecorded(
    string indexed eventId,  // indexed: 필터링 가능
    bytes32 dataHash,        // not indexed: 값만 기록
    address indexed recorder
);
```

### apps/account/src/foundry/contract.rs

```rust
// 주목 1: 외부 프로세스 실행
let output = tokio::process::Command::new("forge")
    .args(["create", "--rpc-url", rpc_url, ...])
    .output()
    .await?;

// 주목 2: stdout 파싱으로 배포 주소 추출
// forge create 출력 형식:
// "Deployed to: 0x5FbDB2315678afecb367f032d93F642f64180aa3"
let address = output.stdout
    .lines()
    .find(|l| l.contains("Deployed to:"))
    .and_then(|l| l.split_whitespace().last())
    .ok_or(anyhow::anyhow!("배포 주소 파싱 실패"))?;
```

### apps/account/src/services/auth.rs

```rust
// 주목 1: Argon2 패스워드 해싱 (bcrypt보다 권장)
use argon2::{Argon2, PasswordHash, PasswordVerifier};

// 주목 2: JWT 클레임 구조
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,    // subject (user id)
    email: String,
    role: String,
    exp: u64,       // expiration timestamp
    iat: u64,       // issued at
}

// 주목 3: refresh token rotation
// 리프레시 토큰 사용 시 새 토큰 발급 + 구 토큰 무효화
pub async fn refresh_token(&self, old_refresh: &str) -> Result<TokenPair> {
    let token = self.validate_refresh_token(old_refresh).await?;
    self.revoke_token(old_refresh).await?;  // 구 토큰 무효화
    self.issue_new_tokens(token.user_id).await  // 새 토큰 쌍 발급
}
```

## Rust 역량 인수인계 체크리스트

platform 코드를 유지보수하려면 다음 역량이 필요하다. 각 항목을 확인해보자.

### 필수 Rust 역량 (10항목)

- [ ] **1. 소유권과 빌림**: `&T`, `&mut T`, `T`의 차이를 코드에서 즉시 파악
  - 확인: `blockchain.rs`의 `&self` vs `&mut self` 메서드 구분
  
- [ ] **2. 라이프타임**: 기본 라이프타임 어노테이션 읽기 (복잡한 것은 나중에)
  - 확인: 컴파일러 에러 메시지에서 라이프타임 힌트 이해
  
- [ ] **3. Result와 ? 연산자**: 에러 전파 체인 추적
  - 확인: `event.rs`의 `?` 체인을 따라가며 에러 흐름 파악
  
- [ ] **4. async/await와 Tokio**: Future, 비동기 함수, `.await` 이해
  - 확인: `tokio::spawn`, `tokio::time::timeout` 사용 패턴
  
- [ ] **5. Arc와 Mutex**: 멀티스레드 공유 상태 처리
  - 확인: `Arc<BlockchainService>`가 여러 요청 핸들러에서 공유되는 방식
  
- [ ] **6. 트레이트 객체**: `dyn Trait`, `impl Trait` 구분
  - 확인: `Provider` 트레이트를 반환하는 함수들
  
- [ ] **7. Serde**: `#[derive(Serialize, Deserialize)]`, `#[serde(rename_all)]` 등
  - 확인: `models.rs`의 JSON 직렬화 설정
  
- [ ] **8. 매크로 읽기**: `sol!`, `sqlx::query!`, `tracing::info!` 등
  - 확인: 매크로 출력이 무엇인지 대략 추측 가능
  
- [ ] **9. 에러 타입 정의**: `thiserror`로 에러 enum 작성
  - 확인: `errors.rs`를 수정하여 새 에러 variant 추가
  
- [ ] **10. cargo 명령어**: `build`, `test`, `clippy`, `fmt`
  - 확인: CI에서 실행되는 명령어 모두 로컬에서 실행 가능

### 필수 블록체인 역량 (10항목)

- [ ] **1. Solidity 기초**: 컨트랙트, 함수 가시성, modifier, event, error
  - 확인: `TraceRecord.sol` 전체를 읽고 동작 설명 가능
  
- [ ] **2. ABI 이해**: 함수 시그니처, 인자 인코딩, 반환값 디코딩
  - 확인: `sol!` 매크로가 생성하는 타입 이름 예측 가능
  
- [ ] **3. 트랜잭션 라이프사이클**: mempool → 채굴 → 확정
  - 확인: `pending.get_receipt()`가 왜 필요한지 설명 가능
  
- [ ] **4. 가스와 비용**: 가스 추정, `with_recommended_fillers`의 역할
  - 확인: Besu에서 가스 비용이 0인 이유 설명 가능
  
- [ ] **5. Alloy Provider**: HTTP Provider, 서명자 연결, `with_recommended_fillers`
  - 확인: 새 Provider를 직접 구성하여 테스트 가능
  
- [ ] **6. sol! 매크로**: 인라인 ABI, JSON ABI, #[sol(rpc)] 속성
  - 확인: 새 컨트랙트 함수를 sol! 에 추가하고 Rust에서 호출 가능
  
- [ ] **7. 이벤트 필터링**: Filter 구성, decode_log, indexed 파라미터
  - 확인: 특정 event를 과거 블록에서 조회하는 코드 작성 가능
  
- [ ] **8. UUPS 프록시**: 구현/프록시 분리, 업그레이드 절차, storage layout
  - 확인: 새 storage 변수를 추가하면 왜 위험한지 설명 가능
  
- [ ] **9. keccak256 해시**: 해시 계산, 입력 정규화, 32바이트 처리
  - 확인: 동일한 이벤트가 항상 같은 해시를 생성하는지 확인 가능
  
- [ ] **10. Besu 설정**: genesis.json, IBFT 2.0, min-gas-price=0
  - 확인: 로컬 Besu 네트워크를 Docker로 시작하고 컨트랙트 배포 가능

## 코드 수정 시 주의사항

### TraceRecord.sol 수정 시

```
위험: storage layout 변경
안전: 새 함수 추가, 이벤트 추가, 새 storage 변수 끝에 추가

절대 금지:
  - 기존 storage 변수 순서 변경
  - 기존 storage 변수 타입 변경
  - 기존 storage 변수 삭제

UUPS 업그레이드 절차:
  1. V2 컨트랙트 작성 (V1 storage 그대로 유지)
  2. 로컬에서 테스트
  3. 테스트넷에 새 구현 배포
  4. upgradeToAndCall() 호출
  5. 검증 후 프로덕션 적용
```

### blockchain.rs의 sol! 매크로 수정 시

컨트랙트와 sol! 정의가 불일치하면 런타임 에러가 발생한다. 컨트랙트를 수정할 때마다 sol!도 동기화해야 한다.

```rust
// 컨트랙트에 새 함수를 추가했다면 sol!에도 추가
sol! {
    #[sol(rpc)]
    contract TraceRecord {
        // 기존
        function recordHash(...) external;
        function getRecord(...) external view returns (Record memory);
        
        // 새로 추가
        function batchRecordHashes(
            string[] calldata eventIds,
            bytes32[] calldata hashes
        ) external;
    }
}
```

### DB 마이그레이션 주의사항

```sql
-- 안전: 새 컬럼 추가 (nullable 또는 default 있어야 함)
ALTER TABLE trace_events ADD COLUMN metadata JSONB;

-- 안전: 새 인덱스 추가 (CONCURRENTLY로 무중단)
CREATE INDEX CONCURRENTLY idx_new ON trace_events(new_column);

-- 위험: 컬럼 삭제 (먼저 코드에서 참조 제거 후 삭제)
-- ALTER TABLE trace_events DROP COLUMN old_column;

-- 위험: 컬럼 타입 변경 (데이터 손실 가능)
-- ALTER TABLE trace_events ALTER COLUMN id TYPE BIGINT;
```

## 1달 후 다음 단계 학습 추천

platform 코드에 익숙해진 후 성장하기 위한 다음 학습 경로:

### Rust 심화

**1순위: 비동기 Rust 심화**
- 교재: [Tokio 공식 튜토리얼](https://tokio.rs/tokio/tutorial)
- 내용: select!, join!, mpsc 채널, 백프레셔 처리
- 왜: platform의 백그라운드 태스크와 재시도 로직 개선에 직접 필요

**2순위: 에러 처리 고급**
- 교재: [Error Handling in Rust - A Deep Dive (corrode.dev)](https://corrode.dev/blog/rust-error-handling/)
- 내용: error context, anyhow vs thiserror 선택, 에러 체인
- 왜: 프로덕션 장애 대응 시 에러 메시지 품질이 핵심

**3순위: Rust 성능 최적화**
- 교재: [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- 내용: 불필요한 clone 제거, 제로 카피 파싱, flamegraph 프로파일링
- 왜: 트랜잭션 수가 증가하면 성능 최적화가 필요

### 블록체인 심화

**1순위: Foundry 고급 사용**
- 교재: [Foundry Book](https://book.getfoundry.sh/)
- 내용: Forge 테스트 작성, fuzz 테스트, invariant 테스트, cast 도구
- 왜: 컨트랙트 업그레이드 전 철저한 테스트가 필수

**2순위: 스마트 컨트랙트 보안**
- 교재: [Cyfrin Updraft - Smart Contract Security](https://updraft.cyfrin.io/)
- 내용: reentrancy, integer overflow, access control 취약점
- 왜: TraceRecord에 잘못된 access control이 있으면 누구나 해시를 덮어쓸 수 있음

**3순위: EVM 내부 구조**
- 교재: [evm.codes](https://www.evm.codes/)
- 내용: 옵코드, storage slot 계산, DELEGATECALL 동작
- 왜: UUPS 프록시의 storage collision 문제를 이해하는 데 필수

### 인프라/운영

**서비스 모니터링**
- Prometheus + Grafana로 Axum 메트릭 수집
- Besu 노드 상태 모니터링
- 알람 설정 (블록 생성 중단, pending 트랜잭션 급증)

**컨트랙트 업그레이드 자동화**
- Foundry의 `forge script`로 업그레이드 스크립트 작성
- 멀티시그(Gnosis Safe)로 업그레이드 권한 분산

## 자주 발생하는 문제와 해결법

### "트랜잭션이 pending 상태에서 멈췄다"

```bash
# Besu 노드 상태 확인
curl -X POST http://besu:8545 \
  -d '{"jsonrpc":"2.0","method":"txpool_content","id":1}'

# 검증자 목록 확인 (IBFT)
curl -X POST http://besu:8545 \
  -d '{"jsonrpc":"2.0","method":"ibft_getValidatorsByBlockNumber","params":["latest"],"id":1}'

# nonce 확인 (낮은 nonce 트랜잭션이 막고 있을 수 있음)
curl -X POST http://besu:8545 \
  -d '{"jsonrpc":"2.0","method":"eth_getTransactionCount","params":["0xYOUR_ADDRESS","latest"],"id":1}'
```

### "컨트랙트 호출이 revert됨"

```bash
# cast로 revert 이유 확인
cast call \
  --rpc-url http://besu:8545 \
  0xCONTRACT_ADDRESS \
  "recordHash(string,bytes32)" \
  "event-001" \
  0x1234...

# 또는 Rust에서
let err = contract.recordHash(id, hash).call().await.unwrap_err();
println!("{:#?}", err);  // AlloyError 상세 출력
```

### "해시 불일치"

데이터가 변조되거나 해시 계산 로직이 다를 때 발생한다.

```rust
// 디버깅: 해시 입력을 출력해서 확인
let hash_input = format!("{}{}{}", event_id, event_type, payload);
println!("해시 입력: {}", hash_input);
println!("해시: {}", hex::encode(keccak256(hash_input.as_bytes())));

// Solidity에서 동일 검증
// bytes32 expected = keccak256(abi.encodePacked(eventId, eventType, payload));
// 주의: Rust의 keccak256(bytes)와 Solidity의 keccak256(abi.encodePacked(...))은 다를 수 있음
```

## 요약

platform 코드 읽기 순서:
1. `main.rs` → 서버 구조 파악
2. `core/app.rs` → 의존성 구조 파악
3. `services/blockchain.rs` → Alloy 패턴 학습
4. `services/event.rs` → 비즈니스 로직과 블록체인 연동
5. `contracts/TraceRecord.sol` → 온체인 데이터 구조
6. `foundry/contract.rs` → 자동 배포 패턴
7. `services/auth.rs` → 인증 구조

체크리스트 10+10항목을 완료하면 platform을 안전하게 유지보수할 수 있다. 이후 Foundry 고급, 컨트랙트 보안, Tokio 심화 순으로 성장하면 된다.

이것으로 22장(Platform 분석)을 마친다. 부록에서는 블록체인 생태계 현황과 Node.js → Rust 전환 가이드를 다룬다.
