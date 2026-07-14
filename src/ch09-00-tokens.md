# 9. 토큰과 ERC-20·ERC-721

블록체인에서 토큰(token)은 컨트랙트나 프로토콜이 관리하는 자산 단위다. 실물이 따로 생기는 것이 아니라, “누가 얼마를 가졌고 어떤 규칙으로 옮길 수 있는가”를 온체인 상태와 함수로 표현한다.

## 코인과 토큰

보통 코인은 체인의 기본 자산을, 토큰은 그 체인 위 프로그램이 발행한 자산을 가리킨다.

- ETH: Ethereum의 기본 자산. 가스비 지불과 프로토콜 경제에 사용한다.
- ERC-20 토큰: Ethereum 컨트랙트가 잔액과 전송 규칙을 관리한다.
- SOL: Solana의 기본 자산.
- SPL Token: Solana의 Token Program 규칙을 따르는 토큰이다.

일상에서는 둘을 넓게 묶어 토큰이라고 부르기도 한다. 기술 문서를 읽을 때는 기본 자산인지, 컨트랙트가 관리하는 자산인지 확인하면 된다.

## EIP, ERC, 숫자의 뜻

EIP는 Ethereum 개선 제안 문서다. 그중 애플리케이션 수준의 표준과 규약은 ERC 범주에 들어간다. EIP-20과 EIP-721의 숫자는 문서 식별 번호다. 순위, 버전, 같은 종류 표준의 순번을 뜻하지 않는다. 두 문서는 각각 대체 가능 토큰과 NFT의 공통 인터페이스를 정의한다.

표준의 목적은 호환성이다. 지갑과 거래소가 토큰마다 제각각인 함수 이름을 알아야 한다면 통합이 어렵다. 공통 인터페이스를 따르면 도구는 컨트랙트 주소와 ABI만으로 잔액을 조회하고 토큰을 전송한다.

## ERC-20 토큰

ERC-20은 대체 가능 토큰(fungible token) 표준이다. 같은 컨트랙트의 1단위는 다른 1단위와 구별하지 않는다. 포인트, 거버넌스 투표권, 결제 자산처럼 수량이 핵심인 모델에 맞는다.

주요 함수는 다음과 같다.

| 함수 | 역할 |
|---|---|
| `totalSupply()` | 전체 발행량 |
| `balanceOf(owner)` | 주소의 잔액 |
| `transfer(to, amount)` | 호출자 토큰 전송 |
| `approve(spender, amount)` | 제3자가 쓸 수 있는 한도 설정 |
| `allowance(owner, spender)` | 남은 사용 한도 조회 |
| `transferFrom(from, to, amount)` | 승인 한도 안에서 대신 전송 |

Vault나 DEX가 사용자의 토큰을 가져갈 때는 보통 두 트랜잭션이 필요하다.

```text
1. 사용자 → Token.approve(vault, 100)
2. 사용자 → Vault.deposit(100)
3. Vault → Token.transferFrom(user, vault, 100)
```

`approve`는 토큰을 바로 보내는 함수가 아니다. 지정한 `spender`에게 인출 권한을 준다. 무제한 승인을 남발하면 해당 컨트랙트가 해킹되거나 악성 업그레이드를 했을 때 지갑 잔액이 위험해진다.

`decimals`는 화면 표시 규칙이다. `decimals = 18`인 토큰의 컨트랙트가 `1_000_000_000_000_000_000`을 저장하면 UI가 1.0으로 보여 줄 수 있다. 컨트랙트 연산은 계속 정수로 한다.

## ERC-721 토큰

ERC-721은 대체 불가능 토큰(NFT) 표준이다. 각 자산은 컨트랙트 안에서 고유한 `tokenId`로 식별된다. 완전한 식별자는 보통 `(chain, contract address, tokenId)` 조합이다.

| 함수 | 역할 |
|---|---|
| `ownerOf(tokenId)` | 해당 NFT 소유자 |
| `balanceOf(owner)` | 주소가 가진 NFT 개수 |
| `safeTransferFrom(...)` | 수신 컨트랙트의 수용 가능 여부를 확인하며 전송 |
| `approve(to, tokenId)` | 특정 NFT 전송 권한 부여 |
| `setApprovalForAll(operator, approved)` | 소유자의 모든 NFT에 운영자 권한 설정 |
| `tokenURI(tokenId)` | 선택적 메타데이터 URI |

NFT가 이미지 자체를 온체인에 저장한다는 보장은 없다. `tokenURI`가 가리키는 JSON과 이미지는 외부 서버나 별도 스토리지에 있기도 한다. 컨트랙트 코드, URI 변경 권한, 저장 위치를 함께 봐야 지속성을 판단할 수 있다.

`safeTransferFrom`은 수신자가 컨트랙트일 때 ERC-721 수신 인터페이스를 구현했는지 확인한다. 단순 `transferFrom`으로 NFT를 받을 줄 모르는 컨트랙트에 보내면 자산이 잠길 수 있다.

## ERC-20과 ERC-721 비교

| 기준 | ERC-20 | ERC-721 |
|---|---|---|
| 식별 방식 | 주소별 수량 | 개별 `tokenId` |
| 대체 가능성 | 같은 단위끼리 대체 가능 | 각 토큰이 구별됨 |
| 대표 상태 | `balances[address]` | `owners[tokenId]` |
| 대표 용도 | 포인트, 결제, 거버넌스 | 수집품, 멤버십, 고유 권리 |

게임 아이템처럼 한 종류를 여러 개 발행하면서 종류별 ID도 필요한 경우에는 ERC-1155가 더 잘 맞을 수 있다. 표준은 유행이 아니라 자산 모델과 통합 요구에 맞춰 고른다.

## 직접 구현보다 OpenZeppelin

학습 목적으로 ERC-20을 직접 구현해 보는 것은 괜찮지만 제품 코드에서는 검토된 구현을 상속하거나 조합하는 편이 안전하다. OpenZeppelin Contracts 5.x로 발행 권한이 있는 토큰을 만들면 다음처럼 쓸 수 있다.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract CourseToken is ERC20, Ownable {
    constructor(address initialOwner)
        ERC20("Course Token", "COURSE")
        Ownable(initialOwner)
    {}

    function mint(address to, uint256 amount) external onlyOwner {
        _mint(to, amount);
    }
}
```

OpenZeppelin 5.x의 `Ownable` constructor는 `initialOwner`를 받는다. 예전 버전 예제를 그대로 복사하면 빌드가 깨질 수 있으므로 설치한 라이브러리 버전의 문서를 기준으로 코드를 맞춘다.

`owner` 한 명이 모든 권한을 갖는 구조가 맞지 않으면 `AccessControl`로 역할을 나눈다. 발행자, 일시정지 관리자, 업그레이드 관리자를 분리하고 각 역할의 이전·회수 절차까지 테스트해야 한다.

## 토큰을 설계할 때 먼저 정할 것

컨트랙트 코드보다 정책이 먼저다.

- 누가 발행하고 소각할 수 있는가?
- 공급량 상한이 있는가?
- 전송을 멈추거나 제한할 권한이 있는가?
- 관리자 키를 잃거나 탈취당하면 어떻게 되는가?
- 수수료 부과, 리베이스, 전송 시 차감 같은 비표준 동작이 있는가?
- 메타데이터와 업그레이드 권한을 누가 바꾸는가?

표준 인터페이스를 구현했다고 경제적 안전성까지 보장되는 것은 아니다. 표준은 호환성의 출발점일 뿐이다.
