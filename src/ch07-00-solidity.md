# 7. Solidity 핵심 문법

Solidity는 EVM 스마트 컨트랙트를 작성하는 언어다. 외부 진입점에는 누구나 신뢰할 수 없는 입력을 보낼 수 있으며 상태나 자산을 바꾸는 함수는 특히 공격 경계가 된다.

예제는 안정적인 0.8 계열을 가리키도록 `^0.8.24`를 사용한다. 실제 프로젝트에서는 최신 릴리스와 변경 사항을 확인하고 컴파일러 버전을 잠근다.

## 가장 작은 상태 컨트랙트

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract Counter {
    uint256 private constant MAX_NUMBER = 1_000_000;
    uint256 public number;

    event NumberChanged(address indexed caller, uint256 newNumber);

    error NumberTooLarge(uint256 attempted);

    function setNumber(uint256 newNumber) external {
        _setNumber(newNumber);
    }

    function increment() external {
        _setNumber(number + 1);
    }

    function _setNumber(uint256 newNumber) private {
        if (newNumber > MAX_NUMBER) {
            revert NumberTooLarge(newNumber);
        }
        number = newNumber;
        emit NumberChanged(msg.sender, newNumber);
    }
}
```

이 코드에서 봐야 할 것은 다섯 가지다.

- `number`는 `storage`에 남는 상태 변수다.
- `public` 상태 변수에는 읽기 함수가 자동 생성된다.
- `external` 함수는 컨트랙트 밖에서 호출하는 진입점이다.
- `msg.sender`는 현재 호출자다. 최초 트랜잭션 발신자와 항상 같지는 않다.
- 이벤트는 영수증 로그에 기록되며 컨트랙트가 다시 읽는 상태 저장소가 아니다.

## 타입과 단위

Solidity는 정수 크기와 부호를 명시한다. 자산 수량에는 보통 `uint256`을 쓴다. 0.8 계열은 기본 산술에서 overflow와 underflow를 검사해 revert한다. `unchecked` 블록은 이 검사를 끄므로 이유와 범위 증명이 있을 때만 쓴다.

주소 타입도 구분된다.

```solidity
address user;
address payable recipient;
```

`address payable`은 ETH 전송을 허용하는 주소다. ETH 단위는 문법으로 쓸 수 있다.

```solidity
uint256 fee = 0.01 ether;
```

온체인 값은 여전히 정수 wei다. `ether`는 사람이 읽기 편한 단위 표기다.

## `storage`, `memory`, `calldata`

문자열, 배열, 구조체처럼 참조형 데이터를 함수에서 다룰 때 위치를 적는다.

```solidity
function hashLabel(string calldata label) external pure returns (bytes32) {
    return keccak256(bytes(label));
}
```

외부 입력을 읽기만 한다면 `calldata`가 자연스럽다. 함수 안에서 수정할 임시 복사본은 `memory`, 영구 상태를 직접 가리키면 `storage`다.

```solidity
struct Profile {
    string name;
    uint256 score;
}

mapping(address => Profile) private profiles;

function addScore(uint256 amount) external {
    Profile storage profile = profiles[msg.sender];
    profile.score += amount;
}
```

`mapping`은 모든 가능한 키에 기본값이 있다고 보는 저장 구조다. 키 목록을 자동으로 열거해 주지 않는다. 목록이 필요하면 별도 배열이나 오프체인 인덱서를 설계해야 한다.

## 함수 가시성과 상태 변경

| 키워드 | 의미 |
|---|---|
| `external` | 주로 외부 호출용 |
| `public` | 내부와 외부에서 호출 가능 |
| `internal` | 현재 컨트랙트와 상속한 컨트랙트에서 사용 |
| `private` | 선언한 컨트랙트 안에서만 사용 |
| `view` | 상태를 읽지만 쓰지 않음 |
| `pure` | 컨트랙트 상태를 읽지도 쓰지도 않음 |
| `payable` | 호출과 함께 ETH를 받을 수 있음 |

`private`은 비밀이라는 뜻이 아니다. 다른 컨트랙트가 Solidity 함수로 접근하지 못할 뿐, 온체인 저장 데이터는 노드에서 읽을 수 있다.

## 실패와 검증

입력이나 권한을 검사할 때 `require`나 custom error를 쓴다.

```solidity
error InvalidOwner();
error Unauthorized(address caller);

address public immutable owner;

constructor(address initialOwner) {
    if (initialOwner == address(0)) revert InvalidOwner();
    owner = initialOwner;
}

function restrictedAction() external {
    if (msg.sender != owner) revert Unauthorized(msg.sender);
}
```

`assert`는 사용자 입력 검증보다 내부 불변식 확인에 쓴다. `revert`가 발생하면 현재 호출 프레임의 상태 변경과 하위 호출 변경이 되돌아간다.

## 이벤트는 조회용 기록이다

이벤트 인자에 `indexed`를 붙이면 로그 토픽으로 검색하기 쉬워진다.

```solidity
event TransferRecorded(
    address indexed from,
    address indexed to,
    uint256 amount,
    bytes32 reference
);
```

컨트랙트는 과거 이벤트 전체를 순회할 수 없다. 오프체인 인덱서는 RPC로 로그를 구독하거나 블록 범위를 조회해 검색 DB를 만든다. 중요한 현재 상태는 이벤트에만 두지 말고 컨트랙트 저장소에도 있어야 한다.

## `receive`와 `fallback`

컨트랙트로 함수 호출 데이터 없이 ETH가 오면 `receive()`가 실행될 수 있다. 호출 데이터가 있는데 일치하는 함수가 없거나 `receive`가 없을 때는 `fallback()`이 대상이 된다.

```solidity
receive() external payable {
    emit Deposit(msg.sender, msg.value);
}

fallback() external payable {
    revert("unknown function");
}
```

여기서 `fallback`은 임시 우회 코드가 아니라 Solidity가 정의한 특수 함수 이름이다. 프록시가 호출을 다른 구현으로 위임할 때도 사용하지만 초보 프로젝트에서 이유 없이 열어 두면 예상치 못한 ETH 수신이나 호출 처리가 생길 수 있다.

## Solidity 실행 환경 요약

Solidity 함수는 결정적인 EVM 실행 안에서 끝나야 한다. 임의의 외부 HTTP 요청을 보낼 수 없고 영구 저장과 다른 컨트랙트 호출에는 가스 비용이 붙는다. `private` 상태도 체인 밖에서 숨겨지지 않으며 외부 호출 입력은 항상 검증해야 한다.

`Counter`를 배포하기 전에 Foundry 테스트로 두 상태 변경 경로가 같은 상한을 지키는지 확인한다.
