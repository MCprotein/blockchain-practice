# EVM과 가스: 이더리움의 실행 엔진

이더리움이 "월드 컴퓨터"라면, **EVM(Ethereum Virtual Machine)**은 그 컴퓨터의 CPU다. 스마트 컨트랙트가 실행되는 환경이자, 모든 이더리움 노드가 동일하게 구현해야 하는 명세다.

---

## 1. EVM이란?

### 1.1 가상 머신의 개념

Node.js 개발자에게 가상 머신은 익숙한 개념이다.

```text
Java의 JVM:
  Java 코드 → 컴파일 → .class(바이트코드) → JVM이 실행
  어떤 OS에서도 동일하게 동작 ("Write once, run anywhere")

Node.js의 V8:
  JavaScript → V8 엔진이 JIT 컴파일 → 네이티브 코드로 실행

EVM:
  Solidity → 컴파일 → EVM 바이트코드 → EVM이 실행
  어떤 이더리움 노드에서도 동일한 결과 보장
```

**핵심 차이**: JVM이나 V8은 파일 시스템, 네트워크 등에 접근할 수 있다. EVM은 철저히 격리(Sandboxed)되어 있다. 외부 세계와의 유일한 접점은 블록체인 상태(스토리지, 잔액)뿐이다.

### 1.2 EVM의 종류

```text
구현체별 EVM:
  Geth    (Go)    → go-ethereum의 EVM
  Besu    (Java)  → Hyperledger Besu의 EVM
  Reth    (Rust)  → Reth의 EVM (revm 라이브러리)
  Nethermind (C#) → Nethermind의 EVM

모두 동일한 이더리움 명세를 따름
→ 같은 컨트랙트를 실행하면 항상 같은 결과
```

---

## 2. EVM 아키텍처: 스택 기반 머신

### 2.1 세 가지 저장 영역

```text
EVM 실행 컨텍스트:

┌─────────────────────────────────────────────────────────┐
│                      EVM                                 │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │   스택       │  │   메모리     │  │   스토리지    │  │
│  │  (Stack)    │  │  (Memory)   │  │  (Storage)   │  │
│  │             │  │             │  │              │  │
│  │ 최대 1024개  │  │ 바이트 배열  │  │ key-value    │  │
│  │ 각 256비트   │  │ 실행 중만   │  │ 영구 저장    │  │
│  │ LIFO 구조   │  │ 존재        │  │ 블록체인에   │  │
│  │             │  │ 동적 확장   │  │ 기록됨       │  │
│  └──────────────┘  └──────────────┘  └───────────────┘  │
│       빠름              중간              느리고 비쌈      │
└─────────────────────────────────────────────────────────┘
```

**스택 (Stack)**
- EVM의 주 작업 공간
- 모든 연산은 스택에서 이루어짐
- LIFO (Last In First Out) 구조
- 최대 깊이: 1024
- 각 슬롯: 256비트 (32바이트)

**메모리 (Memory)**
- 바이트 단위로 주소 지정 가능한 선형 배열
- 함수 호출 동안만 존재 (함수 종료 시 소멸)
- 동적으로 확장 가능 (확장할수록 가스 비용 증가)
- 함수 인자, 반환값, 임시 데이터 저장

**스토리지 (Storage)**
- 컨트랙트별 영구적인 key-value 저장소
- 키: 32바이트, 값: 32바이트
- 블록체인에 영구 기록 → 가장 비쌈
- Solidity의 `state variable`이 여기에 저장됨

### 2.2 Node.js와의 비교

```text
Node.js V8 엔진:
  힙(Heap):     객체, 배열 (GC가 관리)
  스택(Stack):  함수 호출, 지역 변수
  외부 접근:    파일, 네트워크, OS API

EVM:
  스토리지:     영구 상태 (블록체인)
  메모리:       임시 바이트 배열 (실행 중)
  스택:         연산 작업 공간 (256비트 슬롯)
  외부 접근:    불가능! (완전 격리)
  
V8은 JIT 컴파일로 최적화된 네이티브 코드 실행
EVM은 바이트코드를 인터프리팅 (훨씬 느림, 하지만 결정론적)
```

### 2.3 바이트코드와 옵코드

Solidity 코드는 EVM이 이해하는 **바이트코드**로 컴파일된다. 각 바이트는 **옵코드(Opcode)**라는 명령어를 나타낸다.

