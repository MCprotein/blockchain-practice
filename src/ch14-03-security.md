# Chapter 14-3: 스마트 컨트랙트 보안

## 왜 보안이 중요한가

스마트 컨트랙트 보안은 일반 웹 보안과 다른 차원의 위험이 있다.

| | 웹 서버 (Node.js) | 스마트 컨트랙트 |
|--|-----------------|--------------|
| 버그 발견 시 | 패치 배포 | 패치 불가 (불변성) |
| 피해 범위 | 데이터 유출, 서비스 중단 | 즉각적인 자금 탈취 |
| 복구 가능성 | DB 롤백, 백업 복원 | 불가 (트랜잭션 불변) |
| 코드 공개 | 선택적 | 강제 공개 (블록체인) |
| 공격자 | 내부 시스템 접근 필요 | 누구나 공개된 인터페이스로 호출 |

2023년 기준 스마트 컨트랙트 해킹으로 수십억 달러가 탈취됐다. 코드를 배포하기 전 철저한 보안 검토가 필수다.

## 1. 재진입 공격 (Reentrancy Attack)

### The DAO 해킹 사례 (2016)

2016년 "The DAO" 프로젝트에서 재진입 공격으로 3.6백만 ETH(당시 약 6천만 달러)가 탈취됐다. 이 사건으로 이더리움이 ETH/ETC로 하드포크됐다.

### 공격 원리

```
정상 출금 흐름:
1. 사용자가 withdraw() 호출
2. 잔액 확인 (충분한 잔액)
3. ETH 전송
4. 잔액 업데이트 (0으로)

재진입 공격 흐름:
1. 공격자가 withdraw() 호출
2. 잔액 확인 (100 ETH)
3. ETH 전송 → 공격자 컨트랙트의 receive() 트리거
   → receive()에서 withdraw()를 다시 호출 ← 재진입!
   → 잔액이 아직 업데이트 안 됨 → 또 100 ETH 전송
   → 또 재진입...
4. 모든 ETH 소진 후 잔액 업데이트 (이미 늦음)
```

### 취약한 코드

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// 취약한 뱅크 컨트랙트
contract VulnerableBank {
    mapping(address => uint256) public balances;

    function deposit() external payable {
        balances[msg.sender] += msg.value;
    }

    // 취약점: 상태 업데이트 전에 외부 호출
    function withdraw() external {
        uint256 amount = balances[msg.sender];
        require(amount > 0, "No balance");

        // 1. ETH 전송 (외부 호출)
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");

        // 2. 상태 업데이트 (너무 늦음!)
        balances[msg.sender] = 0;
    }
}
```

### 공격자 컨트랙트

```solidity
// 재진입 공격 컨트랙트
contract Attacker {
    VulnerableBank public bank;
    uint256 public attackCount;

    constructor(address _bank) {
        bank = VulnerableBank(_bank);
    }

    function attack() external payable {
        require(msg.value >= 1 ether, "Need 1 ETH");
        bank.deposit{value: 1 ether}();
        bank.withdraw();
    }

    // ETH를 받을 때마다 다시 withdraw() 호출 (재진입!)
    receive() external payable {
        attackCount++;
        if (address(bank).balance >= 1 ether && attackCount < 10) {
            bank.withdraw();  // 재진입!
        }
    }

    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }
}
```

### 방어 방법 1: Checks-Effects-Interactions 패턴

```solidity
contract SecureBank_CEI {
    mapping(address => uint256) public balances;

    function deposit() external payable {
        balances[msg.sender] += msg.value;
    }

    function withdraw() external {
        // Checks (검증)
        uint256 amount = balances[msg.sender];
        require(amount > 0, "No balance");

        // Effects (상태 변경) — 외부 호출 전에!
        balances[msg.sender] = 0;

        // Interactions (외부 호출) — 마지막에
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
    }
}
```

상태를 먼저 변경하면 재진입해도 잔액이 이미 0이므로 추가 인출 불가.

### 방어 방법 2: ReentrancyGuard

```solidity
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract SecureBank_Guard is ReentrancyGuard {
    mapping(address => uint256) public balances;

    function deposit() external payable {
        balances[msg.sender] += msg.value;
    }

    // nonReentrant: 실행 중 재진입 시 revert
    function withdraw() external nonReentrant {
        uint256 amount = balances[msg.sender];
        require(amount > 0, "No balance");
        balances[msg.sender] = 0;
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
    }
}
```

**권장:** CEI 패턴 + nonReentrant를 함께 사용한다. CEI는 로직적 보호, nonReentrant는 이중 안전망.

## 2. 정수 오버플로/언더플로

### Solidity 0.8 이전 (위험)

```solidity
// Solidity 0.7 이하 — 오버플로 체크 없음!
contract OldToken {
    mapping(address => uint256) public balances;

    function transfer(address to, uint256 amount) external {
        // uint256 최대값 + 1 = 0 (오버플로!)
        balances[msg.sender] -= amount;  // 잔액 0인데 빼면 엄청 큰 값이 됨
        balances[to] += amount;
    }
}

