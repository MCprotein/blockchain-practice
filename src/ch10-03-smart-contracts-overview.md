# 스마트 컨트랙트 개요: 블록체인 위의 자동 실행 프로그램

> **스마트 컨트랙트를 한 문장으로**: "조건이 충족되면 자동으로 실행되는, 블록체인 위의 변경 불가능한 프로그램"

Node.js 개발자로서 여러분은 이미 "코드가 비즈니스 로직을 처리한다"는 개념에 익숙하다. 스마트 컨트랙트는 그 코드가 **중앙 서버 대신 블록체인 위에서 실행**된다는 점이 다르다. 한번 배포하면 누구도 — 심지어 개발자 본인도 — 멈추거나 수정할 수 없다.

---

## 1. 스마트 컨트랙트란?

### 1.1 Nick Szabo의 자판기 비유

1994년, Nick Szabo는 스마트 컨트랙트 개념을 처음 제안하며 **자판기**에 비유했다:

```
자판기:
  1. 돈을 넣는다 (조건 충족)
  2. 버튼을 누른다 (함수 호출)
  3. 음료가 나온다 (자동 실행)
  4. 거스름돈이 나온다 (결과 반환)
  
  → 점원(중개자) 없이 규칙대로 자동 동작
  → 조건과 결과가 기계에 내장되어 있음

스마트 컨트랙트:
  1. ETH를 전송한다 (조건 충족)
  2. 함수를 호출한다 (트랜잭션)
  3. 코드가 실행된다 (자동 실행)
  4. 토큰 등이 발행/전송된다 (결과 반환)
  
  → 은행, 변호사, 중개인 없이 코드대로 동작
  → 조건과 결과가 블록체인에 내장되어 있음
```

### 1.2 Node.js 백엔드와의 비교

```javascript
// 전통적인 Node.js 백엔드
app.post('/transfer', authenticate, async (req, res) => {
  const { to, amount } = req.body;
  
  // 이 로직을 서버 운영자가 언제든 바꿀 수 있음
  // 서버가 다운되면 서비스 중단
  // DB 관리자가 잔액을 임의로 수정 가능
  
  await db.query(
    'UPDATE accounts SET balance = balance - $1 WHERE id = $2',
    [amount, req.user.id]
  );
  await db.query(
    'UPDATE accounts SET balance = balance + $1 WHERE id = $2',
    [amount, to]
  );
  res.json({ success: true });
});

// 스마트 컨트랙트 (Solidity)
// 배포 후 이 코드는 영원히 고정됨
// 이더리움이 살아있는 한 항상 동작
// 누구도 임의로 잔액 변경 불가
function transfer(address to, uint256 amount) public {
    require(balances[msg.sender] >= amount, "잔액 부족");
    balances[msg.sender] -= amount;
    balances[to] += amount;
    emit Transfer(msg.sender, to, amount);
}
```

---

## 2. 불변성: 한번 배포하면 수정 불가

### 2.1 불변성의 의미

```
컨트랙트 주소: 0x6B175474E89094C44Da98b954EedeAC495271d0F
(DAI 스테이블코인, 2019년 배포)

이 주소에 있는 코드:
  - DAI 팀도 수정 불가
  - 이더리움 재단도 수정 불가
  - 어떤 정부도 중단 불가
  - 코드에 버그가 있으면 영원히 유지됨 (!)
  - 코드가 올바르면 영원히 신뢰 가능 (!)
```

**불변성이 강점인 이유:**
- 사용자가 코드를 신뢰할 수 있음 (나중에 바뀌지 않음)
- 감사(audit)된 코드가 변경되지 않음
- "코드가 곧 계약(Law)"이 가능

**불변성의 위험:**
- 버그를 수정할 수 없음
- 보안 취약점이 발견되어도 즉시 패치 불가
- 2016년 The DAO 해킹이 대표적 사례

### 2.2 업그레이드 가능한 컨트랙트 패턴

불변성의 단점을 보완하기 위해 **프록시 패턴(Proxy Pattern)**이 개발되었다:

```
프록시 패턴:

사용자 ──▶ Proxy Contract     ──▶ Implementation V1
           (주소 변하지 않음)       (실제 로직, 교체 가능)
           (스토리지 유지)
                    ──업그레이드──▶ Implementation V2
                                   (버그 수정된 버전)

핵심:
  - 사용자는 항상 Proxy 주소를 사용
  - 로직(Implementation)만 교체
  - 데이터(Storage)는 Proxy에 유지됨
  - 투명 프록시, UUPS, 다이아몬드 패턴 등 다양한 변형
```

