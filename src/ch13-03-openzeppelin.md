# Chapter 13-3: OpenZeppelin

## OpenZeppelin이란

OpenZeppelin은 스마트 컨트랙트 보안 회사이자, 이더리움 생태계에서 가장 널리 사용되는 오픈소스 컨트랙트 라이브러리를 제공하는 조직이다. 2016년부터 수많은 보안 감사를 거친 검증된 구현체를 제공한다.

**Node.js 생태계 비유:**

| Node.js | OpenZeppelin |
|---------|-------------|
| `express` | ERC-20, ERC-721 기본 구현 |
| `passport` | AccessControl, Ownable |
| `helmet` | ReentrancyGuard, Pausable |
| `lodash` | SafeMath (0.8+ 이후 내장), Strings |
| `sequelize` | - |

직접 구현보다 OpenZeppelin을 사용하는 이유:
- 수백만 달러 규모의 컨트랙트에서 검증됨
- 보안 취약점 발견 시 빠른 패치
- 커뮤니티 표준으로 자리잡아 감사(audit) 비용 절감
- 최신 EIP 구현 반영

## Foundry에서 설치

```bash
# OpenZeppelin 컨트랙트 설치
forge install OpenZeppelin/openzeppelin-contracts

# 업그레이드 가능 버전 (프록시 패턴용)
forge install OpenZeppelin/openzeppelin-contracts-upgradeable

# remappings.txt에 경로 매핑 추가
echo "@openzeppelin/=lib/openzeppelin-contracts/" >> remappings.txt
```

설치 확인:
```text
lib/
└── openzeppelin-contracts/
    └── contracts/
        ├── access/
        │   ├── Ownable.sol
        │   └── AccessControl.sol
        ├── token/
        │   ├── ERC20/
        │   └── ERC721/
        ├── security/
        │   ├── ReentrancyGuard.sol
        │   └── Pausable.sol
        └── utils/
            ├── Strings.sol
            └── ...
```

## ERC20 상속으로 토큰 만들기

