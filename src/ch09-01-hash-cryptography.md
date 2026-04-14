# 암호학 기초: 해시, 머클 트리, 공개키 암호화

블록체인은 암호학 위에 세워진 시스템이다. 암호학을 이해하지 않으면 블록체인이 왜 안전한지, 어떻게 신뢰를 만들어내는지 알 수 없다. 이 챕터에서는 블록체인에 사용되는 핵심 암호학 개념을 Rust 코드와 함께 깊이 이해한다.

---

## 1. 해시 함수 (Hash Function)

### 1.1 해시 함수란?

해시 함수는 **임의 길이의 입력**을 받아 **고정 길이의 출력**(다이제스트, digest)을 만드는 함수다.

```
입력 (임의 길이)        출력 (고정 길이)
"hello"          ──▶   2cf24dba5fb0a3...  (256비트)
"hello world"    ──▶   b94d27b9934d3e...  (256비트)
10MB 파일        ──▶   a3f5c8d21b09e4...  (256비트)
```

Node.js에서 이미 해시를 써봤을 것이다:

```javascript
// Node.js에서의 해시
const crypto = require('crypto');
const hash = crypto.createHash('sha256')
  .update('hello')
  .digest('hex');
console.log(hash);
// 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

### 1.2 암호학적 해시 함수의 5가지 특성

**1. 결정론적 (Deterministic)**
같은 입력은 항상 같은 출력을 낸다.
```
SHA256("hello") = 2cf24dba... (언제나 동일)
```

**2. 단방향성 (One-way / Preimage Resistance)**
출력에서 입력을 역산하는 것이 계산상 불가능하다.
```
2cf24dba... ──▶ "hello"   ← 이것이 불가능해야 함
```
모든 가능한 입력을 시도(무차별 대입)하는 것 외에 방법이 없다.

**3. 눈사태 효과 (Avalanche Effect)**
입력이 아주 조금 바뀌어도 출력이 완전히 달라진다.
```
SHA256("hello")  = 2cf24dba5fb0a30e26e83b2ac5b9e29e...
SHA256("hellO")  = 185f8db32921bd46d35cc3c8c85b...
```
한 글자만 바꿔도 출력이 50% 이상 바뀐다.

**4. 충돌 저항성 (Collision Resistance)**
서로 다른 두 입력이 같은 출력을 내는 경우(충돌)를 찾는 것이 계산상 불가능하다.
```
find x, y such that SHA256(x) == SHA256(y)  ← 사실상 불가능
```

**5. 빠른 연산**
해시 계산 자체는 매우 빠르다 (역산이 어려운 것이지, 정방향은 빠름).

### 1.3 SHA-256 동작 원리 (개념적)

SHA-256은 다음과 같은 과정을 거친다:

```
입력 메시지
    │
    ▼
┌─────────────────┐
│  패딩 (Padding) │ ← 메시지를 512비트 블록의 배수로 만듦
└────────┬────────┘
         │
    ┌────▼────┐
    │ 블록 1  │ ──▶ 압축 함수 (64라운드) ──▶ 중간 해시값
    └─────────┘           │
    ┌─────────┐           ▼
    │ 블록 2  │ ──▶ 압축 함수 (64라운드) ──▶ 중간 해시값
    └─────────┘           │
         ...              ▼
    ┌─────────┐           │
    │ 블록 N  │ ──▶ 압축 함수 (64라운드) ──▶ 최종 256비트 해시
    └─────────┘
```

각 압축 함수는 비트 연산(AND, OR, XOR, 시프트, 로테이션)과 모듈러 덧셈을 64번 반복한다. 이 과정이 눈사태 효과를 만들어낸다.

---

## 2. Rust로 SHA-256 해싱 구현하기

`Cargo.toml`에 의존성을 추가한다:

```toml
[dependencies]
sha2 = "0.10"
hex = "0.4"
```

```rust
use sha2::{Sha256, Digest};

