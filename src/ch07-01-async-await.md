# 7.1 async/await와 Future

## async fn: 비동기 함수

`async fn`으로 정의한 함수는 `Future`를 반환합니다:

```rust
// 동기 함수
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// 비동기 함수
async fn greet_async(name: &str) -> String {
    // 네트워크 요청, 파일 읽기 등 I/O 작업을 여기서
    format!("Hello, {}!", name)
}
```

`async fn foo() -> T`는 실제로 `fn foo() -> impl Future<Output = T>`로 변환됩니다.

TypeScript와 비교:

```typescript
// TypeScript: async fn은 Promise를 반환
async function greetAsync(name: string): Promise<string> {
    return `Hello, ${name}!`;
}
```

```rust
// Rust: async fn은 Future를 반환 (Future ≈ Promise)
async fn greet_async(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

---

## Future 트레이트

`Future`는 아직 완료되지 않은 비동기 계산을 나타냅니다:

```rust,ignore
// 표준 라이브러리에 이렇게 정의
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),    // 완료됨
    Pending,     // 아직 진행 중
}
```

`Future`는 `poll()`이 호출될 때만 진행됩니다. 생성만으로는 실행되지 않습니다. 이것이 Node.js의 `Promise`와 가장 큰 차이점입니다.

### Promise vs Future

```typescript
// TypeScript Promise: 생성 즉시 실행 시작
const promise = fetchData();  // 이미 실행 중!
// 나중에 await하든 안 하든 실행됨
```

```rust,ignore
// Rust Future: await하기 전까지 실행 안 됨
let future = fetch_data();  // 아직 실행 안 됨!
let result = future.await;  // 여기서 실행 시작
```

---

## .await 사용법

```rust,ignore
async fn fetch_block(height: u64) -> Result<Block, String> {
    // 네트워크 요청 (비동기)
    let url = format!("https://api.blockchain.com/blocks/{}", height);
    // 실제 HTTP 요청은 reqwest 등 사용
    Ok(Block { index: height, hash: String::from("abc") })
}

async fn get_latest_blocks() -> Vec<Block> {
    let mut blocks = Vec::new();

    // 순차 실행 — 하나씩
    let block1 = fetch_block(1).await;  // 완료 후
    let block2 = fetch_block(2).await;  // 실행

    if let Ok(b) = block1 { blocks.push(b); }
    if let Ok(b) = block2 { blocks.push(b); }

    blocks
}

struct Block { index: u64, hash: String }
```

### .await는 async fn 안에서만

```rust,ignore
// 에러! async 아닌 함수에서 .await 사용 불가
fn not_async() {
    let result = fetch_block(1).await;  // 컴파일 에러
}

// 해결책 1: async fn으로 만들기
async fn is_async() {
    let result = fetch_block(1).await;  // OK
}

// 해결책 2: 블로킹 실행 (런타임 필요)
fn blocking_main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(fetch_block(1));
}
```

---

## 실행자(Executor)가 필요한 이유

`Future`는 혼자 실행될 수 없습니다. **실행자(executor)**가 필요합니다. 실행자는 `Future`를 `poll()`로 구동시키는 스케줄러입니다.

```text
Future ──────────────────► Executor
(비동기 계산의 명세)         (실제 실행하는 주체)

poll() → Pending ──────► 다른 Future 실행 (I/O 대기 중)
poll() → Ready(T) ──────► 결과 반환
```

Node.js에서는 V8 엔진과 libuv가 이벤트 루프를 기본으로 제공합니다. Rust에서는 Tokio 등 외부 런타임이 이 역할을 합니다.

```rust,ignore
// Tokio 런타임이 Future를 실행
#[tokio::main]
async fn main() {
    // tokio::main 매크로가 Tokio 런타임을 설정하고
    // main() Future를 실행자에 제출
    let result = fetch_block(1).await;
}

