# Chapter 15: 미니프로젝트 — ERC-20 토큰 + Vault

## 프로젝트 개요

지금까지 배운 내용을 종합해 실제로 동작하는 미니 프로젝트를 완성한다. 이 프로젝트는 두 개의 컨트랙트로 구성된다:

1. **MyToken.sol** — ERC-20 토큰 (OpenZeppelin 기반)
2. **Vault.sol** — 토큰을 예치하고 출금하는 금고 컨트랙트

### 전체 요구사항

**MyToken:**
- ERC-20 표준 준수 (OpenZeppelin ERC20 상속)
- 최대 공급량 제한 (100만 토큰)
- 소유자만 민팅 가능
- 토큰 소각(burn) 기능

**Vault:**
- MyToken만 예치 가능
- 사용자별 예치 잔액 관리
- 예치(deposit)와 출금(withdraw) 기능
- 재진입 공격 방지 (ReentrancyGuard)
- 긴급 정지 기능 (Pausable)
- 모든 주요 동작 이벤트 발행

### 프로젝트 구조

```
token-vault/
├── foundry.toml
├── remappings.txt
├── .env.example
│
├── src/
│   ├── MyToken.sol
│   └── Vault.sol
│
├── test/
│   ├── MyToken.t.sol
│   └── Vault.t.sol
│
└── script/
    └── Deploy.s.sol
```

## MyToken.sol 전체 코드

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC20Burnable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Ownable2Step} from "@openzeppelin/contracts/access/Ownable2Step.sol";

