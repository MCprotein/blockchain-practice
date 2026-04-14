# Chapter 14-2: 프록시 패턴 (Proxy Patterns)

## 왜 업그레이드가 필요한가

스마트 컨트랙트는 한 번 배포하면 코드를 변경할 수 없다. 이것이 블록체인의 **불변성(immutability)** 원칙이다. 그런데 실무에서는:

- 배포 후 버그가 발견된다
- 비즈니스 요구사항이 변경된다
- 새로운 기능이 필요하다
- 보안 취약점이 발견된다

Node.js 서비스라면 코드를 고치고 재배포하면 끝이다. 스마트 컨트랙트는?

```text
문제: 컨트랙트 A에 버그 발견
                    ↓
새 컨트랙트 B를 배포해도...
- 기존 사용자는 A를 가리키고 있음
- A에 있는 데이터(잔액, 상태)는 B로 이동 불가
- 에코시스템(거래소, 프론트엔드)이 A 주소를 사용 중
```

## delegatecall 동작 원리

프록시 패턴의 핵심은 `delegatecall`이다. 일반 `call`과의 차이를 이해해야 한다.

### call vs delegatecall

```solidity
contract Logic {
    uint256 public value;

    function setValue(uint256 _value) public {
        value = _value;  // Logic의 storage에 저장
    }
}

contract Caller {
    uint256 public value;
    Logic logic = Logic(0x0000000000000000000000000000000000001000);

    function useCall() public {
        // call: Logic 컨트랙트의 context에서 실행
        // Logic.value가 변경됨, Caller.value는 그대로
        logic.setValue(42);
    }

    function useDelegatecall() public {
        // delegatecall: Caller 컨트랙트의 context에서 실행
        // Caller.value가 변경됨, Logic.value는 그대로
        (bool success, ) = address(logic).delegatecall(
            abi.encodeWithSignature("setValue(uint256)", 42)
        );
        require(success);
    }
}
```

**delegatecall의 핵심:** 코드는 Logic에서 빌려오지만, **실행 환경(storage, msg.sender, msg.value)은 호출자(Caller)의 것**을 사용한다.

```text
일반 call:
  Caller → [call] → Logic
                    ├─ msg.sender = Caller
                    ├─ storage = Logic's storage
                    └─ code = Logic's code

delegatecall:
  Caller → [delegatecall] → Logic
           ├─ msg.sender = 원래 호출자 (EOA)
           ├─ storage = Caller's storage  ← 핵심!
           └─ code = Logic's code         ← 빌린 코드
```

**Node.js 비유:** JavaScript의 `Function.prototype.call(thisArg)`와 유사하다.

```javascript
const logic = {
    setValue(value) {
        this.value = value;  // 'this'는 호출 시 결정
    }
};

const proxy = { value: 0 };

// call: proxy의 context에서 logic의 함수 실행
logic.setValue.call(proxy, 42);
// proxy.value = 42, logic.value는 그대로
```

## 프록시 패턴 아키텍처

```text
사용자 → Proxy 컨트랙트 → Logic(Implementation) 컨트랙트
          (데이터 저장)      (코드만 있음, 데이터 없음)
          (주소 불변)        (업그레이드 시 교체)
```

업그레이드 시:
```text
사용자 → Proxy 컨트랙트 → Logic V2 컨트랙트 (새 주소)
          (주소 그대로)    (새 코드)
          (데이터 그대로)
```

## Transparent Proxy vs UUPS Proxy

### Transparent Proxy (투명 프록시)

OpenZeppelin이 초기에 제안한 패턴. 관리자(admin)와 일반 사용자가 다른 함수를 호출한다.

```solidity
contract TransparentProxy {
    address public implementation;
    address public admin;

    modifier ifAdmin() {
        if (msg.sender == admin) {
            _;  // 관리자: 프록시 자체 함수 (upgrade 등)
        } else {
            _delegate(implementation);  // 사용자: 구현체로 위임
        }
    }

    function upgrade(address newImplementation) external ifAdmin {
        implementation = newImplementation;
    }

    fallback() external payable {
        _delegate(implementation);
    }

    function _delegate(address impl) internal {
        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }
}
```

**Transparent Proxy의 문제:**
- 모든 호출에서 admin 체크 → 가스 낭비
- ProxyAdmin 컨트랙트가 별도로 필요해 복잡성 증가

### UUPS Proxy (Universal Upgradeable Proxy Standard, EIP-1822)

업그레이드 로직을 **구현체(implementation)에** 두는 방식. 프록시는 단순히 위임만 한다.

```solidity
// 프록시 컨트랙트 (매우 단순)
contract ERC1967Proxy {
    // EIP-1967 표준 슬롯: keccak256("eip1967.proxy.implementation") - 1
    bytes32 private constant IMPLEMENTATION_SLOT =
        0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;

    constructor(address implementation, bytes memory data) {
        _setImplementation(implementation);
        if (data.length > 0) {
            (bool success,) = implementation.delegatecall(data);
            require(success, "Initialization failed");
        }
    }

    fallback() external payable {
        _delegate(_getImplementation());
    }

    receive() external payable {
        _delegate(_getImplementation());
    }

    function _getImplementation() internal view returns (address impl) {
        assembly {
            impl := sload(IMPLEMENTATION_SLOT)
        }
    }

    function _setImplementation(address newImpl) internal {
        assembly {
            sstore(IMPLEMENTATION_SLOT, newImpl)
        }
    }

    function _delegate(address impl) internal {
        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }
}
```

