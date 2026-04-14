# 계정과 트랜잭션: 이더리움의 기본 단위

이더리움의 모든 활동은 **계정(Account)**과 **트랜잭션(Transaction)**으로 이루어진다. Node.js 개발자라면 계정을 "데이터베이스의 레코드", 트랜잭션을 "상태를 변경하는 API 요청"으로 이해하면 쉽다.

---

## 1. 두 종류의 계정

이더리움에는 근본적으로 다른 두 종류의 계정이 있다.

```
이더리움 계정 종류:

┌─────────────────────────────┐  ┌─────────────────────────────┐
│  EOA                        │  │  CA (Contract Account)      │
│  (Externally Owned Account) │  │                             │
│  외부 소유 계정             │  │  컨트랙트 계정              │
├─────────────────────────────┤  ├─────────────────────────────┤
│ ✅ 개인 키(Private Key) 있음 │  │ ❌ 개인 키 없음             │
│ ✅ 트랜잭션 시작 가능        │  │ ❌ 스스로 TX 시작 불가      │
│ ❌ 코드 없음                 │  │ ✅ 코드(바이트코드) 있음    │
│ ❌ 자동 실행 없음            │  │ ✅ 호출 시 코드 실행        │
│                             │  │ ✅ 스토리지(상태) 있음      │
│ 예: 개인 지갑, MetaMask     │  │ 예: Uniswap, ERC-20 토큰   │
└─────────────────────────────┘  └─────────────────────────────┘
```

### 1.1 EOA (외부 소유 계정)

- 사람(또는 프로그램)이 **개인 키**로 제어
- 트랜잭션을 **시작**할 수 있는 유일한 계정 유형
- 주소는 공개 키에서 파생: `address = keccak256(pubkey)[12:]`
- 예: `0x742d35Cc6634C0532925a3b8D4C9b845b3A09b92`

```javascript
// ethers.js로 EOA 생성
const { ethers } = require("ethers");

// 새 지갑(EOA) 생성
const wallet = ethers.Wallet.createRandom();
console.log("주소:", wallet.address);
console.log("비밀키:", wallet.privateKey);
console.log("니모닉:", wallet.mnemonic.phrase);

// 기존 비밀키로 지갑 복원
const restored = new ethers.Wallet("0xac0974bec...");
console.log("복원된 주소:", restored.address);
```

### 1.2 CA (컨트랙트 계정)

- **개인 키가 없다** → 아무도 "소유"하지 않음
- 코드가 규칙. 코드에 정의된 대로만 동작
- EOA 또는 다른 CA에 의해 호출될 때만 실행됨
- 주소는 배포자 주소와 nonce로 결정: `address = keccak256(deployer, nonce)`

```
CA 호출 흐름:
사용자(EOA) ──TX──▶ Uniswap 컨트랙트(CA) ──내부 호출──▶ 토큰 컨트랙트(CA)
                          │
                     코드 실행
                     (Swap 로직)
```

---

## 2. 계정 상태

이더리움 글로벌 상태는 모든 계정의 상태를 담은 **거대한 key-value 저장소**다. 각 계정은 4개의 필드를 가진다.

```
계정 상태 구조:

address → {
  nonce:       u64,      // EOA: 발송한 TX 수 / CA: 생성한 컨트랙트 수
  balance:     u256,     // 보유 ETH (wei 단위)
  storageRoot: bytes32,  // 스토리지 내용의 머클 루트 (CA만 의미 있음)
  codeHash:    bytes32,  // 컨트랙트 코드의 Keccak 해시 (EOA는 빈 해시)
}
```

**실제 예시:**
```
EOA (Alice의 지갑):
{
  nonce:       42,                  // 42번 트랜잭션을 보냄
  balance:     5_000_000_000_000_000_000, // 5 ETH (wei)
  storageRoot: 0x56e81f...0000,    // 빈 스토리지 (EOA는 스토리지 없음)
  codeHash:    0xc5d2460...0000,   // 빈 코드 해시
}

CA (ERC-20 토큰 컨트랙트):
{
  nonce:       1,                   // 컨트랙트 배포 시 1
  balance:     0,                   // 토큰 컨트랙트 자체는 ETH 없음
  storageRoot: 0x3a7f8c...,        // 잔액 맵 등 스토리지
  codeHash:    0xbf1a9c...,        // ERC-20 바이트코드 해시
}
```

