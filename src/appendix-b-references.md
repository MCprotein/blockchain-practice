# 공식 문서와 다음 학습 순서

라이브러리와 체인 도구는 빠르게 바뀐다. 블로그 예제보다 공식 문서와 설치한 버전의 API를 우선한다.

## 이 책의 기준 자료

### Rust

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust 표준 라이브러리 문서](https://doc.rust-lang.org/std/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

### Ethereum과 Solidity

- [Ethereum 개발자 문서](https://ethereum.org/developers/docs/)
- [Ethereum 트랜잭션](https://ethereum.org/developers/docs/transactions/)
- [EVM](https://ethereum.org/developers/docs/evm/)
- [가스와 수수료](https://ethereum.org/developers/docs/gas/)
- [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
- [Solidity 문서](https://docs.soliditylang.org/)
- [Solidity 보안 고려사항](https://docs.soliditylang.org/en/latest/security-considerations.html)

### 도구와 표준

- [Foundry Book](https://getfoundry.sh/)
- [OpenZeppelin Contracts 5.x](https://docs.openzeppelin.com/contracts/5.x/)
- [ERC-20 명세](https://eips.ethereum.org/EIPS/eip-20)
- [ERC-721 명세](https://eips.ethereum.org/EIPS/eip-721)

### Solana와 Anchor

- [Solana 핵심 개념](https://solana.com/docs/core)
- [Solana 계정](https://solana.com/docs/core/accounts)
- [PDA](https://solana.com/docs/core/pda)
- [CPI](https://solana.com/docs/core/cpi)
- [Anchor 문서](https://www.anchor-lang.com/docs)
- [Anchor 설치](https://www.anchor-lang.com/docs/installation)
- [Anchor 1.0 릴리스 노트](https://www.anchor-lang.com/docs/updates/release-notes/1-0-0)
- [LiteSVM 테스트](https://www.anchor-lang.com/docs/testing/litesvm)

## 다음 학습 순서

이 책의 예제를 끝냈다면 관심 분야에 따라 결과물 하나를 정한다.

- Ethereum: Token Vault에 권한 모델과 공격용 mock을 추가하고 Foundry invariant test로 장부 조건을 검증한다.
- Solana: PDA Counter나 Token Vault를 만들고 LiteSVM에서 정상 경로, 잘못된 서명자, 잘못된 시드를 테스트한다.
- 프로토콜: 미니 체인의 서명, 상태 저장, 합의를 서로 분리해 구현하고 각 계층이 보장하지 않는 것도 문서화한다.

어느 갈래든 공통 원칙은 같다. 코드를 배포하기 전에 불변식을 문장으로 쓰고 실패 경로를 테스트하고 관리자와 업그레이드 권한을 문서화한다.
