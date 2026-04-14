# 22-1: 서비스 아키텍처 - NestJS 개발자를 위한 대응 분석

## 개요: NestJS vs Axum

4년간 Node.js 백엔드를 개발했다면 NestJS의 구조에 익숙할 것이다. Axum은 NestJS와 철학이 다르지만, 역할은 대응된다.

```text
NestJS                        Axum (platform 방식)
─────────────────────────     ─────────────────────────
@Module()                  ↔  AppState (구조체)
@Injectable() Service      ↔  Service 구조체
@Controller()              ↔  Router + Handler 함수
@Middleware()              ↔  Tower Layer/middleware
@Guard()                   ↔  Tower middleware 또는 Extractor
DI Container               ↔  Arc<AppState> 수동 주입
Pipes (validation)         ↔  serde + validator 크레이트
Interceptors               ↔  Tower Layer
Exception Filters          ↔  IntoResponse 구현
```

NestJS는 데코레이터와 DI 컨테이너로 "마법처럼" 의존성을 연결한다. Axum은 모든 것이 명시적이다. `Arc<AppState>`를 직접 만들어서 라우터에 붙인다. 더 verbose하지만 무슨 일이 일어나는지 완전히 이해할 수 있다.

## AppState - NestJS의 @Module providers

NestJS에서 `@Module({ providers: [UserService, AuthService, TypeOrmModule] })`으로 의존성을 등록하듯, Axum에서는 `AppState` 구조체에 모든 의존성을 담는다.

### platform의 AppState 패턴

```rust,ignore
// apps/iksan-api/src/core/app.rs

use std::sync::Arc;
use sqlx::PgPool;
use crate::services::{
    blockchain::BlockchainService,
    event::EventService,
    farmer::FarmerService,
    crop::CropService,
};

/// 앱 전체에서 공유되는 상태
/// NestJS의 AppModule providers에 해당
#[derive(Clone)]
pub struct AppState {
    // 데이터베이스 커넥션 풀
    pub db: PgPool,
    
    // 서비스들 - Arc로 감싸 참조 카운팅
    pub blockchain: Arc<BlockchainService>,
    pub event_service: Arc<EventService>,
    pub farmer_service: Arc<FarmerService>,
    pub crop_service: Arc<CropService>,
    
    // 설정
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        // 1. DB 연결
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.db_max_connections)
            .connect(&config.database_url)
            .await?;
        
        // 마이그레이션
        sqlx::migrate!("./migrations").run(&db).await?;
        
        // 2. 블록체인 서비스 초기화
        let blockchain = Arc::new(
            BlockchainService::new(
                config.rpc_url.clone(),
                config.private_key.clone(),
                config.contract_address.clone(),
            ).await?
        );
        
        // 3. 비즈니스 서비스 초기화 (블록체인 의존성 주입)
        let event_service = Arc::new(
            EventService::new(db.clone(), Arc::clone(&blockchain))
        );
        
        let farmer_service = Arc::new(FarmerService::new(db.clone()));
        let crop_service = Arc::new(CropService::new(db.clone()));
        
        Ok(AppState {
            db,
            blockchain,
            event_service,
            farmer_service,
            crop_service,
            config: Arc::new(config),
        })
    }
}
```

`AppState`는 `Clone`을 구현하는데, 실제로 데이터를 복사하지 않는다. `Arc` 참조 카운터만 증가시킨다. Axum이 각 요청 핸들러에 `AppState`를 전달할 때 이 방식으로 작동한다.

### NestJS와 비교

```typescript
// NestJS
@Module({
  imports: [
    TypeOrmModule.forFeature([Farmer, Crop, TraceEvent]),
  ],
  providers: [
    FarmerService,
    CropService,
    EventService,
    BlockchainService,
  ],
  controllers: [FarmerController, CropController, EventController],
})
export class AppModule {}
```

```rust,ignore
// Axum (platform)
let state = AppState::new(config).await?;

let app = Router::new()
    .nest("/farmers", farmer_routes())
    .nest("/crops", crop_routes())
    .nest("/events", event_routes())
    .with_state(state);  // AppState를 라우터에 주입
```

NestJS는 프레임워크가 DI를 관리하고, Axum은 개발자가 직접 주입한다.

## Router - Express/NestJS Controller와 대응

### platform의 라우트 구조

