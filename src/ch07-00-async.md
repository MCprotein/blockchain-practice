# 7장: 비동기 프로그래밍

## 왜 비동기가 필요한가?

블록체인 노드는 동시에 많은 일을 처리합니다:
- 여러 피어와 TCP 연결 유지
- 새 트랜잭션 수신 및 검증
- 블록 동기화
- RPC 요청 처리
- 마이닝 (CPU 집약적)

이런 I/O 집약적 작업을 동기적으로 처리하면 하나의 작업이 끝날 때까지 나머지가 모두 대기합니다.

## Node.js의 비동기 모델

Node.js는 싱글 스레드 + 이벤트 루프로 동시성을 처리합니다:

```javascript
// Node.js — 이벤트 루프가 내장
const response1 = await fetch('https://api.example.com/blocks/1');
const response2 = await fetch('https://api.example.com/blocks/2');
// 순차적 — 1이 끝나야 2 시작

// 병렬 처리
const [r1, r2] = await Promise.all([
    fetch('https://api.example.com/blocks/1'),
    fetch('https://api.example.com/blocks/2'),
]);
```

Node.js는 처음부터 비동기를 전제로 설계되었습니다. 런타임(V8 + libuv)이 이벤트 루프를 제공합니다.

## Rust의 비동기 모델

Rust는 `async/await` 문법을 제공하지만, **런타임은 포함되어 있지 않습니다**. 런타임을 직접 선택해야 합니다.

```rust
// Rust — 런타임을 선택해야 함
use tokio; // 가장 널리 쓰이는 비동기 런타임

#[tokio::main]
async fn main() {
    let response1 = fetch_block(1).await;
    let response2 = fetch_block(2).await;

    // 병렬 처리
    let (r1, r2) = tokio::join!(
        fetch_block(1),
        fetch_block(2),
    );
}
```

### 왜 런타임이 별도인가?

이것은 Rust의 "제로 비용 추상화" 철학의 일부입니다:
- 임베디드 시스템에서는 Tokio 런타임이 너무 무거울 수 있음
- 스마트 컨트랙트(Solana on-chain)에서는 비동기가 필요 없음
- 시스템 프로그래밍에서는 직접 스레드 관리가 필요할 수 있음

Rust는 `async/await` 문법만 언어에 포함하고, 실행 방법은 선택하게 합니다.

## 주요 비동기 런타임

| 런타임 | 특징 | 사용처 |
|--------|------|--------|
| **Tokio** | 가장 널리 사용, 고성능, 풍부한 생태계 | 웹 서버, 블록체인 노드 |
| **async-std** | 표준 라이브러리와 유사한 API | 범용 |
| **smol** | 경량 | 임베디드, 리소스 제한 환경 |

이 책에서는 Tokio를 사용합니다. Ethereum의 ethers-rs, Solana의 tokio 기반 클라이언트 모두 Tokio를 사용합니다.

## 이 장의 구성

1. **async/await** (7.1): Future 트레이트, Node.js와 차이점
2. **Tokio** (7.2): 설치, spawn, channel, HTTP
3. **공유 상태** (7.3): Arc, Mutex, RwLock

다음 챕터에서 async/await를 자세히 배웁니다.
