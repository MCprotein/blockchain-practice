# 7.3 공유 상태: Arc, Mutex, RwLock

## 왜 공유 상태가 필요한가?

여러 비동기 태스크나 스레드가 같은 데이터에 접근해야 하는 경우가 있습니다:
- 블록체인 상태를 여러 RPC 핸들러가 읽고 쓰기
- 메모리 풀(mempool)에 여러 태스크가 동시에 트랜잭션 추가
- 캐시를 여러 요청 핸들러가 공유

Node.js는 싱글 스레드이기 때문에 이런 문제가 없었습니다. Rust의 비동기 런타임(Tokio)은 멀티 스레드로 동작하므로 데이터 레이스를 방지해야 합니다.

---

## 소유권과 스레드 안전성

```rust
// 이건 불가능 — 소유권은 하나
fn main() {
    let data = String::from("blockchain data");

    let t1 = std::thread::spawn(|| {
        println!("{}", data);  // data 캡처 시도
    });

    let t2 = std::thread::spawn(|| {
        println!("{}", data);  // 에러! data는 이미 t1이 가져감
    });
}
```

해결책: 여러 소유자를 허용하는 `Arc<T>`를 사용합니다.

---

## Arc\<T\>: 스레드 간 공유 소유권

`Arc`는 Atomically Reference Counted — 스레드 안전한 참조 카운팅 스마트 포인터입니다.

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);

    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Arc::clone(&data);  // 참조 카운트 증가 (데이터 복사 없음)
        let handle = thread::spawn(move || {
            println!("Thread {}: {:?}", i, data_clone);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 여기서도 data 사용 가능
    println!("Main: {:?}", data);
}
```

`Arc::clone()`은 데이터를 복사하지 않습니다. 내부 참조 카운터만 증가합니다. 마지막 `Arc`가 드롭될 때 데이터가 해제됩니다.

`Rc<T>`(단일 스레드 참조 카운팅) vs `Arc<T>`(멀티 스레드):
- `Rc<T>`: 싱글 스레드 전용, 더 빠름
- `Arc<T>`: 스레드 간 공유 가능, 원자적 연산으로 약간 느림

---

## Mutex\<T\>: 상호 배제

`Arc`는 불변 데이터를 공유합니다. 데이터를 수정하려면 `Mutex`가 필요합니다.

`Mutex`(Mutual Exclusion)는 한 번에 하나의 스레드만 접근할 수 있도록 잠금을 제공합니다.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0u64));

    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter_clone.lock().unwrap();  // 잠금 획득
            *num += 1;
            // num이 스코프를 벗어나면 잠금 자동 해제 (MutexGuard의 Drop)
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Counter: {}", *counter.lock().unwrap());  // 10
}
```

### Mutex의 잠금 획득과 해제

```rust
use std::sync::Mutex;

fn main() {
    let m = Mutex::new(5);

    {
        let mut val = m.lock().unwrap();  // 잠금 획득 → MutexGuard
        *val = 6;
        println!("val = {}", *val);
    }   // MutexGuard 드롭 → 잠금 자동 해제

    println!("m = {:?}", m);  // Mutex { data: 6 }

    // 잠금 상태 확인
    if let Ok(guard) = m.try_lock() {
        println!("Got lock: {}", *guard);
    } else {
        println!("Lock is held by another thread");
    }
}
```

### 데드락 주의

```rust
use std::sync::{Arc, Mutex};

// 데드락 예시 — 주의!
fn deadlock_example() {
    let lock1 = Arc::new(Mutex::new(1));
    let lock2 = Arc::new(Mutex::new(2));

    let l1 = Arc::clone(&lock1);
    let l2 = Arc::clone(&lock2);

    // 스레드 A: lock1 → lock2 순서
    let t1 = std::thread::spawn(move || {
        let _g1 = l1.lock().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _g2 = l2.lock().unwrap();  // lock2 대기 중
    });

    // 스레드 B: lock2 → lock1 순서 — 데드락!
    let t2 = std::thread::spawn(move || {
        let _g2 = lock2.lock().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _g1 = lock1.lock().unwrap();  // lock1 대기 중
    });

    // t1은 lock2를 기다리고, t2는 lock1을 기다림 — 영원히 대기
    t1.join().unwrap();
    t2.join().unwrap();
}

// 해결책: 항상 같은 순서로 잠금 획득
fn safe_locking() {
    let lock1 = Arc::new(Mutex::new(1));
    let lock2 = Arc::new(Mutex::new(2));

    // 두 스레드 모두 lock1 → lock2 순서로 잠금
    // 데드락 없음
}
```

---

## RwLock\<T\>: 읽기/쓰기 락

`Mutex`는 읽기도 독점합니다. 읽기 작업이 많고 쓰기가 드문 경우 `RwLock`이 효율적입니다.

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let blockchain = Arc::new(RwLock::new(vec!["genesis".to_string()]));

    let mut handles = vec![];

    // 여러 읽기 스레드 — 동시에 접근 가능
    for i in 0..5 {
        let chain = Arc::clone(&blockchain);
        let handle = thread::spawn(move || {
            let chain_ref = chain.read().unwrap();  // 읽기 잠금 (여러 동시 가능)
            println!("Reader {}: {} blocks", i, chain_ref.len());
            // 잠금 자동 해제
        });
        handles.push(handle);
    }

    // 하나의 쓰기 스레드 — 독점 접근
    {
        let chain = Arc::clone(&blockchain);
        let handle = thread::spawn(move || {
            let mut chain_ref = chain.write().unwrap();  // 쓰기 잠금 (독점)
            chain_ref.push("block_1".to_string());
            println!("Writer: added block");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final: {:?}", *blockchain.read().unwrap());
}
```

### 읽기/쓰기 비율에 따른 선택

| 상황 | 선택 |
|------|------|
| 읽기 전용 | `Arc<T>` |
| 읽기 >> 쓰기 | `Arc<RwLock<T>>` |
| 읽기 ≈ 쓰기 또는 쓰기 >> 읽기 | `Arc<Mutex<T>>` |

---

## Tokio에서의 공유 상태

Tokio 비동기 환경에서는 `tokio::sync::Mutex`와 `tokio::sync::RwLock`을 사용합니다:

```rust
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    blockchain: Arc<RwLock<Blockchain>>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            blockchain: Arc::new(RwLock::new(Blockchain::new())),
            mempool: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

async fn handle_new_transaction(
    state: AppState,
    tx: Transaction,
) -> Result<(), String> {
    // mempool에 트랜잭션 추가
    let mut pool = state.mempool.lock().await;
    pool.push(tx);
    println!("Mempool size: {}", pool.len());
    Ok(())
    // pool 드롭 → 잠금 해제
}

async fn get_block_count(state: &AppState) -> u64 {
    // 읽기 잠금 — 여러 태스크 동시 접근 가능
    let chain = state.blockchain.read().await;
    chain.blocks.len() as u64
}

async fn mine_new_block(state: AppState, data: String) -> Result<(), String> {
    // 쓰기 잠금 — 독점 접근
    let mut chain = state.blockchain.write().await;
    chain.add_block(data)?;
    Ok(())
}

struct Blockchain { blocks: Vec<String> }
impl Blockchain {
    fn new() -> Self { Blockchain { blocks: vec!["genesis".to_string()] } }
    fn add_block(&mut self, data: String) -> Result<(), String> {
        self.blocks.push(data);
        Ok(())
    }
}
struct Transaction { id: String }

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let state_clone = state.clone();
    tokio::spawn(async move {
        handle_new_transaction(
            state_clone,
            Transaction { id: "tx1".to_string() },
        ).await.unwrap();
    });

    println!("Block count: {}", get_block_count(&state).await);
}
```

---

## .await 중 락 보유의 위험성

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

// 위험한 패턴!
async fn dangerous(state: Arc<Mutex<Vec<String>>>) {
    let mut data = state.lock().await;  // 잠금 획득

    // .await 동안 잠금을 보유!
    // 다른 태스크가 같은 잠금을 얻으려 하면 교착 상태!
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    data.push("hello".to_string());
}   // 잠금 해제

// 안전한 패턴
async fn safe(state: Arc<Mutex<Vec<String>>>) {
    // 방법 1: 잠금 범위를 최소화
    let result = {
        let mut data = state.lock().await;
        data.push("hello".to_string());
        data.len()  // 잠금 해제 전에 필요한 값 추출
    };  // 잠금 해제

    // .await은 잠금 해제 후에
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("Data size: {}", result);

    // 방법 2: 명시적으로 drop
    let mut data = state.lock().await;
    data.push("world".to_string());
    drop(data);  // 명시적 잠금 해제

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
```

**규칙**: `.await` 포인트를 걸치는 Mutex 가드는 Rust 컴파일러가 경고합니다. `std::sync::Mutex`를 async 코드에서 쓰면 컴파일 에러가 나는 경우도 있습니다.

---

## Node.js에서 공유 상태가 필요 없는 이유

```javascript
// Node.js — 싱글 스레드 이벤트 루프
const blockchain = [];  // 전역 변수로 안전하게 공유

app.post('/blocks', async (req, res) => {
    // 이벤트 루프의 한 틱에서 실행
    // await 전까지는 다른 요청이 끼어들 수 없음
    blockchain.push(newBlock);  // 안전
    const result = await saveToDb(newBlock);  // 여기서만 다른 요청 실행 가능
    res.json(result);
});
```

Node.js는 `await` 포인트 사이에서는 완전히 단독으로 실행됩니다. 여러 요청이 동시에 `blockchain` 배열을 수정할 수 없습니다.

Rust/Tokio는 멀티 스레드이므로 여러 태스크가 정말로 동시에 실행됩니다. 따라서 `Arc<Mutex<T>>`가 필요합니다.

---

## 실용 패턴: 블록체인 노드 상태 관리

```rust
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Clone)]
pub struct NodeState {
    // 블록체인 — 읽기 많음
    pub blockchain: Arc<RwLock<Blockchain>>,
    // 피어 목록 — 읽기/쓰기 혼합
    pub peers: Arc<RwLock<Vec<String>>>,
    // 트랜잭션 캐시 — 쓰기 많음
    pub tx_cache: Arc<RwLock<HashMap<String, Transaction>>>,
}

impl NodeState {
    pub fn new() -> Self {
        NodeState {
            blockchain: Arc::new(RwLock::new(Blockchain::new())),
            peers: Arc::new(RwLock::new(Vec::new())),
            tx_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_peer(&self, addr: String) {
        let mut peers = self.peers.write().await;
        if !peers.contains(&addr) {
            peers.push(addr);
        }
    }

    pub async fn get_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    pub async fn cache_transaction(&self, tx: Transaction) {
        let mut cache = self.tx_cache.write().await;
        cache.insert(tx.id.clone(), tx);
    }

    pub async fn get_cached_tx(&self, id: &str) -> Option<Transaction> {
        self.tx_cache.read().await.get(id).cloned()
    }
}

#[derive(Clone)]
struct Transaction { id: String }
struct Blockchain { blocks: Vec<String> }
impl Blockchain { fn new() -> Self { Blockchain { blocks: vec![] } } }
```

---

## 요약

- `Arc<T>`: 스레드 간 공유 소유권 (참조 카운팅)
- `Mutex<T>`: 상호 배제 잠금 — 읽기/쓰기 모두 독점
- `RwLock<T>`: 읽기 잠금(동시 다중) / 쓰기 잠금(독점) 분리
- `Arc<Mutex<T>>` 또는 `Arc<RwLock<T>>`: 가장 흔한 패턴
- Tokio에서는 `tokio::sync::Mutex`, `tokio::sync::RwLock` 사용
- `.await` 중에 락을 보유하지 않도록 주의 (교착 상태 위험)
- Node.js는 싱글 스레드 → 이벤트 루프로 안전, Rust는 멀티 스레드 → 명시적 동기화 필요

다음으로는 Solana 아키텍처를 배웁니다.