// 매크로 없이 직접 런타임 생성
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let result = fetch_block(1).await;
        println!("{:?}", result);
    });
}
```

---

## async 블록

함수 전체가 아닌 일부만 비동기로 만들 때:

```rust,ignore
fn main() {
    let future = async {
        // 이 블록은 async 컨텍스트
        let block = fetch_block(1).await;
        block
    };
    // future는 아직 실행 안 됨

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(future);
}
```

---

## 병렬 실행

### tokio::join! — 여러 Future 동시 실행

```rust,ignore
use tokio;

async fn fetch_block(height: u64) -> String {
    // 네트워크 요청 시뮬레이션
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    format!("block_{}", height)
}

#[tokio::main]
async fn main() {
    // 순차 실행 — 300ms 소요
    let b1 = fetch_block(1).await;
    let b2 = fetch_block(2).await;
    let b3 = fetch_block(3).await;

    // 병렬 실행 — 100ms 소요 (가장 느린 것만큼)
    let (b1, b2, b3) = tokio::join!(
        fetch_block(1),
        fetch_block(2),
        fetch_block(3),
    );

    println!("{}, {}, {}", b1, b2, b3);
}
```

TypeScript와 비교:

```typescript
// TypeScript
const [b1, b2, b3] = await Promise.all([
    fetchBlock(1),
    fetchBlock(2),
    fetchBlock(3),
]);
```

### tokio::select! — 가장 먼저 완료되는 것 선택

```rust,ignore
use tokio;

#[tokio::main]
async fn main() {
    // 두 작업 중 먼저 끝나는 것 선택 (나머지는 취소)
    tokio::select! {
        result = fetch_block(1) => {
            println!("Block 1 finished first: {}", result);
        }
        result = fetch_from_backup(1) => {
            println!("Backup finished first: {}", result);
        }
    }
}

async fn fetch_from_backup(height: u64) -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    format!("backup_block_{}", height)
}
```

TypeScript와 비교:

```typescript
// TypeScript
const result = await Promise.race([
    fetchBlock(1),
    fetchFromBackup(1),
]);
```

---

## 에러 처리와 async

```rust,ignore
use reqwest;
use serde_json::Value;

async fn fetch_block_data(height: u64) -> Result<Value, reqwest::Error> {
    let url = format!("https://api.blockcypher.com/v1/btc/main/blocks/{}", height);
    let response = reqwest::get(&url).await?;  // ?로 에러 전파
    let json = response.json::<Value>().await?;
    Ok(json)
}

#[tokio::main]
async fn main() {
    match fetch_block_data(100).await {
        Ok(data) => println!("Block: {:?}", data),
        Err(e)   => eprintln!("Error: {}", e),
    }

    // ?와 함께 사용
    // main이 Result를 반환하면 ?를 main에서도 사용 가능
}
```

---

## Node.js와 Rust 비동기 비교 정리

| 개념 | Node.js/TypeScript | Rust |
|------|-------------------|------|
| 비동기 함수 | `async function f(): Promise<T>` | `async fn f() -> T` (impl Future<Output=T>) |
| 값 꺼내기 | `await promise` | `future.await` |
| 병렬 실행 | `Promise.all([...])` | `tokio::join!(...)` |
| 경쟁 실행 | `Promise.race([...])` | `tokio::select! { ... }` |
| 에러 처리 | `try/catch` 또는 `.catch()` | `?` 연산자, `match` |
| 런타임 | V8 + libuv (내장) | Tokio 등 (선택) |
| 실행 모델 | 싱글 스레드 이벤트 루프 | 멀티 스레드 (기본) |
| 즉시 실행 | Promise 생성 즉시 | `.await` 호출 시 |

---

## 요약

- `async fn`: `Future`를 반환하는 함수 — `.await`로 완료 대기
- `Future`: 비동기 계산의 명세 — `poll()`이 호출될 때만 진행
- 실행자(Executor): `Future`를 구동하는 스케줄러 — Tokio 등이 담당
- `async fn` 안에서만 `.await` 사용 가능
- `tokio::join!`: 여러 Future 병렬 실행 (`Promise.all`)
- `tokio::select!`: 가장 먼저 완료되는 것 선택 (`Promise.race`)
- Promise와 달리 Future는 `.await` 전까지 실행 안 됨 (lazy)

다음 챕터에서 Tokio 런타임을 자세히 배웁니다.
