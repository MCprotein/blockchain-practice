# 7.2 Tokio 런타임

## Tokio 설치

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

`features = ["full"]`은 모든 Tokio 기능을 활성화합니다. 프로덕션에서는 필요한 기능만 선택합니다:

```toml
# 세밀한 기능 선택
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "time", "sync", "io-util"] }
```

---

## #[tokio::main]

```rust
use tokio;

#[tokio::main]
async fn main() {
    println!("Hello from async main!");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("One second later");
}
```

`#[tokio::main]` 매크로는 다음으로 확장됩니다:

```rust
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            println!("Hello from async main!");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            println!("One second later");
        })
}
```

---

## tokio::spawn: 비동기 태스크 생성

`tokio::spawn`은 새 태스크를 생성합니다. Node.js의 `Promise` 즉시 실행과 유사합니다:

```rust
use tokio;

async fn process_transaction(id: u64) -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    format!("tx_{} processed", id)
}

#[tokio::main]
async fn main() {
    // 태스크 생성 — 즉시 실행 시작 (백그라운드)
    let handle1 = tokio::spawn(process_transaction(1));
    let handle2 = tokio::spawn(process_transaction(2));
    let handle3 = tokio::spawn(process_transaction(3));

    // 각 태스크의 결과 기다리기
    let r1 = handle1.await.unwrap();  // JoinHandle.await → Result<T, JoinError>
    let r2 = handle2.await.unwrap();
    let r3 = handle3.await.unwrap();

    println!("{}, {}, {}", r1, r2, r3);
}
```

TypeScript와 비교:

```typescript
// TypeScript
async function processTransaction(id: number): Promise<string> {
    await sleep(100);
    return `tx_${id} processed`;
}

const p1 = processTransaction(1);  // 즉시 시작
const p2 = processTransaction(2);
const p3 = processTransaction(3);

const [r1, r2, r3] = await Promise.all([p1, p2, p3]);
```

### spawn 주의사항

```rust
#[tokio::main]
async fn main() {
    let data = String::from("hello");

    // data를 스폰된 태스크로 이동 (move 필요)
    let handle = tokio::spawn(async move {
        println!("In task: {}", data);
        // data의 소유권이 이 태스크로 이동
    });

    // println!("{}", data);  // 에러! 이동됨

    handle.await.unwrap();
}
```

스폰된 태스크는 `'static` 수명을 요구합니다. 즉, 캡처하는 모든 변수는 소유되거나 `'static`이어야 합니다.

---

## tokio::time: 타이머

```rust
use tokio::time::{sleep, Duration, timeout, interval};

#[tokio::main]
async fn main() {
    // sleep: N초 대기
    sleep(Duration::from_secs(1)).await;
    sleep(Duration::from_millis(500)).await;

    // timeout: N초 안에 완료되지 않으면 에러
    let result = timeout(
        Duration::from_secs(5),
        fetch_block(100),  // 이 future가 5초 안에 완료되어야 함
    ).await;

    match result {
        Ok(block) => println!("Got block"),
        Err(_)    => println!("Timed out!"),
    }

    // interval: 주기적 실행
    let mut ticker = interval(Duration::from_secs(1));
    for _ in 0..5 {
        ticker.tick().await;  // 1초마다 실행
        println!("Tick!");
    }
}

async fn fetch_block(height: u64) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("block_{}", height)
}
```

---

## 채널 (Channels)

채널은 태스크 간 메시지 전달에 사용합니다. Node.js에는 직접적인 대응이 없지만, `EventEmitter`나 `Queue`와 유사합니다.

### mpsc: 다수 송신자, 단일 수신자

