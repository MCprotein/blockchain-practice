# 부록 B: Node.js 개발자를 위한 Rust 전환 주의사항

## 개요

4년간 Node.js를 사용했다면 Rust로 전환할 때 특정 패턴에서 반복적으로 막히게 된다. 이 부록은 가장 흔한 함정들을 TypeScript/JavaScript 코드와 Rust 코드를 직접 비교하며 설명한다.

출처: corrode.dev TypeScript to Rust Migration Guide (https://corrode.dev/blog/typescript-to-rust/)

## 1. 빌림 검사기와 싸우지 말 것

빌림 검사기(Borrow Checker)는 컴파일러의 일부로, 메모리 안전성을 보장한다. Node.js에는 이런 개념이 없다. GC가 대신 처리하기 때문이다.

**흔한 실수: 이동 후 사용**

```typescript
// TypeScript - 동작함
const data = { name: "apple", count: 5 };
processData(data);
console.log(data.name); // 문제없음
```

```rust,ignore
// Rust - 컴파일 에러
let data = EventData { name: "apple".to_string(), count: 5 };
process_data(data);           // data의 소유권이 이동됨
println!("{}", data.name);    // 에러! data는 이미 이동됨
```

**해결: 참조(&) 또는 Clone 사용**

```rust,ignore
// 해결 1: 참조 전달 (권장)
let data = EventData { name: "apple".to_string(), count: 5 };
process_data(&data);          // 참조만 전달
println!("{}", data.name);    // OK - 소유권 유지됨

// 해결 2: Clone (비용이 있음)
let data = EventData { name: "apple".to_string(), count: 5 };
process_data(data.clone());   // 복사본 전달
println!("{}", data.name);    // OK - 원본 소유권 유지됨

// 함수 시그니처도 맞춰야 함
fn process_data(data: &EventData) {
    println!("{}: {}", data.name, data.count);
}  // 참조 받기
```

**Arc로 여러 곳에서 공유**

```typescript
// TypeScript - 여러 곳에서 참조 가능
const service = new BlockchainService(config);
const handler1 = new EventHandler(service);
const handler2 = new CropHandler(service);
// 모두 같은 service 객체를 참조
```

```rust,ignore
// Rust - Arc로 공유 참조
use std::sync::Arc;

let service = Arc::new(BlockchainService::new(config).await?);
let handler1 = EventHandler::new(Arc::clone(&service));
let handler2 = CropHandler::new(Arc::clone(&service));
// Arc 참조 카운팅으로 공유, 마지막 참조 소멸 시 해제
```

**핵심 규칙**: 빌림 검사기 에러를 무시하지 마라. 억지로 컴파일을 통과시키려 하면 더 복잡해진다. 에러 메시지를 읽고 소유권 모델을 이해하는 것이 빠른 길이다.

## 2. .unwrap() 남발 금지

Node.js에서는 에러 처리를 안 해도 런타임까지 잘 굴러가는 경우가 많다. Rust에서 `.unwrap()`은 `panic!`을 숨기는 시한폭탄이다.

```typescript
// TypeScript (나쁜 패턴이지만 동작함)
const value = JSON.parse(userInput);  // 실패해도 일단 try-catch로
const result = await db.query(sql);   // undefined 체크 없이 사용
```

```rust,ignore
// Rust - 나쁜 패턴 (절대 금지)
let value: serde_json::Value = serde_json::from_str(user_input).unwrap();  // panic 가능
let result = db.fetch_one(query).await.unwrap();  // panic 가능
```

```rust,ignore
// Rust - 올바른 패턴

// 방법 1: ? 연산자 (가장 간결)
async fn handle_request(input: &str) -> Result<Response, AppError> {
    let value: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| AppError::BadRequest(format!("JSON 파싱 실패: {}", e)))?;
    
    let result = db.fetch_one(query).await
        .map_err(AppError::Database)?;
    
    Ok(Response { data: result })
}

// 방법 2: match
match serde_json::from_str::<serde_json::Value>(input) {
    Ok(value) => { /* 사용 */ }
    Err(e) => return Err(AppError::BadRequest(e.to_string())),
}

// 방법 3: if let (Option에서 자주 사용)
if let Some(record) = db.fetch_optional(query).await? {
    // record 사용
} else {
    return Err(AppError::NotFound("레코드 없음".to_string()));
}
```

**언제 unwrap/expect를 써도 되는가:**
```rust,ignore
// 테스트 코드에서
#[test]
fn test_hash_calculation() {
    let hash = calculate_hash("test").unwrap();  // 테스트에서는 OK
    assert_eq!(hash.len(), 64);
}

// 프로그램 시작 시 반드시 있어야 하는 설정
let db_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL 환경변수가 반드시 있어야 합니다");
// expect는 panic 메시지를 더 명확하게 만듦

// 논리적으로 절대 실패할 수 없는 경우
let arr = [1, 2, 3];
let first = arr.first().unwrap();  // 배열이 비어있을 수 없음
```

## 3. String vs &str 혼동

Node.js에는 문자열이 하나다. Rust에는 두 가지가 있다.

```typescript
// TypeScript
const name: string = "apple";
const greeting: string = `Hello, ${name}`;
function greet(name: string): string {
  return `Hello, ${name}`;
}
```

```rust,ignore
// Rust
let name: &str = "apple";           // 정적 문자열 슬라이스 (불변, 스택)
let owned: String = "apple".to_string(); // 힙 할당 문자열 (가변, 소유)
let owned2: String = String::from("apple");

// 함수 파라미터
fn greet(name: &str) -> String {    // &str 받고 String 반환 (권장)
    format!("Hello, {}", name)
}

// 호출 시
greet("apple");                      // &str 직접
greet(&owned);                       // String → &str 자동 변환 (Deref)
```

**언제 무엇을 쓸까:**
```rust,ignore
// &str을 써야 하는 경우:
// - 함수 파라미터 (호출자가 String이든 &str이든 모두 받을 수 있음)
// - 문자열을 수정하지 않을 때
fn process(name: &str) {
    println!("processing {name}");
}

// String을 써야 하는 경우:
// - 구조체 필드 (소유권 필요)
// - 반환값으로 새 문자열 생성
// - 문자열을 수정해야 할 때
struct Event {
    name: String,  // &str이면 라이프타임 문제
}

// 변환
let s: String = "hello".to_string();
let s: String = format!("{}", some_value);
let slice: &str = &s;              // String → &str
let owned: String = slice.to_owned(); // &str → String
```

## 4. 금융 계산에 float 사용 금지

Node.js에서도 금융 계산에 `number` 타입을 쓰면 안 된다는 것을 알고 있을 것이다. Rust에서도 마찬가지다.

```typescript
// TypeScript - 잘못된 예시
const price = 0.1 + 0.2;  // 0.30000000000000004
const total = 1.5 * 100;   // 실제로는 149.99999... 일 수 있음
```

```rust
// Rust - 잘못된 예시
let price: f64 = 0.1 + 0.2;  // 0.30000000000000004
let token_amount: f64 = 1_500_000.0 * 0.001;  // 부동소수점 오차
```

```rust,ignore
// Rust - 올바른 방법: 정수 사용 (최소 단위)

// ETH는 wei 단위 (10^18)
let balance: u128 = 1_000_000_000_000_000_000u128; // 1 ETH = 10^18 wei
let gas_price: u128 = 20_000_000_000u128;           // 20 Gwei = 20 * 10^9

// Alloy의 U256 (256비트 정수)
use alloy::primitives::U256;
let amount: U256 = U256::from(1_000_000_000_000_000_000u128);

// rust_decimal 크레이트 사용 (가격, 환율 등)
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

let price = dec!(1.5);       // 정확한 소수 표현
let fee_rate = dec!(0.003);  // 0.3%
let fee = price * fee_rate;  // 정확히 0.0045

// 표시용으로만 f64 변환
let display: f64 = fee.to_f64().unwrap_or(0.0);
```

**블록체인에서의 규칙:**
- 토큰 잔액: 항상 최소 단위 정수 (wei, lamports, satoshi)
- 가격/환율: `rust_decimal::Decimal`
- 퍼센트: `u32` (예: 300 = 3.00%)
- 표시용 포맷팅 시에만 부동소수점으로 변환

## 5. .await 중 MutexGuard 보유 위험

Node.js는 싱글스레드이므로 이런 문제가 없다. Rust의 async에서는 `.await` 지점에서 다른 태스크로 전환될 수 있어 데드락이 발생한다.

```typescript
// TypeScript - 문제 없음 (싱글스레드)
async function processEvent(mutex: Mutex, event: Event) {
    const lock = mutex.lock();
    const result = await someAsyncOp();  // 다른 태스크 전환 없음
    lock.release();
}
```

```rust,ignore
// Rust - 컴파일 에러 또는 데드락!
async fn process_event(mutex: Arc<Mutex<State>>, event: Event) -> Result<()> {
    let guard = mutex.lock().unwrap();  // 락 획득
    let result = some_async_op().await;  // 여기서 다른 태스크로 전환 가능
    // guard가 .await를 넘어 살아있으면 Future가 Send를 구현하지 않아 컴파일 에러
    Ok(())
}
```

```rust,ignore
// 올바른 패턴 1: 락 범위를 .await 밖으로

async fn process_event(mutex: Arc<Mutex<State>>, event: Event) -> Result<()> {
    // 락을 최소 범위로 사용
    let value = {
        let guard = mutex.lock().unwrap();
        guard.some_value.clone()  // 값을 복사하고
    }; // 여기서 guard 해제
    
    // .await는 락 없이
    let result = some_async_op(value).await?;
    
    // 다시 락이 필요하면
    {
        let mut guard = mutex.lock().unwrap();
        guard.some_value = result;
    }
    
    Ok(())
}

// 올바른 패턴 2: tokio::sync::Mutex 사용 (async-aware)
use tokio::sync::Mutex;

async fn process_event(mutex: Arc<Mutex<State>>, event: Event) -> Result<()> {
    let mut guard = mutex.lock().await;  // .await로 락 획득 (데드락 없음)
    let result = some_async_op(&guard.data).await?;
    guard.result = result;
    Ok(())
}
```

**규칙:**
- `std::sync::Mutex`: sync 코드에서만 사용, `.await` 전에 반드시 해제
- `tokio::sync::Mutex`: async 코드에서 `.await`를 넘어야 할 때
- 가능하면 락 범위를 최소화

## 6. try/catch → Result 전환

```typescript
// TypeScript
async function fetchUser(id: string): Promise<User> {
    try {
        const user = await db.findUser(id);
        if (!user) throw new NotFoundError(`User ${id} not found`);
        return user;
    } catch (error) {
        if (error instanceof NotFoundError) throw error;
        logger.error('DB error:', error);
        throw new InternalError('Database error');
    }
}
```

```rust,ignore
// Rust
async fn fetch_user(db: &PgPool, id: &str) -> Result<User, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "DB 오류");
            AppError::Database(e)
        })?;
    
    user.ok_or_else(|| AppError::NotFound(format!("사용자 없음: {}", id)))
}
```

**패턴 대응표:**

| TypeScript | Rust |
|-----------|------|
| `try { } catch (e) { }` | `match result { Ok(v) => ..., Err(e) => ... }` |
| `throw new Error("msg")` | `return Err(AppError::SomeError("msg".into()))` |
| `Promise<T>` | `Result<T, E>` (async에서) |
| `async function` | `async fn` |
| `await promise` | `future.await` |
| `Promise.all([...])` | `tokio::join!(...)` 또는 `futures::join_all(...)` |
| `catch (e) { if (e instanceof X) }` | `match e { AppError::X(_) => ..., _ => ... }` |

## 7. 컴파일 시간 관리

Rust는 컴파일이 느리다. Node.js에서 `ts-node`로 즉시 실행하던 것과 달리, Rust는 전체 빌드에 분 단위가 걸릴 수 있다.

```bash
# 느린 방법 (전체 빌드)
cargo build           # 처음: 2-10분
cargo build --release # 최적화 빌드: 더 오래 걸림

# 빠른 방법들

# 1. cargo check - 실행 파일 없이 타입만 검사 (가장 빠름)
cargo check           # 컴파일의 ~30% 시간

# 2. cargo clippy - 타입 검사 + 린트 (check보다 약간 느림)
cargo clippy

# 3. mold/lld 링커 사용 (링킹 시간 단축)
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# 4. cargo-watch - 파일 변경 시 자동 cargo check
cargo install cargo-watch
cargo watch -x check           # 저장 시 check
cargo watch -x "run --bin api" # 저장 시 재시작

# 5. sccache - 빌드 캐시
cargo install sccache
RUSTC_WRAPPER=sccache cargo build

# 6. 작업공간 분리 - 자주 바뀌는 코드를 별도 크레이트로
[workspace]
members = ["core", "api", "blockchain"]
# core가 안 바뀌면 재컴파일 불필요
```

**개발 워크플로우 추천:**
```bash
# 코딩 중: check만
cargo check

# PR 전: 전체 검증
cargo clippy -- -D warnings
cargo test
cargo fmt --check
```

## 8. npm → cargo 명령어 대응표

| npm / Node.js | cargo / Rust | 설명 |
|---------------|-------------|------|
| `npm init` | `cargo new my-project` | 새 프로젝트 |
| `npm install` | `cargo build` | 의존성 다운로드 + 빌드 |
| `npm install pkg` | `cargo add pkg` | 의존성 추가 |
| `npm run start` | `cargo run` | 실행 |
| `npm run build` | `cargo build --release` | 릴리즈 빌드 |
| `npm test` | `cargo test` | 테스트 실행 |
| `npm run lint` | `cargo clippy` | 린트 |
| `npx prettier --write` | `cargo fmt` | 코드 포맷 |
| `package.json` | `Cargo.toml` | 프로젝트 설정 |
| `package-lock.json` | `Cargo.lock` | 잠금 파일 |
| `node_modules/` | `~/.cargo/registry/` | 의존성 캐시 |
| `npx ts-node src/index.ts` | `cargo run --bin name` | 특정 바이너리 실행 |
| `npm publish` | `cargo publish` | 패키지 배포 |
| `npm outdated` | `cargo outdated` | 오래된 의존성 확인 |
| `npx tsc --noEmit` | `cargo check` | 타입 검사만 |
| `.npmrc` | `.cargo/config.toml` | 설정 파일 |
| `npm workspaces` | `cargo workspace` | 모노레포 |
| `jest --watch` | `cargo watch -x test` | 테스트 감시 모드 |

**Cargo.toml vs package.json 비교:**

```json
// package.json
{
  "name": "my-app",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0",
    "prisma": "^5.0.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "jest": "^29.0.0"
  },
  "scripts": {
    "start": "node dist/index.js",
    "dev": "ts-node src/index.ts",
    "test": "jest"
  }
}
```

```toml
# Cargo.toml
[package]
name = "my-app"
version = "1.0.0"
edition = "2021"

[dependencies]
axum = "0.7"           # express에 해당
sqlx = "0.8"           # prisma에 해당

[dev-dependencies]
tokio-test = "0.4"     # jest에 해당 (테스트용만)

# scripts에 해당하는 것은 Makefile이나 cargo-make 사용
# [[bin]] 섹션으로 여러 실행 파일 정의
[[bin]]
name = "server"
path = "src/main.rs"

[[bin]]
name = "migrate"
path = "src/bin/migrate.rs"
```

## 9. NestJS → Axum 패턴 대응표 (상세)

### 모듈 구조

```typescript
// NestJS
@Module({
  imports: [DatabaseModule, AuthModule],
  providers: [EventService, BlockchainService],
  controllers: [EventController],
  exports: [EventService],
})
export class EventModule {}
```

```rust,ignore
// Axum - 별도 모듈 시스템 없음, 수동으로 구성
// src/services/event.rs
pub struct EventService {
    db: PgPool,
    blockchain: Arc<BlockchainService>,
}

// src/core/app.rs
pub struct AppState {
    pub event_service: Arc<EventService>,
    pub blockchain: Arc<BlockchainService>,
}
```

### 컨트롤러와 라우터

```typescript
// NestJS
@Controller('events')
@UseGuards(JwtAuthGuard)
export class EventController {
    constructor(private readonly eventService: EventService) {}

    @Post()
    @HttpCode(HttpStatus.CREATED)
    async create(@Body() dto: CreateEventDto, @Req() req: AuthRequest) {
        return this.eventService.create(req.user.id, dto);
    }

    @Get(':id')
    async findOne(@Param('id') id: string) {
        return this.eventService.findOne(id);
    }
}
```

```rust,ignore
// Axum
pub fn event_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_event))
        .route("/:id", get(get_event))
        // 미들웨어는 라우터 레벨에서 적용
        .route_layer(middleware::from_fn_with_state(
            AppState::placeholder(),
            require_auth,
        ))
}

async fn create_event(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,  // JWT에서 추출
    Json(body): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<EventResponse>), AppError> {
    let event = state.event_service.create(user.id, body).await?;
    Ok((StatusCode::CREATED, Json(event)))
}

async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<EventResponse>, AppError> {
    let event = state.event_service.find_one(&id).await?;
    Ok(Json(event))
}
```

### DTO와 검증

```typescript
// NestJS - class-validator
import { IsString, IsNotEmpty, IsObject, MinLength } from 'class-validator';

export class CreateEventDto {
    @IsString()
    @IsNotEmpty()
    @MinLength(1)
    eventType: string;

    @IsObject()
    payload: Record<string, unknown>;
}
```

```rust,ignore
// Axum - serde + validator 크레이트
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateEventRequest {
    #[validate(length(min = 1, message = "event_type은 비어있을 수 없습니다"))]
    pub event_type: String,
    
    pub payload: serde_json::Value,
}

// 핸들러에서 수동 검증
async fn create_event(
    Json(body): Json<CreateEventRequest>,
) -> Result<Json<TraceEventResponse>, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let response = TraceEventResponse::from_request(body);
    Ok(Json(response))
}

// 또는 커스텀 Extractor로 자동 검증
struct ValidatedJson<T>(T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;
    
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        
        value.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        
        Ok(ValidatedJson(value))
    }
}
```

### 환경변수 설정

```typescript
// NestJS - @nestjs/config
@Injectable()
export class AppConfigService {
    constructor(private configService: ConfigService) {}
    
    get databaseUrl(): string {
        return this.configService.get<string>('DATABASE_URL');
    }
    
    get privateKey(): string {
        return this.configService.getOrThrow('PRIVATE_KEY');
    }
}
```

```rust,ignore
// Axum - dotenvy + std::env
use dotenvy::dotenv;

pub struct Config {
    pub database_url: String,
    pub private_key: String,
    pub rpc_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv().ok();  // .env 파일 로드 (없어도 OK)
        
        Ok(Config {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL 환경변수 없음"))?,
            
            private_key: std::env::var("PRIVATE_KEY")
                .map_err(|_| anyhow::anyhow!("PRIVATE_KEY 환경변수 없음"))?,
            
            rpc_url: std::env::var("RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8545".to_string()),
        })
    }
}
```

### 인터셉터와 Tower 레이어

```typescript
// NestJS Interceptor
@Injectable()
export class LoggingInterceptor implements NestInterceptor {
    intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
        const start = Date.now();
        return next.handle().pipe(
            tap(() => {
                const duration = Date.now() - start;
                console.log(`요청 처리 시간: ${duration}ms`);
            })
        );
    }
}
```

```rust,ignore
// Axum - Tower Layer (tower-http 사용)
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/health", get(health_check))
    .route("/events", post(create_event))
    .layer(TraceLayer::new_for_http()); // 자동으로 요청/응답 로깅

// 커스텀 레이어가 필요하면
use tower::{Layer, Service};

#[derive(Clone)]
struct TimingLayer;

impl<S> Layer<S> for TimingLayer {
    type Service = TimingService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        TimingService { inner }
    }
}
```

## 실수별 빠른 참조 카드

```text
에러: "use of moved value"
해결: & 참조 사용 또는 .clone()

에러: "cannot borrow as mutable"  
해결: &mut 참조 또는 RefCell<T>

에러: "future is not Send"
해결: .await 전에 MutexGuard 해제

에러: "the trait bound is not satisfied"
해결: where T: 트레이트 경계 추가 또는 Arc<dyn Trait>

에러: "expected String, found &str"
해결: .to_string() 또는 .to_owned() 추가

경고: "unused Result"
해결: let _ = result; 또는 .ok() 또는 ? 연산자

경고: "unnecessary clone"
해결: & 참조로 전달 가능한지 확인
```

## 요약

Node.js에서 Rust로 전환 시 가장 중요한 10가지:

1. **소유권**: 한 번에 한 소유자, 이동 후 사용 불가 → `&` 참조 사용
2. **unwrap 금지**: `?` 연산자와 `match`로 에러 처리
3. **String vs &str**: 함수 파라미터는 `&str`, 구조체 필드는 `String`
4. **정수 연산**: 금융/토큰 계산은 반드시 `U256`, `u128`, `Decimal`
5. **async Mutex**: `tokio::sync::Mutex` 사용, `.await` 전에 `std::sync::Mutex` 해제
6. **Result 체인**: `?`, `map_err`, `and_then`으로 에러 전파
7. **cargo check**: 빠른 타입 검사로 개발 속도 향상
8. **cargo 명령어**: npm 대신 cargo, 패키지는 crates.io
9. **Axum 패턴**: State extractor, Extension extractor, Router::new()
10. **컴파일러 믿기**: 에러가 나면 억지로 고치지 말고 메시지를 읽어라

Rust 컴파일러는 엄격하지만 친절하다. 에러 메시지에 해결책이 힌트로 들어있는 경우가 많다.
