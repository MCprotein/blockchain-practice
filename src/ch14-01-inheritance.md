# Chapter 14-1: 상속과 인터페이스

## 컨트랙트 상속: is 키워드

Solidity에서 상속은 `is` 키워드를 사용한다. TypeScript의 `extends`와 동일한 역할이다.

```solidity
// TypeScript
class Animal {
    name: string;
    constructor(name: string) { this.name = name; }
    speak(): string { return "..."; }
}
class Dog extends Animal {
    speak(): string { return "Woof!"; }
}

// Solidity
contract Animal {
    string public name;
    constructor(string memory _name) { name = _name; }
    function speak() public virtual returns (string memory) { return "..."; }
}
contract Dog is Animal {
    constructor() Animal("Dog") {}
    function speak() public virtual override returns (string memory) { return "Woof!"; }
}
```

### 부모 함수 호출

```solidity
contract Base {
    function greet() public virtual returns (string memory) {
        return "Hello from Base";
    }
}

contract Child is Base {
    function greet() public virtual override returns (string memory) {
        // super로 부모 함수 호출
        string memory parentGreet = super.greet();
        return string(abi.encodePacked(parentGreet, " and Child"));
    }
}
```

### 부모 생성자 호출

```solidity
contract Ownable {
    address public owner;
    constructor(address _owner) {
        owner = _owner;
    }
}

contract Pausable {
    bool public paused;
    constructor() {
        paused = false;
    }
}

// 여러 부모의 생성자를 모두 호출
contract MyContract is Ownable, Pausable {
    // 방법 1: 선언부에서
    constructor() Ownable(msg.sender) Pausable() {
        // 추가 초기화
    }
}
```

## 다중 상속과 C3 선형화

Solidity는 **다중 상속**을 지원한다. 즉 하나의 컨트랙트가 여러 부모를 가질 수 있다.

```solidity
contract A {
    function foo() public virtual returns (string memory) { return "A"; }
}

contract B is A {
    function foo() public virtual override returns (string memory) { return "B"; }
}

contract C is A {
    function foo() public virtual override returns (string memory) { return "C"; }
}

// B와 C 모두 상속 — 다이아몬드 문제
contract D is B, C {
    // D에서 foo()를 override하지 않으면 컴파일 에러
    // 어느 부모의 foo()를 써야 할지 모호하기 때문
    function foo() public override(B, C) returns (string memory) {
        return super.foo(); // C3 선형화 순서에 따라 C의 foo() 호출
    }
}
```

### C3 선형화 규칙

Solidity는 Python의 C3 선형화 알고리즘을 사용한다. `is` 뒤에 나열하는 순서가 **가장 기본(base)에서 가장 파생(derived)** 순서여야 한다.

```solidity
// 상속 순서: 가장 기본(base) → 가장 파생(derived) 순으로
contract D is A, B, C { ... }
//              ↑ 가장 기본   ↑ 가장 파생

// super.foo() 호출 시 MRO(Method Resolution Order):
// D → C → B → A 순으로 탐색
```

실용적 규칙: `super.foo()`는 상속 목록의 **오른쪽→왼쪽** 순서로 호출된다.

```solidity
contract A { function foo() public virtual returns (string memory) { return "A"; } }
contract B is A { function foo() public virtual override returns (string memory) {
    return string(abi.encodePacked(super.foo(), "B")); // "AB"
}}
contract C is A { function foo() public virtual override returns (string memory) {
    return string(abi.encodePacked(super.foo(), "C")); // "AC"
}}
contract D is B, C { function foo() public override(B, C) returns (string memory) {
    return super.foo(); // "ACB" — C먼저, 그다음 B
}}
```

`D is B, C`에서 MRO: D → C → B → A
따라서 `super.foo()`는 C.foo() → B.foo() → A.foo() 순으로 체인된다.

## 추상 컨트랙트 (Abstract Contract)

구현이 없는 함수가 하나 이상 있는 컨트랙트는 `abstract`로 선언해야 한다. 직접 배포할 수 없고 상속해서만 사용한다.

```solidity
// TypeScript abstract class와 동일
abstract class Shape {
    abstract area(): number;  // 구현 없음
    perimeter(): number { return 0; } // 구현 있음
}
```

```solidity
// Solidity abstract contract
abstract contract Shape {
    // 구현 없는 함수 (자식이 반드시 구현해야)
    function area() public virtual returns (uint256);

    // 구현 있는 함수 (자식이 override 선택 가능)
    function describe() public virtual returns (string memory) {
        return "I am a shape";
    }
}

contract Circle is Shape {
    uint256 public radius;
    constructor(uint256 _radius) { radius = _radius; }

    // 반드시 구현해야 함
    function area() public view override returns (uint256) {
        // π * r^2 (정수 근사: 3141592 * r^2 / 1000000)
        return (3141592 * radius * radius) / 1000000;
    }
}

contract Square is Shape {
    uint256 public side;
    constructor(uint256 _side) { side = _side; }

    function area() public view override returns (uint256) {
        return side * side;
    }
}
```