```rust,ignore
// apps/iksan-api/src/main.rs

use axum::{Router, middleware};
use crate::{
    core::app::AppState,
    routes::{farmer, crop, event, health},
    middleware::auth::auth_middleware,
};

pub fn create_app(state: AppState) -> Router {
    // 공개 라우트 (인증 불필요)
    let public = Router::new()
        .route("/health", get(health::check))
        .route("/events/:id/public", get(event::get_public));
    
    // 보호된 라우트 (JWT 필요)
    let protected = Router::new()
        .nest("/farmers", farmer_routes())
        .nest("/crops", crop_routes())
        .nest("/events", event_routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,  // JWT 검증 미들웨어
        ));
    
    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
}

fn farmer_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(farmer::list).post(farmer::create))
        .route("/:id", get(farmer::get).put(farmer::update).delete(farmer::delete))
        .route("/:id/crops", get(farmer::get_crops))
}

fn event_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(event::list).post(event::create))
        .route("/:id", get(event::get))
        .route("/:id/verify", get(event::verify))
        .route("/:id/blockchain", get(event::get_blockchain_status))
}
```

### NestJS Controller와 비교

```typescript
// NestJS
@Controller('farmers')
@UseGuards(JwtAuthGuard)
export class FarmerController {
  constructor(private readonly farmerService: FarmerService) {}

  @Get()
  list(@Query() query: ListFarmerDto) {
    return this.farmerService.findAll(query);
  }

  @Post()
  create(@Body() dto: CreateFarmerDto) {
    return this.farmerService.create(dto);
  }

  @Get(':id')
  get(@Param('id') id: string) {
    return this.farmerService.findOne(id);
  }
}
```

```rust,ignore
// Axum (platform)
// routes/farmer.rs

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListFarmerParams>,
    Extension(user): Extension<AuthUser>,  // JWT에서 추출된 사용자
) -> Result<Json<Vec<FarmerResponse>>, AppError> {
    let farmers = state.farmer_service.find_all(&params).await?;
    Ok(Json(farmers))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<CreateFarmerRequest>,
) -> Result<(StatusCode, Json<FarmerResponse>), AppError> {
    let farmer = state.farmer_service.create(user.id, body).await?;
    Ok((StatusCode::CREATED, Json(farmer)))
}
```

차이점:
- NestJS: 메서드에 데코레이터로 HTTP 메서드 지정
- Axum: `Router::new().route("/", get(fn).post(fn))`으로 함수와 메서드 분리
- NestJS: DI로 서비스 주입
- Axum: `State(state)` extractor로 AppState 접근

## Tower 미들웨어 - NestJS middleware와 대응

Tower는 Axum이 사용하는 미들웨어 추상화 라이브러리다. `Service` 트레이트를 기반으로 한다.

### 인증 미들웨어

```rust,ignore
// apps/iksan-api/src/middleware/auth.rs

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use crate::core::app::AppState;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub role: String,
}

/// JWT 인증 미들웨어
/// NestJS의 @UseGuards(JwtAuthGuard)에 해당
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Authorization 헤더에서 토큰 추출
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // JWT 검증 (account 서비스에 요청하거나 로컬 검증)
    let claims = verify_jwt(token, &state.config.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // 검증된 사용자 정보를 요청 확장(Extension)에 추가
    req.extensions_mut().insert(AuthUser {
        id: claims.sub,
        email: claims.email,
        role: claims.role,
    });
    
    // 다음 핸들러로 진행
    Ok(next.run(req).await)
}

fn verify_jwt(token: &str, secret: &str) -> anyhow::Result<Claims> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
    
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    
    let data = decode::<Claims>(token, &key, &validation)?;
    Ok(data.claims)
}

#[derive(serde::Deserialize)]
struct Claims {
    sub: String,
    email: String,
    role: String,
    exp: u64,
}
```

### NestJS 가드와 비교

```typescript
// NestJS
@Injectable()
export class JwtAuthGuard extends AuthGuard('jwt') {
  canActivate(context: ExecutionContext): boolean {
    // passport-jwt가 자동으로 토큰 검증
    return super.canActivate(context) as boolean;
  }
}

// 사용
@UseGuards(JwtAuthGuard)
@Get('profile')
getProfile(@Request() req) {
  return req.user; // passport가 주입한 사용자
}
```