Multi-Producer Single-Consumer — 가장 흔한 패턴입니다.

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // 버퍼 크기 32인 채널 생성
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // 여러 송신자 (tx.clone()으로 복제)
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    drop(tx);  // 원본 tx 드롭 (남은 sender가 없으면 rx는 None을 받음)

    // 송신자 태스크 1
    tokio::spawn(async move {
        for i in 0..3 {
            tx1.send(format!("task1: tx_{}", i)).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });

    // 송신자 태스크 2
    tokio::spawn(async move {
        for i in 0..3 {
            tx2.send(format!("task2: tx_{}", i)).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(75)).await;
        }
    });

    // 수신자: 모든 메시지 처리
    while let Some(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }
    println!("All senders dropped, channel closed");
}
```

### oneshot: 단일 응답

```rust
use tokio::sync::oneshot;

async fn compute_hash(data: String, responder: oneshot::Sender<String>) {
    let hash = format!("{:x}", data.len());  // 실제로는 SHA-256
    responder.send(hash).unwrap();
}

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel::<String>();

    tokio::spawn(compute_hash(String::from("block_data"), tx));

    let hash = rx.await.unwrap();
    println!("Hash: {}", hash);
}
```

### 블록체인에서의 채널 패턴

```rust
use tokio::sync::mpsc;

#[derive(Debug)]
enum MinerCommand {
    StartMining { data: String, difficulty: usize },
    StopMining,
}

#[derive(Debug)]
struct MinedBlock {
    data: String,
    hash: String,
    nonce: u64,
}

async fn miner_task(mut cmd_rx: mpsc::Receiver<MinerCommand>, result_tx: mpsc::Sender<MinedBlock>) {
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            MinerCommand::StartMining { data, difficulty } => {
                println!("Mining with difficulty {}...", difficulty);
                let target = "0".repeat(difficulty);
                let mut nonce = 0u64;

                loop {
                    let hash = format!("{:x}", data.len() + nonce as usize);
                    if hash.starts_with(&target) {
                        let block = MinedBlock { data: data.clone(), hash, nonce };
                        result_tx.send(block).await.unwrap();
                        break;
                    }
                    nonce += 1;

                    // CPU 독점 방지 — 주기적으로 다른 태스크에 양보
                    if nonce % 1000 == 0 {
                        tokio::task::yield_now().await;
                    }
                }
            }
            MinerCommand::StopMining => {
                println!("Mining stopped");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (cmd_tx, cmd_rx) = mpsc::channel(10);
    let (result_tx, mut result_rx) = mpsc::channel(10);

    // 마이너 태스크 시작
    tokio::spawn(miner_task(cmd_rx, result_tx));

    // 마이닝 명령 전송
    cmd_tx.send(MinerCommand::StartMining {
        data: String::from("Block 1 data"),
        difficulty: 1,
    }).await.unwrap();

    // 결과 수신
    if let Some(block) = result_rx.recv().await {
        println!("Mined! Hash: {}, Nonce: {}", block.hash, block.nonce);
    }
}
```

---

## HTTP 요청: reqwest

```rust
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct BlockInfo {
    height: u64,
    hash: String,
    time: u64,
    n_tx: u32,
}

#[derive(Debug, Deserialize)]
struct BitcoinBlockResponse {
    height: u64,
    hash: String,
}

async fn get_latest_bitcoin_block() -> Result<BitcoinBlockResponse, reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://blockchain.info/latestblock")
        .header("User-Agent", "rust-blockchain-learner/1.0")
        .send()
        .await?
        .json::<BitcoinBlockResponse>()
        .await?;

    Ok(response)
}

