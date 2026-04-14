# 블록과 체인: 데이터 구조 완전 분석

9.1장에서 해시 함수와 암호학을 배웠다. 이제 그 도구들이 어떻게 블록과 체인을 만드는지 알아보자. 블록체인의 이름 그 자체 — "블록(Block)의 체인(Chain)" — 을 코드 레벨에서 이해한다.

> **참고**: 이 챕터는 3장에서 배운 `struct`, `enum`, `Vec`를 블록체인 데이터 구조에 적용합니다.
> 문법 설명을 다시 반복하기보다, `Transaction`, `Block`, `Blockchain`이 어떤 책임을 나눠 갖는지에 집중하세요.

---

## 1. 블록의 구조

하나의 블록은 크게 **헤더(Header)**와 **바디(Body)**로 나뉜다.

먼저 블록을 “데이터베이스 row 하나”로 생각하면 안 된다. 블록은 여러 트랜잭션을 한 번에 담는 **append-only 기록 묶음**이다. 헤더는 이 묶음을 검증하기 위한 요약 정보이고, 바디는 실제 트랜잭션 목록이다.

```text
┌─────────────────────────────────────────────────────┐
│                      블록 #N                         │
├─────────────────────────────────────────────────────┤
│                    HEADER (헤더)                      │
│  ┌─────────────────────────────────────────────┐    │
│  │ previousHash: "0x3a7f..."  (이전 블록 해시)  │    │
│  │ timestamp:    1700000000   (생성 시각)        │    │
│  │ nonce:        293847       (PoW용 임의값)     │    │
│  │ merkleRoot:   "0xb94d..."  (트랜잭션 요약)   │    │
│  │ difficulty:   0x1d00ffff   (채굴 난이도)      │    │
│  │ blockHash:    "0xf2a1..."  (이 블록의 해시)   │    │
│  └─────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────┤
│                     BODY (바디)                      │
│  ┌──────────────────────────────────────────────┐   │
│  │ transactions: [                               │   │
│  │   { from: "0xAlice", to: "0xBob", ... },     │   │
│  │   { from: "0xBob", to: "0xCarol", ... },     │   │
│  │   ...                                         │   │
│  │ ]                                             │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### 1.1 헤더 필드 상세 설명

**previousHash (이전 블록 해시)**
- 이 블록이 어떤 블록 다음에 오는지를 명시한다
- 체인의 연결 고리 역할
- 최초 블록(제네시스 블록)은 이 값이 `0x000...000`

**timestamp (타임스탬프)**
- 블록이 생성된 Unix 시각 (초 단위)
- 네트워크 시간 조작을 방지하기 위해 규칙이 있음
  - 이전 블록보다 늦어야 함
  - 네트워크 시간에서 너무 멀면 거부됨

**nonce (논스)**
- "Number used ONCE"의 약자
- Proof of Work에서 채굴자가 조정하는 값
- 블록 해시가 난이도 조건을 만족할 때까지 nonce를 바꿔가며 시도

**merkleRoot (머클 루트)**
- 바디의 모든 트랜잭션을 머클 트리로 요약한 해시
- 헤더만 봐도 트랜잭션 변조 여부를 알 수 있음

**difficulty (난이도)**
- 블록 해시가 만족해야 하는 조건
- 예: "해시가 0이 18개로 시작해야 한다"
- 약 2주마다 자동 조절됨

### 1.2 블록 해시 계산

블록 해시는 헤더 전체를 해싱해서 만든다:

```text
blockHash = SHA256(SHA256(
  previousHash + merkleRoot + timestamp + nonce + difficulty
))
```

비트코인은 SHA256을 두 번 적용한다(이중 해싱). 이더리움은 Keccak-256을 사용한다.

---

## 2. 체인이 되는 원리

각 블록이 이전 블록의 해시를 포함함으로써 블록들이 연결된다.

```text
제네시스 블록                블록 #1                    블록 #2
┌─────────────┐          ┌─────────────┐          ┌─────────────┐
│ prevHash:   │          │ prevHash:   │          │ prevHash:   │
│ 0x00000000  │          │ 0xABC123... │◀─────────│ 0xDEF456... │
│             │          │             │          │             │
│ merkleRoot: │          │ merkleRoot: │          │ merkleRoot: │
│ 0x111...    │          │ 0x222...    │          │ 0x333...    │
│             │          │             │          │             │
│ nonce: 0    │          │ nonce: 4829 │          │ nonce: 9103 │
│             │          │             │          │             │
│ blockHash:  │──────────│ blockHash:  │──────────│ blockHash:  │
│ 0xABC123... │          │ 0xDEF456... │          │ 0xGHI789... │
└─────────────┘          └─────────────┘          └─────────────┘
  TX: 코인베이스           TX: Alice→Bob             TX: Bob→Carol
                                                  TX: Carol→Dave
