# Solana 아키텍처: 고성능 단일 레이어 블록체인

## Solana란 무엇인가

Solana는 2020년 Anatoly Yakovenko가 창시한 고성능 레이어 1 블록체인입니다. "세상에서 가장 빠른 블록체인"을 목표로 설계된 Solana는 이더리움이 레이어 2나 샤딩으로 해결하려는 확장성 문제를 **단일 레이어에서** 해결하는 독자적인 접근법을 취합니다.

Node.js 백엔드 개발자 관점에서 비유하자면:
- **이더리움** = 수직 확장에 한계가 있어 마이크로서비스(L2)로 분산시키는 모놀리식 서버
- **Solana** = 처음부터 고성능 싱글 프로세스로 설계된 서버 (Node.js의 이벤트 루프처럼 단일 스레드이지만 극도로 최적화된)

---

## 이더리움과 Solana 핵심 차이점 비교

| 항목 | 이더리움 | Solana |
|------|---------|--------|
| **TPS** | ~15 (L1 기준) | ~4,000 (실측), 이론 65,000+ |
| **블록 확정 시간** | ~12초 | ~400ms |
| **평균 수수료** | $1 ~ $50+ (가스 경매) | $0.00025 (고정에 가까움) |
| **합의 메커니즘** | PoS (Casper) | PoS + PoH (Proof of History) |
| **상태 모델** | 컨트랙트 내부 저장소 | 분리된 Account 모델 |
| **스마트 컨트랙트 언어** | Solidity, Vyper | Rust, C, C++ |
| **개발 프레임워크** | Hardhat, Foundry | Anchor |
| **병렬 처리** | 순차 처리 | Sealevel 병렬 실행 |
| **레이어 구조** | L1 + L2 생태계 | 단일 L1 |

### 상태 모델 차이: 가장 중요한 개념

```text
이더리움 모델:
┌─────────────────────────────┐
│    ERC20 컨트랙트           │
│  ┌───────────────────────┐  │
│  │  balances mapping     │  │  ← 데이터가 컨트랙트 안에 있음
│  │  totalSupply          │  │
│  │  allowances mapping   │  │
│  └───────────────────────┘  │
│    함수: transfer, approve  │
└─────────────────────────────┘

Solana 모델:
┌─────────────┐    ┌──────────────────┐    ┌──────────────────┐
│  Token      │    │  User A의        │    │  User B의        │
│  Program    │───▶│  Token Account   │    │  Token Account   │
│  (코드만)   │    │  balance: 100    │    │  balance: 50     │
└─────────────┘    └──────────────────┘    └──────────────────┘
     프로그램은 상태를 갖지 않음. 데이터는 별도 Account에 저장
```

NestJS에 비유하면:
- **이더리움**: `UserService`가 자체 인메모리 Map으로 데이터를 관리 (서비스 안에 DB가 있는 구조)
- **Solana**: `UserService`는 로직만 갖고, 실제 데이터는 외부 PostgreSQL Account에 저장

### 수수료 모델 차이

```typescript
// 이더리움: 가스 경매 방식 (런타임에 결정)
// 트랜잭션 수수료 = gasUsed × gasPrice
// 네트워크 혼잡 시 gasPrice가 폭등 → 예측 불가

// Solana: 기본 수수료 고정
// 수수료 = 서명 수 × 5,000 lamports
// 1 SOL = 1,000,000,000 lamports
// 서명 1개 = 0.000005 SOL ≈ $0.00025 (SOL이 $50일 때)
```

---

## Solana의 8가지 핵심 혁신

### 1. Proof of History (PoH) - Solana의 심장

PoH는 Solana의 가장 혁신적인 기술입니다. **시간 자체를 블록체인에 기록**하는 방식입니다.

```text
일반 블록체인의 문제:
- 검증자들이 "이 트랜잭션이 언제 발생했는가"를 합의로 결정해야 함
- 매 블록마다 네트워크 통신이 필요 → 느림

PoH의 해결책:
SHA256(이전 해시) → SHA256(SHA256(이전 해시)) → ...
이 체인이 곧 시계 역할을 함

[Hash_0] → [Hash_1, Tx_A] → [Hash_2] → [Hash_3, Tx_B] → [Hash_4]
           ↑ Tx_A가 Hash_0 이후,       ↑ Tx_B가 Tx_A 이후에
             Hash_2 이전에 발생했음이    Hash_4 이전에 발생했음
             수학적으로 증명됨           이 수학적으로 증명됨
```

