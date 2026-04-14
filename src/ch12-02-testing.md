# Chapter 12-2: Foundry 테스트 작성

## Solidity로 테스트를 작성한다

Foundry의 가장 독특한 점은 **테스트를 Solidity로 작성한다**는 것이다. JavaScript 테스트 프레임워크(Jest, Mocha)에 익숙한 Node.js 개발자에게는 처음에 어색하지만, 익숙해지면 타입 안전성과 속도 면에서 큰 이점이 있다.

```typescript
// Hardhat / Jest 스타일 (TypeScript)
describe("Counter", function () {
    let counter: Counter;

    beforeEach(async function () {
        const Counter = await ethers.getContractFactory("Counter");
        counter = await Counter.deploy();
    });

    it("should increment", async function () {
        await counter.increment();
        expect(await counter.number()).to.equal(1);
    });
});
```

```solidity
// Foundry 스타일 (Solidity)
contract CounterTest is Test {
    Counter counter;

    function setUp() public {
        counter = new Counter();
    }

    function test_Increment() public {
        counter.increment();
        assertEq(counter.number(), 1);
    }
}
```

## Test 컨트랙트 구조

### 기본 구조

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// forge-std의 Test를 상속 (어서션, 치트코드 포함)
import {Test, console} from "forge-std/Test.sol";
import {Counter} from "../src/Counter.sol";

contract CounterTest is Test {
    // 테스트에 사용할 컨트랙트 및 변수
    Counter public counter;
    address public alice = makeAddr("alice");  // 테스트용 주소 생성
    address public bob = makeAddr("bob");

    // setUp: 각 테스트 함수 실행 전에 호출 (beforeEach)
    function setUp() public {
        counter = new Counter();
    }

    // test_로 시작하는 함수 = 일반 테스트
    function test_InitialValue() public view {
        assertEq(counter.number(), 0);
    }

    function test_Increment() public {
        counter.increment();
        assertEq(counter.number(), 1);
    }

    function test_SetNumber() public {
        counter.setNumber(42);
        assertEq(counter.number(), 42);
    }

    // testFuzz_로 시작하는 함수 = 퍼즈 테스트
    function testFuzz_SetNumber(uint256 x) public {
        counter.setNumber(x);
        assertEq(counter.number(), x);
    }
}
```

**NestJS 테스트와 대응:**

| Foundry | Jest/NestJS | 역할 |
|---------|------------|------|
| `setUp()` | `beforeEach()` | 각 테스트 전 초기화 |
| `test_XXX()` | `it('should ...')` | 단위 테스트 |
| `testFuzz_XXX()` | 없음 (별도 도구) | 퍼즈 테스트 |
| `assertEq(a, b)` | `expect(a).toBe(b)` | 동등 어서션 |
| `console.log()` | `console.log()` | 디버그 출력 |
| `forge-std/Test.sol` | `@nestjs/testing` | 테스트 프레임워크 |

### 테스트 파일 명명 규칙

```text
test/
├── Counter.t.sol          # 단위 테스트 (.t.sol 확장자)
├── Vault.t.sol
└── integration/
    └── VaultFlow.t.sol    # 통합 테스트
```

## 어서션 (Assertions)

`forge-std`의 `Test` 컨트랙트가 제공하는 주요 어서션들:

### 동등 비교

```solidity
// assertEq(actual, expected) - 두 값이 같은지
assertEq(counter.number(), 42);
assertEq(token.balanceOf(alice), 1000 * 1e18);
assertEq(contract.owner(), alice);

// 지원 타입: uint256, int256, address, bytes32, string, bytes, bool
assertEq(token.name(), "MyToken");
assertEq(token.symbol(), "MTK");

// 오류 메시지 포함
assertEq(counter.number(), 42, "Counter should be 42");
```

### 부등 비교

```solidity
assertNotEq(a, b);              // a != b
assertGt(a, b);                 // a > b  (greater than)
assertGe(a, b);                 // a >= b (greater or equal)
assertLt(a, b);                 // a < b  (less than)
assertLe(a, b);                 // a <= b (less or equal)

