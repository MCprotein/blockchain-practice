# Chapter 13: 토큰 표준

## 토큰이란 무엇인가

블록체인에서 "토큰(token)"은 스마트 컨트랙트가 관리하는 디지털 자산이다. 실물 화폐, 주식, 쿠폰, 게임 아이템, 예술 작품 등 어떤 가치도 토큰으로 표현할 수 있다.

**토큰과 코인의 차이:**

| | 코인 (Coin) | 토큰 (Token) |
|--|------------|-------------|
| 예시 | ETH, BTC | USDC, UNI, BAYC |
| 존재 방식 | 블록체인 자체에 내장 | 스마트 컨트랙트로 구현 |
| 전송 방식 | 네트워크 프로토콜 | 컨트랙트 함수 호출 |
| 발행 주체 | 프로토콜 | 개발자/팀 |
| 저장 위치 | 체인 네이티브 상태 | 컨트랙트 mapping |

ETH는 이더리움 블록체인의 네이티브 코인이다. 반면 USDC(달러 스테이블코인)는 이더리움 위에서 스마트 컨트랙트로 구현된 토큰이다. USDC를 전송할 때 실제로는 USDC 컨트랙트의 `transfer()` 함수를 호출하는 것이다.

```solidity
// ETH 전송: 프로토콜 레벨 (네이티브)
payable(recipient).transfer(1 ether);

// USDC 전송: 컨트랙트 함수 호출
IERC20(USDC_ADDRESS).transfer(recipient, 1_000_000); // USDC는 decimals=6
```

**Node.js 비유:** 코인은 데이터베이스 자체의 기본 기능(예: PostgreSQL의 기본 데이터 타입)이고, 토큰은 그 위에서 애플리케이션 레이어로 구현한 기능(예: 애플리케이션의 포인트 시스템)이다.

```typescript
// Node.js 포인트 시스템 (토큰의 중앙화 버전)
class PointSystem {
    private balances = new Map<string, number>();

    transfer(from: string, to: string, amount: number) {
        const fromBalance = this.balances.get(from) ?? 0;
        if (fromBalance < amount) throw new Error('Insufficient balance');
        this.balances.set(from, fromBalance - amount);
        this.balances.set(to, (this.balances.get(to) ?? 0) + amount);
    }
}
```

```solidity
// Solidity ERC-20 토큰 (탈중앙화 버전)
contract MyToken {
    mapping(address => uint256) private _balances;

    function transfer(address to, uint256 amount) public returns (bool) {
        require(_balances[msg.sender] >= amount, "Insufficient balance");
        _balances[msg.sender] -= amount;
        _balances[to] += amount;
        return true;
    }
}
```

핵심 차이: Node.js 포인트 시스템은 서버 운영자가 임의로 잔액을 수정할 수 있다. 스마트 컨트랙트 토큰은 코드에 정의된 규칙 외에는 아무도 수정할 수 없다.

## 왜 표준이 필요한가

초기 이더리움에서는 개발자마다 토큰을 다르게 구현했다. 어떤 토큰은 `send()`, 어떤 토큰은 `transfer()`, 어떤 토큰은 `pay()`로 전송하는 식이었다. 이로 인해:

- 거래소가 새 토큰을 상장할 때마다 커스텀 통합 코드가 필요했다
- 지갑 앱이 모든 토큰을 개별적으로 지원해야 했다
- DeFi 프로토콜이 임의의 토큰과 상호작용할 수 없었다
- 감사(audit) 비용이 토큰마다 달랐다

**표준의 필요성을 Node.js로 비유하면:**

Node.js의 Stream 인터페이스가 없다면, 모든 라이브러리가 데이터 읽기/쓰기 API를 제각각 구현할 것이다. `fs.createReadStream()`의 출력을 `zlib.createGzip()`에 파이프하고, 다시 `net.Socket`에 파이프할 수 있는 건 모두가 같은 Stream 인터페이스를 구현하기 때문이다.