SHA256은 순차적으로만 계산 가능하므로, 해시 체인의 길이 자체가 경과 시간을 증명합니다. 검증자들이 매번 합의할 필요 없이 리더 혼자 시간을 기록하고 나머지가 병렬로 검증합니다.

### 2. Tower BFT - PoH 기반 합의

전통적인 PBFT(Practical Byzantine Fault Tolerance)를 PoH 시계 위에서 실행합니다.

- 검증자들이 투표할 때 "lockout" 타임아웃을 증가시킴
- 이전 투표를 번복할수록 더 큰 페널티(슬래싱)
- PoH 덕분에 시간 동기화 문제가 해결되어 합의 속도 향상

### 3. Turbine - 블록 전파 프로토콜

BitTorrent에서 영감을 받은 블록 데이터 전파 방식입니다.

```text
전통적 방식:              Turbine 방식:
                          리더
리더 → 모든 검증자        ↙  ↓  ↘
(O(n) 대역폭 필요)      검증자1 검증자2 검증자3
                        ↙↘    ↙↘    ↙↘
                       ...   ...   ...
                       (데이터를 청크로 분산 전파)
```

블록을 작은 패킷(shred)으로 분할하여 트리 구조로 전파함으로써 리더의 대역폭 부담을 O(log n)으로 줄입니다.

### 4. Gulf Stream - 멤풀 없는 트랜잭션 전달

이더리움은 트랜잭션을 멤풀(mempool)에 쌓아두고 채굴자/검증자가 선택합니다. Solana는 멤풀을 없애고 **다음 리더에게 직접 트랜잭션을 전달**합니다.

```text
이더리움:
사용자 → 멤풀(대기열) → 검증자가 선택 → 블록 포함

Solana:
사용자 → 현재 리더, 다음 리더, 다다음 리더에게 미리 전달
→ 리더가 즉시 처리 가능, 멤풀 혼잡 없음
```

### 5. Sealevel - 병렬 스마트 컨트랙트 실행

이더리움의 EVM은 트랜잭션을 순차적으로 처리합니다. Solana의 Sealevel은 **서로 다른 계정에 접근하는 트랜잭션을 병렬 처리**합니다.

```rust,ignore
// Solana 트랜잭션은 사용할 계정 목록을 미리 선언
Transaction {
    accounts: [user_a, token_account_a],  // 이 트랜잭션이 건드리는 계정
}

// 런타임이 계정 충돌 분석:
// Tx1: [A, B] 사용  →  병렬 실행 가능!
// Tx2: [C, D] 사용  →  A, B와 겹치지 않으므로
// Tx3: [A, E] 사용  →  A가 겹치므로 Tx1과 순차 실행
```

### 6. Pipelining - CPU 파이프라인 최적화

트랜잭션 처리를 4단계 파이프라인으로 분리합니다:
1. **데이터 수신** (네트워크 수신 유닛)
2. **서명 검증** (GPU 활용)
3. **뱅킹 처리** (CPU 코어들)
4. **블록 기록** (디스크)

각 단계가 동시에 다른 트랜잭션 배치를 처리합니다.

### 7. Cloudbreak - 수평 확장 계정 DB

Solana의 계정 데이터베이스는 SSD에 최적화된 방식으로 동시 읽기/쓰기를 지원합니다.

### 8. Archivers (현재 미구현) - 분산 원장 저장

계획상 전체 원장을 모든 노드가 저장하지 않고, Archivers(replicators)가 분산 저장하는 구조.

---

## Solana 네트워크 구조

### 클러스터 (Cluster)

Solana 네트워크는 여러 **클러스터**로 구성됩니다:

```text
Mainnet-beta: 실제 운영 네트워크 (https://api.mainnet-beta.solana.com)
Testnet:      스트레스 테스트용 (https://api.testnet.solana.com)
Devnet:       개발/테스트용, 무료 SOL 에어드롭 가능 (https://api.devnet.solana.com)
Localnet:     로컬 개발 환경 (solana-test-validator)
```

Node.js 개발자에게 익숙한 개념으로: `NODE_ENV=development|staging|production`과 유사합니다.

### 검증자 (Validator)

