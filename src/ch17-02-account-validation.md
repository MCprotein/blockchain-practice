# Account 검증: #[derive(Accounts)] 완전 가이드

## 왜 Account 검증이 중요한가?

Solana 프로그램 보안의 핵심은 **계정 검증**입니다. 악의적인 사용자는 다음과 같은 공격을 시도할 수 있습니다:

```
공격 시나리오:
1. 타입 혼동 공격: 다른 구조체로 초기화된 계정을 전달
2. 권한 우회: 자신이 서명하지 않은 트랜잭션으로 타인 계정 수정
3. 가짜 프로그램: Token Program 대신 악의적인 프로그램 주소 전달
4. 재사용 공격: 이미 닫힌 계정을 다시 사용
```

Anchor의 `#[derive(Accounts)]`는 이런 공격들을 컴파일 타임과 런타임에 차단합니다.

---

## Account 타입 종류

### 1. Account<'info, T> - 역직렬화된 계정

가장 많이 사용하는 타입입니다. Anchor가 자동으로:
- Account의 `data`를 `T` 타입으로 역직렬화
- `owner`가 현재 프로그램인지 검증
- discriminator(앞 8바이트)가 `T` 타입과 일치하는지 검증

```rust
#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    // UserProfile 타입으로 자동 역직렬화
    // owner = 현재 프로그램인지 자동 검증
    // discriminator 타입 검증 자동
    pub profile: Account<'info, UserProfile>,
}

// 사용:
pub fn update_profile(ctx: Context<UpdateProfile>, new_name: String) -> Result<()> {
    // .profile로 직접 접근 (이미 역직렬화됨)
    ctx.accounts.profile.name = new_name;
    // mut가 없으면 컴파일 에러! → #[account(mut)] 필요
    Ok(())
}
```

```rust
// 타입 파라미터 T의 조건:
// T는 #[account] 매크로가 적용된 구조체여야 함
#[account]
pub struct UserProfile {
    pub owner: Pubkey,
    pub name: String,
    pub score: u64,
}
```

### 2. Signer<'info> - 서명자 검증

트랜잭션에 서명한 계정을 나타냅니다.

```rust
#[derive(Accounts)]
pub struct MintTokens<'info> {
    // 이 계정이 트랜잭션에 서명했는지 자동 검증
    // 서명 없으면 → MissingRequiredSignature 에러
    pub authority: Signer<'info>,
}

// Signer vs &Signer:
// Signer<'info>   → AccountInfo 데이터 접근 가능
// &Signer<'info>  → 주소만 필요할 때 (더 효율적)
```

```typescript
// 클라이언트에서 서명자 지정
await program.methods
  .mintTokens(new BN(100))
  .accounts({
    authority: wallet.publicKey,  // ← 이 키페어로 서명해야 함
  })
  .signers([wallet])              // ← 실제 서명 키페어
  .rpc();
```

### 3. Program<'info, T> - 프로그램 계정 검증

다른 프로그램의 ID를 검증합니다. CPI에서 필수입니다.

```rust
use anchor_spl::token::Token;
use anchor_lang::system_program::System;

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    // System Program 검증
    // 주소가 11111111...인지 자동 확인
    pub system_program: Program<'info, System>,

    // Token Program 검증
    // 주소가 TokenkegQfe...인지 자동 확인
    pub token_program: Program<'info, Token>,
}

// 악의적인 사용자가 가짜 프로그램 주소를 전달해도 차단!
```

### 4. SystemAccount<'info> - System Program 소유 계정

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    // owner가 System Program인지 검증
    // 일반 SOL 지갑 계정에 사용
    pub payer: SystemAccount<'info>,
}
```

### 5. UncheckedAccount<'info> - 검증 없는 계정 (주의!)

```rust
#[derive(Accounts)]
pub struct Dangerous<'info> {
    /// CHECK: 이 계정은 오직 주소만 사용하고 데이터는 읽지 않음
    /// 주소가 my_config_account와 일치하는지만 확인
    #[account(address = MY_CONFIG_PUBKEY)]
    pub config: UncheckedAccount<'info>,
}