async fn post_transaction(tx_data: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.example.com/transactions")
        .header("Content-Type", "application/json")
        .body(tx_data.to_string())
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if status.is_success() {
        Ok(body)
    } else {
        Err(reqwest::Error::from(
            // 실제로는 커스텀 에러 타입 사용
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match get_latest_bitcoin_block().await {
        Ok(block) => println!("Latest block: height={}, hash={}", block.height, block.hash),
        Err(e)    => eprintln!("Failed: {}", e),
    }

    Ok(())
}
```

### reqwest 클라이언트 재사용

```rust
use reqwest::Client;
use std::sync::Arc;

// 클라이언트를 Arc로 공유 (커넥션 풀 재사용)
#[derive(Clone)]
struct BlockchainClient {
    http: Arc<Client>,
    base_url: String,
}

impl BlockchainClient {
    fn new(base_url: String) -> Self {
        BlockchainClient {
            http: Arc::new(Client::new()),
            base_url,
        }
    }

    async fn get_block(&self, height: u64) -> Result<serde_json::Value, reqwest::Error> {
        let url = format!("{}/blocks/{}", self.base_url, height);
        self.http.get(&url).send().await?.json().await
    }
}

#[tokio::main]
async fn main() {
    let client = BlockchainClient::new("https://api.blockchain.com/v3/btc".to_string());

    // 여러 태스크에서 공유
    let handles: Vec<_> = (0..5).map(|i| {
        let c = client.clone();  // Arc 클론 — 저렴함
        tokio::spawn(async move {
            match c.get_block(i).await {
                Ok(block) => println!("Block {}: {:?}", i, block),
                Err(e)    => eprintln!("Error block {}: {}", i, e),
            }
        })
    }).collect();

    for h in handles {
        h.await.unwrap();
    }
}
```

---

## Express/NestJS와 Axum 비교

```typescript
// NestJS
@Controller('blocks')
export class BlockController {
    constructor(private blockService: BlockService) {}

    @Get(':height')
    async getBlock(@Param('height') height: string): Promise<BlockDto> {
        return this.blockService.findByHeight(parseInt(height));
    }

    @Post()
    async addBlock(@Body() dto: CreateBlockDto): Promise<BlockDto> {
        return this.blockService.create(dto);
    }
}
```

```rust
// Axum (Rust의 웹 프레임워크)
use axum::{
    routing::{get, post},
    Router, Json, Path,
    extract::State,
};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    blockchain: Arc<tokio::sync::RwLock<Blockchain>>,
}

async fn get_block(
    Path(height): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<Block>, String> {
    let chain = state.blockchain.read().await;
    chain.get_block(height)
        .map(|b| Json(b.clone()))
        .ok_or_else(|| format!("Block {} not found", height))
}

async fn add_block(
    State(state): State<AppState>,
    Json(data): Json<CreateBlockRequest>,
) -> Result<Json<Block>, String> {
    let mut chain = state.blockchain.write().await;
    chain.add_block(data.data)
        .map(|b| Json(b.clone()))
        .map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() {
    let state = AppState {
        blockchain: Arc::new(tokio::sync::RwLock::new(Blockchain::new())),
    };

    let app = Router::new()
        .route("/blocks/:height", get(get_block))
        .route("/blocks", post(add_block))
        .with_state(state);

    println!("Server running on http://0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

struct Blockchain { blocks: Vec<Block> }
impl Blockchain {
    fn new() -> Self { Blockchain { blocks: vec![] } }
    fn get_block(&self, height: u64) -> Option<&Block> { self.blocks.get(height as usize) }
    fn add_block(&mut self, data: String) -> Result<&Block, String> {
        let block = Block { index: self.blocks.len() as u64, data, hash: String::from("abc") };
        self.blocks.push(block);
        Ok(self.blocks.last().unwrap())
    }
}

#[derive(Clone, serde::Serialize)]
struct Block { index: u64, data: String, hash: String }

#[derive(serde::Deserialize)]
struct CreateBlockRequest { data: String }
```

---

## 요약

- `#[tokio::main]`: 비동기 main 함수를 위한 매크로
- `tokio::spawn`: 새 태스크 생성 (백그라운드 실행)
- `tokio::time::sleep`: 비동기 대기
- `tokio::time::timeout`: 시간 제한 설정
- `mpsc` 채널: 다수 송신자, 단일 수신자
- `oneshot` 채널: 단일 요청-응답 패턴
- `reqwest`: 비동기 HTTP 클라이언트
- 웹 서버: Axum (NestJS/Express 대응)

다음 챕터에서 스레드 간 안전한 상태 공유를 배웁니다.
