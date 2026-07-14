# Rust로 이해하는 블록체인과 스마트 컨트랙트

특정 언어 경력을 전제하지 않는 한국어 mdBook입니다. 스마트 컨트랙트를 읽는 데 필요한 Rust와 블록체인 실행 모델을 짧은 예제와 실패 테스트로 연결합니다.

- 공개 사이트: https://mcprotein.github.io/blockchain-practice/
- 원문 목차: [`src/SUMMARY.md`](src/SUMMARY.md)
- 실행 가능한 Rust 실습: [`practice/mini-chain`](practice/mini-chain)

## 다루는 범위

1. Cargo, 소유권, `Result`, 구조체·열거형·트레이트
2. 해시 연결, 서명, 상태 전이, 합의의 역할
3. Ethereum 트랜잭션, EIP-1559, EVM, 가스
4. Solidity와 Foundry 테스트
5. ERC-20·ERC-721, OpenZeppelin, Token Vault 보안
6. Solana 계정·PDA·CPI와 Anchor

Alloy, Besu, 특정 서비스 아키텍처, 시점에 따라 금방 낡는 생태계 통계는 핵심 과정에서 제외했습니다. 필요한 주제는 마지막 장의 공식 문서와 다음 학습 순서에서 이어갑니다.

## 로컬 실행

Rust와 mdBook을 설치한 뒤 저장소 루트에서 실행합니다.

```bash
mdbook serve --open
```

정적 빌드와 문서 안 Rust 예제를 검사합니다.

```bash
mdbook build
mdbook test
```

미니 체인을 실행하고 테스트합니다.

```bash
cargo run -p mini-chain
cargo test --workspace
```

## 코드 블록의 검증 범위

- `rust`: `mdbook test`가 컴파일·실행하는 독립 예제
- `rust,compile_fail`: 실패해야 정상인 컴파일 예제
- `rust,ignore`: 외부 크레이트나 프로젝트 문맥이 필요한 읽기용 코드
- `solidity`, `bash`: 각 도구에서 따로 실행할 예제
- `text`: 출력, 구조, 개념도

`mdbook test` 통과는 `rust,ignore`, Solidity, shell 예제까지 실행했다는 뜻이 아닙니다. 저장소에 Foundry와 Anchor 프로젝트를 포함하지 않으므로 해당 예제는 공식 문서와 API 형태를 대조했지만 로컬 실행 검증 대상은 아닙니다.

## 검증 명령

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
mdbook build
mdbook test
git diff --check
```

`main` 브랜치에 push하면 GitHub Actions가 Rust 형식·lint·테스트와 mdBook 예제·빌드를 검사한 뒤 GitHub Pages에 배포합니다.