/// @title MyToken - ERC-20 토큰
/// @notice 최대 공급량이 제한된 ERC-20 토큰
contract MyToken is ERC20, ERC20Burnable, Ownable2Step {
    // ============ 에러 ============
    error ExceedsMaxSupply(uint256 requested, uint256 available);
    error ZeroAddress();
    error ZeroAmount();

    // ============ 상수 ============
    /// @notice 최대 발행 가능 토큰 수 (1,000,000 MTK)
    uint256 public constant MAX_SUPPLY = 1_000_000 * 10 ** 18;

    // ============ 이벤트 ============
    event TokensMinted(address indexed to, uint256 amount, uint256 newTotalSupply);

    // ============ 생성자 ============
    /// @param initialOwner 초기 소유자 주소
    /// @param initialSupply 초기 발행량 (MTK 단위, 18 decimals 적용)
    constructor(
        address initialOwner,
        uint256 initialSupply
    ) ERC20("MyToken", "MTK") Ownable(initialOwner) {
        if (initialOwner == address(0)) revert ZeroAddress();
        if (initialSupply > MAX_SUPPLY) {
            revert ExceedsMaxSupply(initialSupply, MAX_SUPPLY);
        }
        if (initialSupply > 0) {
            _mint(initialOwner, initialSupply);
            emit TokensMinted(initialOwner, initialSupply, totalSupply());
        }
    }

    // ============ 소유자 전용 함수 ============

    /// @notice 새 토큰 발행 (소유자만)
    /// @param to 수령 주소
    /// @param amount 발행 수량 (wei 단위)
    function mint(address to, uint256 amount) external onlyOwner {
        if (to == address(0)) revert ZeroAddress();
        if (amount == 0) revert ZeroAmount();

        uint256 available = MAX_SUPPLY - totalSupply();
        if (amount > available) {
            revert ExceedsMaxSupply(amount, available);
        }

        _mint(to, amount);
        emit TokensMinted(to, amount, totalSupply());
    }

    // ============ 조회 함수 ============

    /// @notice 추가로 발행 가능한 토큰 수
    function remainingMintable() external view returns (uint256) {
        return MAX_SUPPLY - totalSupply();
    }
}
```

### MyToken 설명

- **ERC20Burnable 상속**: `burn(amount)`, `burnFrom(account, amount)` 자동 제공
- **Ownable2Step**: 소유권 이전 시 새 소유자가 `acceptOwnership()` 호출해야 확정됨 (실수 방지)
- **MAX_SUPPLY**: 컴파일 타임 상수 — 가스 비용 없이 storage에서 읽지 않음
- **커스텀 에러**: 파라미터 포함으로 클라이언트에서 상세한 에러 처리 가능

## Vault.sol 전체 코드

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Ownable2Step} from "@openzeppelin/contracts/access/Ownable2Step.sol";

/// @title Vault - MyToken 금고 컨트랙트
/// @notice 사용자가 MyToken을 예치하고 출금할 수 있는 금고
contract Vault is ReentrancyGuard, Pausable, Ownable2Step {
    using SafeERC20 for IERC20;

    // ============ 에러 ============
    error ZeroAmount();
    error ZeroAddress();
    error InsufficientBalance(address user, uint256 available, uint256 requested);
    error ExceedsDepositLimit(uint256 amount, uint256 limit);
    error TotalDepositLimitReached(uint256 totalDeposits, uint256 limit);

    // ============ 상태 변수 ============

    /// @notice 예치할 수 있는 토큰 컨트랙트
    IERC20 public immutable token;

    /// @notice 사용자별 예치 잔액
    mapping(address => uint256) private _deposits;

    /// @notice 총 예치량
    uint256 public totalDeposits;

    /// @notice 사용자 1인당 최대 예치 한도
    uint256 public depositLimit;

    /// @notice 금고 전체 최대 예치 한도
    uint256 public totalDepositLimit;

    // ============ 이벤트 ============

    event Deposited(
        address indexed user,
        uint256 amount,
        uint256 newUserBalance,
        uint256 newTotalDeposits
    );

    event Withdrawn(
        address indexed user,
        uint256 amount,
        uint256 newUserBalance,
        uint256 newTotalDeposits
    );

    event DepositLimitUpdated(uint256 oldLimit, uint256 newLimit);
    event TotalDepositLimitUpdated(uint256 oldLimit, uint256 newLimit);
    event EmergencyWithdrawn(address indexed to, uint256 amount);

    // ============ 생성자 ============

    /// @param _token 예치 토큰 주소 (MyToken)
    /// @param initialOwner 초기 소유자
    /// @param _depositLimit 사용자 1인당 최대 예치량
    /// @param _totalDepositLimit 금고 전체 최대 예치량
    constructor(
        address _token,
        address initialOwner,
        uint256 _depositLimit,
        uint256 _totalDepositLimit
    ) Ownable(initialOwner) {
        if (_token == address(0)) revert ZeroAddress();
        if (initialOwner == address(0)) revert ZeroAddress();

        token = IERC20(_token);
        depositLimit = _depositLimit;
        totalDepositLimit = _totalDepositLimit;
    }

    // ============ 핵심 함수 ============

    /// @notice 토큰을 금고에 예치
    /// @param amount 예치할 토큰 수량 (wei 단위)
    /// @dev 사전에 Vault에 대한 approve 필요
    function deposit(uint256 amount)
        external
        nonReentrant
        whenNotPaused
    {
        // Checks
        if (amount == 0) revert ZeroAmount();

        uint256 newUserBalance = _deposits[msg.sender] + amount;
        if (newUserBalance > depositLimit) {
            revert ExceedsDepositLimit(amount, depositLimit);
        }

        uint256 newTotalDeposits = totalDeposits + amount;
        if (newTotalDeposits > totalDepositLimit) {
            revert TotalDepositLimitReached(totalDeposits, totalDepositLimit);
        }

        // Effects
        _deposits[msg.sender] = newUserBalance;
        totalDeposits = newTotalDeposits;

        // Interactions
        token.safeTransferFrom(msg.sender, address(this), amount);

        emit Deposited(msg.sender, amount, newUserBalance, newTotalDeposits);
    }

    /// @notice 예치한 토큰을 출금
    /// @param amount 출금할 토큰 수량 (wei 단위)
    function withdraw(uint256 amount)
        external
        nonReentrant
        whenNotPaused
    {
        // Checks
        if (amount == 0) revert ZeroAmount();

        uint256 currentBalance = _deposits[msg.sender];
        if (currentBalance < amount) {
            revert InsufficientBalance(msg.sender, currentBalance, amount);
        }

        // Effects
        uint256 newUserBalance = currentBalance - amount;
        _deposits[msg.sender] = newUserBalance;
        totalDeposits -= amount;

        // Interactions
        token.safeTransfer(msg.sender, amount);

        emit Withdrawn(msg.sender, amount, newUserBalance, totalDeposits);
    }

    // ============ 조회 함수 ============

    /// @notice 특정 사용자의 예치 잔액
    function balanceOf(address user) external view returns (uint256) {
        return _deposits[user];
    }

    /// @notice 금고에 실제로 있는 토큰 수
    function totalAssets() external view returns (uint256) {
        return token.balanceOf(address(this));
    }

    /// @notice 금고의 남은 예치 가능량 (전체 한도 기준)
    function remainingCapacity() external view returns (uint256) {
        return totalDepositLimit - totalDeposits;
    }

    /// @notice 특정 사용자의 남은 예치 가능량 (개인 한도 기준)
    function remainingUserCapacity(address user) external view returns (uint256) {
        uint256 used = _deposits[user];
        if (used >= depositLimit) return 0;
        return depositLimit - used;
    }

    // ============ 소유자 전용 함수 ============

    /// @notice 사용자 1인당 예치 한도 변경
    function setDepositLimit(uint256 newLimit) external onlyOwner {
        emit DepositLimitUpdated(depositLimit, newLimit);
        depositLimit = newLimit;
    }

    /// @notice 전체 예치 한도 변경
    function setTotalDepositLimit(uint256 newLimit) external onlyOwner {
        emit TotalDepositLimitUpdated(totalDepositLimit, newLimit);
        totalDepositLimit = newLimit;
    }

    /// @notice 금고 일시 정지
    function pause() external onlyOwner {
        _pause();
    }

    /// @notice 금고 재개
    function unpause() external onlyOwner {
        _unpause();
    }

    /// @notice 긴급 자금 회수 (일시 정지 상태에서만 가능)
    /// @param to 자금을 받을 주소
    function emergencyWithdraw(address to) external onlyOwner whenPaused {
        if (to == address(0)) revert ZeroAddress();
        uint256 balance = token.balanceOf(address(this));
        token.safeTransfer(to, balance);
        emit EmergencyWithdrawn(to, balance);
    }
}
```