```solidity
// 구현체 컨트랙트 (업그레이드 로직 포함)
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

contract MyContractV1 is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    uint256 public value;

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner) public initializer {
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
        value = 0;
    }

    function setValue(uint256 _value) external {
        value = _value;
    }

    // 업그레이드 권한 제어 — 소유자만 업그레이드 가능
    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}
}
```

**UUPS vs Transparent 비교:**

| | Transparent Proxy | UUPS |
|--|-----------------|------|
| 업그레이드 로직 위치 | Proxy | Implementation |
| 가스 효율 | 낮음 (admin 체크) | 높음 |
| 업그레이드 실수 위험 | 낮음 | 높음 (구현체에 _authorizeUpgrade 없으면 영구 잠금) |
| 복잡성 | ProxyAdmin 필요 | 단순 |
| 현재 권장 | 레거시 | 권장 |

## 스토리지 충돌 주의사항

delegatecall을 사용할 때 가장 위험한 문제가 **스토리지 슬롯 충돌**이다.

```solidity
// 프록시
contract Proxy {
    address public implementation;  // slot 0
    address public admin;           // slot 1
}

// 구현체
contract Logic {
    uint256 public value;           // slot 0 ← 충돌!
    address public owner;           // slot 1 ← 충돌!
}
```

delegatecall로 `Logic.setValue(42)`를 호출하면 Logic의 slot 0에 쓰는 게 아니라 **Proxy의 slot 0**, 즉 `implementation` 주소가 42로 덮어써진다!

### EIP-1967 표준 슬롯으로 해결

```solidity
// EIP-1967: 충돌 가능성이 없는 특수 슬롯 사용
// implementation 주소를 일반 slot 0이 아닌 특수 슬롯에 저장

bytes32 constant IMPLEMENTATION_SLOT =
    keccak256("eip1967.proxy.implementation") - 1;
// = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc

bytes32 constant ADMIN_SLOT =
    keccak256("eip1967.proxy.admin") - 1;
// = 0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103
```

이렇게 하면 구현체의 상태 변수(slot 0, 1, 2...)와 절대 충돌하지 않는다.

### 업그레이드 시 스토리지 레이아웃 유지

```solidity
// V1 구현체
contract MyContractV1 is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    uint256 public value;      // slot 0 (OZ 내부 슬롯 제외)
    address public treasury;   // slot 1

    function setValue(uint256 newValue) external onlyOwner {
        value = newValue;
    }
}

// V2 구현체 (올바른 업그레이드)
contract MyContractV2 is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    uint256 public value;      // slot 0 — 유지!
    address public treasury;   // slot 1 — 유지!
    uint256 public newFeature; // slot 2 — 새로 추가 (뒤에만 가능)

    function setNewFeature(uint256 newValue) external onlyOwner {
        newFeature = newValue;
    }
}

// V2 구현체 (위험한 업그레이드 — 하지 말 것)
contract MyContractV2_BAD is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    address public treasury;   // slot 0 — 변경! value가 treasury로 해석됨
    uint256 public value;      // slot 1 — 변경! treasury가 value로 해석됨
}
```

## OpenZeppelin UUPSUpgradeable 사용 예제

### 전체 구현 예시

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";

