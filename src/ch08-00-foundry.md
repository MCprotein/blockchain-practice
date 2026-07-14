# 8. Foundry로 테스트하기

Foundry는 Ethereum 개발 도구 모음이다. 이 책에서 주로 쓰는 도구는 세 가지다.

- Forge: 빌드, 테스트, 배포 스크립트
- Anvil: 로컬 Ethereum 노드
- Cast: RPC 호출, ABI 인코딩, 트랜잭션 조회

공식 설치 도구는 `foundryup`이다. 설치 방법은 운영체제와 시점에 따라 바뀔 수 있으니 실행 전 공식 문서를 확인한다.

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
forge --version
```

## 프로젝트 만들기

```bash
forge init counter-foundry
cd counter-foundry
forge build
forge test
```

핵심 디렉터리는 다음과 같다.

```text
counter-foundry/
├── foundry.toml
├── src/Counter.sol
├── test/Counter.t.sol
├── script/Counter.s.sol
└── lib/forge-std/
```

| 경로 | 역할 |
|---|---|
| `foundry.toml` | 컴파일러, 소스 경로, RPC 등 도구 설정 |
| `src` | 배포할 Solidity 코드 |
| `test` | Forge가 실행할 Solidity 테스트 |
| `script` | 배포와 운영 트랜잭션 스크립트 |
| `lib` | `forge-std`, OpenZeppelin 같은 의존성 |

Solidity 의존성은 보통 `lib` 아래에 설치하고 remapping으로 import 경로를 잡는다.

## 단위 테스트

7장의 `Counter`를 `src/Counter.sol`에 둔 뒤 다음 테스트를 작성한다.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {Counter} from "../src/Counter.sol";

contract CounterTest is Test {
    Counter private counter;
    address private alice = makeAddr("alice");

    function setUp() public {
        counter = new Counter();
    }

    function test_SetNumberChangesState() public {
        vm.prank(alice);
        counter.setNumber(42);

        assertEq(counter.number(), 42);
    }

    function test_RevertWhenNumberIsTooLarge() public {
        vm.expectRevert(abi.encodeWithSelector(Counter.NumberTooLarge.selector, 1_000_001));
        counter.setNumber(1_000_001);
    }

    function test_IncrementCannotBypassMaximum() public {
        counter.setNumber(1_000_000);
        vm.expectRevert(abi.encodeWithSelector(Counter.NumberTooLarge.selector, 1_000_001));
        counter.increment();
    }

    function testFuzz_SetNumber(uint256 value) public {
        value = bound(value, 0, 1_000_000);
        counter.setNumber(value);
        assertEq(counter.number(), value);
    }
}
```

`setUp`은 각 테스트 전에 새로 실행된다. `vm.prank(alice)`는 다음 호출의 `msg.sender`를 바꾼다. cheatcode는 테스트 EVM을 제어하는 기능이며 제품 컨트랙트에서 사용할 수 없다.

함수 이름으로 테스트 의도를 드러낼 수 있다.

| 이름 | 의미 |
|---|---|
| `test_...` | 일반 단위 테스트 |
| `test_RevertWhen...` | 실패 조건을 검증하는 관례적 이름 |
| `testFuzz_...` | 여러 입력을 생성하는 fuzz test |
| `invariant_...` | 호출 순서가 달라도 유지할 속성 |

## 테스트는 예시보다 속성을 본다

한두 입력을 확인하는 example-based test만으로 자산 코드를 지키기 어렵다. 다음 세 층으로 테스트를 늘린다.

1. 정상 상태 전이: 예치 후 잔액이 늘어난다.
2. 실패 조건: 권한이 없거나 잔액이 부족하면 revert한다.
3. 불변식: 어떤 호출 순서에서도 총 부채가 보유 자산보다 커지지 않는다.

Fuzz test는 무작위에 가까운 여러 입력을 넣고 invariant test는 여러 함수를 조합한 호출 시퀀스를 만든다. 생성 범위를 너무 좁히면 테스트가 쉬운 값만 돌 수 있으니 경계값을 별도 단위 테스트로도 남긴다.

## 로그와 디버깅

실패 원인을 볼 때 verbosity를 올린다.

```bash
forge test -vvv
forge test -vvvv --match-test test_SetNumberChangesState
forge test --debug --match-test test_SetNumberChangesState
```

`-vvvv` 출력에는 호출 trace와 이벤트가 포함된다. “테스트가 revert했다”에서 멈추지 말고 어떤 외부 호출과 상태 변경 직전에 실패했는지 찾는다.

## Anvil과 Cast

로컬 노드를 켠다.

```bash
anvil
```

다른 터미널에서 블록 번호를 읽을 수 있다.

```bash
cast block-number --rpc-url http://127.0.0.1:8545
```

Anvil이 보여 주는 기본 개인키는 공개된 개발용 키다. 로컬 노드 밖에서 자산을 보관하는 데 절대 쓰지 않는다.

## 배포 키 관리

`.env`의 평문 개인키를 그대로 배포 명령에 넣는 예제는 따라 하기 쉽지만 습관으로 남기면 위험하다. Foundry 공식 지침은 암호화된 keystore나 하드웨어 지갑 사용을 권한다.

```bash
cast wallet import deployer --interactive
forge script script/Deploy.s.sol \
  --account deployer \
  --rpc-url "$RPC_URL" \
  --broadcast
```

로컬, 테스트넷, 메인넷 키는 분리한다. 배포 전에 `--broadcast` 없이 시뮬레이션하고 체인 ID·배포자·constructor 인자·예상 주소를 확인한다.

실제 프로젝트에서는 `forge build`와 `forge test`의 결과를 기준으로 삼는다.