```

### 왜 과거 데이터를 수정하기 어려운가?

블록 #1의 트랜잭션을 수정하면:
1. 블록 #1의 머클 루트가 바뀐다
2. 블록 #1의 해시가 바뀐다
3. 블록 #2의 `previousHash`가 맞지 않게 된다
4. 블록 #2를 다시 채굴해야 한다
5. 블록 #3도 바뀐다 → 블록 #3도 재채굴
6. ... 현재 최신 블록까지 모두 재채굴 필요

게다가, 정직한 노드들은 계속 새 블록을 채굴하고 있다. 공격자는 전체 네트워크 해시파워의 51% 이상을 확보해야 따라잡을 수 있다. 이것이 **51% 공격**이다.

---

## 3. 트랜잭션 구조

트랜잭션은 **사용자가 체인 상태를 바꿔 달라고 제출하는 서명된 요청**이다. 이더리움에서 ETH 전송, ERC-20 토큰 전송, 스마트 컨트랙트 함수 호출은 모두 트랜잭션으로 표현된다.

백엔드 API 요청과 비교하면 다음과 같다.

| 백엔드 API 요청 | 이더리움 트랜잭션 |
|-----------------|-------------------|
| `POST /transfer` | `to`, `value`, `data`를 담은 트랜잭션 |
| JWT나 세션 쿠키로 인증 | 개인 키 서명(`v`, `r`, `s`)으로 인증 |
| 서버가 DB에 쓰기 | 검증자가 블록에 포함해야 상태 변경 |
| 실패 시 서버가 에러 응답 | 실패 시 revert되고 가스 일부 또는 전부 소비 |

따라서 트랜잭션은 단순한 “송금 기록”이 아니라, 블록체인 상태 전이를 일으키는 입력값이다.

### 3.1 트랜잭션 필드

이더리움 트랜잭션의 기본 구조:

```text
트랜잭션:
{
  nonce:     5,              // 이 계정이 보낸 TX 수 (재전송 공격 방지)
  gasPrice:  20_000_000_000, // wei per gas (20 Gwei)
  gasLimit:  21_000,         // 최대 사용 가능한 gas
  to:        "0xBob...",     // 수신자 주소 (컨트랙트 생성 시 null)
  value:     1_000_000_000_000_000_000, // 1 ETH (wei 단위)
  data:      "0x",           // 컨트랙트 호출 시 입력 데이터
  v:         27,             // 서명 복구 값
  r:         "0x...",        // 서명 r 값
  s:         "0x..."         // 서명 s 값
}
```

**nonce의 두 가지 역할:**
1. **재전송 공격(Replay Attack) 방지**: 같은 서명된 TX를 여러 번 보내는 공격 차단
2. **순서 보장**: 같은 계정에서 발송한 TX가 nonce 순서대로 처리됨

### 3.2 트랜잭션 수명 주기

```text
사용자가 TX 생성 및 서명
        │
        ▼
   P2P 네트워크로 브로드캐스트
        │
        ▼
   각 노드의 Mempool(메모리 풀)에 대기
   ┌──────────────────────────────┐
   │  Mempool (미확인 TX 대기소)   │
   │  [TX_A, TX_B, TX_C, TX_D,…] │
   │  가스비 높은 순으로 정렬       │
   └──────────────────────────────┘
        │
        ▼ (채굴자/검증자가 선택)
   블록에 포함
        │
        ▼
   블록이 네트워크에 전파
        │
        ▼
   다른 노드들이 블록 검증 및 수락
        │
        ▼
   TX 최종 확인 (Confirmation)
   (일반적으로 6 블록 후 = ~1분)