### Vault 설계 포인트

1. **SafeERC20 사용**: `transfer` 반환값 false를 무시하지 않고 자동으로 revert
2. **CEI 패턴**: Checks → Effects(상태 변경) → Interactions(외부 호출)
3. **이중 한도 관리**: 개인 한도 + 전체 한도로 리스크 제어
4. **emergencyWithdraw**: 일시 정지 상태에서만 실행 가능 (정상 운영 중 오용 방지)
5. **immutable token**: 배포 후 변경 불가 — storage 접근보다 저렴

## 테스트 코드

### test/MyToken.t.sol

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {MyToken} from "../src/MyToken.sol";

contract MyTokenTest is Test {
    MyToken public token;

    address public owner = makeAddr("owner");
    address public alice = makeAddr("alice");
    address public bob = makeAddr("bob");

    uint256 public constant INITIAL_SUPPLY = 500_000 * 10 ** 18;

    function setUp() public {
        vm.prank(owner);
        token = new MyToken(owner, INITIAL_SUPPLY);
    }

    // ============ 초기 상태 ============

    function test_InitialState() public view {
        assertEq(token.name(), "MyToken");
        assertEq(token.symbol(), "MTK");
        assertEq(token.decimals(), 18);
        assertEq(token.totalSupply(), INITIAL_SUPPLY);
        assertEq(token.balanceOf(owner), INITIAL_SUPPLY);
        assertEq(token.owner(), owner);
        assertEq(token.MAX_SUPPLY(), 1_000_000 * 10 ** 18);
    }

    function test_RemainingMintable() public view {
        assertEq(token.remainingMintable(), 500_000 * 10 ** 18);
    }

    // ============ 민팅 ============

    function test_Mint() public {
        uint256 mintAmount = 100_000 * 10 ** 18;

        vm.prank(owner);
        token.mint(alice, mintAmount);

        assertEq(token.balanceOf(alice), mintAmount);
        assertEq(token.totalSupply(), INITIAL_SUPPLY + mintAmount);
    }

    function test_Mint_RevertIfNotOwner() public {
        vm.expectRevert(
            abi.encodeWithSignature("OwnableUnauthorizedAccount(address)", alice)
        );
        vm.prank(alice);
        token.mint(alice, 100);
    }

    function test_Mint_RevertIfExceedsMaxSupply() public {
        uint256 available = token.remainingMintable();

        vm.expectRevert(
            abi.encodeWithSelector(
                MyToken.ExceedsMaxSupply.selector,
                available + 1,
                available
            )
        );
        vm.prank(owner);
        token.mint(alice, available + 1);
    }

    function test_Mint_RevertZeroAddress() public {
        vm.expectRevert(MyToken.ZeroAddress.selector);
        vm.prank(owner);
        token.mint(address(0), 100);
    }

    function test_Mint_RevertZeroAmount() public {
        vm.expectRevert(MyToken.ZeroAmount.selector);
        vm.prank(owner);
        token.mint(alice, 0);
    }

    // ============ 소각 ============

    function test_Burn() public {
        uint256 burnAmount = 100 * 10 ** 18;

        vm.prank(owner);
        token.burn(burnAmount);

        assertEq(token.totalSupply(), INITIAL_SUPPLY - burnAmount);
        assertEq(token.balanceOf(owner), INITIAL_SUPPLY - burnAmount);
    }

    function test_BurnFrom() public {
        uint256 burnAmount = 100 * 10 ** 18;

        // alice가 owner에게 소각 권한 부여
        vm.prank(owner);
        token.transfer(alice, burnAmount);

        vm.prank(alice);
        token.approve(owner, burnAmount);

        vm.prank(owner);
        token.burnFrom(alice, burnAmount);

        assertEq(token.balanceOf(alice), 0);
        assertEq(token.totalSupply(), INITIAL_SUPPLY - burnAmount);
    }

    // ============ 소유권 이전 (2단계) ============

    function test_TransferOwnership() public {
        vm.prank(owner);
        token.transferOwnership(alice);

        // 아직 이전 완료 안 됨
        assertEq(token.owner(), owner);
        assertEq(token.pendingOwner(), alice);

        // alice가 수락
        vm.prank(alice);
        token.acceptOwnership();

        assertEq(token.owner(), alice);
    }

    // ============ 퍼즈 테스트 ============

    function testFuzz_MintAndBurn(uint256 mintAmount) public {
        mintAmount = bound(mintAmount, 1, token.remainingMintable());

        vm.prank(owner);
        token.mint(alice, mintAmount);

        assertEq(token.balanceOf(alice), mintAmount);
        assertLe(token.totalSupply(), token.MAX_SUPPLY());

        vm.prank(alice);
        token.burn(mintAmount);

        assertEq(token.balanceOf(alice), 0);
    }
}
```

### test/Vault.t.sol

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {MyToken} from "../src/MyToken.sol";
import {Vault} from "../src/Vault.sol";

contract VaultTest is Test {
    MyToken public token;
    Vault public vault;

    address public owner = makeAddr("owner");
    address public alice = makeAddr("alice");
    address public bob = makeAddr("bob");

    uint256 public constant DEPOSIT_LIMIT = 10_000 * 10 ** 18;
    uint256 public constant TOTAL_LIMIT = 100_000 * 10 ** 18;
    uint256 public constant ALICE_BALANCE = 20_000 * 10 ** 18;

    function setUp() public {
        vm.startPrank(owner);

        // 토큰 배포
        token = new MyToken(owner, 500_000 * 10 ** 18);

        // Vault 배포
        vault = new Vault(
            address(token),
            owner,
            DEPOSIT_LIMIT,
            TOTAL_LIMIT
        );

        // Alice에게 토큰 지급
        token.mint(alice, ALICE_BALANCE);
        token.mint(bob, ALICE_BALANCE);

        vm.stopPrank();

        // Alice가 Vault에 approve
        vm.prank(alice);
        token.approve(address(vault), type(uint256).max);

        vm.prank(bob);
        token.approve(address(vault), type(uint256).max);
    }

    // ============ 예치 테스트 ============

    function test_Deposit() public {
        uint256 amount = 1_000 * 10 ** 18;

        vm.prank(alice);
        vault.deposit(amount);

        assertEq(vault.balanceOf(alice), amount);
        assertEq(vault.totalDeposits(), amount);
        assertEq(token.balanceOf(address(vault)), amount);
        assertEq(token.balanceOf(alice), ALICE_BALANCE - amount);
    }

    function test_Deposit_EmitsEvent() public {
        uint256 amount = 1_000 * 10 ** 18;

        vm.expectEmit(true, false, false, true);
        emit Vault.Deposited(alice, amount, amount, amount);

        vm.prank(alice);
        vault.deposit(amount);
    }

    function test_Deposit_RevertZeroAmount() public {
        vm.expectRevert(Vault.ZeroAmount.selector);
        vm.prank(alice);
        vault.deposit(0);
    }

    function test_Deposit_RevertExceedsUserLimit() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                Vault.ExceedsDepositLimit.selector,
                DEPOSIT_LIMIT + 1,
                DEPOSIT_LIMIT
            )
        );
        vm.prank(alice);
        vault.deposit(DEPOSIT_LIMIT + 1);
    }

    function test_Deposit_MultipleUsers() public {
        uint256 amount = 5_000 * 10 ** 18;

        vm.prank(alice);
        vault.deposit(amount);

        vm.prank(bob);
        vault.deposit(amount);

        assertEq(vault.totalDeposits(), amount * 2);
        assertEq(vault.balanceOf(alice), amount);
        assertEq(vault.balanceOf(bob), amount);
    }

    // ============ 출금 테스트 ============

    function test_Withdraw() public {
        uint256 depositAmount = 5_000 * 10 ** 18;
        uint256 withdrawAmount = 2_000 * 10 ** 18;

        vm.startPrank(alice);
        vault.deposit(depositAmount);
        vault.withdraw(withdrawAmount);
        vm.stopPrank();

        assertEq(vault.balanceOf(alice), depositAmount - withdrawAmount);
        assertEq(vault.totalDeposits(), depositAmount - withdrawAmount);
        assertEq(
            token.balanceOf(alice),
            ALICE_BALANCE - depositAmount + withdrawAmount
        );
    }

    function test_Withdraw_Full() public {
        uint256 amount = 5_000 * 10 ** 18;

        vm.startPrank(alice);
        vault.deposit(amount);
        vault.withdraw(amount);
        vm.stopPrank();

        assertEq(vault.balanceOf(alice), 0);
        assertEq(vault.totalDeposits(), 0);
        assertEq(token.balanceOf(alice), ALICE_BALANCE);
    }

    function test_Withdraw_RevertInsufficientBalance() public {
        uint256 depositAmount = 1_000 * 10 ** 18;

        vm.prank(alice);
        vault.deposit(depositAmount);

        vm.expectRevert(
            abi.encodeWithSelector(
                Vault.InsufficientBalance.selector,
                alice,
                depositAmount,
                depositAmount + 1
            )
        );
        vm.prank(alice);
        vault.withdraw(depositAmount + 1);
    }

    // ============ 일시 정지 ============

    function test_Pause() public {
        vm.prank(owner);
        vault.pause();

        assertTrue(vault.paused());

        vm.expectRevert(abi.encodeWithSignature("EnforcedPause()"));
        vm.prank(alice);
        vault.deposit(1000);
    }

    function test_Unpause() public {
        vm.prank(owner);
        vault.pause();

        vm.prank(owner);
        vault.unpause();

        assertFalse(vault.paused());

        // 재개 후 정상 동작
        vm.prank(alice);
        vault.deposit(1_000 * 10 ** 18);
    }

    function test_EmergencyWithdraw() public {
        uint256 amount = 5_000 * 10 ** 18;

        vm.prank(alice);
        vault.deposit(amount);

        vm.prank(owner);
        vault.pause();

        uint256 ownerBalanceBefore = token.balanceOf(owner);

        vm.prank(owner);
        vault.emergencyWithdraw(owner);

        assertEq(token.balanceOf(owner), ownerBalanceBefore + amount);
        assertEq(token.balanceOf(address(vault)), 0);
    }

    function test_EmergencyWithdraw_RevertWhenNotPaused() public {
        vm.expectRevert(abi.encodeWithSignature("ExpectedPause()"));
        vm.prank(owner);
        vault.emergencyWithdraw(owner);
    }

    // ============ 한도 관리 ============

    function test_SetDepositLimit() public {
        uint256 newLimit = 50_000 * 10 ** 18;

        vm.prank(owner);
        vault.setDepositLimit(newLimit);

        assertEq(vault.depositLimit(), newLimit);
    }

    function test_RemainingCapacity() public {
        uint256 amount = 30_000 * 10 ** 18;

        vm.prank(owner);
        token.mint(alice, amount);

        vm.prank(alice);
        token.approve(address(vault), amount);

        vm.prank(owner);
        vault.setDepositLimit(amount); // 개인 한도 늘리기

        vm.prank(alice);
        vault.deposit(amount);

        assertEq(vault.remainingCapacity(), TOTAL_LIMIT - amount);
        assertEq(vault.remainingUserCapacity(alice), 0);
    }

    // ============ 통합 시나리오 ============

    function test_FullScenario() public {
        // 1. Alice 예치
        uint256 aliceDeposit = 5_000 * 10 ** 18;
        vm.prank(alice);
        vault.deposit(aliceDeposit);

        // 2. Bob 예치
        uint256 bobDeposit = 3_000 * 10 ** 18;
        vm.prank(bob);
        vault.deposit(bobDeposit);

        assertEq(vault.totalDeposits(), aliceDeposit + bobDeposit);

        // 3. Alice 일부 출금
        uint256 aliceWithdraw = 2_000 * 10 ** 18;
        vm.prank(alice);
        vault.withdraw(aliceWithdraw);

        assertEq(vault.balanceOf(alice), aliceDeposit - aliceWithdraw);

        // 4. 비상 상황: 일시 정지 후 긴급 출금
        vm.prank(owner);
        vault.pause();

        vm.prank(owner);
        vault.emergencyWithdraw(owner);

        assertEq(token.balanceOf(address(vault)), 0);
    }

    // ============ 퍼즈 테스트 ============

    function testFuzz_DepositAndWithdraw(uint256 depositAmount, uint256 withdrawAmount) public {
        depositAmount = bound(depositAmount, 1, DEPOSIT_LIMIT);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        // Alice가 충분한 토큰이 있는지 확인
        if (token.balanceOf(alice) < depositAmount) {
            vm.prank(owner);
            token.mint(alice, depositAmount - token.balanceOf(alice));
        }

        vm.startPrank(alice);
        vault.deposit(depositAmount);
        vault.withdraw(withdrawAmount);
        vm.stopPrank();

        assertEq(vault.balanceOf(alice), depositAmount - withdrawAmount);
        // 불변식: vault의 실제 잔액 = 총 예치량
        assertEq(token.balanceOf(address(vault)), vault.totalDeposits());
    }
}
```

