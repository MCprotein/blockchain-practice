# Chapter 11: Solidity 기초

## Solidity란 무엇인가

Solidity는 이더리움 스마트 컨트랙트를 작성하기 위한 정적 타입(statically typed) 프로그래밍 언어다. 2014년 Gavin Wood가 제안하고 이더리움 팀이 개발했으며, 현재는 이더리움 생태계의 표준 스마트 컨트랙트 언어로 자리잡았다.

Node.js 백엔드 개발자 관점에서 보면 Solidity는 JavaScript/TypeScript와 문법적으로 유사한 부분이 많다. 중괄호로 블록을 구분하고, `if/else`, `for`, `while` 같은 제어 흐름도 동일하다. 하지만 실행 환경이 근본적으로 다르다.

**TypeScript 코드가 실행되는 환경:**
- Node.js V8 엔진 위에서 실행
- 서버의 CPU와 메모리 사용
- 실행 비용: 서버 운영비

**Solidity 코드가 실행되는 환경:**
- EVM(Ethereum Virtual Machine) 위에서 실행
- 수천 개의 노드가 동일한 코드를 동시에 실행
- 실행 비용: 가스(Gas) — 실제 ETH로 지불

이 차이가 Solidity의 모든 설계 결정에 영향을 미친다. 반복문을 쓸 때도, 데이터를 저장할 때도, 함수를 호출할 때도 항상 "이게 얼마나 비싼가?"를 생각해야 한다.

## 컨트랙트 구조

모든 Solidity 파일은 세 가지 핵심 요소로 구성된다.

### 1. pragma — 컴파일러 버전 지정

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
```

`pragma solidity ^0.8.20`은 "Solidity 0.8.20 이상, 0.9.0 미만의 컴파일러를 사용하라"는 의미다. `^` 기호는 npm의 semver와 동일한 의미다.

`SPDX-License-Identifier`는 소스 코드의 라이선스를 명시한다. 블록체인에 배포된 코드는 누구나 볼 수 있으므로 라이선스 표시가 중요하다. MIT, Apache-2.0, GPL-3.0 등을 사용할 수 있으며, 라이선스가 없다면 `UNLICENSED`를 쓴다.

### 2. import — 다른 파일 불러오기

```solidity
// 상대 경로 import
import "./Token.sol";

// 특정 심볼만 import
import { ERC20 } from "./ERC20.sol";

// npm 패키지 스타일 (OpenZeppelin 등)
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

// 별칭(alias) 사용
import { Ownable as OwnableContract } from "@openzeppelin/contracts/access/Ownable.sol";
```

TypeScript의 ES module import와 거의 동일하다. `@openzeppelin/...` 같은 패키지 경로는 `remappings.txt` 또는 `foundry.toml`에서 실제 경로로 매핑된다.

### 3. contract — 컨트랙트 정의

```solidity
contract MyContract {
    // 상태 변수 (블록체인에 영구 저장)
    uint256 public count;
    
    // 함수
    function increment() public {
        count += 1;
    }
}
```

`contract` 키워드는 TypeScript의 `class`와 유사하다. 상태 변수는 클래스의 인스턴스 변수처럼 컨트랙트의 데이터를 저장하며, 이 데이터는 블록체인에 영구적으로 기록된다.

## TypeScript class vs Solidity contract 비교

| 개념 | TypeScript | Solidity |
|------|-----------|----------|
| 타입 정의 | `class MyClass {}` | `contract MyContract {}` |
| 인스턴스 변수 | `private count: number` | `uint256 private count` |
| 생성자 | `constructor() {}` | `constructor() {}` |
| 메서드 | `increment(): void {}` | `function increment() public {}` |
| 상속 | `extends BaseClass` | `is BaseContract` |
| 인터페이스 | `implements IFoo` | `is IFoo` (인터페이스도 동일 구문) |
| 읽기 전용 | `readonly` | `view` 또는 `pure` 함수 |
| 접근 제어 | `public/private/protected` | `public/private/internal/external` |

**핵심 차이점:**
- TypeScript 클래스는 메모리에 인스턴스가 생성되고 GC가 관리한다
- Solidity 컨트랙트는 블록체인 주소에 배포되고 영원히 존재한다
- TypeScript는 생성자를 여러 번 호출할 수 있다
- Solidity 생성자는 배포 시 딱 한 번만 실행된다

## Remix IDE로 첫 컨트랙트 작성하기

[Remix IDE](https://remix.ethereum.org)는 브라우저에서 바로 사용할 수 있는 Solidity 개발 환경이다. 설치 없이 바로 시작할 수 있어서 학습하기에 최적이다.

### Remix 시작하기

1. `https://remix.ethereum.org` 접속
2. 좌측 파일 탐색기에서 `contracts/` 폴더 우클릭 → `New File`
3. 파일명: `Counter.sol` 입력