```

**Mempool(멤풀)**은 Node.js의 Redis 큐(Bull/BullMQ)와 비슷하다. 처리 대기 중인 작업들의 목록이다. 차이는 누구나 Mempool에 TX를 제출할 수 있고, 가스비가 높을수록 먼저 처리된다는 점이다.

---

## 4. UTXO 모델 vs Account 모델

블록체인은 잔액을 추적하는 방식이 두 가지로 나뉜다. 비트코인과 이더리움이 서로 다른 방식을 사용한다.

### 4.1 UTXO 모델 (비트코인)

**UTXO = Unspent Transaction Output** (미사용 트랜잭션 출력)

현금을 생각해보자. 10만원짜리 지폐를 가지고 있다가 7만원짜리 물건을 사면:
- 10만원 지폐를 낸다 (소비됨)
- 7만원을 지불하고
- 3만원 거스름돈을 받는다

```text
UTXO 모델 예시:

Alice가 받은 UTXO들:
  UTXO_1: 0.5 BTC (TX_A의 출력)
  UTXO_2: 0.3 BTC (TX_B의 출력)
  UTXO_3: 1.2 BTC (TX_C의 출력)
  총 잔액: 2.0 BTC

Alice가 Bob에게 0.7 BTC를 보낼 때:
  입력: UTXO_1(0.5) + UTXO_2(0.3) = 0.8 BTC  ← 소비됨
  출력1: Bob에게 0.7 BTC                       ← Bob의 새 UTXO
  출력2: Alice에게 0.1 BTC (거스름돈)           ← Alice의 새 UTXO

결과:
  Alice: UTXO_3(1.2 BTC) + UTXO_new(0.1 BTC) = 1.3 BTC
  Bob:   UTXO_new(0.7 BTC) = 0.7 BTC
```

**UTXO 모델의 특징:**
- 잔액이란 개념이 없음. "내가 소유한 미사용 출력들의 합"이 잔액
- 병렬 처리에 유리 (각 UTXO가 독립적)
- 프라이버시에 유리 (주소를 자주 바꿀 수 있음)
- 스마트 컨트랙트 구현이 복잡

### 4.2 Account 모델 (이더리움)

은행 계좌와 동일하다. 각 주소에 잔액이 직접 저장된다.

```text
Account 모델:

상태(State):
  Alice: { balance: 2.0 ETH, nonce: 5 }
  Bob:   { balance: 0.5 ETH, nonce: 2 }

Alice가 Bob에게 0.7 ETH를 보낼 때:
  Alice.balance -= 0.7 ETH → 1.3 ETH
  Bob.balance   += 0.7 ETH → 1.2 ETH
  Alice.nonce   += 1       → 6

결과:
  Alice: { balance: 1.3 ETH, nonce: 6 }
  Bob:   { balance: 1.2 ETH, nonce: 3 }
```

**Account 모델의 특징:**
- 직관적. 일반 데이터베이스처럼 잔액을 저장
- 스마트 컨트랙트 구현에 적합
- 상태 크기가 작음 (UTXO처럼 과거 TX 모두 추적 불필요)
- 재전송 공격 방지를 위해 nonce 필수

| 비교 | UTXO (비트코인) | Account (이더리움) |
|------|----------------|-------------------|
| 잔액 표현 | 미사용 출력들의 합 | 계정에 직접 저장 |
| 프라이버시 | 더 높음 | 낮음 |
| 스마트 컨트랙트 | 어려움 | 쉬움 |
| 병렬 처리 | 유리 | 불리 |
| 상태 크기 | 크게 증가 가능 | 상대적으로 작음 |

---

## 5. Rust로 Block, Transaction 구조체 구현

```rust,ignore
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// 트랜잭션 구조체
#[derive(Debug, Clone)]
struct Transaction {
    from:      String,  // 발신자 주소
    to:        String,  // 수신자 주소
    value:     u64,     // 이체 금액 (wei 단위)
    nonce:     u64,     // 재전송 방지용 순번
    data:      String,  // 컨트랙트 호출 데이터 (일반 전송 시 빈 문자열)
    signature: String,  // ECDSA 서명
}