이더리움의 글로벌 상태 전체는 **패트리샤 머클 트라이(Patricia Merkle Trie)**라는 자료구조에 저장된다. 블록 헤더의 `stateRoot`가 이 트라이의 루트 해시다.

---

## 3. 트랜잭션 타입

이더리움은 역사적으로 여러 트랜잭션 타입이 추가되었다.

### 3.1 타입 0 — 레거시 트랜잭션

EIP-2718 이전의 원래 형식:

```javascript
{
  nonce:    42,
  gasPrice: 20_000_000_000,  // 20 Gwei (고정 가격)
  gasLimit: 21_000,
  to:       "0xBob...",
  value:    ethers.parseEther("1.0"),
  data:     "0x",
  v:        27,   // 체인 ID 반영 (EIP-155)
  r:        "0x...",
  s:        "0x...",
}
```

문제: `gasPrice`가 고정이라 가스비 예측이 어렵고, 채굴자가 모든 수수료를 가져감.

### 3.2 타입 1 — EIP-2930 (Access List)

2021년 도입. 사전에 접근할 스토리지 슬롯을 선언해 가스비 절감:

```javascript
{
  type:       1,
  chainId:    1,
  nonce:      42,
  gasPrice:   20_000_000_000,
  gasLimit:   50_000,
  to:         "0xContract...",
  value:      0,
  data:       "0x...",
  accessList: [          // 미리 선언하면 가스비 절감
    {
      address: "0xToken...",
      storageKeys: ["0x0000...0001"],
    }
  ],
}
```

### 3.3 타입 2 — EIP-1559 (현재 표준)

2021년 London 업그레이드에서 도입. 수수료 시장 개혁:

```javascript
{
  type:                 2,
  chainId:              1,
  nonce:                42,
  maxFeePerGas:         30_000_000_000,  // 최대 30 Gwei (지불 상한)
  maxPriorityFeePerGas: 2_000_000_000,   // 최대 2 Gwei (채굴자 팁)
  gasLimit:             21_000,
  to:                   "0xBob...",
  value:                ethers.parseEther("1.0"),
  data:                 "0x",
  accessList:           [],
}
```

**EIP-1559 수수료 구조:**
```
실제 지불 가스비 = min(maxFeePerGas, baseFee + maxPriorityFeePerGas)

baseFee:    프로토콜이 결정 (소각됨! 채굴자에게 가지 않음)
tip:        maxPriorityFeePerGas (검증자에게 인센티브)
환불:       maxFeePerGas - (baseFee + tip) 는 돌려받음

예:
  baseFee = 15 Gwei
  maxPriorityFeePerGas = 2 Gwei
  maxFeePerGas = 30 Gwei
  
  실제 지불 = 15 + 2 = 17 Gwei/gas
  환불 = 30 - 17 = 13 Gwei/gas
  소각 = 15 Gwei/gas (baseFee)
  검증자 수익 = 2 Gwei/gas (tip)
```

---

## 4. nonce의 역할 (매우 중요!)

nonce는 단순한 카운터처럼 보이지만 두 가지 중요한 역할을 한다.

### 4.1 재전송 공격(Replay Attack) 방지

```
공격 시나리오 (nonce 없이):
  1. Alice가 "Bob에게 1 ETH 전송" TX에 서명
  2. 공격자가 이 서명된 TX를 가로챔
  3. 공격자가 같은 TX를 100번 재전송
  4. Alice의 계좌에서 100 ETH가 빠져나감!

nonce가 있으면:
  1. Alice의 첫 TX: nonce=5
  2. 이더리움이 TX 처리 후 Alice.nonce = 6으로 업데이트
  3. 공격자가 nonce=5 TX를 재전송
  4. 이더리움이 "Alice의 nonce는 이미 6이야, nonce=5 TX는 거부"
```

### 4.2 트랜잭션 순서 보장

```
Alice가 빠르게 3개의 TX를 전송:
  TX_A: nonce=5, "Bob에게 1 ETH"
  TX_B: nonce=6, "컨트랙트 배포"
  TX_C: nonce=7, "배포된 컨트랙트 호출"

이더리움은 반드시 nonce 순서대로 처리:
  nonce=5 처리 → nonce=6 처리 → nonce=7 처리

TX_B(컨트랙트 배포)보다 TX_C(컨트랙트 호출)가 먼저 
처리되는 일은 절대 일어나지 않음!

주의: nonce=6이 멤풀에 없으면 nonce=7은 대기 상태로 막힘
```