fn hash_data(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

fn main() {
    let inputs = vec!["hello", "hello world", "blockchain"];
    
    for input in inputs {
        let hash = hash_data(input);
        println!("SHA256({:?}) = {}", input, hash);
    }
    
    // 눈사태 효과 확인
    println!("\n--- 눈사태 효과 ---");
    println!("SHA256(\"hello\") = {}", hash_data("hello"));
    println!("SHA256(\"hellO\") = {}", hash_data("hellO"));
    // 한 글자 차이지만 완전히 다른 해시!
}
```

**실행 결과:**
```
SHA256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
SHA256("hello world") = b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576fbe1ea5cc1f7f26b
SHA256("blockchain") = ef7797e13d3a75526946a3bcf00daec9fc9d1baea5a3d79434669b9eb6b4b250

--- 눈사태 효과 ---
SHA256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
SHA256("hellO") = 185f8db32921bd46d35cc3c8c85b0068eb62859e80a82f2e1e13d3e5a19f7bc9
```

---

## 3. 머클 트리 (Merkle Tree)

### 3.1 왜 머클 트리가 필요한가?

블록 하나에 수천 개의 트랜잭션이 있다고 하자. 특정 트랜잭션이 블록에 포함되어 있는지 검증하려면 어떻게 해야 할까?

**단순한 방법**: 모든 트랜잭션을 다운로드해서 확인 → 수백 MB 다운로드 필요

**머클 트리 방법**: 몇 개의 해시값만으로 O(log n) 검증 가능

### 3.2 머클 트리 구조

```
                    ┌──────────────┐
                    │  루트 해시   │  ← 머클 루트 (블록 헤더에 저장)
                    │  H(H12+H34) │
                    └──────┬───────┘
                    ┌──────┴───────┐
             ┌──────┴──────┐ ┌────┴──────────┐
             │    H12      │ │      H34      │
             │  H(H1+H2)  │ │   H(H3+H4)   │
             └──────┬──────┘ └──────┬────────┘
           ┌────────┴──────┐  ┌─────┴─────────┐
        ┌──┴───┐      ┌───┴──┐ ┌──┴───┐   ┌──┴────┐
        │  H1  │      │  H2  │ │  H3  │   │  H4   │
        │Hash  │      │Hash  │ │Hash  │   │Hash   │
        │(TX1) │      │(TX2) │ │(TX3) │   │(TX4)  │
        └──────┘      └──────┘ └──────┘   └───────┘
           ▲              ▲        ▲            ▲
          TX1            TX2      TX3          TX4
        (트랜잭션 1)                          (트랜잭션 4)
```

### 3.3 머클 증명 (Merkle Proof)

TX3가 블록에 포함되어 있다는 것을 증명하려면:
- TX3의 해시 (H3)
- H4 (형제 노드)
- H12 (삼촌 노드)
- 머클 루트

이 4개의 값만으로 TX3의 포함 여부를 검증할 수 있다. 수천 개의 트랜잭션 전체를 다운로드할 필요가 없다!

```
검증 과정:
1. H3 = Hash(TX3) 계산
2. H34 = Hash(H3 + H4) 계산
3. Root = Hash(H12 + H34) 계산
4. 계산된 Root == 블록 헤더의 머클 루트? → 검증 성공!
```

### 3.4 Rust로 머클 트리 구현

```rust
use sha2::{Sha256, Digest};

/// 두 해시를 합쳐서 새 해시 생성
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// 단일 데이터의 해시
fn hash_leaf(data: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.finalize().into()
}

/// 간단한 머클 트리 구현
struct MerkleTree {
    leaves: Vec<[u8; 32]>,
}

impl MerkleTree {
    fn new(transactions: &[&str]) -> Self {
        let leaves = transactions.iter()
            .map(|tx| hash_leaf(tx))
            .collect();
        Self { leaves }
    }
    
    /// 머클 루트 계산
    fn root(&self) -> Option<[u8; 32]> {
        if self.leaves.is_empty() {
            return None;
        }
        
        let mut current_level = self.leaves.clone();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            // 두 개씩 짝지어 해시
            let mut i = 0;
            while i < current_level.len() {
                if i + 1 < current_level.len() {
                    // 두 노드를 합쳐 부모 노드 생성
                    next_level.push(hash_pair(&current_level[i], &current_level[i + 1]));
                } else {
                    // 홀수 개인 경우, 마지막 노드를 자기 자신과 합침
                    next_level.push(hash_pair(&current_level[i], &current_level[i]));
                }
                i += 2;
            }
            
            current_level = next_level;
        }
        
        Some(current_level[0])
    }
    
    /// 특정 리프의 머클 증명 경로 생성
    fn proof(&self, index: usize) -> Vec<([u8; 32], bool)> {
        let mut proof = Vec::new();
        let mut current_level = self.leaves.clone();
        let mut current_index = index;
        
        while current_level.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                // 왼쪽 노드면 오른쪽 형제
                (current_index + 1).min(current_level.len() - 1)
            } else {
                // 오른쪽 노드면 왼쪽 형제
                current_index - 1
            };
            
            let is_right = current_index % 2 == 0; // 형제가 오른쪽에 있는지
            proof.push((current_level[sibling_index], is_right));
            
            // 다음 레벨로
            let mut next_level = Vec::new();
            let mut i = 0;
            while i < current_level.len() {
                if i + 1 < current_level.len() {
                    next_level.push(hash_pair(&current_level[i], &current_level[i + 1]));
                } else {
                    next_level.push(hash_pair(&current_level[i], &current_level[i]));
                }
                i += 2;
            }
            
            current_index /= 2;
            current_level = next_level;
        }
        
        proof
    }
}