// UncheckedAccount를 사용하면 Anchor가 주석을 강제함:
// /// CHECK: 이유를 설명해야 컴파일됨
// 주석 없으면 → 컴파일 에러: "doc comment required"
```

사용 시나리오:
- 외부 프로그램의 계정을 주소로만 참조할 때
- 계정 데이터 구조를 직접 파싱해야 할 때
- 성능 최적화가 필요한 특수 케이스

---

## 계정 제약조건 (Account Constraints)

### #[account(init, payer, space)] - 새 계정 생성

```rust
#[derive(Accounts)]
pub struct CreatePost<'info> {
    #[account(
        init,                          // 새 Account 생성 (없으면 에러)
        payer = author,                // 렌트 비용을 author가 지불
        space = 8 + Post::LEN,        // 할당할 바이트 수 (8 = discriminator)
    )]
    pub post: Account<'info, Post>,

    #[account(mut)]                    // author 잔액 감소하므로 mut 필요
    pub author: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// init이 자동으로 처리:
// 1. System Program에 CPI → 새 Account 생성
// 2. owner를 현재 프로그램으로 설정
// 3. discriminator 작성
// 4. rent 면제 lamports 이체
```

### #[account(init_if_needed)] - 없으면 생성, 있으면 사용

```rust
#[derive(Accounts)]
pub struct GetOrCreateProfile<'info> {
    #[account(
        init_if_needed,               // 없으면 생성, 있으면 그냥 로드
        payer = user,
        space = 8 + UserProfile::LEN,
        seeds = [b"profile", user.key().as_ref()],
        bump,
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// 주의: init_if_needed는 Cargo.toml에 feature 추가 필요
// anchor-lang = { version = "0.30.1", features = ["init-if-needed"] }
```

### #[account(mut)] - 수정 가능 계정

```rust
#[derive(Accounts)]
pub struct UpdateScore<'info> {
    #[account(mut)]  // data 또는 lamports를 수정할 때 필수
    pub user_data: Account<'info, UserData>,

    // mut 없이 수정하면?
    // → 런타임에서 "writable" 플래그 없음 에러
    // → 또는 Anchor가 컴파일 타임에 경고
}
```

### #[account(has_one)] - 관계 검증

```rust
#[account]
pub struct Post {
    pub author: Pubkey,   // 작성자 주소 저장
    pub content: String,
}

#[derive(Accounts)]
pub struct DeletePost<'info> {
    #[account(
        mut,
        has_one = author,  // post.author == author.key() 자동 검증!
        close = author,    // 계정 닫고 lamports를 author에게 반환
    )]
    pub post: Account<'info, Post>,

    pub author: Signer<'info>,  // post.author와 같아야 함
}

// has_one 없이 구현하면:
// if post.author != ctx.accounts.author.key() {
//     return Err(MyError::Unauthorized);
// }
// → has_one이 이 코드를 자동으로 생성
```

### #[account(constraint)] - 커스텀 제약조건

```rust
#[derive(Accounts)]
#[instruction(amount: u64)]  // Instruction 인자 접근 필요 시
pub struct Transfer<'info> {
    #[account(
        mut,
        has_one = owner,
        // 커스텀 제약조건: 잔액 충분한지 확인
        constraint = from.balance >= amount @ MyError::InsufficientBalance,
        // @ MyError::... 로 에러 타입 지정 가능
    )]
    pub from: Account<'info, Wallet>,

    #[account(mut)]
    pub to: Account<'info, Wallet>,

    pub owner: Signer<'info>,
}
```

### #[account(seeds, bump)] - PDA 검증

```rust
#[derive(Accounts)]
pub struct UpdateUserData<'info> {
    #[account(
        mut,
        seeds = [b"user-data", user.key().as_ref()],  // PDA seeds
        bump,  // bump를 자동으로 찾아서 검증
        // bump = user_data.bump 처럼 저장된 bump 사용 가능
    )]
    pub user_data: Account<'info, UserData>,

    pub user: Signer<'info>,
}

// seeds + bump가 하는 일:
// 1. find_program_address(seeds, program_id) 실행
// 2. 계산된 PDA == user_data.key() 검증
// 3. 불일치 시 → ConstraintSeeds 에러

// PDA 생성 시 (init + seeds):
#[derive(Accounts)]
pub struct CreateUserData<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + UserData::LEN,
        seeds = [b"user-data", user.key().as_ref()],
        bump,
    )]
    pub user_data: Account<'info, UserData>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
