# Chapter 12-3: Foundry 배포

## 배포 스크립트 (Script 컨트랙트)

Foundry의 배포 스크립트는 Solidity로 작성한다. Hardhat의 `deploy.ts`처럼 JavaScript로 작성하지 않고, `Script` 컨트랙트를 상속해서 Solidity로 배포 로직을 작성한다.

```solidity
// script/Deploy.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {SimpleToken} from "../src/SimpleToken.sol";

contract DeploySimpleToken is Script {
    // setUp은 선택사항
    function setUp() public {}

    // run()이 실제 실행되는 진입점
    function run() external returns (SimpleToken token) {
        // 환경 변수에서 배포자 개인키 읽기
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        console.log("Deploying with address:", deployer);
        console.log("Deployer balance:", deployer.balance);

        // startBroadcast ~ stopBroadcast 사이의 트랜잭션만 실제 전송
        vm.startBroadcast(deployerPrivateKey);

        token = new SimpleToken(
            "SimpleToken",
            "STK",
            1_000_000 * 1e18
        );

        vm.stopBroadcast();

        console.log("SimpleToken deployed at:", address(token));
        console.log("Total supply:", token.totalSupply());

        return token;
    }
}
```

### vm.startBroadcast / vm.stopBroadcast

이 두 함수 사이에 있는 트랜잭션만 실제 블록체인에 전송된다. 그 외 코드는 로컬에서 시뮬레이션만 된다.

```solidity
function run() external {
    // 이 부분은 로컬 시뮬레이션만 (트랜잭션 없음)
    address deployer = vm.addr(privateKey);
    console.log("Simulating deployment...");

    vm.startBroadcast(privateKey);

    // 이 부분만 실제 트랜잭션으로 전송
    Token token = new Token();
    token.mint(deployer, 1000 * 1e18);

    vm.stopBroadcast();

    // 다시 로컬 시뮬레이션
    console.log("Deployed at:", address(token));
}
```

### 복잡한 배포 스크립트

```solidity
// script/DeploySystem.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {MyToken} from "../src/MyToken.sol";
import {Vault} from "../src/Vault.sol";

contract DeploySystem is Script {
    struct DeployedContracts {
        address token;
        address vault;
    }

    function run() external returns (DeployedContracts memory deployed) {
        uint256 deployerKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerKey);
        address treasury = vm.envAddress("TREASURY_ADDRESS");

        console.log("=== Deployment Start ===");
        console.log("Deployer:", deployer);
        console.log("Treasury:", treasury);

        vm.startBroadcast(deployerKey);

        // 1. 토큰 배포
        MyToken token = new MyToken("MyToken", "MTK");
        console.log("Token:", address(token));

        // 2. Vault 배포 (토큰 주소 필요)
        Vault vault = new Vault(address(token), treasury);
        console.log("Vault:", address(vault));

        // 3. 초기 설정
        token.grantRole(token.MINTER_ROLE(), address(vault));
        vault.setDepositLimit(10_000 * 1e18);

        vm.stopBroadcast();

        deployed = DeployedContracts({
            token: address(token),
            vault: address(vault)
        });

        console.log("=== Deployment Complete ===");
    }
}
```

## .env 파일 설정

```bash
# .env (절대 git에 커밋하지 말 것!)
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
TREASURY_ADDRESS=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
ETHERSCAN_API_KEY=YOUR_ETHERSCAN_KEY
SEPOLIA_RPC_URL=https://eth-sepolia.alchemyapi.io/v2/YOUR_KEY
MAINNET_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/YOUR_KEY
```

```bash
# .env.example (커밋 OK, 값은 빈칸)
PRIVATE_KEY=
TREASURY_ADDRESS=
ETHERSCAN_API_KEY=
SEPOLIA_RPC_URL=
MAINNET_RPC_URL=
```

## Anvil로 로컬 노드 실행

배포 전에 로컬 환경에서 먼저 테스트한다.

```bash
# 기본 로컬 노드 실행
anvil

# 출력:
#                              _   _
#                             (_) | |
#       __ _   _ __   __   __ _  | |
#      / _` | | '_ \  \ \ / /| | | |
#     | (_| | | | | |  \ V / | | | |
#      \__,_| |_| |_|   \_/  |_| |_|
#
# Available Accounts
# ==================
# (0) 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (10000 ETH)
# (1) 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (10000 ETH)
# ...
#
# Private Keys
# ==================
# (0) 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
# ...
#
# Listening on 127.0.0.1:8545
```

### Anvil 옵션

```bash
# 포트 변경
anvil --port 8546