## 배포 스크립트

```solidity
// script/Deploy.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {MyToken} from "../src/MyToken.sol";
import {Vault} from "../src/Vault.sol";

contract Deploy is Script {
    // 배포 설정
    uint256 constant INITIAL_SUPPLY = 500_000 * 10 ** 18;
    uint256 constant DEPOSIT_LIMIT = 10_000 * 10 ** 18;
    uint256 constant TOTAL_LIMIT = 100_000 * 10 ** 18;

    function run() external returns (MyToken token, Vault vault) {
        uint256 deployerKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerKey);

        console.log("=== Token Vault Deployment ===");
        console.log("Deployer:", deployer);
        console.log("Network Chain ID:", block.chainid);

        vm.startBroadcast(deployerKey);

        // 1. MyToken 배포
        token = new MyToken(deployer, INITIAL_SUPPLY);
        console.log("MyToken deployed:", address(token));
        console.log("  Initial supply:", INITIAL_SUPPLY / 10 ** 18, "MTK");

        // 2. Vault 배포
        vault = new Vault(
            address(token),
            deployer,
            DEPOSIT_LIMIT,
            TOTAL_LIMIT
        );
        console.log("Vault deployed:", address(vault));
        console.log("  Deposit limit:", DEPOSIT_LIMIT / 10 ** 18, "MTK per user");
        console.log("  Total limit:", TOTAL_LIMIT / 10 ** 18, "MTK");

        // 3. 초기 설정: Vault에 토큰 민팅 권한은 부여하지 않음
        // (Vault는 예치된 토큰만 관리, 새 토큰 발행 없음)

        vm.stopBroadcast();

        console.log("=== Deployment Complete ===");
        console.log("Next steps:");
        console.log("  1. Users approve Vault to spend their tokens");
        console.log("  2. Users call vault.deposit(amount)");
        console.log("  3. Users call vault.withdraw(amount)");
    }
}
```

