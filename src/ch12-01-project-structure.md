# Chapter 12-1: Foundry 프로젝트 구조

## forge init으로 프로젝트 생성

```bash
# 새 디렉토리에 프로젝트 생성
forge init my-project
cd my-project

# 현재 디렉토리에 초기화
forge init .

# 기존 git 저장소에 초기화 (--force로 덮어쓰기)
forge init --force .
```

생성 직후 출력:

```text
Initializing /path/to/my-project...
Installing forge-std in /path/to/my-project/lib/forge-std (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Initialized forge project
```

## 디렉토리 구조

```text
my-project/
├── foundry.toml          # 프로젝트 설정 파일
├── .gitignore
├── .gitmodules           # git 서브모듈 목록
│
├── src/                  # 컨트랙트 소스 (= NestJS의 src/)
│   └── Counter.sol
│
├── test/                 # 테스트 파일 (= NestJS의 test/ 또는 *.spec.ts)
│   └── Counter.t.sol     # .t.sol 확장자가 관례
│
├── script/               # 배포/운영 스크립트 (= NestJS의 scripts/)
│   └── Counter.s.sol     # .s.sol 확장자가 관례
│
└── lib/                  # 외부 라이브러리 (= NestJS의 node_modules/)
    └── forge-std/        # Foundry 표준 라이브러리 (테스트 헬퍼)
        ├── src/
        │   ├── Test.sol
        │   ├── Vm.sol
        │   └── ...
        └── ...
```

**NestJS 프로젝트와 비교:**

| Foundry | NestJS | 역할 |
|---------|--------|------|
| `src/` | `src/` | 메인 소스 코드 |
| `test/` | `test/` 또는 `*.spec.ts` | 테스트 |
| `script/` | `scripts/` | 유틸리티 스크립트 |
| `lib/` | `node_modules/` | 외부 의존성 |
| `foundry.toml` | `package.json` + `tsconfig.json` | 프로젝트 설정 |
| `forge-std` | `@nestjs/testing` | 테스트 프레임워크 |
| `.gitmodules` | `package-lock.json` | 의존성 잠금 |

## foundry.toml 설정 파일

`foundry.toml`은 Foundry 프로젝트의 핵심 설정 파일이다. `package.json`과 `tsconfig.json`을 합친 것으로 생각하면 된다.

### 기본 구조

```toml
# foundry.toml

[profile.default]
src = "src"                    # 컨트랙트 소스 디렉토리
test = "test"                  # 테스트 디렉토리
script = "script"              # 스크립트 디렉토리
out = "out"                    # 컴파일 출력 디렉토리
libs = ["lib"]                 # 라이브러리 경로
```

### 컴파일러 설정

```toml
[profile.default]
solc_version = "0.8.20"        # 특정 solc 버전 고정
optimizer = true               # 옵티마이저 활성화
optimizer_runs = 200           # 옵티마이저 실행 횟수
                               # 낮을수록 배포 비용↓, 높을수록 호출 비용↓
via_ir = false                 # IR 기반 컴파일 (더 강력한 최적화)
evm_version = "paris"          # 대상 EVM 버전
```

**`optimizer_runs` 가이드:**
- `200` (기본): 배포와 호출 비용의 균형
- `1`: 배포 비용 최소화 (한 번만 배포, 자주 호출 안 하는 컨트랙트)
- `1000000`: 호출 비용 최소화 (자주 호출되는 컨트랙트)

### 테스트 설정

```toml
[profile.default]
fuzz_runs = 256                # 퍼즈 테스트 반복 횟수 (기본 256)
verbosity = 0                  # 기본 출력 레벨 (CLI에서 -vvv로 오버라이드 가능)
match_test = ""                # 특정 테스트만 실행 (정규식)
no_match_test = ""             # 특정 테스트 제외

[fuzz]
runs = 1000                    # 퍼즈 테스트 실행 횟수 더 높게 설정
max_test_rejects = 65536       # 퍼즈 입력 거부 최대 횟수
seed = "0x1234"                # 재현 가능한 테스트를 위한 시드

[invariant]
runs = 256                     # 불변식 테스트 실행 횟수
depth = 15                     # 각 실행당 함수 호출 깊이
```

### 포맷터 설정

```toml
[fmt]
line_length = 120              # 한 줄 최대 길이
tab_width = 4                  # 들여쓰기 공백 수
bracket_spacing = false        # 괄호 내부 공백 여부
int_types = "long"             # "long" = uint256, "short" = uint
multiline_func_header = "all"  # 함수 헤더 멀티라인 기준
sort_imports = true            # import 정렬
```

### RPC 설정