# 계정 수와 잔액 설정
anvil --accounts 20 --balance 1000

# 블록 시간 설정 (기본: 즉시 채굴)
anvil --block-time 12

# 체인 ID 설정
anvil --chain-id 31337

# 메인넷 포크 (실제 상태 복제)
anvil --fork-url $MAINNET_RPC_URL --fork-block-number 18000000

# 상태 저장 (재시작 시 복원)
anvil --state ./anvil-state.json
```

**메인넷 포크의 강점:** 실제 메인넷의 토큰, DEX, 프로토콜 상태를 그대로 복제해서 로컬에서 테스트할 수 있다. 예를 들어 Uniswap V3, Aave, Compound 등과의 상호작용을 실제 배포 없이 테스트 가능하다.

## forge script로 배포

### 로컬(Anvil)에 배포

```bash
# 터미널 1: Anvil 실행
anvil

# 터미널 2: 배포
forge script script/Deploy.s.sol:DeploySimpleToken \
    --rpc-url http://127.0.0.1:8545 \
    --broadcast \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    -vvvv
```

### 테스트넷(Sepolia)에 배포

```bash
# .env 로드
source .env

# 시뮬레이션만 (--broadcast 없음)
forge script script/Deploy.s.sol:DeploySimpleToken \
    --rpc-url $SEPOLIA_RPC_URL \
    -vvv

# 실제 배포 (--broadcast 추가)
forge script script/Deploy.s.sol:DeploySimpleToken \
    --rpc-url $SEPOLIA_RPC_URL \
    --broadcast \
    --verify \                    # Etherscan 검증
    --etherscan-api-key $ETHERSCAN_API_KEY \
    -vvvv
```

### 배포 결과 파일

배포 후 `broadcast/` 디렉토리에 트랜잭션 정보가 저장된다:

```
broadcast/
└── Deploy.s.sol/
    ├── 31337/                    # chain ID
    │   └── run-latest.json       # 가장 최근 실행 결과
    └── 11155111/                 # Sepolia chain ID
        └── run-latest.json
```

```json
// broadcast/Deploy.s.sol/31337/run-latest.json
{
    "transactions": [
        {
            "hash": "0x...",
            "contractName": "SimpleToken",
            "contractAddress": "0x5FbDB2315678afecb367f032d93F642f64180aa3",
            "transactionType": "CREATE",
            "arguments": ["SimpleToken", "STK", "1000000000000000000000000"]
        }
    ],
    "receipts": [...],
    "timestamp": 1700000000
}
```

## forge create로 직접 배포

스크립트 없이 CLI로 바로 배포할 수 있다. 간단한 컨트랙트에 유용하다.

```bash
# 기본 배포
forge create src/SimpleToken.sol:SimpleToken \
    --constructor-args "SimpleToken" "STK" 1000000000000000000000000 \
    --rpc-url http://127.0.0.1:8545 \
    --private-key $PRIVATE_KEY

# 출력:
# Deployer: 0xf39Fd6...
# Deployed to: 0x5FbDB2...
# Transaction hash: 0xabc123...

# Etherscan 검증 포함
forge create src/SimpleToken.sol:SimpleToken \
    --constructor-args "SimpleToken" "STK" 1000000000000000000000000 \
    --rpc-url $SEPOLIA_RPC_URL \
    --private-key $PRIVATE_KEY \
    --verify \
    --etherscan-api-key $ETHERSCAN_API_KEY
```

## cast로 컨트랙트 상호작용

배포 후 `cast`로 컨트랙트와 상호작용할 수 있다.

### 읽기 (call — 트랜잭션 없음)

```bash
# 함수 호출: "함수명(파라미터타입)(반환타입)"
cast call $TOKEN_ADDRESS "name()(string)" \
    --rpc-url http://127.0.0.1:8545

# 출력: SimpleToken

cast call $TOKEN_ADDRESS "balanceOf(address)(uint256)" \
    0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
    --rpc-url http://127.0.0.1:8545

# 출력: 1000000000000000000000000

cast call $TOKEN_ADDRESS "totalSupply()(uint256)" \
    --rpc-url http://127.0.0.1:8545
```

### 쓰기 (send — 트랜잭션 발생)

```bash
# 전송 트랜잭션
cast send $TOKEN_ADDRESS \
    "transfer(address,uint256)" \
    0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
    1000000000000000000 \
    --rpc-url http://127.0.0.1:8545 \
    --private-key $PRIVATE_KEY