## 단계별 실행 가이드

### 1단계: 프로젝트 설정

```bash
# 프로젝트 생성
forge init token-vault
cd token-vault

# OpenZeppelin 설치
forge install OpenZeppelin/openzeppelin-contracts

# remappings.txt 설정
echo "@openzeppelin/=lib/openzeppelin-contracts/" > remappings.txt

# src/ 파일 작성 (MyToken.sol, Vault.sol)
# test/ 파일 작성
# script/ 파일 작성
```

### 2단계: 컴파일 및 테스트

```bash
# 컴파일
forge build

# 전체 테스트 실행
forge test -vvv

# 가스 리포트
forge test --gas-report

# 커버리지
forge coverage
```

예상 출력:
```
Running 20 tests for test/Vault.t.sol:VaultTest
[PASS] test_Deposit() (gas: 98234)
[PASS] test_Withdraw() (gas: 75123)
[PASS] test_Pause() (gas: 45678)
...
Test result: ok. 20 passed; 0 failed; finished in 45.23ms
```

### 3단계: 로컬 배포

```bash
# 터미널 1: Anvil 실행
anvil

# 터미널 2: 배포
export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

forge script script/Deploy.s.sol \
    --rpc-url http://127.0.0.1:8545 \
    --broadcast \
    -vvv
```