/// 머클 증명 검증
fn verify_proof(
    leaf: &[u8; 32],
    proof: &[([u8; 32], bool)],
    root: &[u8; 32],
) -> bool {
    let mut current = *leaf;
    
    for (sibling, sibling_is_right) in proof {
        current = if *sibling_is_right {
            hash_pair(&current, sibling)
        } else {
            hash_pair(sibling, &current)
        };
    }
    
    &current == root
}

fn main() {
    let transactions = vec![
        "Alice -> Bob: 1 ETH",
        "Bob -> Carol: 0.5 ETH",
        "Carol -> Dave: 0.2 ETH",
        "Dave -> Eve: 0.1 ETH",
    ];
    
    let tree = MerkleTree::new(&transactions);
    let root = tree.root().unwrap();
    
    println!("머클 루트: {}", hex::encode(root));
    
    // TX2 (인덱스 1)에 대한 증명
    let tx_index = 1;
    let leaf_hash = hash_leaf(transactions[tx_index]);
    let proof = tree.proof(tx_index);
    
    println!("\n'{}' 검증:", transactions[tx_index]);
    println!("리프 해시: {}", hex::encode(leaf_hash));
    
    let is_valid = verify_proof(&leaf_hash, &proof, &root);
    println!("검증 결과: {}", if is_valid { "성공!" } else { "실패!" });
    
    // 조작된 트랜잭션은 검증 실패
    let fake_leaf = hash_leaf("Alice -> Bob: 1000 ETH"); // 조작!
    let is_fake_valid = verify_proof(&fake_leaf, &proof, &root);
    println!("\n조작된 트랜잭션 검증: {}", if is_fake_valid { "성공" } else { "실패 (올바름!)" });
}
```

---

## 4. 공개키/비밀키 암호학

### 4.1 비대칭 암호화의 개념

Node.js에서 JWT를 써봤다면 이미 비대칭 암호화를 경험한 것이다. 블록체인에서는 이 개념이 훨씬 중요하다.

```
비대칭 키 쌍:
┌──────────────┐        ┌──────────────┐
│   비밀키     │        │   공개키     │
│ (Private Key)│        │ (Public Key) │
│              │        │              │
│ 절대 노출 X  │        │ 모두에게 공개│
│ 지갑의 비밀번호│       │ 계좌 번호처럼│
└──────────────┘        └──────────────┘
       │                       │
       │   수학적으로 연결됨    │
       └───────────────────────┘