// 공격: 잔액이 0인 계정에서 1을 빼면
// 0 - 1 = type(uint256).max (약 1.15 * 10^77)
// 사실상 무한한 잔액이 생김
```

이를 방지하기 위해 OpenZeppelin의 `SafeMath` 라이브러리를 사용했다:

```solidity
// 0.7 이하 시절
using SafeMath for uint256;
balances[msg.sender] = balances[msg.sender].sub(amount); // underflow 방지
```

### Solidity 0.8 이후 (자동 보호)

```solidity
// Solidity 0.8+ — 오버플로/언더플로 자동 감지 후 revert
contract NewToken {
    mapping(address => uint256) public balances;

    function transfer(address to, uint256 amount) external {
        // 잔액 부족 시 자동으로 revert (언더플로 방지)
        balances[msg.sender] -= amount;
        balances[to] += amount;
    }
}
```

0.8 이후에는 SafeMath가 불필요하다. 단, 가스 최적화를 위해 `unchecked` 블록을 사용할 때는 직접 체크해야 한다:

```solidity
// unchecked: 오버플로 체크 비활성화 (가스 절약)
// 안전이 보장된 경우에만 사용!
function _transfer(address from, address to, uint256 amount) internal {
    uint256 fromBalance = _balances[from];
    require(fromBalance >= amount, "Insufficient balance"); // 수동 체크 필수!

    unchecked {
        _balances[from] = fromBalance - amount; // 위에서 체크했으므로 안전
        _balances[to] += amount;                // uint256 최대값 초과 시 문제될 수 있음
    }
}
```

## 3. tx.origin vs msg.sender

`tx.origin`은 트랜잭션을 최초 시작한 EOA(외부 계정)다. `msg.sender`는 현재 함수의 직접 호출자다.

```
EOA(Alice) → ContractA → ContractB.foo()

ContractB.foo() 내부:
  msg.sender = ContractA (직접 호출자)
  tx.origin  = Alice (최초 발신자)
```

### tx.origin 사용의 위험

```solidity
// 취약한 지갑 컨트랙트
contract VulnerableWallet {
    address public owner;

    constructor() { owner = msg.sender; }

    // tx.origin 사용 — 위험!
    function transfer(address payable to, uint256 amount) external {
        require(tx.origin == owner, "Not owner");
        to.transfer(amount);
    }
}