### 4단계: cast로 상호작용

```bash
# 배포된 주소 확인
export TOKEN=<MyToken 주소>
export VAULT=<Vault 주소>
export USER=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# 잔액 확인
cast call $TOKEN "balanceOf(address)(uint256)" $USER \
    --rpc-url http://127.0.0.1:8545

# Vault에 approve (10,000 MTK)
cast send $TOKEN \
    "approve(address,uint256)" \
    $VAULT $(cast to-wei 10000 ether) \
    --private-key $PRIVATE_KEY \
    --rpc-url http://127.0.0.1:8545

# 1,000 MTK 예치
cast send $VAULT \
    "deposit(uint256)" \
    $(cast to-wei 1000 ether) \
    --private-key $PRIVATE_KEY \
    --rpc-url http://127.0.0.1:8545

# Vault 잔액 확인
cast call $VAULT "balanceOf(address)(uint256)" $USER \
    --rpc-url http://127.0.0.1:8545

# 500 MTK 출금
cast send $VAULT \
    "withdraw(uint256)" \
    $(cast to-wei 500 ether) \
    --private-key $PRIVATE_KEY \
    --rpc-url http://127.0.0.1:8545
```

### 5단계: 테스트넷 배포 (Sepolia)

```bash
# .env 파일 설정
cat > .env << EOF
PRIVATE_KEY=<your-private-key>
SEPOLIA_RPC_URL=https://eth-sepolia.alchemyapi.io/v2/<your-key>
ETHERSCAN_API_KEY=<your-etherscan-key>
EOF

source .env

# Sepolia 배포
forge script script/Deploy.s.sol \
    --rpc-url $SEPOLIA_RPC_URL \
    --broadcast \
    --verify \
    --etherscan-api-key $ETHERSCAN_API_KEY \
    -vvvv
```

