# Chapter 14: 스마트 컨트랙트 심화

## 개요

앞선 챕터에서 Solidity의 기본 문법과 ERC-20/721 토큰 표준을 살펴봤다. 이 챕터에서는 실무에서 마주치는 더 복잡한 주제들을 다룬다.

## 이 챕터에서 다루는 내용

### 14-1: 상속과 인터페이스

Solidity의 상속 시스템은 TypeScript보다 훨씬 복잡하다. 다중 상속이 허용되고, C3 선형화 알고리즘으로 충돌을 해결한다. 추상 컨트랙트와 인터페이스의 차이, `virtual`/`override` 키워드를 TypeScript의 `extends`/`implements`와 비교해 이해한다.

### 14-2: 프록시 패턴

스마트 컨트랙트는 한 번 배포하면 코드를 바꿀 수 없다. 프록시 패턴은 이 제약을 우회해 로직을 업그레이드할 수 있게 한다. `delegatecall`이 어떻게 작동하는지, Transparent Proxy와 UUPS Proxy의 차이, 스토리지 충돌을 어떻게 피하는지 배운다.

### 14-3: 보안

스마트 컨트랙트는 버그가 곧 자금 손실로 이어진다. 역사상 실제 발생한 해킹 사례(The DAO, 2016)를 분석하고, 재진입 공격·정수 오버플로·tx.origin 오용·프론트러닝 등 주요 취약점과 방어 패턴을 익힌다.

## 왜 심화 내용이 중요한가

Node.js 백엔드에서 버그가 발생하면 서버를 재시작하고 패치를 배포하면 된다. 최악의 경우 데이터베이스를 롤백할 수 있다.

스마트 컨트랙트에서 버그가 발생하면:
- **코드를 수정할 수 없다** — 배포된 컨트랙트는 불변
- **자금이 즉시 탈취될 수 있다** — The DAO 해킹: 하루 만에 6천만 달러 손실
- **트랜잭션은 되돌릴 수 없다** — 블록체인의 불변성은 공격자에게도 유리

이런 이유로 프로 Solidity 개발자는 코드를 작성하는 것만큼 **보안**과 **업그레이드 전략**에 많은 시간을 투자한다.

```text
실무 스마트 컨트랙트 개발 사이클:
1. 요구사항 정의
2. 설계 (업그레이드 전략 포함)
3. 구현 (OpenZeppelin 활용)
4. 단위 테스트 + 퍼즈 테스트
5. 내부 보안 리뷰 (체크리스트 기반)
6. 외부 감사 (audit)
7. 테스트넷 배포 + 버그바운티
8. 메인넷 배포
9. 모니터링 (Tenderly, OpenZeppelin Defender)
```

Node.js 서비스 배포보다 훨씬 신중한 프로세스가 필요하다. 이 챕터는 그 기반을 다진다.

## 실제 해킹 사례 타임라인

심화 내용을 배우기 전에 실제 피해 사례를 먼저 살펴보자. 이것이 왜 이 내용이 중요한지 동기를 부여한다.

| 연도 | 프로젝트 | 취약점 | 피해액 |
|------|---------|--------|--------|
| 2016 | The DAO | 재진입 공격 | $60M |
| 2020 | bZx | 플래시론 + 오라클 조작 | $1M |
| 2021 | Poly Network | 접근 제어 오류 | $611M |
| 2021 | Compound | 거버넌스 버그 | $80M |
| 2022 | Ronin Bridge | 개인키 탈취 + 접근 제어 | $625M |
| 2022 | Wormhole | 서명 검증 오류 | $320M |
| 2023 | Euler Finance | 플래시론 + 로직 오류 | $197M |

이 사례들은 모두 방지 가능했다. 올바른 패턴을 알고, 충분히 테스트하고, 외부 감사를 받았다면 피할 수 있었던 버그들이다.

## Node.js 개발자를 위한 사고방식 전환

Node.js 백엔드 개발자는 다음 사고방식을 스마트 컨트랙트 개발에 맞게 바꿔야 한다.

### 기존: "버그는 고칠 수 있다"