## 인터페이스 (Interface)

인터페이스는 함수 선언만 있고 구현이 전혀 없다. 상태 변수, 생성자, 구현된 함수를 가질 수 없다.

```solidity
// TypeScript interface와 거의 동일
interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
    // 이벤트도 포함 가능
    event Transfer(address indexed from, address indexed to, uint256 value);
}
```

```typescript
// TypeScript
interface IERC20 {
    transfer(to: string, amount: bigint): Promise<boolean>;
    balanceOf(account: string): Promise<bigint>;
}
```

**인터페이스의 제약:**
- 모든 함수는 `external`
- 상태 변수 없음
- 생성자 없음
- 구현된 함수 없음 (Solidity 0.6+ 이전에는 허용됨)
- 다른 인터페이스를 상속할 수 있음

### 인터페이스를 통한 외부 컨트랙트 호출

인터페이스의 핵심 사용법은 **주소를 특정 타입으로 캐스팅**하는 것이다.

```solidity
interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
    function approve(address spender, uint256 amount) external returns (bool);
}

contract TokenSwapper {
    // 임의의 ERC-20 토큰과 상호작용
    function getBalance(address tokenAddress, address account)
        external view returns (uint256)
    {
        // 주소를 IERC20으로 캐스팅
        IERC20 token = IERC20(tokenAddress);
        return token.balanceOf(account);
    }

    function swapTokens(
        address fromToken,
        address toToken,
        uint256 amount
    ) external {
        IERC20(fromToken).transferFrom(msg.sender, address(this), amount);
        // ... 스왑 로직 ...
        IERC20(toToken).transfer(msg.sender, amount);
    }
}
```

**TypeScript 비유:**

```typescript
// TypeScript - interface로 타입 캐스팅
interface IERC20 {
    transfer(to: string, amount: bigint): Promise<boolean>;
    balanceOf(account: string): Promise<bigint>;
}

function getBalance(tokenAddress: string, account: string): Promise<bigint> {
    const token = new ethers.Contract(tokenAddress, ERC20_ABI, provider) as unknown as IERC20;
    return token.balanceOf(account);
}
```

## virtual과 override 키워드

```
virtual  = "자식이 이 함수를 재정의할 수 있다"
override = "이 함수는 부모의 virtual 함수를 재정의한다"
```

```solidity
contract Base {
    // virtual: 자식이 override 가능
    function canOverride() public virtual returns (string memory) {
        return "Base";
    }

    // virtual 없음: 자식이 override 불가
    function cannotOverride() public returns (string memory) {
        return "Fixed";
    }
}

contract Child is Base {
    // override: 부모의 virtual 함수를 재정의
    function canOverride() public virtual override returns (string memory) {
        return "Child";
    }

    // 이건 컴파일 에러:
    // function cannotOverride() public override returns (string memory) { ... }
}

contract GrandChild is Child {
    // Child도 virtual로 선언했으므로 또 override 가능
    function canOverride() public override returns (string memory) {
        return "GrandChild";
    }
}
```

### 다중 상속 시 override

```solidity
contract A {
    function foo() public virtual returns (uint256) { return 1; }
}
contract B is A {
    function foo() public virtual override returns (uint256) { return 2; }
}
contract C is A {
    function foo() public virtual override returns (uint256) { return 3; }
}

contract D is B, C {
    // 여러 부모의 override 목록 명시
    function foo() public override(B, C) returns (uint256) {
        return super.foo(); // C의 foo() (MRO 순서)
    }
}
```

## 추상 컨트랙트 vs 인터페이스 비교

| | 추상 컨트랙트 | 인터페이스 |
|--|-------------|----------|
| 구현된 함수 | 가능 | 불가 |
| 상태 변수 | 가능 | 불가 |
| 생성자 | 가능 | 불가 |
| 이벤트 | 가능 | 가능 |
| 다중 상속 | 가능 | 가능 |
| 사용 목적 | 공통 로직 공유 | 타입 정의/계약 |

**언제 인터페이스를 쓰고 언제 추상 컨트랙트를 쓰나:**

```solidity
// 인터페이스: 외부 컨트랙트와의 상호작용 표준 정의
interface IUniswapV2Router {
    function swapExactTokensForETH(...) external returns (uint[] memory);
}

// 추상 컨트랙트: 공통 로직을 가진 베이스 컨트랙트
abstract contract BaseVault {
    // 공통 상태 변수
    address public asset;
    mapping(address => uint256) public balances;

    // 공통 구현
    function totalAssets() public view returns (uint256) {
        return IERC20(asset).balanceOf(address(this));
    }

    // 자식이 구현해야 할 함수
    function _deposit(address user, uint256 amount) internal virtual;
    function _withdraw(address user, uint256 amount) internal virtual;
}

contract ETHVault is BaseVault {
    function _deposit(address user, uint256 amount) internal override {
        // ETH 전용 예치 로직
        balances[user] += amount;
    }

    function _withdraw(address user, uint256 amount) internal override {
        // ETH 전용 출금 로직
        balances[user] -= amount;
        payable(user).transfer(amount);
    }
}
```

