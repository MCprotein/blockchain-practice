# 부록 C: 전체 참고 자료

이 교재에서 다룬 모든 기술의 공식 문서와 학습 자료를 카테고리별로 정리했다. 각 자료에 URL, 설명, 난이도를 표시한다.

난이도 기준:
- 🟢 **입문**: 프로그래밍 경험만 있으면 시작 가능
- 🟡 **중급**: 해당 분야 기초 지식 필요
- 🔴 **고급**: 깊은 이해와 경험 필요

---

## 1. Rust 학습

### 공식 자료

**The Rust Programming Language (The Book)**
- URL: https://doc.rust-lang.org/book/
- 한국어판: https://rinthel.github.io/rust-lang-book-ko/
- 설명: Rust 공식 교재. 소유권, 빌림, 트레이트, 에러 처리 등 핵심 개념을 순서대로 다룬다. 어떤 Rust 학습 자료보다 이것을 먼저 읽어야 한다.
- 난이도: 🟢 입문
- 분량: 20장 + 부록, 약 30-50시간

**Rustlings**
- URL: https://github.com/rust-lang/rustlings
- 설명: 터미널에서 실행하는 인터랙티브 Rust 연습 문제 모음. 컴파일 에러를 고치면서 Rust를 익힌다. The Book과 병행하면 효과적이다.
- 난이도: 🟢 입문
- 분량: 약 100개 문제, 10-20시간

**Rust by Example**
- URL: https://doc.rust-lang.org/rust-by-example/
- 설명: 실행 가능한 예제 코드로 Rust를 배운다. The Book이 설명 중심이라면, 이것은 코드 중심. 특정 문법이나 기능을 빠르게 확인할 때 유용하다.
- 난이도: 🟢 입문
- 분량: 필요한 부분만 참고

**Tokio 공식 튜토리얼**
- URL: https://tokio.rs/tokio/tutorial
- 설명: Rust 비동기 런타임 Tokio의 공식 튜토리얼. async/await, task, channel, select!, 네트워킹을 직접 구현하며 배운다. platform 코드를 이해하려면 반드시 이수해야 한다.
- 난이도: 🟡 중급
- 분량: 8개 챕터, 10-15시간

### 심화 자료

**corrode.dev - Rust for TypeScript Developers**
- URL: https://corrode.dev/blog/
- 설명: TypeScript/JavaScript 개발자를 위한 Rust 전환 가이드. 이 교재의 부록 B 내용이 여기서 영감을 받았다. 실무적인 조언이 많다.
- 난이도: 🟡 중급

**Rust Async Book**
- URL: https://rust-lang.github.io/async-book/
- 설명: Rust 비동기 프로그래밍의 공식 가이드. Future, executor, waker 등 내부 동작 원리까지 설명한다.
- 난이도: 🔴 고급

**The Rust Performance Book**
- URL: https://nnethercote.github.io/perf-book/
- 설명: Rust 코드 성능 최적화 가이드. 프로파일링, 불필요한 할당 제거, SIMD 등을 다룬다.
- 난이도: 🔴 고급

**Zero To Production In Rust**
- URL: https://www.zero2prod.com/
- 설명: Rust로 실제 프로덕션 웹 서비스를 처음부터 만드는 책. 테스트, CI/CD, 로깅, 보안까지 다룬다. 유료이지만 가장 실용적인 Rust 웹 서비스 교재.
- 난이도: 🟡 중급

---

## 2. 블록체인 기초

**ethereum.org 개발자 문서**
- URL: https://ethereum.org/developers
- 설명: 이더리움 재단이 관리하는 공식 개발자 포털. 블록체인 기초 개념, 이더리움 아키텍처, 스마트 컨트랙트 소개가 잘 정리되어 있다.
- 난이도: 🟢 입문

**Cyfrin Updraft**
- URL: https://updraft.cyfrin.io/
- 설명: Patrick Collins가 만든 무료 블록체인 개발 교육 플랫폼. Solidity 기초부터 스마트 컨트랙트 보안, 고급 DeFi까지 체계적인 커리큘럼을 무료로 제공한다. 이 분야 최고의 무료 학습 자료 중 하나.
- 난이도: 🟢 입문 ~ 🟡 중급
- 특이사항: 영어, 무료, 수료증 발급

**CryptoZombies**
- URL: https://cryptozombies.io/
- 설명: 게임 스토리로 Solidity를 배우는 인터랙티브 튜토리얼. 좀비 게임을 만들면서 ERC-20, ERC-721을 구현한다. 재미있게 시작하기 좋다.
- 난이도: 🟢 입문
- 분량: 6개 레슨, 10-15시간

