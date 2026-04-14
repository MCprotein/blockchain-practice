# CLAUDE.md — Rust + Blockchain 완전 정복

이 파일은 Claude Code AI 어시스턴트를 위한 프로젝트 지침서입니다.

## 프로젝트 개요

**mdBook 기반 한국어 학습 가이드북**으로, Node.js/TypeScript 백엔드 개발자(4년차 수준)가 Rust 언어와 블록체인 개발을 30일 커리큘럼으로 학습하는 자료입니다.

- 사이트: https://mcprotein.github.io/blockchain-practice/
- 언어: 한국어 (Korean)
- 빌드 도구: mdBook
- 배포: GitHub Actions → GitHub Pages (main 브랜치 push 시 자동)

## 빌드 명령어

```bash
# 정적 사이트 빌드 (출력: book/ 디렉토리)
mdbook build

# 로컬 개발 서버 (파일 변경 시 자동 새로고침)
mdbook serve --open

# 코드 블록 doctest 실행
mdbook test
```

빌드 결과물 `book/`은 `.gitignore`에 등록되어 있으므로 커밋하지 않습니다.

## 파일 구조

```
blockchain-practice/
├── book.toml              # mdBook 설정 (제목: "Rust + Blockchain 완전 정복", language: "ko")
├── src/                   # 모든 콘텐츠 마크다운 파일
│   ├── SUMMARY.md         # 목차 — 네비게이션의 단일 진실 공급원 (SSoT)
│   ├── introduction.md    # 서문
│   ├── ch01-XX-*.md       # 1장: Rust 시작하기
│   ├── ch02-XX-*.md       # 2장: 소유권과 빌림
│   ├── ch03-XX-*.md       # 3장: 구조체, 열거형, 패턴 매칭
│   ├── ch04-XX-*.md       # 4장: 에러 처리
│   ├── ch05-XX-*.md       # 5장: 트레이트와 제네릭
│   ├── ch06-XX-*.md       # 6장: 컬렉션과 이터레이터
│   ├── ch07-XX-*.md       # 7장: 비동기 프로그래밍
│   ├── ch08-XX-*.md       # 미니프로젝트: Rust 블록체인
│   ├── ch09-XX-*.md       # 9장: 블록체인 기초
│   ├── ch10-XX-*.md       # 10장: 이더리움 아키텍처
│   ├── ch11-XX-*.md       # 11장: Solidity 기초
│   ├── ch12-XX-*.md       # 12장: Foundry
│   ├── ch13-XX-*.md       # 13장: 토큰 표준 (ERC-20, ERC-721)
│   ├── ch14-XX-*.md       # 14장: 스마트 컨트랙트 심화
│   ├── ch15-XX-*.md       # 미니프로젝트: Token Vault
│   ├── ch16-XX-*.md       # 16장: Solana 아키텍처
│   ├── ch17-XX-*.md       # 17장: Anchor 프레임워크
│   ├── ch18-XX-*.md       # 미니프로젝트: Solana 토큰 프로그램
│   ├── ch19-XX-*.md       # 19장: Rust + Ethereum (Alloy)
│   ├── ch20-XX-*.md       # 20장: 프라이빗 체인 (Hyperledger Besu)
│   ├── ch21-XX-*.md       # 미니프로젝트: Mini Trace
│   ├── ch22-XX-*.md       # 22장: Platform 프로젝트 분석
│   └── appendix-*.md      # 부록 (생태계, Node→Rust 전환, 참고자료)
├── theme/
│   └── custom.css         # 커스텀 CSS (light 테마, 글씨 크기 조정)
├── practice/              # 미니프로젝트 실습 Rust 코드
├── Cargo.toml             # Rust workspace
└── .github/
    └── workflows/
        └── deploy.yml     # GitHub Actions 자동 배포
```

## 챕터 네이밍 규칙

파일명 형식: `chXX-YY-name.md`

- `XX` = 챕터 번호 (두 자리, 예: `01`, `09`, `22`)
- `YY` = 섹션 번호 (두 자리, 예: `00`=챕터 인트로, `01`=첫 번째 섹션)
- `name` = 영문 소문자, 하이픈 구분 (예: `getting-started`, `ownership-rules`)