// 피싱 공격 컨트랙트
contract PhishingContract {
    VulnerableWallet public wallet;

    constructor(address _wallet) {
        wallet = VulnerableWallet(_wallet);
    }

    // 사용자가 이 함수를 호출하도록 속임
    receive() external payable {
        // tx.origin = Alice (속은 사용자)
        // msg.sender = PhishingContract
        // wallet은 tx.origin(Alice)을 owner로 확인 → 통과!
        wallet.transfer(payable(msg.sender), address(wallet).balance);
    }
}
```

공격 흐름:
1. 공격자가 피싱 사이트를 만들어 Alice에게 "무료 NFT를 받으려면 이 컨트랙트에 ETH를 보내세요" 유도
2. Alice가 PhishingContract에 ETH 전송 → receive() 실행
3. tx.origin = Alice이므로 VulnerableWallet의 보안 통과
4. Alice의 지갑에 있는 모든 ETH 탈취

### 올바른 접근법

```solidity
contract SecureWallet {
    address public owner;

    constructor() { owner = msg.sender; }

    // msg.sender 사용 — 안전
    function transfer(address payable to, uint256 amount) external {
        require(msg.sender == owner, "Not owner");
        // msg.sender가 컨트랙트라면 컨트랙트가 owner여야 통과
        // 피싱 공격 시 msg.sender = PhishingContract ≠ owner → revert
        to.transfer(amount);
    }
}
```

**규칙:** 접근 제어에 `tx.origin`을 절대 사용하지 말고 항상 `msg.sender`를 사용하라. `tx.origin`의 유일한 합법적 사용은 "EOA만 허용" (컨트랙트 호출 방지)이지만, 이 역시 권장되지 않는다.

## 4. 프론트러닝 (Front-Running)

블록체인의 트랜잭션은 확정되기 전에 멤풀(mempool)에 공개된다. 채굴자나 다른 참여자가 이 정보를 보고 자신의 트랜잭션을 먼저 끼워넣을 수 있다.

```
Alice의 트랜잭션: DEX에서 토큰 A를 100 ETH로 구매 (멤풀에 공개)
             ↓
공격자(Bot)가 발견: Alice의 구매로 가격이 오를 것을 예측
             ↓
공격자: 더 높은 가스비로 같은 토큰 먼저 구매 (Alice보다 먼저 채굴됨)
             ↓
Alice의 구매 실행: 이미 가격이 올라서 더 비싸게 삼 (슬리피지 손실)
             ↓
공격자: 비싸진 가격에 팔아서 차익 획득
```

이를 **샌드위치 공격(Sandwich Attack)**이라고 한다.

### 방어 방법

```solidity
// 1. 슬리피지 제한 (사용자가 허용할 최소 수량 지정)
function swap(
    address tokenIn,
    address tokenOut,
    uint256 amountIn,
    uint256 minAmountOut  // 이 이상을 못 받으면 revert
) external {
    uint256 amountOut = _calculateOutput(tokenIn, tokenOut, amountIn);
    require(amountOut >= minAmountOut, "Slippage too high");
    // ...
}

// 2. Commit-Reveal 패턴 (게임, 경매에서)
mapping(address => bytes32) public commitments;

// 1단계: 의도를 해시로 숨겨서 제출
function commit(bytes32 commitment) external {
    commitments[msg.sender] = commitment;
}

// 2단계: 나중에 공개
function reveal(uint256 value, bytes32 salt) external {
    require(
        commitments[msg.sender] == keccak256(abi.encodePacked(value, salt)),
        "Invalid reveal"
    );
    // value 사용
    delete commitments[msg.sender];
}
```

## 5. 플래시론 공격 개요

플래시론(Flash Loan)은 동일 트랜잭션 안에서 담보 없이 거액을 빌리고 갚는 DeFi 기능이다. 정당한 용도(차익거래, 담보 교환)도 있지만 공격에 악용될 수 있다.

```
공격 예시 (가격 조작):
1. 플래시론으로 100만 ETH 대출
2. 소형 DEX에서 특정 토큰 대량 매입 → 가격 조작
3. 조작된 가격을 사용하는 프로토콜에서 이익 취득
4. 플래시론 상환
```

**방어:** 외부 가격 피드로 오라클을 사용하거나 (Chainlink), TWAP(시간 가중 평균 가격)을 사용해 순간적인 가격 조작을 무력화한다.

## 6. 기타 주요 취약점

### access control 실수

```solidity
// 취약: initialize를 누구나 호출 가능
function initialize(address owner) external {
    _owner = owner;  // 공격자가 먼저 호출해서 소유권 탈취!
}

