# 목차

[시작하기 전에](./introduction.md)

---

# Part 1: Rust 기초

- [Rust 시작하기](./ch01-00-getting-started.md)
  - [설치와 환경 구성](./ch01-01-installation.md)
  - [Hello, Cargo!](./ch01-02-hello-cargo.md)
- [소유권과 빌림](./ch02-00-ownership.md)
  - [소유권 규칙](./ch02-01-ownership-rules.md)
  - [참조와 빌림](./ch02-02-references.md)
  - [슬라이스](./ch02-03-slices.md)
- [구조체, 열거형, 패턴 매칭](./ch03-00-structs-enums.md)
  - [구조체 정의와 사용](./ch03-01-structs.md)
  - [열거형과 Option](./ch03-02-enums.md)
  - [match와 패턴 매칭](./ch03-03-pattern-matching.md)
- [에러 처리](./ch04-00-error-handling.md)
  - [panic!과 복구 불가능한 에러](./ch04-01-panic.md)
  - [Result와 복구 가능한 에러](./ch04-02-result.md)
  - [에러 전파와 ? 연산자](./ch04-03-propagation.md)
- [트레이트와 제네릭](./ch05-00-traits-generics.md)
  - [제네릭 타입](./ch05-01-generics.md)
  - [트레이트 정의와 구현](./ch05-02-traits.md)
  - [수명(Lifetime)](./ch05-03-lifetimes.md)
- [컬렉션과 이터레이터](./ch06-00-collections.md)
  - [Vec, String, HashMap](./ch06-01-common-collections.md)
  - [클로저](./ch06-02-closures.md)
  - [이터레이터](./ch06-03-iterators.md)
- [비동기 프로그래밍](./ch07-00-async.md)
  - [async/await 기초](./ch07-01-async-await.md)
  - [Tokio 런타임](./ch07-02-tokio.md)
  - [Arc, Mutex와 공유 상태](./ch07-03-shared-state.md)
- [미니프로젝트: Rust로 블록체인 만들기](./ch08-00-project-blockchain.md)

---

# Part 2: 블록체인 기초

- [블록체인이란 무엇인가](./ch09-00-what-is-blockchain.md)
  - [해시 함수와 암호학 기초](./ch09-01-hash-cryptography.md)
  - [블록과 체인 구조](./ch09-02-blocks-and-chain.md)
  - [합의 알고리즘](./ch09-03-consensus.md)
- [이더리움 아키텍처](./ch10-00-ethereum.md)
  - [계정과 트랜잭션](./ch10-01-accounts-transactions.md)
  - [EVM과 가스](./ch10-02-evm-gas.md)
  - [스마트 컨트랙트 개요](./ch10-03-smart-contracts-overview.md)

---

# Part 3: Solidity와 스마트 컨트랙트

- [Solidity 기초](./ch11-00-solidity-basics.md)
  - [타입과 변수](./ch11-01-types-variables.md)
  - [함수와 제어자](./ch11-02-functions-modifiers.md)
  - [매핑, 이벤트, 에러 처리](./ch11-03-mappings-events-errors.md)
- [Foundry 개발 환경](./ch12-00-foundry.md)
  - [프로젝트 구조와 빌드](./ch12-01-project-structure.md)
  - [테스트 작성](./ch12-02-testing.md)
  - [배포와 스크립트](./ch12-03-deployment.md)
- [토큰 표준과 OpenZeppelin](./ch13-00-tokens.md)
  - [ERC-20 토큰](./ch13-01-erc20.md)
  - [ERC-721 NFT](./ch13-02-erc721.md)
  - [OpenZeppelin 활용](./ch13-03-openzeppelin.md)
- [스마트 컨트랙트 심화](./ch14-00-advanced-contracts.md)
  - [상속과 인터페이스](./ch14-01-inheritance.md)
  - [프록시 패턴과 업그레이드](./ch14-02-proxy-patterns.md)
  - [보안과 일반적인 취약점](./ch14-03-security.md)
- [미니프로젝트: Token Vault](./ch15-00-project-token-vault.md)

---

# Part 4: Solana와 Rust 블록체인

- [Solana 아키텍처](./ch16-00-solana-architecture.md)
  - [계정 모델](./ch16-01-account-model.md)
  - [프로그램과 명령어](./ch16-02-programs-instructions.md)
  - [PDA와 CPI](./ch16-03-pda-cpi.md)
- [Anchor 프레임워크](./ch17-00-anchor.md)
  - [프로젝트 구조](./ch17-01-project-structure.md)
  - [계정 검증과 제약 조건](./ch17-02-account-validation.md)
  - [테스트 작성 (TypeScript)](./ch17-03-testing.md)
- [미니프로젝트: Solana 토큰 프로그램](./ch18-00-project-sol-token.md)

---

# Part 5: 실무 통합

- [Rust에서 Ethereum 연동 (Alloy)](./ch19-00-alloy.md)
  - [프로바이더와 컨트랙트 호출](./ch19-01-provider-calls.md)
  - [트랜잭션 서명과 전송](./ch19-02-signing-transactions.md)
  - [sol! 매크로와 ABI](./ch19-03-sol-macro.md)
- [프라이빗 체인과 엔터프라이즈](./ch20-00-private-chains.md)
  - [Hyperledger Besu](./ch20-01-besu.md)
  - [퍼블릭 vs 프라이빗](./ch20-02-public-vs-private.md)
- [미니프로젝트: Mini Trace](./ch21-00-project-mini-trace.md)
- [Platform 프로젝트 분석](./ch22-00-platform-analysis.md)
  - [서비스 아키텍처](./ch22-01-service-architecture.md)
  - [블록체인 연동 흐름](./ch22-02-blockchain-integration.md)
  - [코드 리딩 가이드](./ch22-03-code-reading.md)

---

# 부록

- [블록체인 생태계 현황 (2026)](./appendix-a-ecosystem.md)
- [Node.js에서 Rust로 전환 가이드](./appendix-b-nodejs-to-rust.md)
- [참고 자료 목록](./appendix-c-references.md)