예시:
- `ch01-00-getting-started.md` — 1장 인트로
- `ch01-01-installation.md` — 1장 1절
- `ch09-02-blocks-and-chain.md` — 9장 2절

부록: `appendix-a-ecosystem.md`, `appendix-b-nodejs-to-rust.md`, `appendix-c-references.md`

## SUMMARY.md 관리 규칙

`src/SUMMARY.md`는 mdBook의 목차이자 네비게이션을 정의하는 핵심 파일입니다.

**새 챕터를 추가할 때 반드시 SUMMARY.md에도 등록해야 합니다.** 등록하지 않으면 빌드는 성공하지만 사이트에 해당 챕터가 표시되지 않습니다.

SUMMARY.md 항목 형식:
```markdown
- [챕터 제목](./파일명.md)
  - [섹션 제목](./파일명.md)
```

## 콘텐츠 작성 지침

### 언어

- 모든 내용은 **한국어**로 작성합니다
- 코드, 변수명, 명령어는 영문 그대로 사용합니다
- 기술 용어는 한국어 설명과 함께 영문 원어를 병기합니다 (예: 소유권(Ownership))

### 코드 예제

- 코드 예제는 반드시 **실행 가능**하게 작성합니다
- Rust 예제는 `mdbook test`로 검증 가능한 형태를 권장합니다
- 언어 태그를 명시합니다: ` ```rust `, ` ```solidity `, ` ```typescript `

### Node.js 개발자 대상 비교

이 가이드북의 핵심 독자는 **4년차 Node.js/NestJS 백엔드 개발자**입니다. 새 개념을 설명할 때 TypeScript/Node.js와의 비교를 포함합니다.

비교 예시 패턴:
```markdown
## 소유권 (Ownership)

TypeScript에서는 변수를 여러 곳에서 자유롭게 참조할 수 있지만,
Rust에서는 값의 소유자가 하나뿐이라는 규칙이 있습니다.

```typescript
// TypeScript: 참조 자유
const a = { name: "Alice" };
const b = a; // 같은 객체를 b도 참조
```

```rust
// Rust: 소유권 이동
let a = String::from("Alice");
let b = a; // a의 소유권이 b로 이동
// println!("{}", a); // 컴파일 에러!
```
```

주요 비교 대상:
- TypeScript 타입 시스템 ↔ Rust 타입 시스템
- async/await (Node.js) ↔ async/await + Tokio (Rust)
- try/catch ↔ Result<T, E>
- interface/class ↔ struct + trait
- npm/package.json ↔ cargo/Cargo.toml
- NestJS 의존성 주입 ↔ Rust 모듈 시스템

## 배포

GitHub Actions 워크플로우 (`.github/workflows/deploy.yml`):
- 트리거: `main` 브랜치 push 또는 수동 실행(`workflow_dispatch`)
- 실행 환경: `ubuntu-latest`
- 단계: checkout → mdBook 설치 (`peaceiris/actions-mdbook@v2`) → `mdbook build` → Pages 업로드 → 배포

GitHub Pages 설정: Settings > Pages > Source를 `GitHub Actions`로 설정해야 합니다.

## book.toml 주요 설정

```toml
[book]
title = "Rust + Blockchain 완전 정복"
language = "ko"
src = "src"

[build]
build-dir = "book"

[output.html]
default-theme = "light"
preferred-dark-theme = "light"
additional-css = ["theme/custom.css"]
```

테마 변경이나 새 CSS 추가 시 `theme/custom.css`를 수정합니다.

## practice/ 디렉토리

`practice/` 디렉토리는 가이드북의 미니프로젝트 실습 코드를 담는 Rust workspace입니다.

- 각 미니프로젝트는 별도 crate로 구성합니다
- `Cargo.toml` (루트)에 workspace 멤버로 등록합니다
- 가이드북 챕터에서 이 코드를 참조할 때 경로를 명시합니다

## 주의사항

- `book/` 디렉토리는 빌드 산출물이므로 커밋하지 않습니다 (`.gitignore` 등록됨)
- `.omc/` 디렉토리는 AI 에이전트 상태 파일이므로 커밋하지 않습니다
- `SUMMARY.md`에 없는 파일은 빌드 후 사이트에 노출되지 않습니다
- 챕터 파일을 삭제하거나 이름을 바꿀 때 `SUMMARY.md`도 함께 수정합니다