```text
예: 두 숫자를 더하는 간단한 연산

Solidity:
  uint256 result = a + b;

EVM 바이트코드: (16진수)
  60 05   ← PUSH1 5   (숫자 5를 스택에 푸시)
  60 03   ← PUSH1 3   (숫자 3을 스택에 푸시)
  01      ← ADD       (스택 상위 2개를 꺼내 더한 후 결과를 푸시)

스택 변화:
  초기:       []
  PUSH1 5:   [5]
  PUSH1 3:   [5, 3]
  ADD:        [8]   ← 결과
```

주요 옵코드 목록:

| 옵코드 | 값 | 설명 | 가스 |
|--------|-----|------|------|
| STOP | 0x00 | 실행 중단 | 0 |
| ADD | 0x01 | 스택 상위 2개 더하기 | 3 |
| MUL | 0x02 | 곱하기 | 5 |
| SUB | 0x03 | 빼기 | 3 |
| DIV | 0x04 | 나누기 | 5 |
| SLOAD | 0x54 | 스토리지에서 읽기 | 2,100 |
| SSTORE | 0x55 | 스토리지에 쓰기 | 20,000 |
| MLOAD | 0x51 | 메모리에서 읽기 | 3 |
| MSTORE | 0x52 | 메모리에 쓰기 | 3 |
| CALL | 0xf1 | 다른 컨트랙트 호출 | 가변 |
| CREATE | 0xf0 | 새 컨트랙트 배포 | 32,000 |

---

## 3. 가스 시스템

### 3.1 왜 가스가 필요한가?

```text
가스 없는 이더리움의 문제:

// 이런 컨트랙트를 배포한다면?
contract Malicious {
  function attack() public {
    while(true) {
      // 무한 루프!
    }
  }
}

→ 모든 이더리움 노드가 영원히 이 루프를 실행
→ 네트워크 전체 마비 (DoS 공격)
```

가스는 두 가지 문제를 동시에 해결한다:
1. **중단 문제(Halting Problem) 완화**: 가스가 소진되면 실행 중단
2. **자원 비용 반영**: 더 많은 계산 = 더 많은 가스 = 더 많은 비용

### 3.2 가스 기본 개념

```text
가스 = 계산 작업량의 단위

gasLimit: 이 TX에 최대 얼마나 쓸 것인가 (사용자 설정)
gasUsed:  실제로 얼마나 썼는가 (실행 후 결정됨)
gasPrice: 가스 1단위당 얼마를 낼 것인가 (wei)

총 수수료 = gasUsed × effectiveGasPrice

예:
  단순 ETH 전송: 21,000 gas (고정)
  ERC-20 transfer: ~65,000 gas
  Uniswap 스왑: ~150,000 gas
  컨트랙트 배포: 수십만 ~ 수백만 gas
```

**가스 한도 (gasLimit)가 충분하지 않으면?**
```text
gasLimit = 10,000 gas
실제 필요 = 21,000 gas

→ 10,000 gas 소진 시점에 'out of gas' 에러
→ 트랜잭션 revert (모든 상태 변경 롤백)
→ 이미 소비한 10,000 gas 수수료는 환불 안 됨!
→ 나머지 11,000 gas에 대한 수수료는 환불
```

### 3.3 EIP-1559: 기본 수수료 + 팁

2021년 London 업그레이드에서 도입된 수수료 개혁:

```text
EIP-1559 이전 (경매 방식):
  채굴자가 gasPrice 높은 TX부터 처리
  → 가스비 예측 불가
  → 급한 상황에서 경쟁적 입찰로 수수료 폭등

EIP-1559 이후 (프로토콜 결정):
  baseFee: 프로토콜이 자동 결정 (전 블록 가스 사용량 기반)
    - 블록이 50% 이상 찼으면 baseFee 상승 (최대 12.5%)
    - 블록이 50% 미만이면 baseFee 하락 (최대 12.5%)
  tip: 사용자가 설정하는 검증자 인센티브
  
baseFee는 소각(burn)됨! ETH 공급량 감소 효과
```

**baseFee 조절 메커니즘:**
```text
목표 블록 가스: 15,000,000 gas
최대 블록 가스: 30,000,000 gas

이전 블록 가스 사용: 20,000,000 (목표의 133%)
→ baseFee 상승: 현재 baseFee × (1 + 0.125 × (20M-15M)/15M)
            = 현재 baseFee × 1.0417 (약 4% 상승)

이전 블록 가스 사용: 10,000,000 (목표의 67%)
→ baseFee 하락: 현재 baseFee × (1 - 0.125 × (15M-10M)/15M)
            = 현재 baseFee × 0.9583 (약 4% 하락)
```