앞 챕터에서 직접 구현한 ERC-20 코드가 수백 줄이었다. OpenZeppelin을 사용하면:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {ERC20Burnable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import {ERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";

/// @title MyToken - OpenZeppelin 기반 ERC-20 토큰
contract MyToken is ERC20, ERC20Burnable, ERC20Permit, Ownable {
    uint256 public constant MAX_SUPPLY = 100_000_000 * 1e18; // 1억 토큰

    constructor(
        address initialOwner,
        uint256 initialSupply
    )
        ERC20("MyToken", "MTK")
        ERC20Permit("MyToken")   // EIP-2612: 서명으로 approve
        Ownable(initialOwner)
    {
        require(initialSupply <= MAX_SUPPLY, "Exceeds max supply");
        _mint(initialOwner, initialSupply);
    }

    /// @notice 소유자가 새 토큰 발행 (최대 공급량 제한)
    function mint(address to, uint256 amount) external onlyOwner {
        require(totalSupply() + amount <= MAX_SUPPLY, "Exceeds max supply");
        _mint(to, amount);
    }
}
```

`ERC20Burnable`은 `burn()`, `burnFrom()` 함수를 자동으로 추가한다.
`ERC20Permit`은 EIP-2612 gasless approve를 지원한다 (서명으로 approve, 트랜잭션 없이).

### ERC721 상속 예시

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ERC721URIStorage} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import {ERC721Enumerable} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

contract MyNFT is ERC721, ERC721Enumerable, ERC721URIStorage, Ownable {
    using Strings for uint256;

    uint256 private _nextTokenId;
    uint256 public maxSupply;
    string private _baseURIValue;

    constructor(
        address initialOwner,
        uint256 _maxSupply,
        string memory baseURI
    ) ERC721("MyNFT", "MNFT") Ownable(initialOwner) {
        maxSupply = _maxSupply;
        _baseURIValue = baseURI;
    }

    function mint(address to) external onlyOwner returns (uint256) {
        require(_nextTokenId < maxSupply, "Max supply reached");
        uint256 tokenId = _nextTokenId++;
        _safeMint(to, tokenId);
        return tokenId;
    }

    function _baseURI() internal view override returns (string memory) {
        return _baseURIValue;
    }

    // ERC721Enumerable + ERC721URIStorage 충돌 해결
    function _update(address to, uint256 tokenId, address auth)
        internal
        override(ERC721, ERC721Enumerable)
        returns (address)
    {
        return super._update(to, tokenId, auth);
    }

    function _increaseBalance(address account, uint128 value)
        internal
        override(ERC721, ERC721Enumerable)
    {
        super._increaseBalance(account, value);
    }

    function tokenURI(uint256 tokenId)
        public view
        override(ERC721, ERC721URIStorage)
        returns (string memory)
    {
        return super.tokenURI(tokenId);
    }

    function supportsInterface(bytes4 interfaceId)
        public view
        override(ERC721, ERC721Enumerable, ERC721URIStorage)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }
}
```

## Ownable — 소유권 관리

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract MyContract is Ownable {
    uint256 public value;

    constructor(address initialOwner) Ownable(initialOwner) {}

    // onlyOwner 제어자 자동 제공
    function setValue(uint256 newValue) external onlyOwner {
        value = newValue;
    }

    // 소유권 포기 (address(0)으로 이전)
    // renounceOwnership() 자동 제공

    // 소유권 이전
    // transferOwnership(address newOwner) 자동 제공
}
```

OpenZeppelin v5부터 생성자에서 초기 소유자를 명시적으로 전달해야 한다(`Ownable(initialOwner)`).

### Ownable2Step — 2단계 소유권 이전

실수로 잘못된 주소로 소유권을 이전하는 사고를 방지한다:

```solidity
import {Ownable2Step, Ownable} from "@openzeppelin/contracts/access/Ownable2Step.sol";

contract SafeOwnable is Ownable2Step {
    constructor(address initialOwner) Ownable(initialOwner) {}

    function adminAction() external onlyOwner {
        // 소유자만 실행 가능
    }
}

// 사용:
// 1. contract.transferOwnership(newOwner) — 제안
// 2. newOwner가 contract.acceptOwnership() 호출 — 수락
// → 새 소유자가 수락하지 않으면 소유권 이전 안 됨
```

## AccessControl — 역할 기반 접근 제어

`Ownable`은 소유자 1명만 관리할 수 있다. 여러 역할이 필요할 때는 `AccessControl`을 사용한다.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract RoleBasedToken is ERC20, AccessControl {
    // 역할 정의: keccak256 해시로 식별
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant BURNER_ROLE = keccak256("BURNER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    bool public paused;

    constructor(address admin) ERC20("RoleToken", "RTK") {
        // admin에게 DEFAULT_ADMIN_ROLE 부여 (역할 관리자)
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        // admin에게 MINTER_ROLE도 부여
        _grantRole(MINTER_ROLE, admin);
    }

    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyRole(BURNER_ROLE) {
        _burn(from, amount);
    }

    function pause() external onlyRole(PAUSER_ROLE) {
        paused = true;
    }

    function unpause() external onlyRole(PAUSER_ROLE) {
        paused = false;
    }

    // 역할 관리 (DEFAULT_ADMIN_ROLE만 가능)
    // grantRole(role, account) — 자동 제공
    // revokeRole(role, account) — 자동 제공
    // renounceRole(role, account) — 자동 제공
}
```

```typescript
// NestJS의 @Roles() 데코레이터 + RolesGuard와 개념적으로 동일
@Roles('admin', 'minter')
@UseGuards(RolesGuard)
@Post('/mint')
async mint(@Body() dto: MintDto) {
  return this.tokenService.mint(dto.to, dto.amount);
}
```

```solidity
// Solidity - onlyRole 제어자로 동일한 패턴
function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
    _mint(to, amount);
}
```

**AccessControl 사용 예:**

```bash
# 역할 부여 (DEFAULT_ADMIN_ROLE 보유자만)
cast send $TOKEN "grantRole(bytes32,address)" \
    $(cast keccak "MINTER_ROLE") $MINTER_ADDRESS \
    --private-key $ADMIN_KEY

# 역할 확인
cast call $TOKEN "hasRole(bytes32,address)" \
    $(cast keccak "MINTER_ROLE") $MINTER_ADDRESS
```

## ReentrancyGuard — 재진입 공격 방지

재진입 공격(Reentrancy Attack)은 외부 컨트랙트 호출 중에 다시 같은 함수를 호출하는 공격이다. The DAO 해킹(2016, 6천만 달러)의 원인이 된 공격 유형이다.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract SafeVault is ReentrancyGuard {
    mapping(address => uint256) public balances;

    function deposit() external payable {
        balances[msg.sender] += msg.value;
    }

    // nonReentrant: 재진입 시 revert
    function withdraw(uint256 amount) external nonReentrant {
        require(balances[msg.sender] >= amount, "Insufficient balance");

        // 상태 먼저 변경 (Checks-Effects-Interactions)
        balances[msg.sender] -= amount;

        // 외부 호출 (이 호출 중 재진입 시도하면 nonReentrant가 막음)
        (bool success, ) = payable(msg.sender).call{value: amount}("");
        require(success, "Transfer failed");
    }
}
```

`nonReentrant`는 내부적으로 `_status` 변수로 잠금을 구현한다:

```solidity
// OpenZeppelin ReentrancyGuard 내부 (단순화)
modifier nonReentrant() {
    require(_status != _ENTERED, "ReentrancyGuard: reentrant call");
    _status = _ENTERED;
    _;
    _status = _NOT_ENTERED;
}
```

## Pausable — 긴급 정지

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC20Pausable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Pausable.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract PausableToken is ERC20, ERC20Pausable, Ownable {
    constructor(address initialOwner)
        ERC20("PausableToken", "PTK")
        Ownable(initialOwner)
    {}

    // 긴급 정지 (소유자만)
    function pause() external onlyOwner {
        _pause(); // whenNotPaused 상태로 잠금
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    // ERC20Pausable이 transfer 시 _requireNotPaused() 자동 호출
    function _update(address from, address to, uint256 value)
        internal
        override(ERC20, ERC20Pausable)
    {
        super._update(from, to, value);
    }
}
```

## Upgradeable 컨트랙트 기초

스마트 컨트랙트는 한 번 배포하면 수정이 불가능하다. 업그레이드 가능한 컨트랙트는 **프록시 패턴**으로 이 문제를 해결한다.

자세한 내용은 14-02 챕터에서 다루고, 여기서는 OpenZeppelin의 업그레이드 가능 컨트랙트 사용법만 간략히 소개한다.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// 업그레이드 가능 버전은 -upgradeable 패키지에서
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

/// @custom:oz-upgrades-unsafe-allow constructor
contract UpgradeableToken is
    Initializable,
    ERC20Upgradeable,
    OwnableUpgradeable,
    UUPSUpgradeable
{
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers(); // 구현체 직접 초기화 방지
    }

    // constructor 대신 initialize 사용
    function initialize(
        string memory name,
        string memory symbol,
        address initialOwner
    ) public initializer {
        __ERC20_init(name, symbol);
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();

        _mint(initialOwner, 1_000_000 * 1e18);
    }

    // 업그레이드 권한 제어 (소유자만 업그레이드 가능)
    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}

    function mint(address to, uint256 amount) external onlyOwner {
        _mint(to, amount);
    }
}
```

**업그레이드 가능 컨트랙트의 핵심 규칙:**
- `constructor` 대신 `initialize` 함수 사용 (`initializer` 제어자로 한 번만 실행)
- 상태 변수 순서를 업그레이드 시 변경하면 안 됨 (스토리지 충돌)
- `immutable` 변수 사용 금지

## OpenZeppelin 주요 확장 목록

```solidity
// ERC-20 확장
import {ERC20Burnable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
// burn(), burnFrom() 추가

import {ERC20Capped} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Capped.sol";
// 최대 공급량 제한

import {ERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
// EIP-2612: 서명으로 approve (가스리스)

import {ERC20Votes} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";
// 거버넌스 투표권 (체크포인트 기반)

import {ERC20FlashMint} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20FlashMint.sol";
// 플래시론 지원

// ERC-721 확장
import {ERC721Enumerable} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
// 토큰 목록 순회 가능

import {ERC721URIStorage} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
// 토큰별 URI 저장

import {ERC721Royalty} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721Royalty.sol";
// EIP-2981: 로열티 정보

// 유틸리티
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {Address} from "@openzeppelin/contracts/utils/Address.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {SafeCast} from "@openzeppelin/contracts/utils/math/SafeCast.sol";
```

## 완성된 프로덕션급 토큰 예시

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC20Burnable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import {ERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import {ERC20Votes} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Nonces} from "@openzeppelin/contracts/utils/Nonces.sol";

contract GovernanceToken is
    ERC20,
    ERC20Burnable,
    ERC20Permit,
    ERC20Votes,
    AccessControl,
    ReentrancyGuard
{
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    uint256 public constant MAX_SUPPLY = 1_000_000_000 * 1e18; // 10억

    constructor(address admin)
        ERC20("GovernanceToken", "GOV")
        ERC20Permit("GovernanceToken")
    {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(MINTER_ROLE, admin);
    }

    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        require(totalSupply() + amount <= MAX_SUPPLY, "Exceeds max supply");
        _mint(to, amount);
    }

    // ERC20Votes + ERC20Permit 충돌 해결
    function _update(address from, address to, uint256 value)
        internal
        override(ERC20, ERC20Votes)
    {
        super._update(from, to, value);
    }

    function nonces(address owner)
        public view
        override(ERC20Permit, Nonces)
        returns (uint256)
    {
        return super.nonces(owner);
    }
}
```

## 정리

OpenZeppelin 사용의 핵심:

1. **검증된 구현 재사용** — 직접 구현 대신 검증된 코드를 상속
2. **역할 기반 접근 제어** — `Ownable`(단순) vs `AccessControl`(복잡)
3. **보안 패턴** — `ReentrancyGuard`, `Pausable`은 거의 항상 포함
4. **확장성** — 필요한 기능을 믹스인(Mixin)처럼 추가
5. **업그레이드** — `Upgradeable` 버전으로 업그레이드 가능한 컨트랙트 구현

다음 챕터에서는 상속, 프록시 패턴, 보안 취약점 등 고급 주제를 다룬다.
