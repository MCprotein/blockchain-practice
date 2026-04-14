# Rust + Blockchain 완전 정복

> 4년차 Node.js 백엔드 개발자를 위한 30일 Rust + 블록체인 집중 가이드북

[![Deploy mdBook to GitHub Pages](https://github.com/MCProtein/blockchain-practice/actions/workflows/deploy.yml/badge.svg)](https://github.com/MCProtein/blockchain-practice/actions/workflows/deploy.yml)

## 사이트

**https://mcprotein.github.io/blockchain-practice/**

## 프로젝트 소개

이 가이드북은 Node.js/TypeScript 백엔드 개발 경험이 있는 개발자가 **Rust 언어**와 **블록체인 개발**을 체계적으로 학습할 수 있도록 설계된 한국어 학습 자료입니다.

- TypeScript와의 비교를 통해 Rust 개념을 직관적으로 이해
- 블록체인 이론과 실습을 병행하는 4주 커리큘럼
- 매 주차마다 미니프로젝트로 실력 점검
- 실무에서 바로 사용할 수 있는 코드 예제

## 목차

### 1주차: Rust 기초 + 블록체인 첫걸음

- Rust 시작하기 (설치, 환경 구성, Cargo)
- 블록체인이란 무엇인가 (해시 함수, 암호학 기초)
- 소유권과 빌림 (소유권 규칙, 참조, 슬라이스)
- 블록과 체인 구조
- 구조체, 열거형, 패턴 매칭
- 합의 알고리즘
- **미니프로젝트**: Rust로 블록체인 만들기

### 2주차: Rust 심화 + 이더리움/Solidity

- 에러 처리 (panic!, Result, ? 연산자)
- 이더리움 아키텍처 (계정, 트랜잭션, EVM, 가스)
- 트레이트와 제네릭 (제네릭 타입, 트레이트, 수명)
- Solidity 기초 (타입, 함수, 매핑, 이벤트)
- Foundry 개발 환경 (빌드, 테스트, 배포)
- 토큰 표준과 OpenZeppelin (ERC-20, ERC-721)
- **미니프로젝트**: Token Vault

### 3주차: 비동기 Rust + Solana + 컨트랙트 심화

- 컬렉션과 이터레이터 (Vec, String, HashMap, 클로저)
- 스마트 컨트랙트 심화 (상속, 프록시 패턴, 보안)
- 비동기 프로그래밍 (async/await, Tokio, Arc/Mutex)
- Solana 아키텍처 (계정 모델, 프로그램, PDA/CPI)
- Anchor 프레임워크 (계정 검증, 테스트)
- **미니프로젝트**: Solana 토큰 프로그램

### 4주차: 실무 통합 + Platform 프로젝트

- Rust에서 Ethereum 연동 (Alloy, 트랜잭션 서명, sol! 매크로)
- 프라이빗 체인과 엔터프라이즈 (Hyperledger Besu)
- **미니프로젝트**: Mini Trace
- Platform 프로젝트 분석 (서비스 아키텍처, 블록체인 연동)

### 부록

- 블록체인 생태계 현황 (2026)
- Node.js에서 Rust로 전환 가이드
- 참고 자료 목록

## 로컬에서 실행하기

### 사전 준비

**Rust 설치 (rustup)**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**mdBook 설치**

```bash
cargo install mdbook
```

### 로컬 서버 실행

```bash
# 프로젝트 루트에서 실행
mdbook serve --open
```

브라우저가 자동으로 열리며 `http://localhost:3000`에서 가이드북을 확인할 수 있습니다. 파일을 수정하면 자동으로 새로고침됩니다.

### 빌드만 실행

```bash
mdbook build
```

빌드 결과물은 `book/` 디렉토리에 생성됩니다 (`.gitignore`로 제외됨).

## 배포 방법

이 프로젝트는 GitHub Actions를 통해 `main` 브랜치에 push할 때 자동으로 GitHub Pages에 배포됩니다.

### 자동 배포 흐름

1. `main` 브랜치에 push
2. GitHub Actions가 `.github/workflows/deploy.yml` 워크플로우를 실행
3. ubuntu-latest 환경에서 `mdbook build`로 정적 사이트 생성
4. `actions/upload-pages-artifact`로 빌드 결과물 업로드
5. `actions/deploy-pages`로 GitHub Pages에 자동 배포

### 최초 GitHub Pages 설정 (저장소 관리자용)

1. GitHub 저장소 > **Settings** > **Pages**로 이동
2. **Source**를 `GitHub Actions`로 설정
3. `main` 브랜치에 push하면 첫 배포가 시작됩니다

### 수동 배포 트리거

GitHub Actions 탭에서 `Deploy mdBook to GitHub Pages` 워크플로우를 선택하고 **Run workflow** 버튼을 클릭하면 수동으로 배포를 실행할 수 있습니다.

## 파일 구조

```
blockchain-practice/
├── book.toml              # mdBook 설정 (제목, 언어, 테마)
├── src/                   # 챕터 마크다운 파일
│   ├── SUMMARY.md         # 목차 (네비게이션 정의 — 필수)
│   ├── introduction.md    # 시작하기 전에
│   ├── ch01-*.md          # Rust 시작하기
│   ├── ch02-*.md          # 소유권과 빌림
│   ├── ch03-*.md          # 구조체, 열거형, 패턴 매칭
│   ├── ch04-*.md          # 에러 처리
│   ├── ch05-*.md          # 트레이트와 제네릭
│   ├── ch06-*.md          # 컬렉션과 이터레이터
│   ├── ch07-*.md          # 비동기 프로그래밍
│   ├── ch08-*.md          # 미니프로젝트: 블록체인
│   ├── ch09-*.md          # 블록체인 기초
│   ├── ch10-*.md          # 이더리움 아키텍처
│   ├── ch11-*.md          # Solidity 기초
│   ├── ch12-*.md          # Foundry
│   ├── ch13-*.md          # 토큰 표준
│   ├── ch14-*.md          # 스마트 컨트랙트 심화
│   ├── ch15-*.md          # 미니프로젝트: Token Vault
│   ├── ch16-*.md          # Solana 아키텍처
│   ├── ch17-*.md          # Anchor 프레임워크
│   ├── ch18-*.md          # 미니프로젝트: Solana 토큰
│   ├── ch19-*.md          # Rust + Ethereum (Alloy)
│   ├── ch20-*.md          # 프라이빗 체인
│   ├── ch21-*.md          # 미니프로젝트: Mini Trace
│   ├── ch22-*.md          # Platform 프로젝트 분석
│   └── appendix-*.md      # 부록
├── theme/
│   └── custom.css         # 커스텀 스타일 (글씨 크기, light 테마)
├── practice/              # 미니프로젝트 실습 코드 (Rust crate)
├── Cargo.toml             # Rust workspace 설정
└── .github/
    └── workflows/
        └── deploy.yml     # GitHub Pages 자동 배포 워크플로우
```

## 기여 방법

1. 저장소를 fork합니다
2. 새 브랜치를 생성합니다: `git checkout -b feature/챕터명`
3. `src/` 디렉토리에 새 챕터 파일을 추가합니다
   - 네이밍 규칙: `chXX-YY-name.md` (XX=챕터번호, YY=섹션번호)
4. `src/SUMMARY.md`에 챕터를 등록합니다 (등록하지 않으면 사이트에 표시되지 않음)
5. 로컬에서 `mdbook build`로 오류가 없는지 확인합니다
6. Pull Request를 생성합니다

### 작성 가이드라인

- 모든 내용은 **한국어**로 작성합니다
- 코드 예제는 **실행 가능**하게 작성합니다
- Node.js/TypeScript 개발자를 대상으로 비교 설명을 포함합니다
- 새 챕터는 반드시 `SUMMARY.md`에 등록해야 사이트에 표시됩니다

## 라이선스

이 프로젝트는 학습 목적으로 제작되었습니다.