**Alchemy University**
- URL: https://university.alchemy.com/
- 설명: Alchemy(블록체인 인프라 회사)가 운영하는 무료 교육 과정. Ethereum Developer Bootcamp, JavaScript 블록체인 개발 등 실용적인 과정을 무료로 제공한다.
- 난이도: 🟢 입문 ~ 🟡 중급

**Mastering Ethereum (책)**
- URL: https://github.com/ethereumbook/ethereumbook (무료 온라인)
- 설명: Andreas Antonopoulos와 Gavin Wood가 쓴 이더리움 기술서. EVM 내부 구조, 암호학 기초, 보안까지 깊이 있게 다룬다. 이더리움을 진지하게 공부하고 싶다면 필독.
- 난이도: 🟡 중급 ~ 🔴 고급

---

## 3. Solidity / EVM

**Solidity by Example**
- URL: https://solidity-by-example.org/
- 설명: 실제 작동하는 Solidity 예제 코드 모음. Hello World부터 DeFi 프로토콜까지 다양한 패턴을 코드로 보여준다. 특정 기능을 빠르게 확인할 때 최고의 참고자료.
- 난이도: 🟢 입문 ~ 🟡 중급

**Foundry Book**
- URL: https://book.getfoundry.sh/
- 설명: Rust로 만든 Solidity 개발 도구 Foundry의 공식 문서. forge 테스트 작성, fuzz 테스트, invariant 테스트, cast 도구 사용법이 상세히 나와 있다. platform 개발에 직접 필요한 자료.
- 난이도: 🟡 중급

**OpenZeppelin Contracts**
- URL: https://docs.openzeppelin.com/contracts/
- GitHub: https://github.com/OpenZeppelin/openzeppelin-contracts
- 설명: 감사받은 스마트 컨트랙트 라이브러리. ERC-20, ERC-721, Access Control, UUPS Proxy, Ownable 등 platform에서 사용하는 패턴의 원본 구현이다.
- 난이도: 🟡 중급

**evm.codes**
- URL: https://www.evm.codes/
- 설명: EVM 옵코드 레퍼런스. PUSH, ADD, SLOAD 등 모든 EVM 명령어의 gas 비용, 스택 효과, 설명이 있다. UUPS 프록시의 DELEGATECALL이나 storage slot 계산을 이해하려면 참고해야 한다.
- 난이도: 🔴 고급

**EVM from Scratch**
- URL: https://evm.codes/playground
- 설명: EVM 바이트코드를 직접 실행해볼 수 있는 플레이그라운드. 스마트 컨트랙트가 어떻게 컴파일되고 실행되는지 실제로 보여준다.
- 난이도: 🔴 고급

**Solidity 공식 문서**
- URL: https://docs.soliditylang.org/
- 설명: Solidity 언어의 완전한 레퍼런스. 문법, 내장 함수, 타입 시스템 등 의문이 생길 때 최종 참고자료.
- 난이도: 🟡 중급

---

## 4. Alloy / Ethereum Rust

**Alloy 공식 문서**
- URL: https://alloy.rs/
- GitHub: https://github.com/alloy-rs/alloy
- 설명: Alloy 라이브러리 공식 문서. Provider, Signer, sol! 매크로, Network 추상화 등 모든 API가 문서화되어 있다.
- 난이도: 🟡 중급

**Alloy 예제 코드**
- URL: https://github.com/alloy-rs/examples
- 설명: Alloy 공식 예제 저장소. Provider 사용, 트랜잭션 전송, 컨트랙트 배포, 이벤트 구독 등 다양한 실용 예제가 있다. 새 기능을 사용할 때 여기서 먼저 찾아보라.
- 난이도: 🟡 중급

**Reth (Rust Ethereum)**
- URL: https://reth.rs/
- GitHub: https://github.com/paradigmxyz/reth
- 설명: Paradigm이 개발한 Rust 기반 이더리움 실행 클라이언트. Alloy와 같은 팀이 만들어 긴밀하게 통합된다. 이더리움 내부 구조를 Rust 코드로 이해하는 데 좋다.
- 난이도: 🔴 고급

---

## 5. Solana

**Solana 개발자 포털**
- URL: https://solana.com/developers
- 설명: Solana 공식 개발자 시작 지점. 문서, 튜토리얼, 예제 코드, SDK 링크가 모여 있다.
- 난이도: 🟢 입문

**Solana 개발자 문서**
- URL: https://docs.solanalabs.com/
- 설명: Solana 프로토콜과 클라이언트의 공식 기술 문서. Proof of History, 계정 모델, 프로그램 구조 등.
- 난이도: 🟡 중급