// 예시
assertGt(token.balanceOf(alice), 0, "Alice should have tokens");
assertLe(fee, maxFee, "Fee should not exceed maximum");
```

### 불리언 어서션

```solidity
assertTrue(condition);
assertFalse(condition);

// 예시
assertTrue(vault.isActive(), "Vault should be active");
assertFalse(token.paused(), "Token should not be paused");
```

### 근사값 비교 (부동소수점 대신)

```solidity
// assertApproxEqAbs(actual, expected, maxDelta) - 절대 오차
assertApproxEqAbs(actual, expected, 1e15);  // 0.001 ETH 오차 허용

// assertApproxEqRel(actual, expected, maxPercentDelta) - 상대 오차 (1e18 = 100%)
assertApproxEqRel(actual, expected, 1e16);  // 1% 오차 허용
```

### 배열 어서션

```solidity
uint256[] memory expected = new uint256[](3);
expected[0] = 1;
expected[1] = 2;
expected[2] = 3;

assertEq(actual, expected);  // 배열 전체 비교
```

## 치트코드 (Cheatcodes)

치트코드는 테스트 환경에서만 사용할 수 있는 특수 함수들이다. `vm` 객체를 통해 접근한다. 블록체인 상태를 자유롭게 조작해 다양한 시나리오를 테스트할 수 있다.

### vm.prank() — 다른 주소로 위장

```solidity
// 다음 트랜잭션 한 번만 다른 주소로 실행
vm.prank(alice);
token.transfer(bob, 100);
// 이 시점부터는 다시 원래 주소

// 여러 트랜잭션을 같은 주소로 실행
vm.startPrank(alice);
token.approve(vault, 1000);
vault.deposit(500);
vm.stopPrank();
```

**NestJS 비유:** JWT 토큰을 변경해서 다른 사용자로 API를 호출하는 것. 실제 개인키 없이도 어떤 주소로든 트랜잭션을 보낼 수 있다.

```typescript
// NestJS 테스트에서 다른 사용자로 요청
const response = await request(app.getHttpServer())
    .get('/profile')
    .set('Authorization', `Bearer ${aliceToken}`);
```

```solidity
// Foundry에서 다른 주소로 호출
vm.prank(alice);
contract.doSomething();
```

### vm.expectRevert() — 에러 검증

```solidity
// 다음 호출이 revert될 것을 예상
vm.expectRevert("ERC20: insufficient balance");
token.transfer(bob, 999999 ether);

// 커스텀 에러 검증
vm.expectRevert(
    abi.encodeWithSelector(InsufficientBalance.selector, alice, 0, 100)
);
token.transfer(bob, 100);

// 에러 타입만 검증 (파라미터 무시)
vm.expectRevert(InsufficientBalance.selector);
token.transfer(bob, 100);

// 빈 revert
vm.expectRevert();
maliciousCall();
```

**Jest 비유:**

```typescript
// Jest
expect(() => service.transfer(bob, 999999)).toThrow('insufficient balance');
```

### vm.deal() — ETH 잔액 설정

```solidity
// alice에게 10 ETH 부여
vm.deal(alice, 10 ether);

// 컨트랙트에 ETH 부여
vm.deal(address(vault), 100 ether);

// 잔액 확인
assertEq(alice.balance, 10 ether);
```

### vm.warp() — 시간 조작

```solidity
// 현재 블록 타임스탬프 변경
vm.warp(block.timestamp + 1 days);
vm.warp(block.timestamp + 365 days);

// 특정 시점으로 이동
vm.warp(1700000000); // Unix 타임스탬프

// 블록 번호 변경
vm.roll(block.number + 100);
```

**사용 예시 - 타임락 테스트:**

```solidity
function test_TimelockExpiry() public {
    // 출금 요청
    vault.requestWithdrawal(1 ether);

    // 타임락 전에는 불가
    vm.expectRevert("Timelock not expired");
    vault.executeWithdrawal();

    // 24시간 후
    vm.warp(block.timestamp + 1 days);

    // 이제 가능
    vault.executeWithdrawal();
    assertEq(alice.balance, 1 ether);
}
```

### vm.mockCall() — 외부 호출 모킹

```solidity
// 특정 주소의 특정 함수 호출을 모킹
address mockOracle = address(0x1234);
vm.mockCall(
    mockOracle,
    abi.encodeWithSelector(IOracle.getPrice.selector, address(token)),
    abi.encode(2000 * 1e8)  // $2000 반환
);