```

**비밀키 → 공개키**: 비밀키에서 공개키를 계산할 수 있다 (단방향)
**공개키 → 비밀키**: 공개키에서 비밀키를 역산하는 것은 불가능 (이산 대수 문제)

### 4.2 타원 곡선 암호 (ECDSA)

이더리움과 비트코인은 **secp256k1** 타원 곡선을 사용한다.

```
타원 곡선: y² = x³ + 7 (mod p)

    y
    │      ╭────╮
    │    ╭─╯    ╰─╮
    │   ╱           ╲
    │  │             │
    ├──┼─────────────┼──── x
    │  │             │
    │   ╲           ╱
    │    ╰─╮    ╭─╯
    │      ╰────╯
```

비밀키는 무작위 256비트 정수, 공개키는 이 값에 타원 곡선의 생성원 G를 곱한 점이다.

### 4.3 이더리움 지갑 주소 생성 과정

```
1. 비밀키 생성 (256비트 난수)
   예: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

2. 타원 곡선 연산으로 공개키 생성 (512비트)
   비밀키 × G(생성원) = 공개키 점(x, y)

3. 공개키를 Keccak-256으로 해싱

4. 해시의 마지막 20바이트 = 이더리움 주소

   비밀키(32바이트) ──ECDSA──▶ 공개키(64바이트)
                                     │
                               Keccak-256 해시
                                     │
                                 32바이트
                                     │
                            마지막 20바이트 추출
                                     │
                                     ▼
                      0x742d35Cc6634C0532925a3b8D4C9...
                      (이더리움 지갑 주소)
```

### 4.4 디지털 서명: 서명과 검증

```
서명 과정 (송신자):
┌─────────────────────────────────────────┐
│  메시지: "Alice → Bob: 1 ETH"            │
│     ↓ Keccak-256                        │
│  메시지 해시                             │
│     ↓ + 비밀키                          │
│  서명 (r, s, v) — 64바이트 + 1바이트    │
└─────────────────────────────────────────┘

검증 과정 (수신자):
┌─────────────────────────────────────────┐
│  서명 (r, s, v) + 메시지 해시           │
│     ↓ + 공개키                          │
│  서명이 유효한가? (Yes/No)              │
│                                         │
│  ※ 비밀키 없이도 검증 가능!             │
└─────────────────────────────────────────┘
```

**중요한 점**: 서명을 검증할 때 비밀키가 필요하지 않다. 공개키만 있으면 서명의 유효성을 확인할 수 있다. 이것이 블록체인에서 트랜잭션 인증이 작동하는 원리다.

### 4.5 Rust로 키쌍 생성, 서명, 검증

```toml
[dependencies]
secp256k1 = { version = "0.27", features = ["rand"] }
sha3 = "0.10"
rand = "0.8"
hex = "0.4"
```

```rust
use secp256k1::{Secp256k1, Message, SecretKey, PublicKey};
use sha3::{Keccak256, Digest};
use rand::rngs::OsRng;

/// 메시지를 Keccak-256으로 해싱
fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// 공개키에서 이더리움 주소 계산
fn public_key_to_address(public_key: &PublicKey) -> String {
    // 비압축 공개키 (65바이트: 04 + x + y)
    let serialized = public_key.serialize_uncompressed();
    
    // 앞의 '04' 바이트를 제외한 64바이트를 해싱
    let hash = keccak256(&serialized[1..]);
    
    // 마지막 20바이트가 주소
    let address = &hash[12..];
    format!("0x{}", hex::encode(address))
}