**Solana Cookbook**
- URL: https://solanacookbook.com/
- 설명: 실용적인 Solana 개발 패턴 모음. "이걸 어떻게 하지?"에 대한 답을 코드 예제로 보여준다. TypeScript(web3.js), Rust(Anchor) 예제 모두 있다.
- 난이도: 🟡 중급

**Anchor 프레임워크 문서**
- URL: https://www.anchor-lang.com/
- GitHub: https://github.com/coral-xyz/anchor
- 설명: Solana 스마트 컨트랙트(Program) 개발을 쉽게 만드는 프레임워크. Rust로 Solana Program을 작성할 때 사실상 표준. 매크로로 보일러플레이트를 크게 줄여준다.
- 난이도: 🟡 중급

**Helius Solana 가이드**
- URL: https://www.helius.dev/blog
- 설명: Helius(Solana 인프라 회사) 블로그. Solana의 고급 개념을 깊이 있게 설명하는 기술 글이 많다. 특히 Firedancer, 합의, 성능 최적화 관련 글이 훌륭하다.
- 난이도: 🟡 중급 ~ 🔴 고급

---

## 6. Hyperledger Besu

**Hyperledger Besu 공식 문서**
- URL: https://besu.hyperledger.org/
- 설명: Besu의 완전한 공식 문서. 설치, 설정, IBFT 2.0, QBFT, 프라이빗 트랜잭션, 권한 관리 등 모든 내용이 있다.
- 난이도: 🟡 중급

**Besu GitHub**
- URL: https://github.com/hyperledger/besu
- 설명: Besu 소스 코드와 이슈 트래커. 버그 보고나 특정 동작의 원인을 이해하려면 여기를 확인한다.
- 난이도: 🔴 고급

**Besu 네트워크 빠른 시작**
- URL: https://besu.hyperledger.org/private-networks/tutorials/ibft
- 설명: IBFT 2.0 프라이빗 네트워크를 처음 구성하는 단계별 가이드.
- 난이도: 🟡 중급

---

## 7. 심화 / 전문가 과정

**Blockchain from Scratch**
- URL: https://github.com/anders94/blockchain-A-to-Z
- 설명: 블록체인을 처음부터 직접 구현한다. 해싱, 체인 구조, P2P 네트워크, PoW를 코드로 구현하며 블록체인의 본질을 이해한다.
- 난이도: 🔴 고급

**EVM from Scratch**
- URL: https://github.com/w1nt3r-eth/evm-from-scratch
- 설명: 순수 코드로 미니 EVM을 구현하는 과제. PUSH, ADD, SSTORE, CALL 등 옵코드를 직접 구현한다. EVM 내부 동작을 가장 확실하게 이해하는 방법.
- 난이도: 🔴 고급

**Polkadot Blockchain Academy (PBA)**
- URL: https://polkadot.com/blockchain-academy
- 설명: Polkadot 생태계가 운영하는 블록체인 심화 교육. 암호학 기초, 합의 알고리즘, 경제학, Substrate 개발을 전문가 수준으로 가르친다. 선발 과정이 있으며, 수료 후 Polkadot 생태계 취업 연계.
- 난이도: 🔴 고급
- 형태: 집중 부트캠프 (현장)

**DeFi 보안 - Secureum**
- URL: https://secureum.xyz/
- 설명: 스마트 컨트랙트 보안 전문가 양성 교육. Epoch0부터 CARE4 시리즈까지 무료로 제공한다. 보안 감사(audit) 분야로 진출하고 싶다면 필수.
- 난이도: 🔴 고급

---

## 8. 도구 및 인프라

**SQLx 문서**
- URL: https://docs.rs/sqlx/latest/sqlx/
- GitHub: https://github.com/launchbadge/sqlx
- 설명: Rust용 비동기 SQL 라이브러리. platform이 사용하는 ORM/쿼리 빌더. 마이그레이션, 컴파일 타임 쿼리 검증, PostgreSQL/MySQL/SQLite 지원.
- 난이도: 🟡 중급

**Axum 문서**
- URL: https://docs.rs/axum/latest/axum/
- GitHub: https://github.com/tokio-rs/axum
- 설명: Axum 공식 API 문서. Extractor, Router, 미들웨어 등 모든 API가 코드 예제와 함께 설명된다.
- 난이도: 🟡 중급

**Tower 미들웨어**
- URL: https://docs.rs/tower/latest/tower/
- 설명: Axum이 내부적으로 사용하는 미들웨어 프레임워크. 커스텀 Layer, Service를 만들 때 참고.
- 난이도: 🔴 고급