// 안전: initializer 제어자 사용
function initialize(address owner) external initializer {
    _owner = owner;
}
```

### block.timestamp 조작

```solidity
// 취약: 채굴자가 타임스탬프를 약간 조작 가능 (±15초)
function isLotteryOpen() public view returns (bool) {
    return block.timestamp % 7 == 0;  // 예측/조작 가능
}

// 안전: 타임스탬프는 큰 범위에서만 신뢰 (초 단위 정밀도는 금물)
function hasExpired() public view returns (bool) {
    return block.timestamp > deadline;  // 수 분 이상 차이는 괜찮음
}
```

### 정수 나눗셈 버림

```solidity
// 버림(truncation) 주의
uint256 fee = (amount * 3) / 100;
// amount = 10이면: (10 * 3) / 100 = 30 / 100 = 0 (수수료 없음!)

// 충분한 정밀도 유지
uint256 fee = (amount * 300) / 10000;  // basis points 사용
// amount = 10이면: (10 * 300) / 10000 = 3000 / 10000 = 0 (여전히 0)
// 최소 금액 제한을 두거나 더 큰 단위로 계산해야 함
```

### 외부 컨트랙트 신뢰

```solidity
// 취약: 토큰 컨트랙트가 악의적일 수 있음
function deposit(address token, uint256 amount) external {
    IERC20(token).transferFrom(msg.sender, address(this), amount);
    balances[msg.sender] += amount;  // 실제로 받은 양 ≠ amount일 수 있음
    // 전송 수수료가 있는 토큰(fee-on-transfer)이면 실제 수령량이 적음
}