## TypeScript extends/implements와의 비교

```typescript
// TypeScript
interface IShape {
    area(): number;
}

abstract class BaseShape implements IShape {
    abstract area(): number;
    describe(): string { return "I am a shape"; }
}

class Circle extends BaseShape {
    constructor(private radius: number) { super(); }
    area(): number { return Math.PI * this.radius ** 2; }
}
```

```solidity
// Solidity
interface IShape {
    function area() external returns (uint256);
}

abstract contract BaseShape is IShape {
    // area()는 IShape에서 선언됐으므로 자동으로 virtual처럼 동작
    function describe() public virtual returns (string memory) {
        return "I am a shape";
    }
}

contract Circle is BaseShape {
    uint256 public radius;
    constructor(uint256 _radius) { radius = _radius; }

    function area() public view override returns (uint256) {
        return (3141592 * radius * radius) / 1000000;
    }
}
```

**주요 차이점:**

| TypeScript | Solidity |
|-----------|---------|
| `class ... extends ... implements ...` | `contract ... is ..., ...` |
| `extends`(상속)와 `implements`(인터페이스) 구분 | `is` 키워드로 통일 |
| `abstract` 메서드 = 구현 없음 | `virtual` + 구현 없음 = 추상 함수 |
| `super.method()` | `super.method()` 또는 `ContractName.method()` |
| 다중 인터페이스 구현 가능 | 다중 상속 가능 |
| MRO 없음 (단일 상속) | C3 선형화 MRO |

## 실전 상속 패턴

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// 1. 인터페이스: 외부 계약
interface IVault {
    function deposit(uint256 amount) external;
    function withdraw(uint256 amount) external;
    function balanceOf(address user) external view returns (uint256);
    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
}

// 2. 추상 베이스: 공통 로직
abstract contract BaseVault is IVault {
    mapping(address => uint256) internal _balances;
    uint256 internal _totalDeposited;

    // 공통 구현 (모든 Vault에서 동일)
    function balanceOf(address user) public view override returns (uint256) {
        return _balances[user];
    }

    function totalDeposited() public view returns (uint256) {
        return _totalDeposited;
    }

    // 자식이 구현해야 할 훅
    function _beforeDeposit(address user, uint256 amount) internal virtual {}
    function _afterDeposit(address user, uint256 amount) internal virtual {}
    function _beforeWithdraw(address user, uint256 amount) internal virtual {}
    function _afterWithdraw(address user, uint256 amount) internal virtual {}

    // 공통 deposit 로직 (훅 포함)
    function deposit(uint256 amount) public virtual override {
        _beforeDeposit(msg.sender, amount);
        _balances[msg.sender] += amount;
        _totalDeposited += amount;
        _afterDeposit(msg.sender, amount);
        emit IVault.Deposited(msg.sender, amount);
    }

    function withdraw(uint256 amount) public virtual override {
        require(_balances[msg.sender] >= amount, "Insufficient");
        _beforeWithdraw(msg.sender, amount);
        _balances[msg.sender] -= amount;
        _totalDeposited -= amount;
        _afterWithdraw(msg.sender, amount);
        emit IVault.Withdrawn(msg.sender, amount);
    }
}

// 3. 구체 구현: 특정 기능 추가
contract FeeVault is BaseVault {
    uint256 public feeRate = 100; // 1% (basis points)
    address public feeRecipient;
    uint256 public totalFees;

    constructor(address _feeRecipient) {
        feeRecipient = _feeRecipient;
    }

    // 출금 시 수수료 차감
    function _beforeWithdraw(address user, uint256 amount) internal override {
        uint256 fee = (amount * feeRate) / 10000;
        _balances[feeRecipient] += fee;
        totalFees += fee;
    }
}
```

## 정리

Solidity 상속의 핵심:

1. **`is` 키워드** — 상속과 인터페이스 구현 모두 동일한 키워드
2. **`virtual`/`override`** — 재정의 가능 여부를 명시적으로 선언
3. **C3 선형화** — 다중 상속 시 메서드 해석 순서, `is` 목록의 오른쪽 → 왼쪽
4. **추상 컨트랙트** — 공통 로직 + 미구현 함수를 가진 베이스
5. **인터페이스** — 순수 타입 정의, 외부 컨트랙트와의 상호작용에 필수

다음 챕터에서는 스마트 컨트랙트 업그레이드를 위한 프록시 패턴을 배운다.
