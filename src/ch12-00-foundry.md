# Chapter 12: Foundry — Solidity 개발 도구

## Foundry란 무엇인가

Foundry는 Rust로 작성된 고성능 Ethereum 스마트 컨트랙트 개발 프레임워크다. 2021년 Paradigm이 개발했으며, 현재 전문 Solidity 개발자들이 가장 많이 선택하는 도구다.

Foundry의 가장 큰 특징은 **테스트를 Solidity로 작성한다**는 점이다. JavaScript/TypeScript로 테스트를 작성하는 Hardhat과 달리, Foundry는 Solidity 컨트랙트로 테스트를 작성한다. 처음에는 낯설게 느껴지지만, 타입 안전성과 실행 속도에서 큰 이점이 있다.

## Hardhat vs Foundry 비교

| 항목 | Hardhat | Foundry |
|------|---------|---------|
| 언어 | JavaScript/TypeScript | Rust |
| 테스트 언어 | JavaScript/TypeScript | Solidity |
| 속도 | 보통 | 매우 빠름 |
| 플러그인 생태계 | 풍부 | 성장 중 |
| 퍼즈 테스트 | 별도 도구 필요 | 기본 내장 |
| 가스 리포트 | 플러그인 필요 | 기본 내장 |
| 학습 곡선 | 낮음 (JS 친숙) | 중간 (Solidity 테스트) |
| 설정 | `hardhat.config.ts` | `foundry.toml` |
| 패키지 관리 | npm | git 서브모듈 |

**Node.js 개발자 관점:**

Hardhat은 NestJS 프로젝트처럼 JavaScript 생태계에 완전히 녹아있다. npm으로 의존성을 관리하고, Jest/Mocha 스타일로 테스트를 작성한다. 이미 TypeScript를 잘 안다면 빠르게 시작할 수 있다.

Foundry는 Go나 Rust 도구체인에 더 가깝다. 빠르고, 자기완결적이며, Solidity 세계에 집중한다. 실무에서는 두 도구를 함께 사용하는 경우도 많다(Hardhat으로 스크립트, Foundry로 테스트).

## Foundry의 4가지 도구

Foundry는 단일 바이너리가 아니라 4개의 전문 도구로 구성된다. Node.js 생태계의 도구들과 비교해보자.

### forge — 컴파일러 + 테스트 러너 + 배포 도구

```bash
forge build          # 컴파일
forge test           # 테스트 실행
forge script         # 배포 스크립트 실행
forge create         # 컨트랙트 직접 배포
forge fmt            # 코드 포맷
forge coverage       # 커버리지 리포트
forge snapshot       # 가스 스냅샷
forge inspect        # 컨트랙트 ABI, 바이트코드 등 조회
```

Node.js 비유: `tsc` (컴파일) + `jest` (테스트) + 배포 스크립트가 하나로 합쳐진 것.

### cast — EVM 상호작용 CLI

배포된 컨트랙트와 블록체인 데이터를 조회하고 트랜잭션을 전송하는 커맨드라인 도구.

```bash
cast call <주소> "balanceOf(address)(uint256)" <지갑주소>
cast send <주소> "transfer(address,uint256)" <to> <amount> --private-key <key>
cast balance <주소>
cast block latest
cast tx <txhash>
cast abi-encode "transfer(address,uint256)" <to> <amount>
cast to-wei 1.5 ether
cast from-wei 1500000000000000000
cast keccak "hello"
```

Node.js 비유: `curl` + `ethers.js` 커맨드라인 버전. Hardhat의 `npx hardhat console`과 유사하지만 더 강력.

### anvil — 로컬 테스트 노드

개발용 로컬 Ethereum 노드. 무한 ETH를 가진 테스트 계정들을 제공한다.

```bash
anvil                           # 기본 실행 (포트 8545)
anvil --port 8546               # 포트 변경
anvil --fork-url <rpc-url>      # 메인넷 포크 (실제 상태 복제)
anvil --chain-id 1337           # 체인 ID 설정
anvil --block-time 12           # 12초마다 블록 생성
```

Node.js 비유: `jest`의 `--testEnvironment` 또는 로컬 개발용 SQLite 같은 개념. Hardhat의 Hardhat Network와 동일한 역할이지만 독립 프로세스로 실행.

```
anvil 실행 시 출력:
Available Accounts
==================
(0) 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (10000 ETH)
(1) 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (10000 ETH)
...

Private Keys
==================
(0) 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
...

Listening on 127.0.0.1:8545
```

### chisel — Solidity REPL

Solidity 코드를 대화형으로 실행하는 셸. 빠른 프로토타이핑과 학습에 유용.

```bash
chisel
# 프롬프트가 나오면 Solidity 코드를 바로 입력
```

```
➜ uint256 x = 100;
➜ uint256 y = 200;
➜ x + y
Type: uint256
Hex: 0x12c
Decimal: 300

➜ address(0x1234).balance
Type: uint256
Decimal: 0

➜ keccak256(abi.encodePacked("hello"))
Type: bytes32
Hex: 0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
```

Node.js 비유: `node` REPL 또는 TypeScript의 `ts-node` 인터랙티브 모드.

## 설치 확인

### 설치 방법

```bash
# foundryup 설치 (Foundry 버전 관리자)
curl -L https://foundry.paradigm.xyz | bash

# 셸 재시작 후
foundryup
```

### 설치 확인

```bash
forge --version
# forge 0.2.0 (xxxxxxx 20xx-xx-xx T00:00:00.000000000Z)

cast --version
# cast 0.2.0 (xxxxxxx 20xx-xx-xx T00:00:00.000000000Z)

anvil --version
# anvil 0.2.0 (xxxxxxx 20xx-xx-xx T00:00:00.000000000Z)

chisel --version
# chisel 0.2.0 (xxxxxxx 20xx-xx-xx T00:00:00.000000000Z)
```

### 업데이트

```bash
foundryup          # 최신 안정 버전으로 업데이트
foundryup --pr 1234  # 특정 PR 버전 설치
foundryup --branch master  # 개발 브랜치 버전
```

## Foundry가 빠른 이유

Rust로 작성된 Foundry는 Node.js 기반 Hardhat보다 테스트 실행 속도가 10~100배 빠른 경우가 많다. 이는 다음 이유들 때문이다:

1. **EVM을 Rust로 구현** — revm(Rust EVM)을 사용해 네이티브 속도
2. **병렬 테스트 실행** — 기본적으로 멀티코어 활용
3. **JIT 컴파일 없음** — Node.js의 초기화 비용 없음
4. **Solidity 테스트** — ABI 인코딩/디코딩 오버헤드 없음

실제로 100개의 테스트를 Hardhat은 10초가 걸릴 때, Foundry는 0.5초 이내에 처리하는 경우가 흔하다.

## Foundry 도구 체계 정리

```
Foundry 생태계
├── forge        - 빌드/테스트/배포 (메인 도구)
├── cast         - 블록체인 상호작용 CLI
├── anvil        - 로컬 테스트 노드
└── chisel       - Solidity REPL

Node.js 생태계 대응
├── forge build  ↔  tsc
├── forge test   ↔  jest / mocha
├── forge script ↔  ts-node deploy.ts
├── cast         ↔  curl + ethers.js CLI
├── anvil        ↔  hardhat node / ganache
└── chisel       ↔  node / ts-node (REPL)
```

다음 챕터에서는 Foundry 프로젝트를 직접 생성하고 구조를 파악한다.