**tracing 크레이트**
- URL: https://docs.rs/tracing/latest/tracing/
- 설명: Rust용 구조화된 로깅 프레임워크. platform 전체에서 사용하는 `tracing::info!`, `tracing::error!` 등의 공식 문서.
- 난이도: 🟡 중급

**Docker 공식 문서**
- URL: https://docs.docker.com/
- 설명: platform과 Besu 네트워크를 컨테이너로 운영하는 데 필요한 Docker/Compose 레퍼런스.
- 난이도: 🟢 입문 ~ 🟡 중급

---

## 9. 커뮤니티와 뉴스레터

**Rust 공식 포럼**
- URL: https://users.rust-lang.org/
- 설명: Rust 사용자 포럼. 질문, 토론, 프로젝트 공유. 막히는 부분이 있을 때 검색하거나 질문하면 커뮤니티가 친절하게 답변한다.

**Ethereum Research**
- URL: https://ethresear.ch/
- 설명: 이더리움 연구자들의 기술 논의 포럼. EIP 초안, 프로토콜 개선 제안, 암호학 연구가 올라온다.

**Week in Ethereum News**
- URL: https://weekinethereumnews.com/
- 설명: 매주 이더리움 생태계의 중요 뉴스와 개발 소식을 정리하는 뉴스레터. 생태계 동향을 파악하는 가장 효율적인 방법.

**Solana Dev Newsletter**
- URL: https://solana.com/news (또는 Helius 블로그)
- 설명: Solana 개발 소식과 생태계 업데이트.

---

## 학습 경로 추천

### 이 교재를 끝낸 독자를 위한 다음 단계

**Rust 백엔드 개발자 (3개월)**
```
월 1: Zero To Production In Rust (웹 서비스 전체 스택)
월 2: Tokio 심화 + 비동기 패턴 고급
월 3: platform에 새 기능 추가 (실전)
```

**블록체인 개발자 (3개월)**
```
월 1: Cyfrin Updraft 전체 과정 + Foundry 테스트 작성
월 2: OpenZeppelin 컨트랙트 분석 + DeFi 프로토콜 이해
월 3: 스마트 컨트랙트 보안 (Secureum)
```

**Solana 전문가 (3개월)**
```
월 1: Anchor 튜토리얼 전체 + 기본 Program 구현
월 2: Token Program, Metaplex NFT 이해
월 3: Firedancer 아키텍처 이해 + 고성능 Program 최적화
```

**풀스택 블록체인 엔지니어 (6개월)**
```
모두 포함 + Polkadot Blockchain Academy 지원
```

---

## 참고 자료 요약표

| 카테고리 | 자료명 | URL | 난이도 | 무료 |
|---------|-------|-----|-------|------|
| Rust | The Book | doc.rust-lang.org/book | 🟢 | ✅ |
| Rust | Rustlings | github.com/rust-lang/rustlings | 🟢 | ✅ |
| Rust | Tokio Tutorial | tokio.rs/tokio/tutorial | 🟡 | ✅ |
| Rust | Zero To Production | zero2prod.com | 🟡 | ❌ |
| 블록체인 | ethereum.org | ethereum.org/developers | 🟢 | ✅ |
| 블록체인 | Cyfrin Updraft | updraft.cyfrin.io | 🟢 | ✅ |
| 블록체인 | Mastering Ethereum | github.com/ethereumbook | 🟡 | ✅ |
| Solidity | Solidity by Example | solidity-by-example.org | 🟢 | ✅ |
| Solidity | Foundry Book | book.getfoundry.sh | 🟡 | ✅ |
| Solidity | OpenZeppelin | docs.openzeppelin.com | 🟡 | ✅ |
| EVM | evm.codes | evm.codes | 🔴 | ✅ |
| Alloy | Alloy Docs | alloy.rs | 🟡 | ✅ |
| Solana | Anchor Docs | anchor-lang.com | 🟡 | ✅ |
| Solana | Solana Cookbook | solanacookbook.com | 🟡 | ✅ |
| Besu | Besu Docs | besu.hyperledger.org | 🟡 | ✅ |
| 심화 | Secureum | secureum.xyz | 🔴 | ✅ |
| 심화 | PBA | polkadot.com/blockchain-academy | 🔴 | 선발 |

이 부록의 모든 URL은 2026년 4월 기준으로 유효하다. 빠르게 변하는 블록체인 생태계 특성상 일부 링크는 변경될 수 있다. 검색 엔진에서 자료명으로 검색하면 최신 버전을 찾을 수 있다.