fn main() {
    let secp = Secp256k1::new();
    
    // 1. 키쌍 생성
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    
    println!("=== 키쌍 생성 ===");
    println!("비밀키: 0x{}", hex::encode(secret_key.secret_bytes()));
    println!("공개키: 0x{}", hex::encode(public_key.serialize()));
    println!("주소:   {}", public_key_to_address(&public_key));
    
    // 2. 메시지 서명
    let message_text = "Alice -> Bob: 1 ETH";
    let message_hash = keccak256(message_text.as_bytes());
    let message = Message::from_digest(message_hash);
    
    let signature = secp.sign_ecdsa(&message, &secret_key);
    
    println!("\n=== 트랜잭션 서명 ===");
    println!("메시지: {}", message_text);
    println!("서명: {}", hex::encode(signature.serialize_compact()));
    
    // 3. 서명 검증
    let is_valid = secp.verify_ecdsa(&message, &signature, &public_key).is_ok();
    println!("\n=== 서명 검증 ===");
    println!("검증 결과: {}", if is_valid { "유효함!" } else { "무효!" });
    
    // 4. 잘못된 공개키로 검증 시도 (실패해야 함)
    let (_, wrong_public_key) = secp.generate_keypair(&mut OsRng);
    let is_wrong_valid = secp.verify_ecdsa(&message, &signature, &wrong_public_key).is_ok();
    println!("다른 공개키로 검증: {}", if is_wrong_valid { "통과 (문제!)" } else { "실패 (올바름!)" });
    
    // 5. 서명에서 공개키 복구 (이더리움이 실제로 하는 방식)
    println!("\n=== 발신자 복구 ===");
    let recoverable_sig = secp.sign_ecdsa_recoverable(&message, &secret_key);
    let recovered_key = secp.recover_ecdsa(&message, &recoverable_sig).unwrap();
    println!("복구된 주소: {}", public_key_to_address(&recovered_key));
    println!("원래 주소:   {}", public_key_to_address(&public_key));
    println!("일치: {}", recovered_key == public_key);
}
```

---

## 5. 실제 이더리움에서의 활용

이더리움에서 트랜잭션이 서명되는 전체 흐름:

```
사용자 행동: "1 ETH를 Bob에게 보낸다"
     │
     ▼
트랜잭션 객체 생성:
{
  nonce: 5,
  gasPrice: 20 gwei,
  gasLimit: 21000,
  to: "0xBob...",
  value: 1 ETH,
  data: ""
}
     │
     ▼
RLP 인코딩 → 바이트 배열
     │
     ▼
Keccak-256 해시
     │
     ▼
비밀키로 ECDSA 서명 → (r, s, v)
     │
     ▼
서명된 트랜잭션을 네트워크에 브로드캐스트
     │
     ▼
검증자(노드)가 서명 검증:
  - 서명에서 공개키 복구
  - 공개키에서 주소 계산
  - 트랜잭션의 from 주소와 일치하면 유효!
```

---

## 6. 핵심 정리

| 개념 | 설명 | 블록체인에서의 역할 |
|------|------|-------------------|
| **SHA-256** | 256비트 해시 생성 | 블록 해시, PoW |
| **Keccak-256** | 이더리움 표준 해시 | 트랜잭션 해시, 주소 생성 |
| **머클 트리** | 트랜잭션 요약 트리 | 블록 내 TX 검증, 경량 클라이언트 |
| **ECDSA** | 타원 곡선 서명 | 트랜잭션 인증, 소유권 증명 |
| **비밀키** | 256비트 랜덤 숫자 | 지갑 소유권, 서명 생성 |
| **공개키** | 비밀키 × G | 주소 생성, 서명 검증 |
| **지갑 주소** | Keccak(공개키)[-20바이트] | 계정 식별자 |

다음 챕터에서는 이 암호학 기초 위에 블록과 체인이 어떻게 구성되는지 알아본다.