### 3.4 주요 가스 비용 예시

```text
스토리지 관련 (가장 비쌈):
  SSTORE (새 값, 0→비영): 20,000 gas
  SSTORE (기존 값 변경):   2,900 gas
  SSTORE (값 삭제 0으로): -15,000 gas (환불!)
  SLOAD  (스토리지 읽기):  2,100 gas (cold) / 100 gas (warm)

메모리 관련:
  MLOAD  (메모리 읽기):  3 gas
  MSTORE (메모리 쓰기):  3 gas
  메모리 확장: 확장할수록 quadratic 증가

기본 연산:
  ADD, SUB:              3 gas
  MUL, DIV:              5 gas
  SHA3 (Keccak256):     30 gas + 6 gas/word
  
트랜잭션 기본:
  기본 비용:            21,000 gas
  calldata 0 바이트:      4 gas/byte
  calldata 비0 바이트:   16 gas/byte

컨트랙트:
  CREATE (배포):        32,000 gas + 코드 크기 비용
  CALL:                 2,600 gas (cold address)
  DELEGATECALL:         2,600 gas
```

---

## 4. Solidity에서 EVM까지: 실행 흐름

### 4.1 컴파일 과정

```text
Solidity 소스코드 (.sol)
        │
        ▼ solc 컴파일러
        │
  ┌─────┴──────────────────────┐
  │  EVM 바이트코드             │  ← 블록체인에 저장되는 것
  │  ABI (Application Binary   │  ← 외부 인터페이스 명세
  │       Interface)           │
  └────────────────────────────┘
        │ (배포 트랜잭션)
        ▼
  이더리움 네트워크
        │
        ▼ (함수 호출 트랜잭션)
  EVM이 바이트코드 실행
        │
        ▼
  상태 변경 / 반환값 / 이벤트
```

### 4.2 간단한 Solidity 코드와 EVM 실행 추적

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SimpleCounter {
    uint256 public count;  // 스토리지 슬롯 0에 저장
    
    function increment() public {
        count += 1;  // SLOAD(슬롯0) → ADD → SSTORE(슬롯0)
    }
    
    function getCount() public view returns (uint256) {
        return count;  // SLOAD(슬롯0) → RETURN
    }
}
```

`increment()` 함수 호출 시 EVM 실행:
```text
1. PUSH1 0x00    스택: [0]         → 스토리지 슬롯 0 주소
2. SLOAD         스택: [현재count]  → 슬롯 0에서 값 읽기 (2,100 gas)
3. PUSH1 0x01    스택: [현재count, 1]
4. ADD           스택: [count+1]    → 더하기 (3 gas)
5. PUSH1 0x00    스택: [count+1, 0]
6. SSTORE        스택: []           → 슬롯 0에 저장 (20,000 gas)
7. STOP                             → 종료
```

**가스 계산:**
```text
기본 TX 비용:           21,000
calldata (함수 시그니처):    64  (4바이트 × 16)
SLOAD (cold):            2,100
ADD:                         3
SSTORE (새 값):          20,000
기타 옵코드:              ~200
────────────────────────
총 약:                  43,367 gas
```

---

## 5. calldata, memory, storage 차이

Node.js 개발자에게 친숙한 방식으로 비교:

```text
calldata:
  역할: 함수 호출 시 전달되는 입력 데이터 (읽기 전용)
  Node.js 비유: req.body (HTTP 요청 본문)
  가스: 저렴 (0 바이트=4gas, 비0 바이트=16gas)
  
  예:
  function transfer(address to, uint256 amount) external {
    // 'to'와 'amount'는 calldata에 있음 (수정 불가)
  }

memory:
  역할: 함수 실행 중 임시 데이터 저장
  Node.js 비유: 함수 내 지역 변수
  가스: 중간 (확장할수록 비쌈)
  수명: 함수 실행 중에만 존재
  
  예:
  function processData(bytes calldata input) external {
    bytes memory temp = new bytes(input.length);  // memory 할당
    // temp는 이 함수가 끝나면 사라짐
  }

