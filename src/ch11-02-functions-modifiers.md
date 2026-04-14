# Chapter 11-2: 함수와 제어자 (Functions & Modifiers)

## 함수 선언 문법

Solidity 함수의 전체 문법 구조:

```solidity
function transfer(address to, uint256 amount) public returns (bool) {
    balances[msg.sender] -= amount;
    balances[to] += amount;
    return true;
}
```

실제 예시로 각 요소를 확인해보자:

```solidity
// 기본 함수
function add(uint256 a, uint256 b) public pure returns (uint256) {
    return a + b;
}

// 여러 값 반환
function getMinMax(uint256[] memory arr) public pure returns (uint256 min, uint256 max) {
    min = arr[0];
    max = arr[0];
    for (uint256 i = 1; i < arr.length; i++) {
        if (arr[i] < min) min = arr[i];
        if (arr[i] > max) max = arr[i];
    }
    // named return이면 return문 생략 가능
}

// 반환값 명시적으로
function divide(uint256 a, uint256 b) public pure returns (uint256 quotient, uint256 remainder) {
    return (a / b, a % b);
}
```

TypeScript와의 문법 비교:

```typescript
// TypeScript
function add(a: number, b: number): number {
    return a + b;
}

// 구조분해 반환
function getMinMax(arr: number[]): { min: number; max: number } {
    return { min: Math.min(...arr), max: Math.max(...arr) };
}
```

```solidity
// Solidity - 타입이 앞에, 이름이 뒤에
function add(uint256 a, uint256 b) public pure returns (uint256) {
    return a + b;
}
```

## 가시성 (Visibility)

### public

외부 계정(EOA), 다른 컨트랙트, 그리고 이 컨트랙트 내부 어디서든 호출 가능.

```solidity
function transfer(address to, uint256 amount) public {
    balances[msg.sender] -= amount;
    balances[to] += amount;
}
```

### external

오직 외부에서만 호출 가능. 컨트랙트 내부에서는 `this.funcName()`으로 호출해야 한다. 파라미터를 calldata에서 직접 읽어 `public`보다 가스 효율이 좋다.

```solidity
// 인터페이스 구현 함수, 외부에서만 호출될 함수에 적합
function deposit(uint256 amount) external payable {
    require(msg.value == amount, "Value mismatch");
    balances[msg.sender] += amount;
}
```

### internal

이 컨트랙트와 이를 상속한 자식 컨트랙트에서만 호출 가능. TypeScript의 `protected`와 유사.

```solidity
function _mint(address to, uint256 amount) internal {
    totalSupply += amount;
    balances[to] += amount;
}

// 자식 컨트랙트에서 호출 가능
contract MyToken is BaseToken {
    function publicMint() external {
        _mint(msg.sender, 100 * 10**18);
    }
}
```

### private

오직 이 컨트랙트에서만 호출 가능. 상속 컨트랙트에서도 접근 불가.

```solidity
function _validateAmount(uint256 amount) private pure returns (bool) {
    return amount > 0 && amount <= type(uint256).max;
}
```

**관례:** internal/private 함수는 이름 앞에 `_` 언더스코어를 붙이는 것이 관례다 (OpenZeppelin 스타일).

## 상태 변경성 (State Mutability)

### view

상태를 읽기만 하고 변경하지 않는다. 트랜잭션 없이 무료로 호출 가능 (로컬 노드에서 실행).

```solidity
uint256 private _totalSupply;
mapping(address => uint256) private _balances;

function totalSupply() public view returns (uint256) {
    return _totalSupply;
}

function balanceOf(address account) public view returns (uint256) {
    return _balances[account];
}

// 계산 로직도 view로 가능 (상태 읽기는 허용)
function getPercentage(address account) public view returns (uint256) {
    if (_totalSupply == 0) return 0;
    return (_balances[account] * 100) / _totalSupply;
}
```

### pure

상태를 읽지도, 변경하지도 않는다. 순수하게 입력값만으로 계산.

```solidity
function multiply(uint256 a, uint256 b) public pure returns (uint256) {
    return a * b;
}

function hashData(address addr, uint256 amount) public pure returns (bytes32) {
    return keccak256(abi.encodePacked(addr, amount));
}

// 수학 유틸리티 함수들은 보통 pure
function min(uint256 a, uint256 b) internal pure returns (uint256) {
    return a < b ? a : b;
}
```