```rust,ignore
// Axum
// 미들웨어가 Extension에 AuthUser를 추가
// 핸들러에서 Extension extractor로 접근

pub async fn get_profile(
    Extension(user): Extension<AuthUser>,
) -> Json<serde_json::Value> {
    Json(json!({ "id": user.id, "email": user.email }))
}
```

### CORS 미들웨어

```rust,ignore
// main.rs에서 tower-http의 CORS 레이어 사용

use tower_http::cors::{CorsLayer, Any};
use axum::http::{HeaderName, Method};

let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any)
    .allow_origin(Any);  // 프로덕션에서는 특정 도메인만

let app = create_app(state)
    .layer(cors)
    .layer(TraceLayer::new_for_http());
```

### 요청 로깅 미들웨어

```rust,ignore
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;

let trace_layer = TraceLayer::new_for_http()
    .make_span_with(
        DefaultMakeSpan::new()
            .level(Level::INFO)
            .include_headers(false)
    )
    .on_response(
        DefaultOnResponse::new()
            .level(Level::INFO)
            .latency_unit(tower_http::LatencyUnit::Millis)
    );
```

`TraceLayer`는 NestJS의 `LoggingInterceptor`와 유사하다. 모든 HTTP 요청/응답에 대해 자동으로 tracing 스팬을 생성한다.

## 각 서비스 상세 분석

### account 서비스

**역할**: 인증, 계정, DID, 요금제

```text
account/
├── src/
│   ├── main.rs
│   ├── core/
│   │   ├── app.rs          ← AppState (DB + blockchain + services)
│   │   └── config.rs
│   ├── routes/
│   │   ├── auth.rs         ← POST /auth/login, /auth/register, /auth/refresh
│   │   ├── account.rs      ← GET/PUT /accounts/me
│   │   ├── did.rs          ← POST /did/register, GET /did/:did
│   │   └── subscription.rs ← GET/POST /subscriptions
│   ├── services/
│   │   ├── auth.rs         ← JWT 발급, 검증, 리프레시
│   │   ├── account.rs      ← 계정 CRUD
│   │   ├── did.rs          ← DID 생성, 블록체인 등록
│   │   └── subscription.rs ← 요금제 관리
│   ├── foundry/
│   │   └── contract.rs     ← 컨트랙트 배포 로직
│   └── models/
│       ├── account.rs
│       └── did.rs
```

account 서비스의 특이점은 Foundry를 Rust에서 직접 실행한다는 것이다. 새 고객이 가입하면, 해당 고객 전용 스마트 컨트랙트를 자동으로 배포한다:

```rust,ignore
// apps/account/src/foundry/contract.rs
pub async fn deploy_trace_contract(
    rpc_url: &str,
    deployer_key: &str,
) -> anyhow::Result<Address> {
    // forge create 명령을 서브프로세스로 실행
    let output = tokio::process::Command::new("forge")
        .args([
            "create",
            "--rpc-url", rpc_url,
            "--private-key", deployer_key,
            "contracts/src/TraceRecord.sol:TraceRecord",
        ])
        .output()
        .await?;
    
    // 출력에서 배포된 주소 파싱
    let stdout = String::from_utf8(output.stdout)?;
    let address = parse_deployed_address(&stdout)?;
    
    Ok(address)
}
```

### traceability 서비스

**역할**: 이력 추적 비즈니스 로직, 제품 관리, 규정 준수

```text
traceability/
├── src/
│   ├── routes/
│   │   ├── product.rs      ← 제품 등록/조회/QR 생성
│   │   ├── trace.rs        ← 이력 조회, 타임라인
│   │   └── compliance.rs   ← 규정 준수 체크
│   ├── services/
│   │   ├── product.rs      ← 제품 비즈니스 로직
│   │   ├── trace.rs        ← 이력 집계 및 검증
│   │   └── qr.rs           ← QR 코드 생성
│   └── repositories/
│       ├── product.rs      ← 제품 DB 쿼리
│       └── trace.rs        ← 이력 DB 쿼리
```

traceability 서비스는 iksan-api에서 기록한 데이터를 읽어서 소비자에게 보여주는 역할을 한다. 블록체인에 직접 쓰지 않고, iksan-api의 API를 통해 데이터를 가져온다.