storage:
  역할: 컨트랙트의 영구 상태
  Node.js 비유: 데이터베이스 레코드
  가스: 매우 비쌈 (읽기 2,100 / 쓰기 20,000)
  수명: 영구적 (블록체인에 기록)
  
  예:
  contract MyContract {
    uint256 public totalSupply;    // storage 변수
    mapping(address => uint256) public balances;  // storage 맵
  }
```

**실용적인 가스 최적화 패턴:**

```solidity
// 비효율적: storage를 루프에서 반복 접근
function badSum(uint256[] storage arr) internal view returns (uint256) {
    uint256 sum = 0;
    for (uint256 i = 0; i < arr.length; i++) {
        sum += arr[i];  // 매번 SLOAD (2,100 gas × n번)
    }
    return sum;
}

// 효율적: storage를 memory로 캐싱
function goodSum(uint256[] storage arr) internal view returns (uint256) {
    uint256[] memory localArr = arr;  // 한 번만 SLOAD
    uint256 sum = 0;
    for (uint256 i = 0; i < localArr.length; i++) {
        sum += localArr[i];  // MLOAD (3 gas × n번)
    }
    return sum;
}
```

---

## 6. EVM 실행의 결정론적 특성

이더리움의 가장 중요한 속성 중 하나:

```text
결정론적 실행:
  동일한 상태 + 동일한 TX → 항상 동일한 결과

이것이 왜 중요한가:
  - 전 세계 수천 개 노드가 같은 TX를 실행
  - 모두가 같은 결과를 얻어야 함
  - 그래야 블록체인의 상태 합의가 가능

EVM의 격리 이유:
  - 타임스탬프: 블록 헤더의 값만 사용 (시스템 시계 X)
  - 난수: 없음 (or PREVRANDAO 사용, 블록에서 가져옴)
  - 외부 API: 접근 불가
  - 파일 시스템: 접근 불가
```

---

## 7. ethers.js로 EVM 상태 조회

```javascript
const { ethers } = require("ethers");

const provider = new ethers.JsonRpcProvider("http://localhost:8545");

async function inspectEVMState() {
  const contractAddress = "0xYourContract...";
  
  // 1. 스토리지 슬롯 직접 읽기 (저수준)
  const slot0 = await provider.getStorage(contractAddress, 0);
  console.log("스토리지 슬롯 0:", slot0);
  // → 0x0000000000000000000000000000000000000000000000000000000000000042
  //   = 66 (uint256)
  
  // 2. 코드 읽기 (바이트코드)
  const code = await provider.getCode(contractAddress);
  console.log("바이트코드 크기:", (code.length - 2) / 2, "bytes");
  console.log("바이트코드 (앞 20바이트):", code.slice(0, 42));
  
  // 3. 가스 추정
  const gasEstimate = await provider.estimateGas({
    to:   contractAddress,
    data: "0xd09de08a",  // increment() 함수 시그니처
  });
  console.log("예상 가스:", gasEstimate.toString());
  
  // 4. eth_call로 상태 변경 없이 실행 (view 함수)
  const result = await provider.call({
    to:   contractAddress,
    data: "0x06661abd",  // count() getter 시그니처
  });
  const count = BigInt(result);
  console.log("현재 count:", count.toString());
  
  // 5. 트랜잭션 추적 (디버깅용, 일부 노드에서 지원)
  // const trace = await provider.send("debug_traceTransaction", [txHash, {}]);
}

inspectEVMState().catch(console.error);
```

---

## 8. 핵심 정리

```text
EVM 아키텍처:
  ┌────────────────────────────────────────┐
  │  스택: 연산 작업 (빠름, 싸다)          │
  │  메모리: 임시 저장 (중간)              │
  │  스토리지: 영구 저장 (느림, 비싸다)    │
  └────────────────────────────────────────┘

가스 시스템:
  - 모든 연산에 가스 비용 부과 (무한 루프 방지)
  - SSTORE이 가장 비쌈 (20,000 gas)
  - EIP-1559: baseFee(소각) + tip(검증자)
  
최적화 팁:
  - storage 읽기를 최소화 → memory에 캐싱
  - calldata는 storage보다 훨씬 싸다
  - 불필요한 스토리지 변수 삭제 시 가스 환불
```

다음 챕터에서는 이 EVM 위에서 실행되는 **스마트 컨트랙트**의 전체 개요와 ABI, 배포 과정을 살펴본다.