### payable

ETH를 받을 수 있는 함수. `msg.value`가 0이 아닌 트랜잭션을 수락한다.

```solidity
function deposit() external payable {
    require(msg.value > 0, "Must send ETH");
    balances[msg.sender] += msg.value;
    emit Deposited(msg.sender, msg.value);
}

// payable이 없으면 ETH와 함께 호출 시 자동 revert
function regularFunction() external {
    // msg.value는 항상 0
}
```

**상태 변경성 규칙 요약:**

| 키워드 | 상태 읽기 | 상태 쓰기 | ETH 수신 | 비용 |
|-------|---------|---------|---------|------|
| (없음) | O | O | X | 가스 필요 |
| `view` | O | X | X | 무료 (로컬) |
| `pure` | X | X | X | 무료 (로컬) |
| `payable` | O | O | O | 가스 필요 |

## 생성자 (Constructor)

컨트랙트 배포 시 딱 한 번만 실행되는 특수 함수.

```solidity
contract Token {
    string public name;
    string public symbol;
    uint256 public totalSupply;
    address public owner;
    mapping(address => uint256) private _balances;
    
    constructor(
        string memory _name,
        string memory _symbol,
        uint256 initialSupply
    ) {
        name = _name;
        symbol = _symbol;
        owner = msg.sender;
        
        // 초기 공급량을 배포자에게 지급
        totalSupply = initialSupply;
        _balances[msg.sender] = initialSupply;
    }
}
```

배포 시 생성자 인수를 전달한다:

```bash
# Foundry로 배포 시
forge create Token --constructor-args "MyToken" "MTK" 1000000000000000000000000
```

**상속과 생성자:**

```solidity
contract Ownable {
    address public owner;
    
    constructor() {
        owner = msg.sender;
    }
}

contract Token is Ownable {
    string public name;
    
    // 부모 생성자는 자동 호출 (인수 없는 경우)
    constructor(string memory _name) {
        name = _name;
    }
}

// 부모 생성자에 인수가 필요한 경우
contract ChildToken is BaseToken {
    constructor(string memory _name) BaseToken(_name, "CHILD") {
        // BaseToken의 생성자에 인수 전달
    }
}
```

## receive()와 fallback() 함수

컨트랙트가 ETH를 받거나 알 수 없는 함수가 호출될 때 실행되는 특수 함수들.

### receive()

순수하게 ETH만 전송될 때 (calldata가 비어있을 때) 호출된다.

```solidity
contract ETHReceiver {
    event Received(address sender, uint256 amount);
    
    // ETH를 받기 위한 함수
    receive() external payable {
        emit Received(msg.sender, msg.value);
        // 추가 로직 가능
    }
}
```

```javascript
// 외부에서 단순 ETH 전송 (TypeScript/ethers.js)
await signer.sendTransaction({
    to: contractAddress,
    value: ethers.parseEther("1.0")
    // data 없음 -> receive() 호출
});
```

### fallback()

1. 매칭되는 함수가 없을 때
2. calldata가 있는데 receive()가 없을 때

```solidity
contract Proxy {
    address public implementation;
    
    constructor(address _impl) {
        implementation = _impl;
    }
    
    // 모든 호출을 구현체로 전달 (프록시 패턴)
    fallback() external payable {
        address impl = implementation;
        assembly {
            // calldata를 그대로 구현체에 전달
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }
    
    receive() external payable {}
}
```

**receive vs fallback 호출 흐름:**

```text
ETH 전송 또는 함수 호출
         |
    calldata 있음?
    /           \
  없음           있음
   |              |
receive()    함수 선택자 매칭?
존재?        /           \
/    \     있음          없음
있음  없음   |              |
|     |   해당 함수    fallback()
receive() fallback()    존재?
                       /      \
                     있음     없음
                      |         |
                  fallback()  revert
```

## 함수 제어자 (Modifier)

제어자는 함수 실행 전후에 공통 로직을 삽입하는 메커니즘이다. NestJS의 Guard, Middleware와 개념적으로 유사하다.

### 기본 제어자 패턴