---

## 5. ethers.js로 트랜잭션 전체 흐름 구현

```javascript
const { ethers } = require("ethers");

async function demonstrateTransactions() {
  // 1. 프로바이더 연결 (로컬 Besu 노드)
  const provider = new ethers.JsonRpcProvider("http://localhost:8545");
  
  // 2. 지갑 생성 (Besu 개발 계정 사용)
  const privateKey = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
  const wallet = new ethers.Wallet(privateKey, provider);
  
  console.log("=== 계정 정보 ===");
  console.log("주소:", wallet.address);
  
  // 3. 계정 상태 조회
  const balance = await provider.getBalance(wallet.address);
  const nonce = await provider.getTransactionCount(wallet.address);
  console.log("잔액:", ethers.formatEther(balance), "ETH");
  console.log("nonce:", nonce);
  
  // 4. 트랜잭션 수동 구성 (타입 2, EIP-1559)
  const recipient = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
  
  const txRequest = {
    to:                   recipient,
    value:                ethers.parseEther("0.1"),
    gasLimit:             21_000n,
    maxFeePerGas:         ethers.parseUnits("30", "gwei"),
    maxPriorityFeePerGas: ethers.parseUnits("2", "gwei"),
    nonce:                nonce,
    chainId:              31337n,  // Hardhat/Besu 로컬
    type:                 2,
  };
  
  // 5. 트랜잭션 서명 (아직 전송 안 함)
  const signedTx = await wallet.signTransaction(txRequest);
  console.log("\n=== 서명된 트랜잭션 ===");
  console.log("서명된 TX (RLP encoded):", signedTx.slice(0, 40) + "...");
  
  // 6. 트랜잭션 전송 (더 간단한 방법)
  console.log("\n=== 트랜잭션 전송 ===");
  const tx = await wallet.sendTransaction({
    to: recipient,
    value: ethers.parseEther("0.1"),
  });
  
  console.log("TX 해시:", tx.hash);
  console.log("nonce:", tx.nonce);
  console.log("gasLimit:", tx.gasLimit.toString());
  console.log("maxFeePerGas:", ethers.formatUnits(tx.maxFeePerGas ?? 0n, "gwei"), "Gwei");
  
  // 7. 확인 대기
  console.log("\n블록 포함 대기 중...");
  const receipt = await tx.wait(1); // 1 컨펌 대기
  
  // 8. 트랜잭션 영수증 분석
  console.log("\n=== 트랜잭션 영수증 ===");
  console.log("상태:", receipt.status === 1 ? "성공" : "실패");
  console.log("블록 번호:", receipt.blockNumber);
  console.log("가스 사용:", receipt.gasUsed.toString());
  
  const gasCost = receipt.gasUsed * (receipt.gasPrice ?? 0n);
  console.log("실제 가스비:", ethers.formatEther(gasCost), "ETH");
  console.log("이벤트 로그 수:", receipt.logs.length);
  
  // 9. 잔액 변화 확인
  const newBalance = await provider.getBalance(wallet.address);
  console.log("\n=== 잔액 변화 ===");
  console.log("이전 잔액:", ethers.formatEther(balance), "ETH");
  console.log("이후 잔액:", ethers.formatEther(newBalance), "ETH");
  console.log("차이:", ethers.formatEther(balance - newBalance), "ETH (전송량 + 가스비)");
}

demonstrateTransactions().catch(console.error);
```

### 5.1 트랜잭션 영수증 (Receipt) 상세