```toml
[rpc_endpoints]
mainnet = "https://eth-mainnet.alchemyapi.io/v2/${ALCHEMY_API_KEY}"
sepolia = "https://eth-sepolia.alchemyapi.io/v2/${ALCHEMY_API_KEY}"
polygon = "https://polygon-mainnet.alchemyapi.io/v2/${ALCHEMY_API_KEY}"
localhost = "http://127.0.0.1:8545"

[etherscan]
mainnet = { key = "${ETHERSCAN_API_KEY}" }
sepolia = { key = "${ETHERSCAN_API_KEY}", url = "https://api-sepolia.etherscan.io/api" }
```

### 프로필별 설정

NestJS의 NODE_ENV처럼, Foundry도 환경별로 다른 설정을 사용할 수 있다:

```toml
# 기본 프로필
[profile.default]
solc_version = "0.8.20"
optimizer = true
optimizer_runs = 200

# CI 환경 (더 엄격한 테스트)
[profile.ci]
fuzz_runs = 10000
verbosity = 4

# 프로덕션 배포 (최적화 극대화)
[profile.production]
optimizer_runs = 1000000
via_ir = true
```

```bash
# 프로필 선택
FOUNDRY_PROFILE=ci forge test
FOUNDRY_PROFILE=production forge build
```

### 완성된 foundry.toml 예시

```toml
[profile.default]
src = "src"
test = "test"
script = "script"
out = "out"
libs = ["lib"]

solc_version = "0.8.20"
optimizer = true
optimizer_runs = 200
evm_version = "paris"

[profile.default.fuzz]
runs = 256

[profile.ci]
fuzz_runs = 10000

[fmt]
line_length = 120
tab_width = 4
sort_imports = true

[rpc_endpoints]
mainnet = "${MAINNET_RPC_URL}"
sepolia = "${SEPOLIA_RPC_URL}"
localhost = "http://127.0.0.1:8545"

[etherscan]
mainnet = { key = "${ETHERSCAN_API_KEY}" }
sepolia = { key = "${ETHERSCAN_API_KEY}", url = "https://api-sepolia.etherscan.io/api" }
```

## remappings — 의존성 경로 매핑

remappings는 import 경로의 별칭을 정의한다. npm의 `paths` (tsconfig.json)와 동일한 개념이다.

### remappings.txt

```text
# remappings.txt (루트 디렉토리)
@openzeppelin/=lib/openzeppelin-contracts/
@uniswap/v3-core/=lib/v3-core/
forge-std/=lib/forge-std/src/
```

### foundry.toml에 직접 설정

```toml
[profile.default]
remappings = [
    "@openzeppelin/=lib/openzeppelin-contracts/",
    "forge-std/=lib/forge-std/src/",
    "solmate/=lib/solmate/src/",
]
```

**remappings 적용 전후:**

```solidity
// remappings 없을 때 (번거로운 상대 경로)
import "../../lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

// remappings 적용 후 (깔끔한 절대 경로)
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
```

```typescript
// TypeScript tsconfig.json paths와 비교
{
  "compilerOptions": {
    "paths": {
      "@/*": ["./src/*"],
      "@common/*": ["./src/common/*"]
    }
  }
}
// import { UserService } from '@common/user.service';
```

### remappings 자동 감지

```bash
# 설치된 라이브러리의 remappings 자동 감지
forge remappings

# 출력 예시:
# ds-test/=lib/forge-std/lib/ds-test/src/
# forge-std/=lib/forge-std/src/
# @openzeppelin/=lib/openzeppelin-contracts/
```

## forge install로 라이브러리 설치

Foundry는 npm 대신 **git 서브모듈**로 의존성을 관리한다.

### 기본 설치

```bash
# GitHub 저장소 설치
forge install OpenZeppelin/openzeppelin-contracts

# 특정 버전(태그) 설치
forge install OpenZeppelin/openzeppelin-contracts@v5.0.0

# 여러 패키지 동시 설치
forge install OpenZeppelin/openzeppelin-contracts Uniswap/v3-core

# 커밋 해시로 고정
forge install OpenZeppelin/openzeppelin-contracts@a1948c5
```

### 자주 사용하는 라이브러리

```bash
# OpenZeppelin - 표준 컨트랙트 라이브러리
forge install OpenZeppelin/openzeppelin-contracts

# OpenZeppelin Upgradeable - 업그레이드 가능한 컨트랙트
forge install OpenZeppelin/openzeppelin-contracts-upgradeable

# solmate - 가스 최적화된 컨트랙트 모음
forge install transmissions11/solmate

# Uniswap V3 Core
forge install Uniswap/v3-core

# Chainlink - 오라클
forge install smartcontractkit/chainlink
```

### 설치 후 remappings 추가

```bash
# OpenZeppelin 설치 후
forge install OpenZeppelin/openzeppelin-contracts

# remappings.txt에 추가
echo "@openzeppelin/=lib/openzeppelin-contracts/" >> remappings.txt
```

