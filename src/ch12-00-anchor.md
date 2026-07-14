# 12. Anchor로 프로그램 만들기

Anchor는 Solana 프로그램의 직렬화, 계정 검증, 오류, IDL 생성을 돕는 Rust 프레임워크다. 보안 검사를 없애는 도구가 아니라 반복되는 검사를 구조체와 매크로로 드러내는 도구에 가깝다.

이 장의 명령과 프로젝트 구조는 Anchor 1.0.x 기준이다. 생태계 도구는 빠르게 바뀌므로 설치할 때는 공식 설치 문서와 릴리스 노트를 함께 확인한다.

## 프로젝트 시작

공식 설치 절차로 Rust와 Anchor CLI를 준비한 뒤 프로젝트를 만든다.

```bash
anchor --version
anchor init counter-anchor
cd counter-anchor
anchor build
anchor test
```

현재 Anchor는 기본 프로젝트를 모듈 구조로 생성한다.

```text
programs/counter-anchor/src/
├── lib.rs
├── constants.rs
├── error.rs
├── instructions/
└── state/
```

작은 예제는 한 파일로 읽는 편이 쉬워 아래 코드를 `lib.rs` 형태로 모았다. 프로젝트가 커지면 instruction과 state를 생성된 디렉터리로 나눈다.

## PDA 상태를 쓰는 Counter

```rust,ignore
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWxTWqkZtS6W2BeZ7FEfcYkgMQHG");

#[program]
pub mod counter_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.authority = ctx.accounts.authority.key();
        counter.value = 0;
        counter.bump = ctx.bumps.counter;
        Ok(())
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.value = counter
            .value
            .checked_add(1)
            .ok_or(CounterError::Overflow)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Counter::INIT_SPACE,
        seeds = [b"counter", authority.key().as_ref()],
        bump
    )]
    pub counter: Account<'info, Counter>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"counter", authority.key().as_ref()],
        bump = counter.bump
    )]
    pub counter: Account<'info, Counter>,

    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub authority: Pubkey,
    pub value: u64,
    pub bump: u8,
}

#[error_code]
pub enum CounterError {
    #[msg("counter 값이 u64 범위를 넘었습니다")]
    Overflow,
}
```

`declare_id!` 문자열은 예제용 주소다. 실제 프로젝트에서는 `anchor init`이 만든 program ID로 바꾼다. 이 코드는 Anchor 의존성과 프로젝트 문맥이 필요하므로 `rust,ignore`로 표시했다.

## 매크로가 하는 일

| 매크로·타입 | 역할 |
|---|---|
| `#[program]` | 외부에서 호출할 instruction handler 모듈 |
| `Context<T>` | instruction에 필요한 검증된 계정 묶음 |
| `#[derive(Accounts)]` | 계정 목록과 제약 검증 코드 생성 |
| `Signer<'info>` | 트랜잭션 서명이 필요한 계정 |
| `Account<'info, T>` | 프로그램 소유권과 discriminator를 확인하고 `T`로 역직렬화 |
| `Program<'info, System>` | 전달된 주소가 System Program인지 확인 |
| `#[account]` | Anchor 계정 데이터 타입 표시 |
| `#[error_code]` | 프로그램 오류 코드 정의 |

`#[derive(Accounts)]`는 필드 형식만 확인하지 않는다. signer, owner, PDA seed, 쓰기 권한, 계정 초기화 비용 같은 실행 조건을 instruction handler 진입 전에 검증하는 코드를 만든다.

## 제약 조건을 읽는 순서

`Initialize`를 위에서 아래로 읽으면 다음 정책이 보인다.

1. `counter`를 새 계정으로 만든다.
2. 계정 생성 비용은 `authority`가 낸다.
3. discriminator 8바이트와 데이터 크기만큼 공간을 잡는다.
4. 주소는 `b"counter"` 바이트 시드와 authority 공개키로 만든 PDA다.
5. authority는 쓰기 가능하고 실제로 서명해야 한다.
6. 계정 생성에는 정확한 System Program을 쓴다.