### iksan-api 서비스

**역할**: 농업인/작물 관리 + 블록체인 연동 핵심

```text
iksan-api/
├── src/
│   ├── routes/
│   │   ├── farmer.rs       ← 농업인 CRUD
│   │   ├── crop.rs         ← 작물 CRUD
│   │   └── event.rs        ← 이벤트 생성/조회/검증
│   ├── services/
│   │   ├── farmer.rs
│   │   ├── crop.rs
│   │   ├── event.rs        ← 이벤트 비즈니스 로직
│   │   └── blockchain.rs   ← Alloy 연동 핵심
│   └── contracts/
│       └── src/
│           └── TraceRecord.sol
```

## 핵심 데이터베이스 스키마

### account 서비스 테이블

```sql
-- 계정
CREATE TABLE accounts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name        VARCHAR(100),
    role        VARCHAR(50) DEFAULT 'user',
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW()
);

-- DID (Decentralized Identifier)
CREATE TABLE dids (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID REFERENCES accounts(id) ON DELETE CASCADE,
    did             VARCHAR(255) UNIQUE NOT NULL,  -- did:ethr:0x...
    public_key      TEXT NOT NULL,
    contract_address VARCHAR(42),  -- DID 컨트랙트 주소
    blockchain_tx   VARCHAR(66),   -- 등록 TX 해시
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- 요금제
CREATE TABLE subscriptions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id  UUID REFERENCES accounts(id) ON DELETE CASCADE,
    plan        VARCHAR(50) NOT NULL,  -- 'free', 'basic', 'enterprise'
    status      VARCHAR(50) DEFAULT 'active',
    starts_at   TIMESTAMPTZ NOT NULL,
    ends_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ DEFAULT NOW()
);
```

### iksan-api 서비스 테이블

```sql
-- 농업인
CREATE TABLE farmers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id  UUID NOT NULL,  -- account 서비스의 ID (외래키 아님, 서비스 분리)
    name        VARCHAR(100) NOT NULL,
    phone       VARCHAR(20),
    address     TEXT,
    farm_name   VARCHAR(200),
    certification_number VARCHAR(100),  -- 농업인 확인증
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW()
);

-- 작물
CREATE TABLE crops (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    farmer_id   UUID REFERENCES farmers(id) ON DELETE CASCADE,
    name        VARCHAR(100) NOT NULL,
    variety     VARCHAR(100),   -- 품종
    planting_date DATE,
    expected_harvest DATE,
    location    TEXT,           -- 재배지
    area_sqm    DECIMAL(10, 2), -- 재배 면적 (㎡)
    status      VARCHAR(50) DEFAULT 'growing',
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

-- 이벤트 (핵심 테이블)
CREATE TABLE trace_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    crop_id         UUID REFERENCES crops(id),
    farmer_id       UUID REFERENCES farmers(id),
    event_type      VARCHAR(100) NOT NULL,  -- 'planting', 'harvest', 'inspection', 'transport'
    payload         JSONB NOT NULL,         -- 이벤트 상세 데이터
    data_hash       VARCHAR(66) NOT NULL,   -- keccak256 (0x + 64자)
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- 블록체인 기록 (별도 테이블로 분리)
CREATE TABLE blockchain_records (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id    UUID REFERENCES trace_events(id) ON DELETE CASCADE,
    tx_hash     VARCHAR(66) NOT NULL,  -- 0x + 64자
    block_number BIGINT,
    recorded_at TIMESTAMPTZ DEFAULT NOW(),
    status      VARCHAR(50) DEFAULT 'pending'  -- 'pending', 'confirmed', 'failed'
);

-- 인덱스
CREATE INDEX idx_trace_events_crop_id ON trace_events(crop_id);
CREATE INDEX idx_trace_events_farmer_id ON trace_events(farmer_id);
CREATE INDEX idx_trace_events_event_type ON trace_events(event_type);
CREATE INDEX idx_trace_events_created_at ON trace_events(created_at DESC);
CREATE INDEX idx_blockchain_records_event_id ON blockchain_records(event_id);
```

`trace_events`와 `blockchain_records`를 분리한 이유:
- 이벤트 생성(빠른 DB 쓰기)과 블록체인 기록(느린 TX)을 비동기로 처리
- 블록체인 기록 실패 시 재시도 가능
- 블록체인 기록 상태를 독립적으로 추적