토큰 표준도 마찬가지다. 모든 ERC-20 토큰이 `transfer()`, `balanceOf()`, `approve()` 같은 동일한 인터페이스를 구현하기 때문에, Uniswap 같은 DEX는 어떤 ERC-20 토큰과도 상호작용할 수 있다.

```typescript
// 표준 덕분에 가능한 범용 코드 (ethers.js)
const ERC20_ABI = ['function balanceOf(address) view returns (uint256)'];

async function getBalance(tokenAddress: string, walletAddress: string) {
    // USDC든 UNI든 WETH든 동일한 코드로 조회 가능
    const token = new ethers.Contract(tokenAddress, ERC20_ABI, provider);
    return token.balanceOf(walletAddress);
}
```

```solidity
// Solidity에서 범용 토큰 처리
function swapAnyERC20(address token, uint256 amount) external {
    // 어떤 ERC-20이든 동일한 인터페이스로 처리
    IERC20(token).transferFrom(msg.sender, address(this), amount);
    uint256 outputAmount = quoteOutput(token, amount);
    require(outputAmount > 0, "ZERO_OUTPUT");
    IERC20(token).transfer(msg.sender, outputAmount);
}
```

## ERC 표준 체계

**ERC(Ethereum Request for Comments)**는 이더리움 커뮤니티가 토큰과 컨트랙트 표준을 제안하고 논의하는 과정이다. Node.js의 RFC, Python의 PEP와 유사한 개념이다.

**EIP(Ethereum Improvement Proposal)**로 제안되고, 커뮤니티 검토를 거쳐 **ERC**로 확정된다.

### 표준 제안 과정

```text
1. 개발자가 EIP 제안 (GitHub PR)
2. 커뮤니티 토론 및 피드백
3. 레퍼런스 구현 작성
4. 광범위한 검토 및 수정
5. "Final" 상태로 확정 → ERC가 됨
```

### 주요 토큰 표준

| 표준 | 이름 | 특징 | 대표 예시 |
|------|------|------|-----------|
| ERC-20 | 대체 가능 토큰 | 모든 단위가 동일 | USDC, UNI, WETH |
| ERC-721 | 대체 불가능 토큰(NFT) | 각 토큰이 고유 | CryptoPunks, BAYC |
| ERC-1155 | 멀티 토큰 | ERC-20 + ERC-721 통합 | 게임 아이템 |
| ERC-4626 | 토큰화된 금고 | yield-bearing vault | Yearn, Aave |
| ERC-2612 | Permit | 서명으로 approve | USDC v2, DAI |
| ERC-4337 | 계정 추상화 | 스마트 지갑 | Safe, Biconomy |

### ERC-20 — 대체 가능 토큰 (Fungible Token)

1 USDC = 다른 1 USDC. 완전히 동일하고 상호 교환 가능하다. 지폐처럼 개별 구분이 없다.

```text
사용 사례:
- 스테이블코인 (USDC, USDT, DAI)
- 거버넌스 토큰 (UNI, COMP, AAVE)
- 래핑된 자산 (WETH, WBTC)
- 유동성 풀 토큰 (LP tokens)
- 프로젝트 유틸리티 토큰
```

ERC-20이 만들어진 2015년 이후 이더리움 생태계의 근간이 됐다. 현재 수천 개의 ERC-20 토큰이 존재하며, 총 시가총액은 수조 달러에 달한다.

### ERC-721 — 대체 불가능 토큰 (Non-Fungible Token)

각 토큰이 고유한 ID를 가지며 서로 다르다. 토큰 #1과 토큰 #2는 같은 컨트랙트에서 발행됐어도 다른 자산이다. 실물 미술품처럼 각각이 독립적인 가치를 가진다.

```text
사용 사례:
- 디지털 아트 (CryptoPunks, BAYC)
- 게임 아이템 (특정 캐릭터, 무기)
- 도메인 이름 (ENS)
- 부동산 권리증
- 이벤트 티켓
- 신원 증명서
```

### ERC-1155 — 멀티 토큰

하나의 컨트랙트에서 대체 가능 토큰과 대체 불가능 토큰을 모두 관리할 수 있다. 게임에서 금화(대체 가능)와 전설 검(대체 불가능)을 하나의 컨트랙트로 관리하는 식이다.