## 프로젝트 확장 아이디어

이 프로젝트를 기반으로 다음 기능을 추가해볼 수 있다:

```
1. 이자 기능
   - 예치 기간에 비례한 이자 계산
   - 블록 번호나 타임스탬프 기반 이자율

2. 유동성 토큰 (Vault Share Token)
   - 예치 시 vMTK 토큰 발행
   - 출금 시 vMTK 소각
   - ERC-4626 표준 구현

3. 프록시 업그레이드
   - UUPS 프록시로 업그레이드 가능하게
   - V2에 수수료 기능 추가

4. 거버넌스
   - 토큰 보유자 투표로 파라미터 변경
   - OpenZeppelin Governor 활용
```

## 정리

이 미니프로젝트에서 사용한 핵심 패턴들:

| 패턴 | 적용 | 이유 |
|------|------|------|
| OpenZeppelin ERC20 상속 | MyToken | 검증된 구현 재사용 |
| Ownable2Step | 두 컨트랙트 모두 | 소유권 이전 실수 방지 |
| ReentrancyGuard | Vault | 재진입 공격 방지 |
| Pausable | Vault | 긴급 정지 |
| SafeERC20 | Vault | 안전한 토큰 전송 |
| CEI 패턴 | Vault | 재진입 방지 이중 보호 |
| 커스텀 에러 | 두 컨트랙트 모두 | 가스 효율 + 타입 안전 |
| immutable | Vault.token | 가스 절약 |
| 퍼즈 테스트 | 테스트 | 엣지 케이스 자동 탐지 |
