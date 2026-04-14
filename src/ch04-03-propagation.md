# 4.3 에러 전파와 ? 연산자

## ? 연산자

`?` 연산자는 `Result`와 `Option`에서 에러를 상위 함수로 전파하는 단축 문법입니다.

### 기본 동작

```rust
// ? 없이 — 장황한 코드
fn read_block_from_file(path: &str) -> Result<Block, std::io::Error> {
    let content = match std::fs::read_to_string(path) {
        Ok(c)  => c,
        Err(e) => return Err(e),  // 에러면 즉시 반환
    };
    let block = match serde_json::from_str(&content) {
        Ok(b)  => b,
        Err(e) => return Err(e),  // 에러면 즉시 반환 (타입이 다르므로 실제로는 변환 필요)
    };
    Ok(block)
}

// ? 사용 — 간결한 코드
fn read_block_from_file(path: &str) -> Result<Block, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;  // 에러면 즉시 return Err(e)
    let block = serde_json::from_str(&content)?;   // 에러면 즉시 return Err(e)
    Ok(block)
}
```

`?`는 다음과 동등합니다:

```rust
// 이 두 코드는 동일
let value = some_result?;

let value = match some_result {
    Ok(v)  => v,
    Err(e) => return Err(e.into()),  // .into()로 에러 타입 변환
};
```

---

## ? 연산자의 에러 타입 변환

`?`는 내부적으로 `From` 트레이트를 이용해 에러 타입을 변환합니다.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),          // std::io::Error → AppError 자동 변환

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),     // serde_json::Error → AppError 자동 변환

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}

fn load_blockchain(path: &str) -> Result<Blockchain, AppError> {
    // std::io::Error → AppError::Io 자동 변환 (?가 From::from 호출)
    let content = std::fs::read_to_string(path)?;

    // serde_json::Error → AppError::Json 자동 변환
    let chain: Blockchain = serde_json::from_str(&content)?;

    Ok(chain)
}
```

### From 트레이트 직접 구현

```rust
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
enum MyError {
    Io(io::Error),
    Parse(ParseIntError),
}

// io::Error → MyError 변환 구현
impl From<io::Error> for MyError {
    fn from(e: io::Error) -> Self {
        MyError::Io(e)
    }
}

// ParseIntError → MyError 변환 구현
impl From<ParseIntError> for MyError {
    fn from(e: ParseIntError) -> Self {
        MyError::Parse(e)
    }
}

fn load_height_from_file(path: &str) -> Result<u64, MyError> {
    let content = std::fs::read_to_string(path)?;  // io::Error → MyError::Io
    let height = content.trim().parse::<u64>()?;   // ParseIntError → MyError::Parse
    Ok(height)
}
```

---

## Option에서 ?

`?`는 `Option`에서도 동작합니다:

```rust
fn get_first_block_hash(chain: &Blockchain) -> Option<&str> {
    let first = chain.blocks.first()?;  // None이면 즉시 return None
    let hash = first.hash.get(..8)?;    // None이면 즉시 return None
    Some(hash)
}
```

단, `Option`을 반환하는 함수에서만 `?`를 쓸 수 있습니다. `Result`를 반환하는 함수에서 `Option`에 `?`를 쓰려면 변환이 필요합니다:

```rust
fn process(chain: &Blockchain) -> Result<String, AppError> {
    let first = chain.blocks.first()
        .ok_or(AppError::EmptyChain)?;  // Option → Result 변환 후 ?
    Ok(first.hash.clone())
}
```

---

## main 함수에서 Result 반환

`main` 함수도 `Result`를 반환할 수 있습니다:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let content = std::fs::read_to_string("blockchain.json")?;
    let chain: Blockchain = serde_json::from_str(&content)?;
    println!("Loaded {} blocks", chain.blocks.len());
    Ok(())
}
```

에러가 발생하면 프로그램이 에러 메시지를 출력하고 종료합니다:

```
Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

`anyhow`를 사용하면 더 나은 에러 출력:

```rust
fn main() -> anyhow::Result<()> {
    let content = std::fs::read_to_string("blockchain.json")
        .context("Failed to open blockchain.json")?;

    let chain: Blockchain = serde_json::from_str(&content)
        .context("Invalid JSON in blockchain.json")?;

    println!("Loaded {} blocks", chain.blocks.len());
    Ok(())
}
```

---

## NestJS의 HttpException과 비교

NestJS에서 에러 처리 패턴:

```typescript
// NestJS — 예외를 던지고 글로벌 필터가 처리
@Injectable()
export class BlockService {
    async findByHeight(height: number): Promise<Block> {
        const block = await this.repo.findOne({ where: { height } });
        if (!block) {
            throw new NotFoundException(`Block at height ${height} not found`);
        }
        return block;
    }