```

### #[account(close)] - 계정 닫기 (렌트 회수)

```rust
#[derive(Accounts)]
pub struct CloseAccount<'info> {
    #[account(
        mut,
        has_one = owner,
        close = owner,  // 계정을 닫고 lamports를 owner에게 반환
    )]
    pub data_account: Account<'info, MyData>,

    #[account(mut)]     // lamports를 받으므로 mut 필요
    pub owner: Signer<'info>,
}

// close가 자동으로 처리:
// 1. data_account.lamports → owner.lamports로 이전
// 2. data_account.data → 0으로 초기화
// 3. data_account.owner → System Program으로 변경
// → 계정이 완전히 삭제됨

// 주의: "closing attack" 방지를 위해
// Anchor가 자동으로 discriminator를 CLOSED_ACCOUNT_DISCRIMINATOR로 설정
```

### #[account(address)] - 특정 주소 검증

```rust
use anchor_lang::solana_program::sysvar::clock::ID as CLOCK_ID;

#[derive(Accounts)]
pub struct CheckTime<'info> {
    // Clock sysvar의 정확한 주소인지 검증
    #[account(address = CLOCK_ID)]
    pub clock: AccountInfo<'info>,

    // 또는 상수 주소 검증
    #[account(address = MY_TREASURY_PUBKEY)]
    pub treasury: SystemAccount<'info>,
}
```

---

## Space 계산 방법

Anchor 계정의 공간을 계산하는 것은 중요한 기술입니다:

```rust
#[account]
pub struct MemoAccount {
    pub author: Pubkey,      // 32 bytes
    pub content: String,     // 4 + max_len bytes
    pub timestamp: i64,      // 8 bytes
    pub likes: u32,          // 4 bytes
    pub is_public: bool,     // 1 byte
    pub tags: Vec<String>,   // 4 + (4 + max_tag_len) * max_tags bytes
    pub bump: u8,            // 1 byte
}

impl MemoAccount {
    pub const MAX_CONTENT_LEN: usize = 280;   // 트위터처럼 280자
    pub const MAX_TAG_LEN: usize = 20;
    pub const MAX_TAGS: usize = 5;

    pub const LEN: usize =
        32 +                                    // author: Pubkey
        4 + Self::MAX_CONTENT_LEN +             // content: String
        8 +                                     // timestamp: i64
        4 +                                     // likes: u32
        1 +                                     // is_public: bool
        4 + (4 + Self::MAX_TAG_LEN) * Self::MAX_TAGS + // tags: Vec<String>
        1;                                      // bump: u8
    // = 32 + 284 + 8 + 4 + 1 + 124 + 1 = 454 bytes
}

// 계정 생성 시:
// space = 8 + MemoAccount::LEN  (8 = Anchor discriminator)
// space = 8 + 454 = 462 bytes
```

### Rust 타입별 Borsh 크기 참조표

```
타입            크기 (bytes)
─────────────────────────────────────────────
bool            1
u8 / i8         1
u16 / i16       2
u32 / i32       4
u64 / i64       8
u128 / i128     16
f32             4
f64             8
Pubkey          32
String          4 (length) + 문자열 바이트 수
Vec<T>          4 (length) + T의 크기 × 요소 수
Option<T>       1 (Some/None) + T의 크기 (Some일 때)
[T; N]          T의 크기 × N
```

---

## 전체 예제: 메모 프로그램

생성, 업데이트, 삭제 기능을 갖춘 완전한 메모 프로그램입니다.

```rust
use anchor_lang::prelude::*;

declare_id!("Memo1111111111111111111111111111111111111111");

// ============================================================
// 에러 정의
// ============================================================
#[error_code]
pub enum MemoError {
    #[msg("메모 내용이 너무 깁니다 (최대 280자)")]
    ContentTooLong,
    #[msg("빈 메모는 생성할 수 없습니다")]
    EmptyContent,
    #[msg("권한이 없습니다")]
    Unauthorized,
}

// ============================================================
// 이벤트 정의
// ============================================================
#[event]
pub struct MemoCreated {
    pub author: Pubkey,
    pub memo_id: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MemoUpdated {
    pub memo_id: Pubkey,
    pub timestamp: i64,
}

// ============================================================
// 계정 구조체
// ============================================================
#[account]
pub struct Memo {
    pub author: Pubkey,       // 32
    pub content: String,      // 4 + 280
    pub created_at: i64,      // 8
    pub updated_at: i64,      // 8
    pub bump: u8,             // 1
}

impl Memo {
    pub const MAX_CONTENT_LEN: usize = 280;
    pub const LEN: usize = 32 + 4 + Self::MAX_CONTENT_LEN + 8 + 8 + 1;
}

// ============================================================
// 프로그램
// ============================================================
#[program]
pub mod memo_program {
    use super::*;