// 이후 mockOracle.getPrice(token) 호출은 2000 * 1e8 반환
uint256 price = IOracle(mockOracle).getPrice(address(token));
assertEq(price, 2000 * 1e8);
```

### vm.expectEmit() — 이벤트 검증

```solidity
// 이벤트가 발생할 것을 예상
// expectEmit(checkTopic1, checkTopic2, checkTopic3, checkData)
vm.expectEmit(true, true, false, true);
emit Transfer(alice, bob, 100);  // 예상하는 이벤트

// 실제 호출 (이 호출에서 위 이벤트가 발생해야 함)
token.transfer(bob, 100);
```

### vm.label() — 주소에 이름 붙이기

```solidity
vm.label(alice, "Alice");
vm.label(address(token), "MyToken");
// 테스트 실패 시 주소 대신 이름으로 표시
```

### 기타 유용한 치트코드

```solidity
// 환경 변수 읽기
string memory key = vm.envString("PRIVATE_KEY");

// storage 직접 읽기/쓰기 (private 변수도 가능!)
bytes32 value = vm.load(address(contract), bytes32(0));  // slot 0 읽기
vm.store(address(contract), bytes32(0), bytes32(uint256(42)));  // 값 쓰기

// 특정 블록에서 포크 (메인넷 상태 복제)
vm.createFork("mainnet", 18000000);

// 가스 측정
uint256 gasBefore = gasleft();
contract.expensiveFunction();
uint256 gasUsed = gasBefore - gasleft();
```

## 퍼즈 테스트 (Fuzz Testing)

퍼즈 테스트는 Foundry가 자동으로 다양한 입력값을 생성해 테스트하는 기능이다. 개발자가 미처 생각하지 못한 엣지 케이스를 자동으로 찾아준다.

### 기본 퍼즈 테스트

```solidity
// testFuzz_로 시작하는 함수가 퍼즈 테스트
function testFuzz_Transfer(
    address to,
    uint256 amount
) public {
    // Foundry가 to와 amount에 다양한 값을 자동으로 넣어서 실행
    vm.assume(to != address(0));  // 제약 조건
    vm.assume(amount <= 1000 ether);

    deal(address(token), alice, amount);

    vm.prank(alice);
    token.transfer(to, amount);

    assertEq(token.balanceOf(to), amount);
}
```

`foundry.toml`의 `fuzz.runs = 256`에 따라 256가지 다른 입력 조합으로 테스트한다.

### vm.assume() — 입력값 제약

```solidity
function testFuzz_Deposit(uint256 amount) public {
    // 조건이 false인 입력은 건너뜀 (해당 실행을 카운트하지 않음)
    vm.assume(amount > 0);
    vm.assume(amount <= type(uint128).max);  // 오버플로 방지

    vm.deal(alice, amount);
    vm.prank(alice);
    vault.deposit{value: amount}();

    assertEq(vault.balanceOf(alice), amount);
}
```

### bound() — 범위로 입력값 제한

`vm.assume()`은 조건을 만족하지 못하면 해당 실행을 건너뛴다. `bound()`는 입력값을 범위 내로 조정해 거부율을 줄인다:

```solidity
function testFuzz_PartialWithdraw(uint256 depositAmount, uint256 withdrawAmount) public {
    // 범위 내로 조정 (건너뛰지 않고 값을 변환)
    depositAmount = bound(depositAmount, 1, 100 ether);
    withdrawAmount = bound(withdrawAmount, 1, depositAmount);

    vm.deal(alice, depositAmount);
    vm.startPrank(alice);
    vault.deposit{value: depositAmount}();
    vault.withdraw(withdrawAmount);
    vm.stopPrank();

    assertEq(vault.balanceOf(alice), depositAmount - withdrawAmount);
}
```

## forge test 출력 읽는 법

### 기본 출력

```bash
forge test
```

```text
Running 5 tests for test/Token.t.sol:TokenTest
[PASS] test_InitialSupply() (gas: 12345)
[PASS] test_Transfer() (gas: 45678)
[FAIL. Counterexample: calldata=0x... args=[0x0000...0000]] testFuzz_Transfer(address,uint256)
[PASS] test_Approval() (gas: 23456)
[PASS] testFuzz_Mint(uint256) (runs: 256, μ: 34567, ~: 34567)