검증자는 Solana 네트워크의 노드입니다. 각 검증자는:
- 전체 블록체인 상태를 유지
- 트랜잭션을 검증하고 블록에 투표
- 스테이킹된 SOL 양에 비례하여 리더로 선택

현재 ~2,000개 이상의 검증자가 운영 중입니다.

### 리더 (Leader) / 슬롯 (Slot)

```text
Epoch (약 2-3일)
├── Slot 0: 리더 A (400ms)
├── Slot 1: 리더 A (400ms)
├── Slot 2: 리더 A (400ms)
├── Slot 3: 리더 A (400ms)  ← 연속 4 슬롯
├── Slot 4: 리더 B (400ms)
├── Slot 5: 리더 B (400ms)
├── ...
└── Slot N: ...
```

- 하나의 **슬롯** = ~400ms, 하나의 블록에 해당
- 각 슬롯마다 한 명의 **리더**(슬롯 리더)가 트랜잭션을 처리하고 블록을 생성
- 리더 스케줄은 에폭 시작 시 스테이크 가중치에 따라 미리 결정됨
- Gulf Stream 덕분에 다음 리더가 누구인지 알고 트랜잭션을 미리 전달 가능

---

## 현재 Solana의 발전 현황

### Firedancer

Jump Crypto가 개발 중인 Solana의 두 번째 검증자 클라이언트입니다 (2024년 mainnet 출시).

```text
현재:  Solana Labs 클라이언트 (Rust) ← 모든 검증자가 사용
목표:  Firedancer (C/C++)  ← Jump Crypto 개발, 100만 TPS 목표

이더리움과의 비교:
이더리움은 클라이언트 다양성(Geth, Prysm, Lighthouse 등)이 잘 갖춰져 있음
Solana는 Firedancer로 이 약점을 보완 중
```

**Solana Labs 클라이언트가 Rust를 선택한 이유:**
- **GC 없는 예측 가능한 레이턴시**: PoH 해시 체인은 SHA256을 연속으로 계산하는 단일 경로로, GC 일시정지가 끼어들면 슬롯 타이밍(400ms)이 어긋남
- **Sealevel 병렬 처리**: 서로 다른 계정에 접근하는 트랜잭션을 동시에 실행할 때 데이터 레이스를 컴파일 타임에 차단
- **두려움 없는 동시성**: 검증자 내부의 네트워크 수신·서명 검증·뱅킹·디스크 기록 4단계 파이프라인을 스레드 안전하게 구현
- **WASM 컴파일 지원**: 스마트 컨트랙트(Program)를 Rust로 작성해 BPF 바이트코드로 컴파일, 단일 언어로 인프라와 컨트랙트를 모두 개발 가능

Firedancer의 특징:
- C/C++로 재작성하여 극한의 성능 최적화
- 독립적인 코드베이스로 클라이언트 다양성 확보
- 목표 TPS: 100만+ (이론적)

### Alpenglow

2025년 발표된 Solana의 새로운 합의 프로토콜입니다. Tower BFT를 대체하는 것이 목표입니다:

- **Rotor**: Turbine의 개선 버전, 더 효율적인 블록 전파
- **Votor**: 새로운 투표 메커니즘, 단일 슬롯 내 확정(single-slot finality) 목표

현재 Tower BFT는 완전한 확정(finality)에 32개 슬롯 (~12.8초)이 필요합니다. Alpenglow는 이를 **단일 슬롯(~400ms)** 내에 달성하는 것을 목표로 합니다.

---

## 개발 환경 설정

```bash
# Solana CLI 설치
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# 설치 확인
solana --version

# Devnet으로 설정
solana config set --url devnet

# 현재 설정 확인
solana config get
# Config File: ~/.config/solana/cli/config.yml
# RPC URL: https://api.devnet.solana.com
# WebSocket URL: wss://api.devnet.solana.com/
# Keypair Path: ~/.config/solana/id.json

# 새 지갑 생성
solana-keygen new

# Devnet에서 무료 SOL 받기 (에어드롭)
solana airdrop 2

# 잔액 확인
solana balance
# 2 SOL

# 로컬 테스트 검증자 실행
solana-test-validator
```

---

## 다음 장 미리보기

이제 Solana의 전체적인 구조를 이해했습니다. 다음 장에서는 Solana의 가장 핵심적인 개념인 **Account 모델**을 깊이 파고들겠습니다. "Solana에서는 모든 것이 Account다"라는 말의 의미를 코드와 함께 살펴봅니다.