```javascript
// 트랜잭션 영수증 구조
const receipt = {
  // 상태
  status:            1,          // 1=성공, 0=실패(revert)
  
  // 블록 정보
  blockHash:         "0x...",    // 포함된 블록의 해시
  blockNumber:       18500000,   // 포함된 블록 번호
  transactionIndex:  42,         // 블록 내 TX 순서
  
  // 가스 정보
  gasUsed:           21000n,     // 실제 사용된 가스
  cumulativeGasUsed: 500000n,    // 블록 내 이 TX까지 누적 가스
  effectiveGasPrice: 17000000000n, // 실제 지불된 가스 가격 (wei)
  
  // 컨트랙트 관련
  contractAddress:   null,       // 컨트랙트 배포 시 새 주소, 일반 TX는 null
  
  // 이벤트 로그 (스마트 컨트랙트가 발행한 이벤트)
  logs: [
    {
      address: "0xToken...",      // 이벤트 발행 컨트랙트
      topics:  ["0xddf252..."],   // Transfer 이벤트 시그니처
      data:    "0x00...0001",     // 이벤트 데이터
    }
  ],
};
```

---

## 6. 트랜잭션 타입별 사용 가이드

```javascript
// 단순 ETH 전송 (ethers.js가 자동으로 EIP-1559 사용)
const simpleTx = await wallet.sendTransaction({
  to: recipient,
  value: ethers.parseEther("1.0"),
});

// 가스비 수동 설정이 필요한 경우
const urgentTx = await wallet.sendTransaction({
  to: recipient,
  value: ethers.parseEther("1.0"),
  maxFeePerGas:         ethers.parseUnits("100", "gwei"), // 긴급, 높은 상한
  maxPriorityFeePerGas: ethers.parseUnits("5", "gwei"),   // 높은 팁
});

// 현재 가스 가격 조회 (EIP-1559)
const feeData = await provider.getFeeData();
console.log("baseFee:", ethers.formatUnits(feeData.gasPrice ?? 0n, "gwei"), "Gwei");
console.log("maxFeePerGas:", ethers.formatUnits(feeData.maxFeePerGas ?? 0n, "gwei"), "Gwei");
console.log("maxPriorityFeePerGas:", ethers.formatUnits(feeData.maxPriorityFeePerGas ?? 0n, "gwei"), "Gwei");

// 트랜잭션 조회 (이미 전송된 것)
const txDetails = await provider.getTransaction("0xabcd...");
console.log("from:", txDetails?.from);
console.log("to:", txDetails?.to);
console.log("value:", ethers.formatEther(txDetails?.value ?? 0n));

// 컨트랙트 배포 트랜잭션 (to가 null)
const deployTx = await wallet.sendTransaction({
  to:   null,   // 또는 undefined
  data: "0x6080604052...", // 컨트랙트 바이트코드
  value: 0n,
});
```

---

## 7. 실용적인 패턴: 트랜잭션 모니터링

```javascript
// 특정 주소의 트랜잭션을 실시간 모니터링
async function watchAddress(provider, address) {
  console.log(`${address} 모니터링 시작...`);
  
  // 새 블록마다 확인
  provider.on("block", async (blockNumber) => {
    const block = await provider.getBlock(blockNumber, true); // TX 포함
    
    for (const tx of block.prefetchedTransactions) {
      if (tx.from === address || tx.to === address) {
        console.log(`\n새 TX 감지! 블록 #${blockNumber}`);
        console.log("  해시:", tx.hash);
        console.log("  from:", tx.from);
        console.log("  to:", tx.to);
        console.log("  값:", ethers.formatEther(tx.value), "ETH");
      }
    }
  });
}

// pending TX 모니터링 (Mempool 관찰)
provider.on("pending", (txHash) => {
  console.log("새 Pending TX:", txHash);
});
```

---

## 8. 핵심 정리

| 개념 | 설명 | Node.js 비유 |
|------|------|-------------|
| **EOA** | 개인 키로 제어하는 계정 | API 클라이언트 (요청 시작자) |
| **CA** | 코드로 동작하는 계정 | 서버 엔드포인트 (로직 실행자) |
| **nonce** | TX 순서 번호 | API 요청의 시퀀스 ID |
| **gasLimit** | 최대 허용 계산량 | API 타임아웃 설정 |
| **baseFee** | 네트워크 기본 수수료 | 서버 기본 처리 비용 |
| **tip** | 검증자 인센티브 | 빠른 처리를 위한 프리미엄 |
| **receipt** | TX 처리 결과 | HTTP 응답 (status, body) |
| **logs** | 컨트랙트 이벤트 | 서버 측 이벤트 (SSE, WebSocket) |

다음 챕터에서는 이더리움의 심장부인 **EVM(이더리움 가상 머신)**과 **가스 시스템**을 깊이 파고든다.