설치 후 디렉토리 구조:

```text
lib/
├── forge-std/                     # 기본 설치됨
│   └── src/
│       ├── Test.sol
│       └── ...
└── openzeppelin-contracts/        # 새로 설치
    └── contracts/
        ├── token/
        │   ├── ERC20/
        │   │   ├── ERC20.sol
        │   │   └── extensions/
        │   └── ERC721/
        ├── access/
        │   ├── Ownable.sol
        │   └── AccessControl.sol
        └── ...
```

### 의존성 업데이트와 제거

```bash
# 특정 라이브러리 업데이트
forge update lib/openzeppelin-contracts

# 모든 라이브러리 업데이트
forge update

# 라이브러리 제거
forge remove openzeppelin-contracts
# 또는
forge remove lib/openzeppelin-contracts
```

### .gitmodules 파일

git 서브모듈로 관리되므로 `.gitmodules`에 의존성이 기록된다:

```text
[submodule "lib/forge-std"]
    path = lib/forge-std
    url = https://github.com/foundry-rs/forge-std
    branch = v1

[submodule "lib/openzeppelin-contracts"]
    path = lib/openzeppelin-contracts
    url = https://github.com/OpenZeppelin/openzeppelin-contracts
    branch = v5.0.0
```

새 팀원이 프로젝트를 클론할 때:

```bash
git clone --recursive <repo-url>
# 또는
git clone <repo-url>
git submodule update --init --recursive
```

npm의 `npm install`에 해당하는 것이 `git submodule update --init --recursive`다.

## 초기 파일 내용

`forge init` 후 생성되는 기본 파일들:

### src/Counter.sol

```solidity
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Counter {
    uint256 public number;

    function setNumber(uint256 newNumber) public {
        number = newNumber;
    }

    function increment() public {
        number++;
    }
}
```

### test/Counter.t.sol

```solidity
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Counter} from "../src/Counter.sol";

contract CounterTest is Test {
    Counter public counter;

    function setUp() public {
        counter = new Counter();
        counter.setNumber(0);
    }

    function test_Increment() public {
        counter.increment();
        assertEq(counter.number(), 1);
    }

    function testFuzz_SetNumber(uint256 x) public {
        counter.setNumber(x);
        assertEq(counter.number(), x);
    }
}
```

### script/Counter.s.sol

```solidity
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Counter} from "../src/Counter.sol";

contract CounterScript is Script {
    Counter public counter;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        counter = new Counter();

        vm.stopBroadcast();
    }
}
```

## 첫 빌드와 테스트

```bash
# 컴파일
forge build

# 출력:
# [⠒] Compiling...
# [⠢] Compiling 24 files with 0.8.20
# [⠆] Solc 0.8.20 finished in 2.34s
# Compiler run successful!

# 테스트 실행
forge test

# 출력:
# [⠒] Compiling...
# No files changed, compilation skipped
#
# Running 2 tests for test/Counter.t.sol:CounterTest
# [PASS] testFuzz_SetNumber(uint256) (runs: 256, μ: 27553, ~: 27553)
# [PASS] test_Increment() (μ: 28334, ~: 28334)
# Test result: ok. 2 passed; 0 failed; 0 skipped; finished in 8.25ms
```

## 프로젝트 구조 최종 정리

실무 프로젝트의 완성된 구조:

```text
my-defi-project/
├── foundry.toml
├── remappings.txt
├── .env                        # 환경 변수 (절대 커밋하지 말 것!)
├── .env.example                # 환경 변수 예시 (커밋 OK)
├── .gitignore
├── .gitmodules
│
├── src/
│   ├── interfaces/             # 인터페이스 정의
│   │   ├── IToken.sol
│   │   └── IVault.sol
│   ├── libraries/              # 내부 라이브러리
│   │   └── Math.sol
│   ├── Token.sol               # 메인 컨트랙트들
│   └── Vault.sol
│
├── test/
│   ├── unit/                   # 단위 테스트
│   │   ├── Token.t.sol
│   │   └── Vault.t.sol
│   ├── integration/            # 통합 테스트
│   │   └── VaultIntegration.t.sol
│   └── mocks/                  # 목 컨트랙트
│       └── MockERC20.sol
│
├── script/
│   ├── Deploy.s.sol            # 배포 스크립트
│   └── Interactions.s.sol     # 상호작용 스크립트
│
├── out/                        # 컴파일 산출물 (gitignore)
│   ├── Token.sol/
│   │   └── Token.json          # ABI + 바이트코드
│   └── ...
│
└── lib/
    ├── forge-std/
    └── openzeppelin-contracts/
```

다음 챕터에서는 Foundry로 테스트를 작성하는 방법을 자세히 다룬다.