impl Transaction {
    fn new(from: &str, to: &str, value: u64, nonce: u64) -> Self {
        Transaction {
            from: from.to_string(),
            to: to.to_string(),
            value,
            nonce,
            data: String::new(),
            signature: String::new(), // 실제로는 비밀키로 서명
        }
    }

    /// 트랜잭션 해시 계산
    fn hash(&self) -> String {
        let content = format!(
            "{}{}{}{}{}",
            self.from, self.to, self.value, self.nonce, self.data
        );
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// 블록 헤더
#[derive(Debug, Clone)]
struct BlockHeader {
    index:         u64,    // 블록 번호
    previous_hash: String, // 이전 블록의 해시
    timestamp:     u64,    // Unix timestamp
    merkle_root:   String, // 트랜잭션들의 머클 루트
    nonce:         u64,    // PoW를 위한 논스
    difficulty:    usize,  // 해시 앞에 0이 몇 개 있어야 하는가
}

/// 블록 구조체
#[derive(Debug, Clone)]
struct Block {
    header:       BlockHeader,
    transactions: Vec<Transaction>,
    hash:         String,
}

impl Block {
    /// 제네시스 블록 생성
    fn genesis() -> Self {
        let header = BlockHeader {
            index:         0,
            previous_hash: "0".repeat(64),
            timestamp:     current_timestamp(),
            merkle_root:   "0".repeat(64),
            nonce:         0,
            difficulty:    2, // 앞에 0이 2개 ('00'으로 시작)
        };
        
        let mut block = Block {
            hash: String::new(),
            header,
            transactions: vec![],
        };
        block.hash = block.calculate_hash();
        block
    }
    
    /// 새 블록 생성 (채굴 포함)
    fn new(
        index: u64,
        previous_hash: String,
        transactions: Vec<Transaction>,
        difficulty: usize,
    ) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        let mut header = BlockHeader {
            index,
            previous_hash,
            timestamp: current_timestamp(),
            merkle_root,
            nonce: 0,
            difficulty,
        };
        
        let mut block = Block {
            hash: String::new(),
            header,
            transactions,
        };
        
        // PoW: 난이도를 만족하는 nonce 탐색
        block.mine();
        block
    }
    
    /// 블록 해시 계산
    fn calculate_hash(&self) -> String {
        let h = &self.header;
        let content = format!(
            "{}{}{}{}{}{}",
            h.index,
            h.previous_hash,
            h.timestamp,
            h.merkle_root,
            h.nonce,
            h.difficulty
        );
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    /// 트랜잭션들의 머클 루트 계산 (간략 버전)
    fn calculate_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0".repeat(64);
        }
        
        let mut hashes: Vec<String> = transactions.iter()
            .map(|tx| tx.hash())
            .collect();
        
        while hashes.len() > 1 {
            let mut next = Vec::new();
            let mut i = 0;
            while i < hashes.len() {
                let left = &hashes[i];
                let right = if i + 1 < hashes.len() {
                    &hashes[i + 1]
                } else {
                    &hashes[i] // 홀수일 때 마지막 반복
                };
                
                let combined = format!("{}{}", left, right);
                let mut hasher = Sha256::new();
                hasher.update(combined.as_bytes());
                next.push(hex::encode(hasher.finalize()));
                i += 2;
            }
            hashes = next;
        }
        
        hashes[0].clone()
    }
    
    /// Proof of Work: 조건을 만족하는 nonce 탐색
    fn mine(&mut self) {
        let target = "0".repeat(self.header.difficulty);
        println!("블록 #{} 채굴 시작 (난이도: {})...", self.header.index, self.header.difficulty);
        
        loop {
            let hash = self.calculate_hash();
            if hash.starts_with(&target) {
                self.hash = hash;
                println!(
                    "채굴 완료! nonce={}, hash={}",
                    self.header.nonce,
                    &self.hash[..16]
                );
                return;
            }
            self.header.nonce += 1;
        }
    }
    
    /// 블록 유효성 검사
    fn is_valid(&self) -> bool {
        // 1. 해시가 올바른가?
        if self.hash != self.calculate_hash() {
            return false;
        }
        // 2. 난이도 조건을 만족하는가?
        let target = "0".repeat(self.header.difficulty);
        self.hash.starts_with(&target)
    }
}