/// @title UpgradeableVault V1
contract UpgradeableVaultV1 is
    Initializable,
    OwnableUpgradeable,
    ReentrancyGuardUpgradeable,
    UUPSUpgradeable
{
    // ============ 상태 변수 (순서 절대 변경 금지) ============
    ERC20Upgradeable public token;
    mapping(address => uint256) public deposits;
    uint256 public totalDeposits;

    // ============ 이벤트 ============
    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);

    // ============ 생성자 비활성화 ============
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    // ============ 초기화 (constructor 대체) ============
    function initialize(
        address _token,
        address initialOwner
    ) public initializer {
        __Ownable_init(initialOwner);
        __ReentrancyGuard_init();
        __UUPSUpgradeable_init();

        token = ERC20Upgradeable(_token);
    }

    // ============ 핵심 기능 ============
    function deposit(uint256 amount) external nonReentrant {
        require(amount > 0, "Amount must be positive");
        token.transferFrom(msg.sender, address(this), amount);
        deposits[msg.sender] += amount;
        totalDeposits += amount;
        emit Deposited(msg.sender, amount);
    }

    function withdraw(uint256 amount) external nonReentrant {
        require(deposits[msg.sender] >= amount, "Insufficient deposit");
        deposits[msg.sender] -= amount;
        totalDeposits -= amount;
        token.transfer(msg.sender, amount);
        emit Withdrawn(msg.sender, amount);
    }

    // ============ 업그레이드 권한 ============
    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}

    // ============ 버전 조회 ============
    function version() external pure returns (string memory) {
        return "V1";
    }
}
```

```solidity
// V2: 수수료 기능 추가
contract UpgradeableVaultV2 is UpgradeableVaultV1 {
    // ============ 새 상태 변수 (기존 변수 뒤에 추가) ============
    uint256 public feeRate;       // 추가 (slot N)
    uint256 public totalFees;     // 추가 (slot N+1)
    address public feeRecipient;  // 추가 (slot N+2)

    // ============ 새 초기화 (reinitializer) ============
    function initializeV2(
        uint256 _feeRate,
        address _feeRecipient
    ) public reinitializer(2) {
        feeRate = _feeRate;
        feeRecipient = _feeRecipient;
    }

    // ============ 기존 함수 override ============
    function withdraw(uint256 amount) external override nonReentrant {
        require(deposits[msg.sender] >= amount, "Insufficient deposit");

        uint256 fee = (amount * feeRate) / 10000;
        uint256 netAmount = amount - fee;

        deposits[msg.sender] -= amount;
        totalDeposits -= amount;
        totalFees += fee;

        if (fee > 0) token.transfer(feeRecipient, fee);
        token.transfer(msg.sender, netAmount);

        emit Withdrawn(msg.sender, netAmount);
    }

    function version() external pure override returns (string memory) {
        return "V2";
    }
}
```

### 배포 스크립트

```solidity
// script/DeployUpgradeable.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {UpgradeableVaultV1} from "../src/UpgradeableVaultV1.sol";

contract DeployUpgradeable is Script {
    function run() external {
        uint256 deployerKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerKey);
        address tokenAddress = vm.envAddress("TOKEN_ADDRESS");

        vm.startBroadcast(deployerKey);

        // 1. 구현체 배포
        UpgradeableVaultV1 implementation = new UpgradeableVaultV1();

        // 2. 초기화 데이터 인코딩
        bytes memory initData = abi.encodeCall(
            UpgradeableVaultV1.initialize,
            (tokenAddress, deployer)
        );

        // 3. 프록시 배포 (구현체 + 초기화 데이터)
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            initData
        );

        vm.stopBroadcast();

        console.log("Implementation:", address(implementation));
        console.log("Proxy:", address(proxy));
        console.log("Version:", UpgradeableVaultV1(address(proxy)).version());
    }
}
```

```solidity
// script/UpgradeVault.s.sol
contract UpgradeVault is Script {
    function run() external {
        address proxyAddress = vm.envAddress("PROXY_ADDRESS");
        uint256 deployerKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(deployerKey);

        // 1. V2 구현체 배포
        UpgradeableVaultV2 implementationV2 = new UpgradeableVaultV2();

        // 2. 프록시를 통해 업그레이드 + V2 초기화
        UpgradeableVaultV1 proxy = UpgradeableVaultV1(proxyAddress);
        proxy.upgradeToAndCall(
            address(implementationV2),
            abi.encodeCall(
                UpgradeableVaultV2.initializeV2,
                (100, feeRecipient)  // 1% fee
            )
        );

        vm.stopBroadcast();

        console.log("Upgraded to V2:", address(implementationV2));
        console.log("Version:", UpgradeableVaultV2(proxyAddress).version());
    }
}
```

## NestJS DI와의 비유

```typescript
// NestJS: 의존성 주입으로 구현체 교체
// module에서 토큰으로 구현체를 바인딩
@Module({
    providers: [
        {
            provide: 'PAYMENT_SERVICE',
            useClass: StripePaymentService,  // V1
        },
    ],
})
export class AppModule {}

// 교체 시: 코드 변경 + 재배포
// useClass: PaypalPaymentService  // V2
```

```solidity
// Solidity: 프록시 패턴으로 구현체 교체
// 프록시가 구현체 주소를 가리킴
// 업그레이드 = 새 구현체 주소로 변경
proxy.upgradeToAndCall(address(implementationV2), "");
// 사용자는 같은 프록시 주소를 사용 — 투명하게 교체됨
```

**핵심 차이:** NestJS DI는 코드 레벨에서 교체하고 서버를 재시작한다. 스마트 컨트랙트 프록시는 체인 위에서 교체하고 데이터가 유지된다.

## 정리

1. **delegatecall** — 코드는 빌리고 storage와 context는 호출자 것을 사용
2. **Transparent Proxy** — admin/user 분리, 가스 비효율, 레거시
3. **UUPS Proxy** — 업그레이드 로직이 구현체에, 가스 효율적, 현재 표준
4. **스토리지 충돌** — EIP-1967 표준 슬롯으로 해결
5. **업그레이드 규칙** — 기존 변수 순서 유지, 새 변수는 뒤에만 추가
6. **_disableInitializers()** — 구현체 직접 초기화 방지 필수

다음 챕터에서는 스마트 컨트랙트 보안 취약점과 방어 패턴을 다룬다.