    /// 새 메모 생성
    pub fn create_memo(ctx: Context<CreateMemo>, content: String) -> Result<()> {
        // 입력 검증
        require!(!content.is_empty(), MemoError::EmptyContent);
        require!(
            content.len() <= Memo::MAX_CONTENT_LEN,
            MemoError::ContentTooLong
        );

        let clock = Clock::get()?;
        let memo = &mut ctx.accounts.memo;

        memo.author = ctx.accounts.author.key();
        memo.content = content;
        memo.created_at = clock.unix_timestamp;
        memo.updated_at = clock.unix_timestamp;
        memo.bump = ctx.bumps.memo;  // Anchor가 자동으로 bump 제공

        emit!(MemoCreated {
            author: memo.author,
            memo_id: ctx.accounts.memo.key(),
            timestamp: clock.unix_timestamp,
        });

        msg!("메모 생성: {}", ctx.accounts.memo.key());
        Ok(())
    }

    /// 메모 내용 업데이트
    pub fn update_memo(ctx: Context<UpdateMemo>, new_content: String) -> Result<()> {
        require!(!new_content.is_empty(), MemoError::EmptyContent);
        require!(
            new_content.len() <= Memo::MAX_CONTENT_LEN,
            MemoError::ContentTooLong
        );

        let clock = Clock::get()?;
        let memo = &mut ctx.accounts.memo;

        memo.content = new_content;
        memo.updated_at = clock.unix_timestamp;

        emit!(MemoUpdated {
            memo_id: ctx.accounts.memo.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// 메모 삭제 (렌트 반환)
    pub fn delete_memo(_ctx: Context<DeleteMemo>) -> Result<()> {
        // close = author 제약조건이 자동으로 처리
        msg!("메모 삭제 완료");
        Ok(())
    }
}

// ============================================================
// 계정 검증 구조체
// ============================================================
#[derive(Accounts)]
#[instruction(content: String)]  // content로 seeds 접근 가능 (여기선 미사용)
pub struct CreateMemo<'info> {
    #[account(
        init,
        payer = author,
        space = 8 + Memo::LEN,
        // 사용자당 하나의 메모: author 주소로 PDA 생성
        // 실제로는 메모 ID(랜덤 키페어)를 사용할 수도 있음
        seeds = [b"memo", author.key().as_ref()],
        bump,
    )]
    pub memo: Account<'info, Memo>,

    #[account(mut)]
    pub author: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateMemo<'info> {
    #[account(
        mut,
        seeds = [b"memo", author.key().as_ref()],
        bump = memo.bump,   // 저장된 bump 사용 (더 효율적)
        has_one = author @ MemoError::Unauthorized,  // 커스텀 에러 지정
    )]
    pub memo: Account<'info, Memo>,

    pub author: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeleteMemo<'info> {
    #[account(
        mut,
        seeds = [b"memo", author.key().as_ref()],
        bump = memo.bump,
        has_one = author @ MemoError::Unauthorized,
        close = author,     // 계정 닫고 lamports를 author에게 반환
    )]
    pub memo: Account<'info, Memo>,

    #[account(mut)]         // lamports를 받으므로 mut
    pub author: Signer<'info>,
}
```

---

## 제약조건 빠른 참조

```
#[account(init)]                  새 계정 생성
#[account(init_if_needed)]        없으면 생성
#[account(mut)]                   수정 가능
#[account(has_one = field)]       account.field == field.key()
#[account(constraint = expr)]     커스텀 불리언 표현식
#[account(address = pubkey)]      특정 주소 강제
#[account(owner = program)]       특정 프로그램 소유 강제
#[account(seeds = [...], bump)]   PDA 검증
#[account(close = target)]        계정 닫기, lamports → target
#[account(signer)]                서명자 강제 (드물게 사용)
#[account(zero)]                  데이터가 0으로 초기화된 계정
#[account(rent_exempt = enforce)] rent-exempt 강제
```

다음 장에서는 TypeScript로 이 프로그램을 테스트하는 방법을 배웁니다.