    async addBlock(data: CreateBlockDto): Promise<Block> {
        try {
            const block = this.repo.create(data);
            return await this.repo.save(block);
        } catch (error) {
            if (error.code === '23505') {  // unique violation
                throw new ConflictException('Block already exists');
            }
            throw new InternalServerErrorException('Database error');
        }
    }
}
```

```rust
// Rust + axum — Result를 반환하고 IntoResponse 구현으로 HTTP 변환
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Block at height {0} not found")]
    NotFound(u64),

    #[error("Block already exists")]
    Conflict,

    #[error("Database error: {0}")]
    Database(String),
}

// AppError → HTTP 응답 자동 변환
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(h)  => (StatusCode::NOT_FOUND, format!("Block {} not found", h)),
            AppError::Conflict     => (StatusCode::CONFLICT, self.to_string()),
            AppError::Database(e)  => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

async fn get_block(Path(height): Path<u64>) -> Result<Json<Block>, AppError> {
    let block = db::find_block(height).await
        .map_err(|e| AppError::Database(e.to_string()))?;

    block.ok_or(AppError::NotFound(height)).map(Json)
}

async fn add_block(Json(data): Json<CreateBlockRequest>) -> Result<Json<Block>, AppError> {
    let block = db::insert_block(data).await
        .map_err(|e| match e.kind() {
            db::ErrorKind::UniqueViolation => AppError::Conflict,
            _ => AppError::Database(e.to_string()),
        })?;

    Ok(Json(block))
}
```

**비교 정리:**

| NestJS | Rust (axum) |
|--------|-------------|
| `throw new NotFoundException(...)` | `return Err(AppError::NotFound(...))` |
| `@UseFilters(HttpExceptionFilter)` | `impl IntoResponse for AppError` |
| 에러 타입이 런타임에 결정 | 에러 타입이 컴파일 타임에 결정 |
| try/catch로 처리 | `?`로 전파, `match`로 처리 |

---

## 에러 처리 체이닝 패턴

실제 코드에서 여러 에러 처리를 체이닝하는 패턴:

```rust
use anyhow::{Context, Result};

async fn process_new_block(
    blockchain: &mut Blockchain,
    tx_data: &str,
) -> Result<String> {
    // 1. 트랜잭션 데이터 검증
    let tx = parse_transaction(tx_data)
        .context("Failed to parse transaction data")?;

    // 2. 잔액 확인
    let balance = get_balance(&tx.from).await
        .with_context(|| format!("Failed to get balance for {}", tx.from))?;

    if balance < tx.amount {
        anyhow::bail!(
            "Insufficient balance: {} < {}",
            balance,
            tx.amount
        );
    }

    // 3. 블록 추가
    let block = blockchain.add_block(tx_data.to_string())
        .context("Failed to add block to chain")?;

    // 4. 영속화
    save_blockchain(blockchain).await
        .context("Failed to save blockchain")?;

    Ok(block.hash.clone())
}
```

### 에러 로깅 패턴

```rust
use tracing::{error, warn, info};

async fn handle_transaction(tx: Transaction) {
    match process_transaction(&tx).await {
        Ok(result) => {
            info!("Transaction {} processed: {:?}", tx.id, result);
        }
        Err(e) => {
            // 에러 체인 전체 출력
            error!("Transaction {} failed: {:#}", tx.id, e);

            // 에러 종류에 따른 다른 처리
            if let Some(db_err) = e.downcast_ref::<DatabaseError>() {
                // DB 에러면 재시도
                warn!("DB error, will retry: {}", db_err);
            }
        }
    }
}
```

---

## 요약

- `?` 연산자: `Result`/`Option`의 에러를 상위 함수로 전파하는 단축 문법
- `?`는 내부적으로 `From::from()`을 호출해 에러 타입을 변환
- `#[from]` (thiserror): 에러 타입 변환 자동 구현
- `main` 함수도 `Result<(), E>`를 반환할 수 있음
- `Option`에서도 `?` 사용 가능 (Option을 반환하는 함수에서)
- NestJS의 예외 기반 에러처리 vs Rust의 반환값 기반 에러 처리

다음 장에서 제네릭과 트레이트를 배웁니다.