### 첫 컨트랙트: Counter

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Counter - 간단한 카운터 컨트랙트
/// @notice 숫자를 증가/감소시키는 기본 컨트랙트
contract Counter {
    // 상태 변수: 블록체인에 저장되는 카운터 값
    uint256 private _count;
    
    // 소유자 주소
    address public owner;
    
    // 이벤트: 카운터가 변경될 때 로그를 남긴다
    event CountChanged(uint256 newCount, address changedBy);
    
    // 생성자: 배포 시 딱 한 번 실행
    constructor() {
        owner = msg.sender;  // 배포한 사람의 주소
        _count = 0;
    }
    
    /// @notice 카운터를 1 증가시킨다
    function increment() public {
        _count += 1;
        emit CountChanged(_count, msg.sender);
    }
    
    /// @notice 카운터를 1 감소시킨다 (0 미만으로는 내려가지 않음)
    function decrement() public {
        require(_count > 0, "Counter: cannot decrement below zero");
        _count -= 1;
        emit CountChanged(_count, msg.sender);
    }
    
    /// @notice 특정 값으로 리셋 (소유자만 가능)
    function reset(uint256 newValue) public {
        require(msg.sender == owner, "Counter: only owner can reset");
        _count = newValue;
        emit CountChanged(_count, msg.sender);
    }
    
    /// @notice 현재 카운터 값 조회 (읽기 전용)
    function getCount() public view returns (uint256) {
        return _count;
    }
}
```

### Remix에서 컴파일하기

1. 좌측 메뉴에서 **Solidity Compiler** (두 번째 아이콘) 클릭
2. `Compiler` 버전을 `0.8.20`으로 선택
3. **Compile Counter.sol** 버튼 클릭
4. 컴파일 성공 시 아이콘에 초록 체크 표시

### Remix에서 배포하기

1. 좌측 메뉴에서 **Deploy & Run Transactions** (세 번째 아이콘) 클릭
2. `Environment`를 `Remix VM (Shanghai)` 선택 (로컬 테스트 환경)
3. `Contract` 드롭다운에서 `Counter` 선택
4. **Deploy** 버튼 클릭
5. 하단에 배포된 컨트랙트 주소가 생성됨

### Remix에서 함수 호출하기

배포 후 하단 `Deployed Contracts` 섹션에서:

- **주황색 버튼**: 상태를 변경하는 함수 (트랜잭션 발생, 가스 소비)
  - `increment`: 카운터 +1
  - `decrement`: 카운터 -1
  - `reset`: 입력값으로 리셋

- **파란색 버튼**: 읽기 전용 함수 (트랜잭션 없음, 가스 소비 없음)
  - `getCount`: 현재 값 조회
  - `owner`: 소유자 주소 조회

`increment`를 3번 클릭한 후 `getCount`를 클릭하면 `3`이 반환된다.

## 카운터 컨트랙트 상세 설명

### msg 전역 객체

```solidity
msg.sender  // 현재 함수를 호출한 주소 (EOA 또는 컨트랙트)
msg.value   // 함수에 함께 전송된 ETH 양 (wei 단위)
msg.data    // 함수 호출 시 전달된 전체 calldata
```

`msg.sender`는 NestJS의 `@Req() req`에서 꺼내는 `req.user`와 비슷한 개념이다. 현재 호출자의 신원을 알 수 있다. 단, 블록체인에서는 서명으로 신원을 증명하므로 위조가 불가능하다.

### require — 조건 검증

```solidity
require(조건, "실패 메시지");
```

조건이 `false`면 트랜잭션을 되돌리고(revert) 메시지를 반환한다. 가스는 이미 사용된 만큼만 소비된다.

Node.js에서의 가드 클로즈(guard clause)와 유사하다:

```typescript
// TypeScript
if (!user) throw new UnauthorizedException('User not found');

// Solidity 동등한 코드
require(msg.sender != address(0), "Invalid sender");
```

### event와 emit

```solidity
event CountChanged(uint256 newCount, address changedBy);
emit CountChanged(_count, msg.sender);
```

이벤트는 블록체인의 로그에 기록된다. 컨트랙트 외부(프론트엔드, 백엔드)에서 이 로그를 구독할 수 있다. 상태 변수보다 훨씬 저렴하게 데이터를 기록하는 방법이다.

### view 함수

```solidity
function getCount() public view returns (uint256) {
    return _count;
}
```

`view` 키워드는 "이 함수는 상태를 읽기만 하고 변경하지 않는다"는 의미다. TypeScript의 getter와 동일하다. `view` 함수는 트랜잭션 없이 무료로 호출할 수 있다.

## 정리

Solidity는 TypeScript와 문법이 유사하지만, 블록체인이라는 특수한 실행 환경 때문에 다른 사고방식이 필요하다:

1. **모든 코드는 공개된다** — 배포된 컨트랙트 코드는 누구나 볼 수 있다
2. **상태 변경은 비용이 든다** — 가스를 소비하므로 효율적으로 작성해야 한다
3. **한번 배포하면 수정 불가** — 업그레이드 패턴을 미리 설계해야 한다
4. **신뢰 없는 환경** — 외부 입력은 항상 검증해야 한다

다음 챕터에서는 Solidity의 타입 시스템을 TypeScript와 비교하며 자세히 살펴본다.
