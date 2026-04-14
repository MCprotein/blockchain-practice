# 4장: 에러 처리

## Rust의 에러 처리 철학

Rust에는 예외(exception)가 없습니다. `try/catch`가 없습니다. `throw`가 없습니다.

이것은 버그가 아니라 의도적인 설계입니다.

### 왜 예외를 없앴는가?

TypeScript/Node.js에서 예외 기반 에러 처리의 문제점:

```typescript
// 이 함수가 에러를 던질 수 있다는 걸 타입에서 알 수 없음
async function fetchBlock(height: number): Promise<Block> {
    const response = await fetch(`/api/blocks/${height}`);
    const block = await response.json();
    return block;
}

// 호출하는 쪽에서 try/catch를 반드시 써야 하는지 모름
const block = await fetchBlock(100);  // 예외가 날 수 있는지 알 수 없음
```

```typescript
// try/catch를 써도 에러 타입이 unknown
try {
    const block = await fetchBlock(100);
} catch (error) {
    // error는 any/unknown 타입
    // 어떤 에러인지 타입 체크해야 함
    if (error instanceof NetworkError) { /* ... */ }
    else if (error instanceof ParseError) { /* ... */ }
}
```

**문제점:**
1. 함수 시그니처만으로 에러 가능성을 알 수 없음
2. 예외를 처리하지 않아도 컴파일 에러 없음 (런타임에서야 발견)
3. 에러 타입이 타입 시스템에서 보장되지 않음

### Rust의 해결책: 에러는 값이다

```rust
// 반환 타입에 에러 가능성이 명시됨
fn fetch_block(height: u64) -> Result<Block, NetworkError> {
    // ...
}

// 호출하는 쪽에서 에러를 반드시 처리해야 함
let block = fetch_block(100);  // Result<Block, NetworkError>
// block을 바로 사용하려면 에러 처리 필요

match block {
    Ok(b)  => println!("Got block: {}", b.hash),
    Err(e) => println!("Failed: {}", e),
}
```

**장점:**
1. 함수 시그니처에서 에러 가능성이 보임
2. 에러를 처리하지 않으면 컴파일러 경고/에러
3. 에러 타입이 명확히 지정됨

## Rust의 두 가지 에러 종류

### 1. 복구 불가능한 에러: `panic!`

프로그래밍 버그, 불변식 위반 등 계속 실행이 의미 없는 상황:

```rust
fn get_block(index: usize) -> &Block {
    // 인덱스가 범위를 벗어나면 panic (버그)
    &blocks[index]  // 범위 초과 시 panic!
}
```

### 2. 복구 가능한 에러: `Result<T, E>`

파일 없음, 네트워크 에러, 파싱 실패 등 정상적인 에러 상황:

```rust
fn parse_block(json: &str) -> Result<Block, ParseError> {
    // 파싱 실패는 예상된 상황 — Result로 처리
    serde_json::from_str(json).map_err(|e| ParseError::Json(e))
}
```

## 이 장의 구성

1. **panic!** (4.1): 언제 쓰고, 블록체인에서 왜 위험한가
2. **Result\<T, E\>** (4.2): Ok/Err, unwrap, 커스텀 에러 타입
3. **에러 전파** (4.3): `?` 연산자, From 트레이트

## NestJS와 비교

```typescript
// NestJS 에러 처리
@Get('/block/:height')
async getBlock(@Param('height') height: string): Promise<BlockDto> {
    const h = parseInt(height);
    if (isNaN(h)) {
        throw new BadRequestException('Invalid block height');
    }
    const block = await this.blockService.findByHeight(h);
    if (!block) {
        throw new NotFoundException(`Block at height ${h} not found`);
    }
    return block;
}
// HttpException을 던지면 NestJS가 자동으로 적절한 HTTP 응답으로 변환
```

```rust
// Rust 에러 처리 (axum 사용)
async fn get_block(Path(height): Path<u64>) -> Result<Json<Block>, AppError> {
    let block = block_service::find_by_height(height).await
        .map_err(AppError::Database)?;

    match block {
        Some(b) => Ok(Json(b)),
        None => Err(AppError::NotFound(format!("Block {} not found", height))),
    }
}
// Result<Json<Block>, AppError>가 자동으로 HTTP 응답으로 변환
```

핵심 차이: Rust는 에러가 반환 타입에 명시되고, 처리를 강제합니다.

다음 챕터에서 `panic!`부터 시작합니다.
