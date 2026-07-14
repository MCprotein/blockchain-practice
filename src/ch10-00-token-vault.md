# 10. Token Vault 보안 실습

Token Vault는 사용자가 ERC-20을 맡기고 나중에 같은 수량을 찾는 컨트랙트다. 기능은 단순하지만 allowance, 외부 호출, 재진입, 회계 불변식을 한 번에 연습하기 좋다. 아래 코드는 보안 패턴을 설명하는 학습용 기준선이며 감사받은 제품용 Vault가 아니다.

## 먼저 불변식을 적는다

코드보다 다음 조건을 먼저 테스트 대상으로 만든다.

```text
사용자 장부 합계 == Vault의 totalDeposits
Vault가 지원 토큰을 정상 전송받았다면 실제 보유량 >= totalDeposits
사용자는 자신의 장부보다 많이 출금할 수 없음
실패한 예치·출금은 장부를 바꾸지 않음
```

마지막 자산 보유량 조건은 토큰 종류에 따라 복잡해진다. 전송 수수료 토큰, 리베이스 토큰, 콜백을 실행하는 토큰은 일반 ERC-20과 다른 회계 모델이 필요하다. 이 실습은 요청한 수량과 실제 수령량이 같은 토큰만 받는다.

## 컨트랙트

OpenZeppelin의 `SafeERC20`은 반환값이 없거나 `false`를 반환하는 일부 ERC-20 구현을 더 안전하게 다룬다. `ReentrancyGuard`는 외부 토큰 호출 중 같은 진입점으로 다시 들어오는 것을 막는다.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract TokenVault is ReentrancyGuard {
    using SafeERC20 for IERC20;

    mapping(IERC20 token => mapping(address user => uint256 amount)) public balanceOf;
    mapping(IERC20 token => uint256 amount) public totalDeposits;

    error ZeroAmount();
    error InsufficientBalance(uint256 available, uint256 requested);
    error UnsupportedTokenBehavior(uint256 requested, uint256 received);

    event Deposited(IERC20 indexed token, address indexed user, uint256 amount);
    event Withdrawn(IERC20 indexed token, address indexed user, uint256 amount);

    function deposit(IERC20 token, uint256 amount) external nonReentrant {
        if (amount == 0) revert ZeroAmount();

        uint256 beforeBalance = token.balanceOf(address(this));
        token.safeTransferFrom(msg.sender, address(this), amount);
        uint256 received = token.balanceOf(address(this)) - beforeBalance;

        if (received != amount) {
            revert UnsupportedTokenBehavior(amount, received);
        }

        balanceOf[token][msg.sender] += amount;
        totalDeposits[token] += amount;
        emit Deposited(token, msg.sender, amount);
    }

    function withdraw(IERC20 token, uint256 amount) external nonReentrant {
        if (amount == 0) revert ZeroAmount();

        uint256 available = balanceOf[token][msg.sender];
        if (available < amount) {
            revert InsufficientBalance(available, amount);
        }

        balanceOf[token][msg.sender] = available - amount;
        totalDeposits[token] -= amount;

        token.safeTransfer(msg.sender, amount);
        emit Withdrawn(token, msg.sender, amount);
    }
}
```

## 왜 출금 장부를 먼저 줄이나

출금은 checks-effects-interactions(CEI) 순서를 따른다.

```text
Checks       출금 수량과 잔액 확인
Effects      사용자 잔액과 총예치액 감소
Interactions 토큰 컨트랙트 호출
```

외부 호출을 먼저 하고 장부를 나중에 줄이면 외부 토큰 컨트랙트가 악성 동작이나 재진입을 시도할 때 `withdraw`가 다시 실행될 여지가 생긴다. `nonReentrant`도 사용했지만 CEI를 함께 지키는 편이 방어층이 두껍다.

예치는 실제 토큰 잔액 변화를 확인해야 해서 외부 호출 뒤 장부를 늘린다. 이 구간은 `nonReentrant`로 보호한다. 지원할 토큰을 allowlist로 제한하는 것도 제품 설계에서 흔한 방어다.

## Foundry 테스트

테스트용 토큰과 Vault를 배포한다.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {TokenVault} from "../src/TokenVault.sol";

contract MockToken is ERC20 {
    constructor() ERC20("Mock Token", "MOCK") {}

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract TokenVaultTest is Test {
    MockToken private token;
    TokenVault private vault;
    address private alice = makeAddr("alice");

    function setUp() public {
        token = new MockToken();
        vault = new TokenVault();
        token.mint(alice, 100 ether);

        vm.prank(alice);
        token.approve(address(vault), type(uint256).max);
    }

    function test_DepositAndWithdraw() public {
        vm.startPrank(alice);
        vault.deposit(IERC20(address(token)), 40 ether);
        vault.withdraw(IERC20(address(token)), 15 ether);
        vm.stopPrank();

        assertEq(vault.balanceOf(IERC20(address(token)), alice), 25 ether);
        assertEq(vault.totalDeposits(IERC20(address(token))), 25 ether);
        assertEq(token.balanceOf(address(vault)), 25 ether);
        assertEq(token.balanceOf(alice), 75 ether);
    }

    function test_RevertWhenWithdrawalExceedsBalance() public {
        vm.prank(alice);
        vault.deposit(IERC20(address(token)), 10 ether);

        vm.expectRevert(
            abi.encodeWithSelector(TokenVault.InsufficientBalance.selector, 10 ether, 11 ether)
        );
        vm.prank(alice);
        vault.withdraw(IERC20(address(token)), 11 ether);
    }

    function testFuzz_DepositPreservesAccounting(uint96 rawAmount) public {
        uint256 amount = bound(uint256(rawAmount), 1, 100 ether);

        vm.prank(alice);
        vault.deposit(IERC20(address(token)), amount);

        assertEq(vault.balanceOf(IERC20(address(token)), alice), amount);
        assertEq(vault.totalDeposits(IERC20(address(token))), amount);
        assertEq(token.balanceOf(address(vault)), amount);
    }
}
```

이 테스트만으로 보안 검토가 끝나지는 않는다. 다음 공격 모델을 별도 mock으로 추가해야 한다.

- 토큰 함수가 `false`를 반환하거나 반환값이 없음
- 전송 수수료 때문에 요청량보다 적게 도착함
- 토큰 호출 중 Vault로 재진입 시도
- 관리자 기능을 추가했다면 권한 탈취와 잘못된 역할 이전
- 예상 밖 토큰이 Vault 주소로 직접 전송됨

## `tx.origin`을 권한 검사에 쓰지 않는다

권한 확인은 보통 `msg.sender`를 기준으로 한다. `tx.origin`은 호출 체인의 최초 EOA를 가리키므로, 사용자가 악성 컨트랙트를 호출했을 때 그 컨트랙트가 권한 있는 사용자를 대신해 Vault를 호출하는 피싱 경로가 생길 수 있다.

## 배포 전 체크

제품용 Vault라면 최소한 다음을 결정해야 한다.

- 어떤 토큰을 지원하는가?
- 회계 단위는 원금 수량인가, 지분 share인가?
- 토큰이 리베이스되거나 손실되면 누가 부담하는가?
- 긴급 중지와 복구 권한은 누구에게 있는가?
- 업그레이드 가능한가? 가능하다면 관리자와 지연 장치는 무엇인가?
- 독립 리뷰, fuzz test, invariant test, 포크 테스트를 했는가?

스마트 컨트랙트 보안은 패턴 하나를 붙여 끝나는 일이 아니다. 자산 모델과 외부 호출 경계를 먼저 정하고 그 가정을 테스트로 공격해야 한다.