// 안전: 실제 받은 양 확인
function deposit(address token, uint256 amount) external {
    uint256 before = IERC20(token).balanceOf(address(this));
    IERC20(token).transferFrom(msg.sender, address(this), amount);
    uint256 actual = IERC20(token).balanceOf(address(this)) - before;
    balances[msg.sender] += actual;  // 실제 수령량 사용
}
```

## 안전한 컨트랙트 작성 체크리스트

```
[ ] CEI 패턴 준수 (Checks → Effects → Interactions)
[ ] nonReentrant 적용 (ETH/토큰 전송이 있는 모든 함수)
[ ] Solidity 0.8+ 사용 (자동 오버플로 방지)
[ ] tx.origin 미사용 (접근 제어에)
[ ] msg.sender 기반 접근 제어
[ ] 슬리피지 보호 (DEX 관련)
[ ] 입력값 전체 검증 (address(0), 0 값, 범위 초과)
[ ] 외부 컨트랙트 호출 최소화
[ ] 실제 수령량 확인 (fee-on-transfer 토큰 대비)
[ ] 초기화 함수 보호 (initializer 제어자)
[ ] 긴급 정지(Pause) 메커니즘
[ ] 다단계 소유권 이전 (Ownable2Step)
[ ] 업그레이드 가능 여부 설계
[ ] 이벤트 발행 (모든 중요 상태 변경)
[ ] 퍼즈 테스트 실행 (다양한 입력값)
[ ] 코드 커버리지 100% 목표
[ ] 외부 감사(audit) 진행
[ ] 버그 바운티 프로그램 운영
```

## 완전한 보안 컨트랙트 예시

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Ownable2Step, Ownable} from "@openzeppelin/contracts/access/Ownable2Step.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title SecureVault - 모든 보안 패턴이 적용된 금고
contract SecureVault is ReentrancyGuard, Ownable2Step, Pausable {
    using SafeERC20 for IERC20;

    // ============ 에러 ============
    error ZeroAmount();
    error ZeroAddress();
    error InsufficientBalance(uint256 available, uint256 required);
    error ExceedsDepositLimit(uint256 amount, uint256 limit);

    // ============ 상태 변수 ============
    IERC20 public immutable token;
    mapping(address => uint256) private _balances;
    uint256 public totalDeposits;
    uint256 public depositLimit;

    // ============ 이벤트 ============
    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
    event DepositLimitUpdated(uint256 oldLimit, uint256 newLimit);

    // ============ 생성자 ============
    constructor(address _token, address initialOwner, uint256 _depositLimit)
        Ownable(initialOwner)
    {
        if (_token == address(0)) revert ZeroAddress();
        if (initialOwner == address(0)) revert ZeroAddress();

        token = IERC20(_token);
        depositLimit = _depositLimit;
    }

    // ============ 핵심 함수 (CEI + nonReentrant + whenNotPaused) ============

    function deposit(uint256 amount)
        external
        nonReentrant
        whenNotPaused
    {
        // Checks
        if (amount == 0) revert ZeroAmount();
        if (totalDeposits + amount > depositLimit) {
            revert ExceedsDepositLimit(amount, depositLimit);
        }

        // Effects (상태 변경 먼저)
        _balances[msg.sender] += amount;
        totalDeposits += amount;

        // Interactions (외부 호출 마지막)
        // SafeERC20: transfer 실패 시 revert (반환값 false 처리)
        // fee-on-transfer 토큰 대비: 실제 수령량 확인 가능
        token.safeTransferFrom(msg.sender, address(this), amount);

        emit Deposited(msg.sender, amount);
    }

    function withdraw(uint256 amount)
        external
        nonReentrant
        whenNotPaused
    {
        // Checks
        if (amount == 0) revert ZeroAmount();
        uint256 balance = _balances[msg.sender];
        if (balance < amount) revert InsufficientBalance(balance, amount);

        // Effects
        _balances[msg.sender] = balance - amount;
        totalDeposits -= amount;

        // Interactions
        token.safeTransfer(msg.sender, amount);

        emit Withdrawn(msg.sender, amount);
    }

    // ============ 조회 ============

    function balanceOf(address user) external view returns (uint256) {
        return _balances[user];
    }

    // ============ 소유자 전용 ============

    function setDepositLimit(uint256 newLimit) external onlyOwner {
        emit DepositLimitUpdated(depositLimit, newLimit);
        depositLimit = newLimit;
    }

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    // 긴급 자금 회수 (일시 정지 상태에서만)
    function emergencyWithdraw(address to) external onlyOwner whenPaused {
        if (to == address(0)) revert ZeroAddress();
        uint256 balance = token.balanceOf(address(this));
        token.safeTransfer(to, balance);
    }
}
```

이 컨트랙트는 다음 보안 패턴을 모두 적용했다:
- **ReentrancyGuard** — 재진입 공격 방지
- **CEI 패턴** — Effects가 Interactions 전에 완료
- **Ownable2Step** — 2단계 소유권 이전
- **Pausable** — 긴급 정지
- **SafeERC20** — 안전한 토큰 전송
- **커스텀 에러** — 가스 효율적 에러
- **이벤트** — 모든 상태 변경 기록
- **입력 검증** — 모든 파라미터 검증

## 정리

스마트 컨트랙트 보안의 핵심 원칙:

1. **CEI 패턴** — Checks → Effects → Interactions 순서를 절대 지킨다
2. **nonReentrant** — ETH/토큰 전송이 포함된 함수에 항상 적용
3. **msg.sender** — 접근 제어는 항상 msg.sender, tx.origin은 사용 금지
4. **입력 검증** — 모든 외부 입력값을 철저히 검증
5. **최소 권한** — 각 역할에 필요한 최소한의 권한만 부여
6. **외부 감사** — 큰 금액이 걸린 컨트랙트는 반드시 전문가 감사