/// 블록체인
struct Blockchain {
    chain:      Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    fn new() -> Self {
        let genesis = Block::genesis();
        Blockchain {
            chain: vec![genesis],
            difficulty: 2,
        }
    }
    
    fn last_block(&self) -> &Block {
        self.chain.last().unwrap()
    }
    
    fn add_block(&mut self, transactions: Vec<Transaction>) {
        let index = self.chain.len() as u64;
        let previous_hash = self.last_block().hash.clone();
        let block = Block::new(index, previous_hash, transactions, self.difficulty);
        self.chain.push(block);
    }
    
    /// 체인 전체 유효성 검사
    fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];
            
            // 현재 블록 해시가 올바른가?
            if !current.is_valid() {
                return false;
            }
            
            // 이전 블록과 연결이 올바른가?
            if current.header.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn main() {
    let mut blockchain = Blockchain::new();
    
    // 트랜잭션 생성
    let tx1 = Transaction::new("0xAlice", "0xBob", 1_000_000_000_000_000_000, 0);
    let tx2 = Transaction::new("0xBob", "0xCarol", 500_000_000_000_000_000, 1);
    
    blockchain.add_block(vec![tx1, tx2]);
    
    let tx3 = Transaction::new("0xCarol", "0xDave", 200_000_000_000_000_000, 0);
    blockchain.add_block(vec![tx3]);
    
    // 체인 상태 출력
    println!("\n=== 블록체인 상태 ===");
    for block in &blockchain.chain {
        println!(
            "블록 #{}: hash={}... (tx 수: {})",
            block.header.index,
            &block.hash[..16],
            block.transactions.len()
        );
    }
    
    println!("\n체인 유효성: {}", blockchain.is_valid());
    
    // 데이터 조작 시도
    println!("\n=== 데이터 조작 시도 ===");
    blockchain.chain[1].transactions[0].value = 999_999_999_999_999_999_999;
    println!("블록 #1의 TX 금액을 변조함");
    println!("체인 유효성: {}", blockchain.is_valid()); // false가 되어야 함
}
```

이 예제는 길지만, 읽는 순서는 단순하다.

| 코드 덩어리 | 먼저 볼 것 |
|-------------|------------|
| `struct Transaction` | 트랜잭션이 어떤 필드로 구성되는지 |
| `impl Transaction` | 트랜잭션을 만드는 함수와 해시 계산 함수 |
| `struct BlockHeader` | 블록 검증에 필요한 요약 정보 |
| `struct Block` | 헤더와 트랜잭션 목록이 어떻게 묶이는지 |
| `impl Block` | 제네시스 생성, 새 블록 생성, 머클 루트 계산, 채굴 |
| `struct Blockchain` | 블록 목록과 난이도를 어떤 상태로 보관하는지 |
| `impl Blockchain` | 마지막 블록 조회, 블록 추가, 체인 검증 |

처음 읽을 때 `&self`, `Vec<Transaction>`, `Self`, `clone()`을 모두 완벽히 이해할 필요는 없다. 지금은 “블록체인 프로그램은 데이터를 구조체로 모델링하고, 검증/해싱/추가 동작을 `impl`에 붙인다”는 큰 흐름을 잡으면 된다.

---

## 6. 핵심 정리

- **블록 = 헤더 + 바디**: 헤더에는 이전 블록 해시, 머클 루트, 논스 등이 있고 바디에는 트랜잭션 목록이 있다
- **체인 연결**: 각 블록이 이전 블록의 해시를 포함해 역방향 변조가 불가능해진다
- **트랜잭션 nonce**: 재전송 공격을 방지하고 처리 순서를 보장한다
- **UTXO(비트코인)**: 미사용 출력들의 합이 잔액. 프라이버시 유리, 스마트 컨트랙트 불리
- **Account(이더리움)**: 계정에 잔액 직접 저장. 스마트 컨트랙트 구현에 최적화

다음 챕터에서는 블록들이 서로 경쟁하며 추가될 때 네트워크가 어떻게 합의에 도달하는지 — **합의 알고리즘** — 을 다룬다.