```text
사용 사례:
- 블록체인 게임 아이템
- 이벤트 티켓 (같은 좌석 등급 = 대체 가능, 특정 좌석 = 대체 불가능)
- 한정판 에디션 (100개 중 하나)
```

ERC-1155의 장점: 여러 토큰을 하나의 트랜잭션으로 배치 전송 가능 → 가스 비용 절약.

### ERC-4626 — 토큰화된 금고 (Tokenized Vault)

수익률(yield)을 발생시키는 금고의 표준 인터페이스다. 예치(deposit), 출금(withdraw), 수익 계산 등의 표준화된 API를 제공한다.

```solidity
interface IERC4626 is IERC20 {
    function asset() external view returns (address);
    function totalAssets() external view returns (uint256);
    function deposit(uint256 assets, address receiver) external returns (uint256 shares);
    function withdraw(uint256 assets, address receiver, address owner) external returns (uint256 shares);
    function convertToShares(uint256 assets) external view returns (uint256 shares);
    function convertToAssets(uint256 shares) external view returns (uint256 assets);
}
```

## 표준의 실제 작동 방식

표준은 코드가 아니라 **인터페이스 명세**다. 다음 함수들을 반드시 구현해야 한다고 정의한다.

```solidity
// ERC-20 표준 인터페이스
interface IERC20 {
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint256);
    function approve(address spender, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}
```

이 인터페이스를 구현하면 그게 ERC-20 토큰이다. 내부 구현은 자유롭게 할 수 있다.

```typescript
// TypeScript 인터페이스와 정확히 같은 개념
interface IERC20 {
    totalSupply(): Promise<bigint>;
    balanceOf(account: string): Promise<bigint>;
    transfer(to: string, amount: bigint): Promise<boolean>;
    allowance(owner: string, spender: string): Promise<bigint>;
    approve(spender: string, amount: bigint): Promise<boolean>;
    transferFrom(from: string, to: string, amount: bigint): Promise<boolean>;
}
```

## 토큰 생태계 현황

### 시가총액 기준 주요 ERC-20 (2024년 기준)

```text
스테이블코인:
- USDT (Tether)       - 약 1000억 달러
- USDC (Circle)       - 약 400억 달러
- DAI (MakerDAO)      - 약 50억 달러

DeFi 거버넌스 토큰:
- UNI (Uniswap)       - 탈중앙화 거래소
- AAVE (Aave)         - 대출 프로토콜
- COMP (Compound)     - 대출 프로토콜
- MKR (MakerDAO)      - 스테이블코인 거버넌스

래핑된 자산:
- WETH                - ETH의 ERC-20 버전
- WBTC                - Bitcoin의 이더리움 버전
```

### 토큰이 가능하게 한 혁신

토큰 표준이 생긴 이후 이더리움 생태계에서 폭발적인 혁신이 일어났다:

```text
ICO (2017-2018):
- 누구나 토큰을 발행하고 자금 조달 가능
- 기존 VC 투자 없이 글로벌 크라우드펀딩

DeFi (2020~):
- 탈중앙화 거래소 (Uniswap): 토큰끼리 자동 교환
- 대출 프로토콜 (Aave): 토큰을 담보로 대출
- 수익 최적화 (Yearn): 토큰으로 자동 투자

NFT (2021~):
- 디지털 소유권의 새로운 형태
- 크리에이터 이코노미
```

## 이 챕터에서 다룰 내용

```text
13-1: ERC-20 직접 구현
  - 6개 필수 함수 전체 구현
  - approve + transferFrom 2단계 패턴
  - Foundry 테스트

13-2: ERC-721 구현
  - NFT의 개념과 구조
  - tokenURI와 메타데이터
  - safeTransfer의 중요성

13-3: OpenZeppelin 활용
  - 검증된 구현 라이브러리 활용
  - 접근 제어 패턴
  - 보안 유틸리티
```

다음 챕터에서는 ERC-20을 처음부터 직접 구현한다.