```typescript
// Node.js - 배포 후 수정 가능
app.get('/transfer', async (req, res) => {
    // 버그 발견 → 코드 수정 → 재배포
    await transferService.transfer(req.body);
});
```

### 새로운: "버그는 영구적이다"

```solidity
// Solidity - 배포 후 수정 불가
function transfer(address to, uint256 amount) public {
    // 이 코드에 버그가 있다면 영원히 존재
    // 유일한 해결책: 처음부터 올바르게 작성
    _transfer(msg.sender, to, amount);
}
```

### 기존: "실패하면 로그 보고 디버깅"

```typescript
// Node.js - 스택 트레이스, 로그, 디버거 사용 가능
try {
    await complexOperation();
} catch (err) {
    logger.error(err);  // 상세 로그
    // 재시도, 수동 수정 가능
}
```

### 새로운: "실패하면 트랜잭션 revert, 가스만 소비"

```solidity
// Solidity - revert되면 상태 변화 없이 가스만 소비
function complexOperation() external {
    // 이 중간에 revert되면 모든 상태 변화가 롤백됨
    // 하지만 소비한 가스는 돌려받지 못함
    step1();
    step2(); // 여기서 실패하면 step1도 롤백
    step3();
}
```

### 기존: "사용자 입력은 미들웨어에서 검증"

```typescript
// NestJS - DTO 검증
@Post('/deposit')
async deposit(@Body() dto: DepositDto) {
    // 클래스-벨리데이터가 이미 검증함
}
```

### 새로운: "모든 입력은 컨트랙트에서 직접 검증"

```solidity
// Solidity - 컨트랙트 자체가 최후의 방어선
function deposit(uint256 amount) external {
    // 외부 검증에 의존하지 말고 직접 검증
    require(amount > 0, "Amount must be positive");
    require(amount <= MAX_DEPOSIT, "Exceeds limit");
    require(!paused, "Contract is paused");
    balances[msg.sender] += amount;
    totalDeposits += amount;
    emit Deposited(msg.sender, amount);
}
```

## 이 챕터 학습 순서

### 1단계: 상속 이해 (14-1)

기본 상속부터 다중 상속, C3 선형화까지 차근차근 이해한다. OpenZeppelin 라이브러리가 이 패턴을 어떻게 활용하는지 보면 자연스럽게 이해된다.

```text
단일 상속 → 다중 상속 → 추상 컨트랙트 → 인터페이스
```

### 2단계: 프록시 패턴 이해 (14-2)

`delegatecall`의 동작을 완전히 이해하는 것이 핵심이다. 처음에는 헷갈리지만, storage 슬롯이 어떻게 공유되는지 이해하면 모든 게 명확해진다.

```text
delegatecall 원리 → Transparent Proxy → UUPS Proxy → 스토리지 레이아웃
```

### 3단계: 보안 패턴 습득 (14-3)

실제 해킹 코드를 보고 분석한 다음 방어 코드를 작성한다. 공격을 이해해야 방어할 수 있다.

```text
재진입 공격 → CEI 패턴 → 오버플로 → tx.origin → 프론트러닝 → 체크리스트
```

## 필수 선행 지식 확인

이 챕터를 시작하기 전에 다음을 확인하자:

```text
[ ] Solidity 기본 문법 (ch11)
[ ] Foundry 테스트 작성 (ch12)
[ ] ERC-20 구조 이해 (ch13-01)
[ ] OpenZeppelin 기본 사용법 (ch13-03)
[ ] modifier 작성 경험
[ ] mapping과 이벤트 이해
```

이 지식이 없다면 앞 챕터를 먼저 학습하고 오자. 특히 프록시 패턴은 storage 슬롯에 대한 깊은 이해가 필요하다.

## 다음 챕터 미리보기

이 챕터를 마치면:

1. **복잡한 상속 구조**를 읽고 이해할 수 있다
2. **OpenZeppelin의 업그레이드 가능 컨트랙트**를 직접 사용할 수 있다
3. **보안 취약점을 코드 리뷰에서 발견**할 수 있다
4. **미니프로젝트(ch15)**를 자신감 있게 완성할 수 있다

스마트 컨트랙트 개발의 핵심은 처음부터 올바르게 작성하는 것이다. 시작해보자.