**그러나 주의**: 업그레이드 가능한 컨트랙트는 "신뢰의 중앙화"를 일부 포기한다. 업그레이드 권한을 가진 주소(보통 멀티시그)가 생기기 때문이다.

---

## 3. 컨트랙트의 수명 주기

```
1. 작성 (Write)
   개발자가 Solidity로 비즈니스 로직 작성
   
   ┌─────────────────────────┐
   │  MyToken.sol            │
   │  pragma solidity ^0.8;  │
   │  contract MyToken {     │
   │    ...                  │
   │  }                      │
   └─────────────────────────┘

2. 컴파일 (Compile)
   solc 컴파일러가 바이트코드 + ABI 생성
   
   ┌──────────────────┐   ┌──────────────┐
   │  바이트코드       │   │   ABI        │
   │  6080604052...   │   │  [{          │
   │  (EVM 기계어)    │   │    "name":   │
   │                  │   │    "transfer"│
   │                  │   │    ...       │
   └──────────────────┘   └──────────────┘

3. 테스트 (Test)
   로컬 환경에서 단위 테스트 / 통합 테스트
   Hardhat, Foundry 등 사용
   
4. 감사 (Audit) — 선택적이지만 중요
   보안 전문가가 코드 검토
   Slither, MythX 등 자동화 도구 사용
   
5. 배포 (Deploy)
   바이트코드를 담은 트랜잭션을 전송
   새 계약 주소가 생성됨
   
   TX { to: null, data: 바이트코드 }
   → 컨트랙트 주소: 0xAbCd...

6. 상호작용 (Interact)
   사용자 또는 다른 컨트랙트가 함수 호출
   
   TX { to: 컨트랙트주소, data: 함수+인자 }

7. (선택) 자기소멸 (Self-Destruct)
   SELFDESTRUCT 옵코드로 컨트랙트 파기
   잔여 ETH는 지정 주소로 전송
   → Cancun 업그레이드 이후 기능 제한됨
```

---

## 4. ABI: 컨트랙트의 API 명세

### 4.1 ABI란?

**ABI (Application Binary Interface)**는 컨트랙트의 **함수와 이벤트 목록**이다. Node.js 개발자에게는 REST API의 **OpenAPI/Swagger 명세**와 동일한 역할을 한다.

```
REST API 세계:
  Swagger/OpenAPI → API 명세 (엔드포인트, 파라미터, 응답 형식)
  클라이언트가 이 명세를 보고 API 호출 방법을 앎

블록체인 세계:
  ABI → 컨트랙트 명세 (함수, 이벤트, 파라미터 타입)
  클라이언트가 ABI를 보고 컨트랙트 호출 방법을 앎
```

### 4.2 ABI 구조

```json
[
  {
    "type": "function",
    "name": "transfer",
    "inputs": [
      { "name": "to",     "type": "address" },
      { "name": "amount", "type": "uint256" }
    ],
    "outputs": [
      { "name": "", "type": "bool" }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "balanceOf",
    "inputs": [
      { "name": "account", "type": "address" }
    ],
    "outputs": [
      { "name": "", "type": "uint256" }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "Transfer",
    "inputs": [
      { "name": "from",  "type": "address", "indexed": true },
      { "name": "to",    "type": "address", "indexed": true },
      { "name": "value", "type": "uint256", "indexed": false }
    ]
  },
  {
    "type": "constructor",
    "inputs": [
      { "name": "initialSupply", "type": "uint256" }
    ],
    "stateMutability": "nonpayable"
  }
]
```

### 4.3 함수 시그니처와 셀렉터

컨트랙트 함수 호출 시 어떤 함수를 호출하는지를 4바이트로 인코딩한다:

```
함수 시그니처: transfer(address,uint256)
Keccak-256:   a9059cbb2ab09eb219583f4a59a5d0623ade346d962bcd4e46b11da047c9049b
앞 4바이트:   0xa9059cbb  ← 이것이 함수 셀렉터

TX의 data 필드:
  0xa9059cbb                               ← 함수 셀렉터 (4바이트)
  000000000000000000000000742d35cc...      ← to 주소 (32바이트 패딩)
  0000000000000000000000000000000000000000000000000de0b6b3a7640000  ← amount (1 ETH in wei)
```

### 4.4 ethers.js로 ABI 활용