Test result: FAILED. 4 passed; 1 failed; finished in 123.45ms
```

- `gas: 12345` — 해당 테스트의 가스 사용량
- `runs: 256` — 퍼즈 테스트 실행 횟수
- `μ: 34567` — 가스 사용량 평균
- `~: 34567` — 가스 사용량 중앙값

### -v 플래그로 상세 출력

```bash
forge test -v      # 실패한 테스트의 로그
forge test -vv     # 모든 테스트의 로그
forge test -vvv    # 스택 트레이스 포함
forge test -vvvv   # 전체 콜 트레이스
```

```bash
forge test -vvv
```

```text
[FAIL. Counterexample: calldata=... args=[0]]
    testFuzz_Transfer(uint256)
    
    Traces:
      [45678] TokenTest::testFuzz_Transfer(0)
        ├─ [0] VM::assume(false)  <-- vm.assume이 실패하면 아닌데...
        ├─ [12345] Token::transfer(0xalice, 0)
        │   └─ ← revert: "Amount must be positive"
        └─ ← [Revert]
    
    Error: Amount must be positive
```

스택 트레이스를 통해 어떤 순서로 함수가 호출되고 어디서 실패했는지 파악할 수 있다.

### 특정 테스트만 실행

```bash
# 테스트 함수명으로 필터링
forge test --match-test test_Transfer

# 파일명으로 필터링
forge test --match-path test/Token.t.sol

# 컨트랙트명으로 필터링
forge test --match-contract TokenTest