`Increment`의 `has_one = authority`는 저장된 `counter.authority`와 전달된 authority 키가 같은지 확인한다. `Signer`까지 있으므로 공개키만 맞춰 넣는 것으로는 부족하다.

## 계정 공간

Anchor 계정은 앞 8바이트에 타입 discriminator를 둔다. `InitSpace`가 필드 공간을 계산하므로 `space = 8 + Counter::INIT_SPACE`로 잡는다.

고정 길이 숫자와 공개키는 계산하기 쉽지만 `String`과 `Vec<T>`는 최대 길이를 정해야 한다.

```rust,ignore
#[account]
#[derive(InitSpace)]
pub struct Memo {
    pub authority: Pubkey,
    #[max_len(280)]
    pub content: String,
}
```

최대 길이를 너무 작게 잡으면 직렬화가 실패하고 지나치게 크게 잡으면 계정 생성에 더 많은 lamports가 필요하다.

## 테스트 층을 나눈다

Anchor 1.0.x의 `anchor init`은 Rust로 작성하는 LiteSVM 테스트를 기본 생성한다. LiteSVM은 별도 RPC 노드를 띄우지 않고 같은 프로세스에서 프로그램을 실행하므로 빠른 상태 전이와 실패 경로 검증에 적합하다. 생성된 테스트를 출발점으로 다음 경우를 추가한다.

- `initialize` 뒤 authority, value, bump가 예상대로 저장되는가?
- authority가 아닌 서명자의 `increment`가 실패하는가?
- 다른 시드로 만든 계정을 전달하면 실패하는가?
- `u64` 경계에서 overflow 오류가 반환되는가?

다른 테스트 러너가 필요하면 프로젝트 생성 시 `--test-template mollusk`, `--test-template mocha`, `--test-template jest` 가운데 하나를 선택할 수 있다. 기본값을 바꿀 이유가 없다면 생성된 Rust LiteSVM 테스트를 그대로 쓰는 편이 가장 단순하다.

테스트 목적에 따라 실행 환경도 나눈다.

| 층 | 도구 | 확인할 것 |
|---|---|---|
| 빠른 프로그램 테스트 | LiteSVM | instruction 성공·실패, 계정 상태, 권한 제약 |
| 로컬 통합 테스트 | `anchor test` | 배포 흐름, RPC를 포함한 클라이언트 상호작용 |
| 실제 클러스터 점검 | devnet 등 | 네트워크 설정, 배포 권한, 실제 RPC 차이 |

Anchor 1.0.x의 `anchor test`는 기본 로컬 validator로 Surfpool을 사용한다. 기존 `solana-test-validator` 동작이 꼭 필요하면 `anchor test --validator legacy`를 사용한다. 테스트 키와 RPC 자격 증명은 저장소에 커밋하지 않는다.

## 프로그램 보안 체크

Anchor가 생성한 타입만 믿고 끝내지 않는다.

- 모든 authority 계정이 `Signer`인가?
- 상태 계정의 PDA seed가 사용자나 자산과 올바르게 묶였는가?
- `has_one`, `owner`, `address`, token constraint가 필요한 곳에 있는가?
- 서로 달라야 할 계정이 같은 주소가 될 수 없는가?
- 산술은 `checked_*`를 쓰는가?
- CPI 대상 프로그램과 mint, token account 관계를 검증하는가?
- 계정 close와 realloc 뒤 데이터가 안전한가?
- upgrade authority를 누가 보유하며 폐기·이전 절차가 있는가?

코드 길이보다 중요한 것은 신뢰 경계다. 각 instruction이 어떤 계정을 신뢰하고 어떤 제약을 검사한 뒤 어떤 상태만 바꾸는지 설명할 수 있어야 한다.