```javascript
const { ethers } = require("ethers");

const provider = new ethers.JsonRpcProvider("http://localhost:8545");
const wallet = new ethers.Wallet(privateKey, provider);

// ERC-20 ABI (최소 버전)
const ERC20_ABI = [
  "function name() view returns (string)",
  "function symbol() view returns (string)",
  "function decimals() view returns (uint8)",
  "function totalSupply() view returns (uint256)",
  "function balanceOf(address account) view returns (uint256)",
  "function transfer(address to, uint256 amount) returns (bool)",
  "function approve(address spender, uint256 amount) returns (bool)",
  "function allowance(address owner, address spender) view returns (uint256)",
  "event Transfer(address indexed from, address indexed to, uint256 value)",
  "event Approval(address indexed owner, address indexed spender, uint256 value)",
];

// 컨트랙트 인스턴스 생성 (ABI + 주소)
const tokenAddress = "0xYourToken...";
const token = new ethers.Contract(tokenAddress, ERC20_ABI, wallet);

async function interactWithToken() {
  // 1. view 함수 호출 (읽기, 가스 없음)
  const name = await token.name();
  const symbol = await token.symbol();
  const decimals = await token.decimals();
  const totalSupply = await token.totalSupply();
  
  console.log(`토큰: ${name} (${symbol})`);
  console.log(`소수점: ${decimals}`);
  console.log(`총 발행량: ${ethers.formatUnits(totalSupply, decimals)}`);
  
  // 2. 잔액 조회
  const myBalance = await token.balanceOf(wallet.address);
  console.log(`내 잔액: ${ethers.formatUnits(myBalance, decimals)} ${symbol}`);
  
  // 3. 토큰 전송 (쓰기 함수, 가스 필요)
  const recipient = "0xRecipient...";
  const amount = ethers.parseUnits("100", decimals); // 100 토큰
  
  const tx = await token.transfer(recipient, amount);
  console.log("TX 해시:", tx.hash);
  
  const receipt = await tx.wait();
  console.log("블록에 포함됨:", receipt.blockNumber);
  
  // 4. 이벤트 로그 파싱
  for (const log of receipt.logs) {
    try {
      const parsed = token.interface.parseLog(log);
      if (parsed?.name === "Transfer") {
        console.log(`Transfer: ${parsed.args.from} → ${parsed.args.to}`);
        console.log(`  금액: ${ethers.formatUnits(parsed.args.value, decimals)}`);
      }
    } catch {}
  }
  
  // 5. 과거 이벤트 조회
  const filter = token.filters.Transfer(wallet.address, null);
  const events = await token.queryFilter(filter, -1000); // 최근 1000블록
  console.log(`내가 보낸 Transfer 이벤트: ${events.length}개`);
}
```

---

## 5. 대표적인 스마트 컨트랙트 유형

### 5.1 토큰 컨트랙트 (ERC-20, ERC-721)

```
ERC-20 (대체 가능 토큰):
  - 모든 토큰이 동등 (1 USDC == 1 USDC)
  - 화폐, 거버넌스 토큰, 유틸리티 토큰
  - 예: USDC, DAI, UNI, LINK
  - 핵심 함수: transfer, approve, transferFrom, balanceOf

ERC-721 (대체 불가능 토큰, NFT):
  - 각 토큰이 고유 (ID가 있음)
  - 디지털 아트, 게임 아이템, 도메인
  - 예: CryptoPunks, BAYC, ENS
  - 핵심 함수: transferFrom, ownerOf, tokenURI

ERC-1155 (멀티 토큰):
  - ERC-20 + ERC-721 혼합
  - 게임 아이템 (여러 종류, 각 종류마다 여러 개)
  - 예: Enjin, OpenSea 컨트랙트
```

### 5.2 DEX (탈중앙화 거래소)

```
AMM (Automated Market Maker) 방식:

전통 거래소:           AMM (Uniswap 등):
  매수 주문 ──▶ 오더북    유동성 풀 ──▶ 자동 가격 결정
  매도 주문 ──▶ 매칭      x × y = k  (불변 곱 공식)
  
예: ETH/USDC 풀
  ETH 100개, USDC 200,000개 (1 ETH = 2,000 USDC)
  
  사용자가 1 ETH를 USDC로 교환:
  새 ETH = 100 + 1 = 101
  새 USDC = 200,000 × 100 / 101 ≈ 198,020
  받는 USDC = 200,000 - 198,020 ≈ 1,980 USDC
  (슬리피지 때문에 정확히 2,000이 아님)
```

### 5.3 대출/차입 프로토콜