```solidity
contract Ownable {
    address public owner;
    
    constructor() {
        owner = msg.sender;
    }
    
    // 제어자 정의
    modifier onlyOwner() {
        require(msg.sender == owner, "Ownable: caller is not the owner");
        _;  // 이 위치에 실제 함수 코드가 삽입됨
    }
    
    // 제어자 적용
    function transferOwnership(address newOwner) public onlyOwner {
        require(newOwner != address(0), "Ownable: new owner is the zero address");
        owner = newOwner;
    }
    
    function renounceOwnership() public onlyOwner {
        owner = address(0);
    }
}
```

`_;` 는 "여기에 함수 본문을 실행하라"는 플레이스홀더다. 제어자 코드가 `_;` 전에 오면 함수 실행 전 체크, 후에 오면 함수 실행 후 체크다.

### NestJS Guards와의 비교

```typescript
// NestJS - Guard
@Injectable()
export class AuthGuard implements CanActivate {
    canActivate(context: ExecutionContext): boolean {
        const request = context.switchToHttp().getRequest();
        return request.user?.role === 'admin';
    }
}

// Controller에 적용
@UseGuards(AuthGuard)
@Post('/admin/action')
async adminAction() {
    // Guard 통과 후 실행
}
```

```solidity
// Solidity - Modifier
modifier onlyOwner() {
    require(msg.sender == owner, "Not owner");
    _;
}

// 함수에 적용
function adminAction() public onlyOwner {
    // require 통과 후 실행
}
```

**핵심 차이:** NestJS Guard는 HTTP 레이어에서 작동하고 DI(의존성 주입)을 활용한다. Solidity Modifier는 컴파일 시점에 인라인으로 삽입된다 (매크로와 유사).

### 파라미터가 있는 제어자

```solidity
modifier minimumAmount(uint256 minimum) {
    require(msg.value >= minimum, "Insufficient ETH sent");
    _;
}

function premiumDeposit() external payable minimumAmount(0.1 ether) {
    // 0.1 ETH 이상만 가능
    premiumBalances[msg.sender] += msg.value;
}
```

### 여러 제어자 조합

```solidity
contract AccessControl {
    address public owner;
    bool public paused;
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    modifier whenNotPaused() {
        require(!paused, "Contract is paused");
        _;
    }
    
    modifier validAddress(address addr) {
        require(addr != address(0), "Zero address");
        _;
    }
    
    // 여러 제어자를 순서대로 적용
    function transfer(
        address to,
        uint256 amount
    ) public whenNotPaused validAddress(to) {
        // whenNotPaused 체크 -> validAddress 체크 -> 함수 실행
        balances[msg.sender] -= amount;
        balances[to] += amount;
    }
    
    function pause() public onlyOwner {
        paused = true;
    }
    
    function unpause() public onlyOwner {
        paused = false;
    }
}
```

### 실행 전후 로직

```solidity
// ReentrancyGuard 패턴 - 재진입 공격 방지
modifier nonReentrant() {
    require(!locked, "ReentrancyGuard: reentrant call");
    locked = true;
    _;              // 함수 실행
    locked = false; // 함수 실행 후
}
```