# 가스 리포트 출력
forge test --gas-report
```

## 전체 테스트 예제

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";

// 테스트 대상 컨트랙트
contract SimpleToken {
    error InsufficientBalance(address from, uint256 available, uint256 required);
    error InvalidRecipient();

    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Mint(address indexed to, uint256 amount);

    string public name;
    string public symbol;
    uint256 public totalSupply;
    address public owner;
    mapping(address => uint256) public balanceOf;

    constructor(string memory _name, string memory _symbol, uint256 initialSupply) {
        name = _name;
        symbol = _symbol;
        owner = msg.sender;
        _mint(msg.sender, initialSupply);
    }

    function transfer(address to, uint256 amount) external returns (bool) {
        if (to == address(0)) revert InvalidRecipient();
        if (balanceOf[msg.sender] < amount) {
            revert InsufficientBalance(msg.sender, balanceOf[msg.sender], amount);
        }
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        emit Transfer(msg.sender, to, amount);
        return true;
    }

    function mint(address to, uint256 amount) external {
        require(msg.sender == owner, "Not owner");
        _mint(to, amount);
    }

    function _mint(address to, uint256 amount) internal {
        totalSupply += amount;
        balanceOf[to] += amount;
        emit Mint(to, amount);
    }
}

// 테스트 컨트랙트
contract SimpleTokenTest is Test {
    SimpleToken public token;

    address public owner = makeAddr("owner");
    address public alice = makeAddr("alice");
    address public bob = makeAddr("bob");

    uint256 public constant INITIAL_SUPPLY = 1_000_000 * 1e18;

    // 각 테스트 전 실행
    function setUp() public {
        vm.prank(owner);
        token = new SimpleToken("SimpleToken", "STK", INITIAL_SUPPLY);
    }

    // ============ 초기 상태 테스트 ============

    function test_InitialState() public view {
        assertEq(token.name(), "SimpleToken");
        assertEq(token.symbol(), "STK");
        assertEq(token.totalSupply(), INITIAL_SUPPLY);
        assertEq(token.owner(), owner);
        assertEq(token.balanceOf(owner), INITIAL_SUPPLY);
    }

    // ============ 전송 테스트 ============

    function test_Transfer() public {
        uint256 amount = 100 * 1e18;

        // owner -> alice 전송
        vm.prank(owner);
        token.transfer(alice, amount);

        assertEq(token.balanceOf(owner), INITIAL_SUPPLY - amount);
        assertEq(token.balanceOf(alice), amount);
        assertEq(token.totalSupply(), INITIAL_SUPPLY); // 총 공급량 불변
    }

    function test_Transfer_EmitsEvent() public {
        uint256 amount = 100 * 1e18;

        // 이벤트 검증
        vm.expectEmit(true, true, false, true);
        emit SimpleToken.Transfer(owner, alice, amount);

        vm.prank(owner);
        token.transfer(alice, amount);
    }

    function test_Transfer_RevertOnInsufficientBalance() public {
        uint256 amount = INITIAL_SUPPLY + 1;

        vm.expectRevert(
            abi.encodeWithSelector(
                SimpleToken.InsufficientBalance.selector,
                owner,
                INITIAL_SUPPLY,
                amount
            )
        );
        vm.prank(owner);
        token.transfer(alice, amount);
    }

    function test_Transfer_RevertOnZeroAddress() public {
        vm.expectRevert(SimpleToken.InvalidRecipient.selector);
        vm.prank(owner);
        token.transfer(address(0), 100);
    }

    // ============ 민팅 테스트 ============

    function test_Mint_OnlyOwner() public {
        uint256 mintAmount = 500 * 1e18;

        vm.prank(owner);
        token.mint(alice, mintAmount);

        assertEq(token.balanceOf(alice), mintAmount);
        assertEq(token.totalSupply(), INITIAL_SUPPLY + mintAmount);
    }

    function test_Mint_RevertIfNotOwner() public {
        vm.expectRevert("Not owner");
        vm.prank(alice);
        token.mint(alice, 100);
    }

    // ============ 퍼즈 테스트 ============

    function testFuzz_Transfer(address to, uint256 amount) public {
        vm.assume(to != address(0));
        vm.assume(to != owner);  // 같은 주소면 잔액 계산이 복잡해짐
        amount = bound(amount, 1, INITIAL_SUPPLY);

        vm.prank(owner);
        token.transfer(to, amount);

        assertEq(token.balanceOf(to), amount);
        assertEq(token.balanceOf(owner), INITIAL_SUPPLY - amount);
        // 불변식: 총 공급량은 절대 변하지 않음
        assertEq(token.totalSupply(), INITIAL_SUPPLY);
    }

    function testFuzz_Mint(address to, uint256 amount) public {
        vm.assume(to != address(0));
        amount = bound(amount, 1, type(uint128).max);  // 오버플로 방지

        uint256 supplyBefore = token.totalSupply();

        vm.prank(owner);
        token.mint(to, amount);

        assertEq(token.totalSupply(), supplyBefore + amount);
        assertEq(token.balanceOf(to), amount);
    }

    // ============ 디버깅 ============

    function test_Debug() public view {
        // console.log는 forge test -vv 이상에서 출력
        console.log("Owner:", owner);
        console.log("Initial supply:", INITIAL_SUPPLY);
        console.log("Balance:", token.balanceOf(owner));

        // console.logBytes32, console.logAddress 등도 사용 가능
    }
}
```

## 테스트 실행

```bash
# 전체 테스트
forge test

# 상세 출력
forge test -vvv

# 가스 리포트
forge test --gas-report

# 커버리지 확인
forge coverage

# 특정 테스트만
forge test --match-test test_Transfer -vvv
```

커버리지 출력 예시:
```text
| File                | % Lines  | % Statements | % Branches | % Funcs  |
|---------------------|----------|--------------|------------|----------|
| src/SimpleToken.sol | 100.00%  | 100.00%      | 87.50%     | 100.00%  |
```

다음 챕터에서는 배포 스크립트와 실제 배포 과정을 다룬다.