# ETH와 함께 전송 (payable 함수)
cast send $VAULT_ADDRESS \
    "deposit()" \
    --value 1ether \
    --rpc-url http://127.0.0.1:8545 \
    --private-key $PRIVATE_KEY
```

### 유틸리티 명령어

```bash
# ETH 잔액 조회
cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
    --rpc-url http://127.0.0.1:8545

# 단위 변환
cast to-wei 1.5 ether       # 1500000000000000000
cast from-wei 1500000000000000000  # 1.5 (ETH)
cast to-wei 100 gwei        # 100000000000

# 해시 계산
cast keccak "transfer(address,uint256)"
# 0xa9059cbb2ab09eb219583f4a59a5d0623ade346d962bcd4e46b11da047c9049b

# 함수 선택자 (처음 4바이트)
cast sig "transfer(address,uint256)"
# 0xa9059cbb

# ABI 인코딩
cast abi-encode "transfer(address,uint256)" \
    0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
    1000000000000000000

# 블록 정보
cast block latest --rpc-url http://127.0.0.1:8545
cast block 100 --rpc-url http://127.0.0.1:8545

# 트랜잭션 정보
cast tx 0xabc123... --rpc-url http://127.0.0.1:8545

# 트랜잭션 영수증
cast receipt 0xabc123... --rpc-url http://127.0.0.1:8545
```

## 전체 배포 워크플로

### 1단계: 로컬 테스트

```bash
# 테스트 통과 확인
forge test -vvv

# 가스 리포트
forge test --gas-report
```

### 2단계: 로컬 배포 테스트

```bash
# 터미널 1
anvil

# 터미널 2
forge script script/Deploy.s.sol \
    --rpc-url http://127.0.0.1:8545 \
    --broadcast \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# 배포된 주소 확인 후 상호작용 테스트
export TOKEN=0x5FbDB2315678afecb367f032d93F642f64180aa3
cast call $TOKEN "totalSupply()(uint256)" --rpc-url http://127.0.0.1:8545
```

### 3단계: 테스트넷 배포

```bash
source .env

# 먼저 시뮬레이션
forge script script/Deploy.s.sol \
    --rpc-url $SEPOLIA_RPC_URL \
    -vvv

# 실제 배포
forge script script/Deploy.s.sol \
    --rpc-url $SEPOLIA_RPC_URL \
    --broadcast \
    --verify \
    --etherscan-api-key $ETHERSCAN_API_KEY \
    -vvvv
```

### 4단계: 검증 확인

```bash
# Etherscan에서 컨트랙트 검증 상태 확인
# https://sepolia.etherscan.io/address/<CONTRACT_ADDRESS>

# 또는 cast로 확인
cast etherscan-source $CONTRACT_ADDRESS \
    --chain sepolia \
    --etherscan-api-key $ETHERSCAN_API_KEY
```

### Makefile로 워크플로 자동화

```makefile
# Makefile
.PHONY: build test deploy-local deploy-sepolia

include .env

build:
	forge build

test:
	forge test -vvv

test-gas:
	forge test --gas-report

deploy-local:
	forge script script/Deploy.s.sol \
		--rpc-url http://127.0.0.1:8545 \
		--broadcast \
		--private-key $(PRIVATE_KEY)

deploy-sepolia:
	forge script script/Deploy.s.sol \
		--rpc-url $(SEPOLIA_RPC_URL) \
		--broadcast \
		--verify \
		--etherscan-api-key $(ETHERSCAN_API_KEY) \
		-vvvv

verify:
	forge verify-contract \
		$(CONTRACT_ADDRESS) \
		src/SimpleToken.sol:SimpleToken \
		--chain sepolia \
		--etherscan-api-key $(ETHERSCAN_API_KEY)

anvil:
	anvil --chain-id 31337
```

```bash
# 사용법
make build
make test
make deploy-local
make deploy-sepolia
```

## 정리

Foundry 배포 워크플로:

1. `forge test` — 테스트 통과 확인
2. `anvil` 실행 — 로컬 노드 시작
3. `forge script --broadcast` (로컬) — 로컬 배포 및 검증
4. `forge script --broadcast` (테스트넷) — 테스트넷 배포
5. `cast call/send` — 배포된 컨트랙트 상호작용
6. Etherscan 검증 — 소스코드 공개

다음 챕터에서는 ERC-20, ERC-721 토큰 표준을 살펴본다.