## 접근 제어가 있는 완전한 컨트랙트 예제

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SimpleVault - 접근 제어가 있는 금고 컨트랙트
contract SimpleVault {
    // ============ 상태 변수 ============
    address public owner;
    address public pendingOwner;
    bool public paused;
    bool private _locked;
    
    mapping(address => uint256) private _balances;
    mapping(address => bool) private _operators;
    
    // ============ 이벤트 ============
    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
    event OwnershipTransferProposed(address indexed newOwner);
    event OwnershipTransferred(address indexed oldOwner, address indexed newOwner);
    event OperatorSet(address indexed operator, bool status);
    event Paused(address indexed by);
    event Unpaused(address indexed by);
    
    // ============ 제어자 ============
    modifier onlyOwner() {
        require(msg.sender == owner, "Vault: not owner");
        _;
    }
    
    modifier onlyOperatorOrOwner() {
        require(
            msg.sender == owner || _operators[msg.sender],
            "Vault: not authorized"
        );
        _;
    }
    
    modifier whenNotPaused() {
        require(!paused, "Vault: paused");
        _;
    }
    
    modifier nonReentrant() {
        require(!_locked, "Vault: reentrant call");
        _locked = true;
        _;
        _locked = false;
    }
    
    modifier validAmount(uint256 amount) {
        require(amount > 0, "Vault: amount must be positive");
        _;
    }
    
    // ============ 생성자 ============
    constructor() {
        owner = msg.sender;
        emit OwnershipTransferred(address(0), msg.sender);
    }
    
    // ============ ETH 수신 ============
    receive() external payable {
        _deposit(msg.sender, msg.value);
    }
    
    // ============ 외부 함수 ============
    
    /// @notice ETH를 금고에 예치
    function deposit() external payable whenNotPaused validAmount(msg.value) {
        _deposit(msg.sender, msg.value);
    }
    
    /// @notice ETH를 출금
    function withdraw(uint256 amount)
        external
        whenNotPaused
        nonReentrant
        validAmount(amount)
    {
        require(_balances[msg.sender] >= amount, "Vault: insufficient balance");
        
        // Checks-Effects-Interactions 패턴
        _balances[msg.sender] -= amount;  // 상태 먼저 변경
        
        (bool success, ) = payable(msg.sender).call{value: amount}("");
        require(success, "Vault: transfer failed");
        
        emit Withdrawn(msg.sender, amount);
    }
    
    /// @notice 특정 사용자의 잔액 조회
    function balanceOf(address user) external view returns (uint256) {
        return _balances[user];
    }
    
    /// @notice 이 컨트랙트의 총 ETH 잔액
    function totalAssets() external view returns (uint256) {
        return address(this).balance;
    }
    
    // ============ 소유자 전용 함수 ============
    
    /// @notice 오퍼레이터 권한 설정
    function setOperator(address operator, bool status) external onlyOwner {
        require(operator != address(0), "Vault: zero address");
        _operators[operator] = status;
        emit OperatorSet(operator, status);
    }
    
    /// @notice 소유권 이전 제안 (2단계 이전)
    function proposeOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "Vault: zero address");
        pendingOwner = newOwner;
        emit OwnershipTransferProposed(newOwner);
    }
    
    /// @notice 소유권 이전 수락
    function acceptOwnership() external {
        require(msg.sender == pendingOwner, "Vault: not pending owner");
        address oldOwner = owner;
        owner = pendingOwner;
        pendingOwner = address(0);
        emit OwnershipTransferred(oldOwner, owner);
    }
    
    // ============ 오퍼레이터/소유자 함수 ============
    
    function pause() external onlyOperatorOrOwner {
        paused = true;
        emit Paused(msg.sender);
    }
    
    function unpause() external onlyOwner {
        paused = false;
        emit Unpaused(msg.sender);
    }
    
    // ============ 내부 함수 ============
    
    function _deposit(address user, uint256 amount) internal {
        _balances[user] += amount;
        emit Deposited(user, amount);
    }
    
    // ============ 조회 함수 ============
    
    function isOperator(address account) external view returns (bool) {
        return _operators[account];
    }
}
```

이 컨트랙트는 실무에서 자주 보이는 패턴들을 담고 있다:

1. **2단계 소유권 이전**: `proposeOwnership` → `acceptOwnership` — 실수로 잘못된 주소로 이전하는 사고를 방지
2. **Checks-Effects-Interactions 패턴**: 재진입 공격 방지
3. **nonReentrant 제어자**: 이중 인출 방지
4. **이벤트 발행**: 모든 중요한 상태 변경을 로그에 기록

## 정리

Solidity 함수는 TypeScript 함수와 문법은 유사하지만 독특한 개념들이 있다:

1. **가시성**은 외부 호환성과 가스 비용에 영향을 준다 — `external`이 `public`보다 저렴
2. **상태 변경성**은 함수가 블록체인과 어떻게 상호작용하는지 정의한다
3. **receive/fallback**은 컨트랙트가 ETH를 받거나 알 수 없는 호출을 처리하는 방법
4. **modifier**는 반복되는 검증 로직을 선언적으로 재사용하는 강력한 도구

다음 챕터에서는 mapping, 이벤트, 에러 처리를 자세히 다룬다.