```
Aave / Compound 방식:

공급자:  ETH 예치 → aETH 토큰 수령 → 이자 수익
차입자:  담보 예치 → 다른 자산 차입 → 이자 지불

과담보(Over-collateralized):
  100 ETH 예치 (담보)
  → 최대 75 ETH 가치의 USDC 차입 가능
  
청산(Liquidation):
  ETH 가격 하락 → 담보 비율 위험 수준
  → 청산인이 담보를 싸게 구매해 부채 상환
  → 자동으로 스마트 컨트랙트가 실행
```

### 5.4 DAO (탈중앙화 자율 조직)

```
전통 조직:          DAO:
  이사회 의결       토큰 보유자 투표
  CEO 결정          거버넌스 컨트랙트가 자동 실행
  법적 구조         코드가 규칙
  
예: Uniswap DAO
  UNI 토큰 보유자가 프로토콜 수수료, 업그레이드 등 투표
  투표 결과가 스마트 컨트랙트로 자동 실행
  (인간의 개입 최소화)
```

---

## 6. 스마트 컨트랙트의 한계와 주의사항

```
주의사항 1: 오라클 문제 (Oracle Problem)
  스마트 컨트랙트는 블록체인 밖의 데이터를 모름
  "ETH 현재 가격은?" → 알 수 없음!
  해결: Chainlink 같은 오라클 서비스가 외부 데이터 공급
  
주의사항 2: 가스 비용
  모든 계산에 비용 발생
  복잡한 로직 = 비싼 가스비 = 사용자 부담
  
주의사항 3: 보안 취약점
  재진입 공격 (Reentrancy): The DAO 해킹의 원인
  정수 오버플로우: Solidity 0.8 이후 자동 방어
  플래시 론 공격: 순식간에 대량 자본 조달 후 조작
  
주의사항 4: 업그레이드 불가 (기본)
  버그 수정을 위해 새 컨트랙트 배포 필요
  사용자가 새 주소로 마이그레이션해야 함
  
주의사항 5: 공개 코드
  모든 코드가 블록체인에 공개됨
  비즈니스 로직 비밀 유지 불가
  (영지식 증명으로 일부 해결 가능)
```

---

## 7. 다음 파트 미리보기: 직접 Solidity 작성하기

다음 파트(Chapter 11)에서 직접 작성할 내용:

```
Ch11. Solidity 기초
  - 변수 타입, 함수, 제어문
  - storage vs memory vs calldata
  - 접근 제어 (public, private, internal, external)
  - modifier로 권한 제어
  
Ch12. 토큰 구현
  - ERC-20 표준 직접 구현
  - OpenZeppelin 라이브러리 활용
  - 민팅, 소각, 전송 로직
  
Ch13. 고급 패턴
  - 재진입 공격 방어
  - 업그레이드 가능한 컨트랙트 (프록시 패턴)
  - 이벤트와 로그 활용
  
Ch14. 테스트와 배포
  - Hardhat으로 단위 테스트
  - 로컬 Besu 네트워크에 배포
  - 스크립트 자동화
```

Node.js 개발자로서 여러분이 이미 가진 기술들:
```
알고 있는 것 → 스마트 컨트랙트에서 대응

REST API 설계     → ABI 설계
DB 스키마 설계    → storage 변수 설계  
비즈니스 로직     → Solidity 함수
미들웨어 (auth)   → modifier
이벤트 에밋       → emit Event
API 테스트       → Hardhat 테스트
환경 변수 관리    → .env + hardhat.config
배포 스크립트     → Hardhat deploy scripts
```

---

## 8. 핵심 정리

- **스마트 컨트랙트 = 블록체인 위의 자동 실행 프로그램**: 중개자 없이 코드가 계약을 집행
- **불변성**: 배포 후 수정 불가 → 보안과 신뢰의 근거이자 동시에 위험 요소
- **수명 주기**: 작성 → 컴파일 → 테스트 → 감사 → 배포 → 상호작용
- **ABI**: 컨트랙트의 OpenAPI 명세 — 클라이언트가 함수를 어떻게 호출하는지 정의
- **주요 유형**: ERC-20 토큰, DEX(AMM), 대출 프로토콜, DAO
- **한계**: 오라클 문제, 가스 비용, 보안 취약점, 공개 코드

이제 블록체인의 개념적 기초(Chapter 9)와 이더리움 플랫폼(Chapter 10)을 완주했다. 다음 파트에서는 실제 Solidity 코드를 한 줄씩 작성하며 스마트 컨트랙트 개발자로 발전한다.