## 서비스 간 HTTP 통신

platform에서 서비스 간 통신은 단순한 HTTP 요청이다. NestJS의 `HttpModule`이나 gRPC 없이 `reqwest` 크레이트를 사용한다.

```rust,ignore
// traceability 서비스가 iksan-api에서 이벤트를 조회하는 예
// apps/traceability/src/services/trace.rs

use reqwest::Client;

pub struct TraceService {
    http_client: Client,
    iksan_api_url: String,
}

impl TraceService {
    pub async fn get_crop_trace_history(
        &self,
        crop_id: &str,
        jwt_token: &str,
    ) -> anyhow::Result<Vec<TraceEvent>> {
        let url = format!("{}/crops/{}/events", self.iksan_api_url, crop_id);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "iksan-api 요청 실패: {}", response.status()
            ));
        }
        
        let events: Vec<TraceEvent> = response.json().await?;
        Ok(events)
    }
}
```

이것은 단순하지만, 프로덕션에서는 서킷 브레이커(tower의 `timeout`, `retry`)를 추가해야 한다.

## 에러 처리 패턴

platform 전체에서 일관된 에러 처리 패턴을 사용한다:

```rust,ignore
// 각 서비스의 errors.rs

use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("찾을 수 없음: {0}")]
    NotFound(String),
    
    #[error("권한 없음")]
    Unauthorized,
    
    #[error("요청 오류: {0}")]
    BadRequest(String),
    
    #[error("데이터베이스 오류")]
    Database(#[from] sqlx::Error),
    
    #[error("블록체인 오류: {0}")]
    Blockchain(String),
    
    #[error("외부 서비스 오류: {0}")]
    ExternalService(String),
    
    #[error("내부 오류")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg.clone()
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "인증이 필요합니다".to_string()
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg.clone()
            ),
            AppError::Database(e) => {
                tracing::error!(error = %e, "DB 오류");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "데이터베이스 오류가 발생했습니다".to_string()
                )
            },
            AppError::Blockchain(msg) => {
                tracing::warn!(error = %msg, "블록체인 오류");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "BLOCKCHAIN_ERROR",
                    msg.clone()
                )
            },
            AppError::ExternalService(msg) => (
                StatusCode::BAD_GATEWAY,
                "EXTERNAL_SERVICE_ERROR",
                msg.clone()
            ),
            AppError::Internal(e) => {
                tracing::error!(error = %e, "내부 오류");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "내부 서버 오류가 발생했습니다".to_string()
                )
            },
        };
        
        (
            status,
            Json(serde_json::json!({
                "success": false,
                "error": {
                    "code": code,
                    "message": message
                }
            }))
        ).into_response()
    }
}
```

NestJS의 `ExceptionFilter`와 비교:

```typescript
// NestJS
@Catch()
export class AllExceptionsFilter implements ExceptionFilter {
  catch(exception: unknown, host: ArgumentsHost) {
    const ctx = host.switchToHttp();
    const response = ctx.getResponse<Response>();
    
    const status = exception instanceof HttpException
      ? exception.getStatus()
      : HttpStatus.INTERNAL_SERVER_ERROR;
    
    response.status(status).json({
      success: false,
      error: { message: 'error' }
    });
  }
}
```

Rust의 접근이 더 타입 안전하다. 가능한 에러 타입이 모두 명시되어 있고, 처리되지 않은 케이스는 컴파일 에러가 난다.

## 요약

| NestJS 개념 | Axum/platform 구현 |
|-------------|-------------------|
| `@Module()` | `AppState` 구조체 |
| `@Injectable()` | 일반 구조체 + `Arc<T>` |
| `@Controller()` | `Router::new().route()` |
| `@Get()`, `@Post()` | `get(fn)`, `post(fn)` |
| `@UseGuards()` | `middleware::from_fn()` |
| `@UseInterceptors()` | Tower Layer |
| `@Param()`, `@Body()`, `@Query()` | `Path`, `Json`, `Query` Extractor |
| `DI Container` | `Arc<AppState>` 수동 주입 |
| `ExceptionFilter` | `IntoResponse` 구현 |
| `HttpException` | `AppError` enum |

다음 장에서는 블록체인 연동 흐름을 더 자세히 분석한다.
