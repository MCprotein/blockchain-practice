# Anchor: Solana의 개발 프레임워크

## Anchor란 무엇인가?

Anchor는 Solana 프로그램 개발을 위한 프레임워크입니다. Solana 생태계에서 Anchor의 위치를 다른 도구들과 비교하면 이렇습니다:

```text
이더리움 개발 스택:
  Solidity 언어 + Foundry/Hardhat 프레임워크

Solana 개발 스택:
  Rust 언어 + Anchor 프레임워크

Node.js 백엔드 스택:
  TypeScript 언어 + NestJS 프레임워크
```

Anchor를 "Solana의 NestJS"라고 부르는 이유는, NestJS가 Express.js의 보일러플레이트를 데코레이터와 의존성 주입으로 줄여주듯이, Anchor가 Native Solana 프로그램의 반복적이고 위험한 코드를 매크로(macro)로 대폭 줄여주기 때문입니다.

---

## 왜 Anchor를 써야 하는가?

앞 장에서 Native Rust로 카운터 프로그램을 작성했을 때를 기억하시나요? 다음과 같은 코드를 직접 작성해야 했습니다:

```rust,ignore
// Native Rust: 개발자가 직접 해야 할 것들

// 1. 계정 파싱 (보일러플레이트)
let accounts_iter = &mut accounts.iter();
let counter_account = next_account_info(accounts_iter)?;
let authority = next_account_info(accounts_iter)?;
let system_program = next_account_info(accounts_iter)?;

// 2. 서명자 검증 (보안 취약점 가능성)
if !authority.is_signer {
    return Err(ProgramError::MissingRequiredSignature);
}

// 3. 계정 owner 검증
if counter_account.owner != program_id {
    return Err(ProgramError::IllegalOwner);
}

// 4. 데이터 역직렬화
let mut counter_data =
    CounterAccount::try_from_slice(&counter_account.data.borrow())?;

// 5. 비즈니스 로직
counter_data.count += amount;

// 6. 데이터 재직렬화
counter_data.serialize(&mut *counter_account.data.borrow_mut())?;

// 7. Instruction 디스패치 (열거형 매칭)
match instruction {
    CounterInstruction::Initialize => initialize(counter_account),
    CounterInstruction::Increment { amount } => increment(counter_account, amount),
    CounterInstruction::Reset => reset(counter_account),
}
```

이 코드에는 두 가지 문제가 있습니다:
1. **반복 작업**: 모든 함수마다 동일한 검증 코드를 작성해야 함
2. **보안 취약점**: 검증 코드를 실수로 빠뜨리면 치명적인 버그 발생

Anchor는 이를 매크로로 해결합니다:

```rust,ignore
// Anchor: 같은 기능, 훨씬 적은 코드

#[program]
pub mod counter {
    use super::*;

    pub fn increment(ctx: Context<Increment>, amount: u64) -> Result<()> {
        // 비즈니스 로직만!
        ctx.accounts.counter.count += amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut, has_one = authority)]  // owner, writable, authority 검증 자동!
    pub counter: Account<'info, CounterAccount>,
    pub authority: Signer<'info>,         // 서명자 검증 자동!
}
```

---

## Native vs Anchor 코드 비교

같은 "카운터 초기화" 기능을 두 방식으로 구현한 코드를 비교합니다:

### Native Rust (약 80줄)

```rust,ignore
fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if counter_account.data_len() > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(CounterAccount::LEN);

    invoke(
        &system_instruction::create_account(
            authority.key,
            counter_account.key,
            required_lamports,
            CounterAccount::LEN as u64,
            program_id,
        ),
        &[authority.clone(), counter_account.clone(), system_program.clone()],
    )?;

    let counter_data = CounterAccount {
        count: 0,
        authority: *authority.key,
        is_initialized: true,
    };
    counter_data.serialize(&mut *counter_account.data.borrow_mut())?;
    Ok(())
}
```

### Anchor (약 20줄)

```rust,ignore
pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.count = 0;
    counter.authority = ctx.accounts.authority.key();
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,                           // 계정 생성 자동
        payer = authority,              // 비용 지불자
        space = 8 + CounterAccount::LEN // 공간 할당
    )]
    pub counter: Account<'info, CounterAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
```

**코드 양이 75% 감소하고, 보안 검증은 오히려 더 철저합니다.**

---

## Anchor가 자동으로 처리하는 것들

```text
Anchor 매크로가 생성하는 코드:
┌─────────────────────────────────────────────────────────┐
│  #[derive(Accounts)] 가 자동 생성하는 것:               │
│  ✓ 계정 파싱 (next_account_info 반복 제거)              │
│  ✓ Account discriminator 검증 (8바이트 타입 식별자)     │
│  ✓ owner 프로그램 검증                                  │
│  ✓ 서명자 검증 (Signer<'info>)                         │
│  ✓ writable 검증 (#[account(mut)])                     │
│  ✓ 초기화 여부 검증                                     │
│  ✓ PDA 검증 (#[account(seeds, bump)])                  │
│  ✓ 관계 검증 (#[account(has_one)])                     │
│                                                         │
│  #[program] 이 자동 생성하는 것:                        │
│  ✓ entrypoint 등록                                     │
│  ✓ Instruction 디스패치 (8바이트 discriminator 기반)   │
│  ✓ Context 구조체 주입                                  │
│  ✓ 에러 처리 및 변환                                    │
│                                                         │
│  #[account] 가 자동 생성하는 것:                        │
│  ✓ Borsh 직렬화/역직렬화                                │
│  ✓ Account discriminator 추가 (앞 8바이트)              │
│  ✓ space 계산 헬퍼                                     │
└─────────────────────────────────────────────────────────┘
```

---

## Anchor 설치 방법

### AVM (Anchor Version Manager) 사용

Node.js 개발자에게 친숙한 nvm(Node Version Manager)과 동일한 개념입니다:

```bash
# AVM 설치
cargo install --git https://github.com/coral-xyz/anchor avm --locked

# 설치 확인
avm --version

# 최신 Anchor 버전 설치
avm install latest
avm use latest

# 특정 버전 설치 및 사용
avm install 0.30.1
avm use 0.30.1

# 현재 버전 확인
anchor --version
# anchor-cli 0.30.1

# 설치된 버전 목록
avm list
```

### 전제 조건 확인

```bash
# Rust 설치 (없으면)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# Solana CLI 설치 (없으면)
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Node.js 18+ (이미 설치되어 있을 것)
node --version

# Yarn (Anchor 프로젝트 기본 패키지 매니저)
npm install -g yarn

# 모든 도구 버전 확인
rustc --version        # rustc 1.75+
solana --version       # solana-cli 1.18+
anchor --version       # anchor-cli 0.30+
node --version         # v18+
```

---

## Anchor IDL (Interface Definition Language)

Anchor의 숨은 강력한 기능 중 하나는 **IDL 자동 생성**입니다. 이더리움의 ABI와 동일한 개념입니다:

```json
// Anchor가 자동 생성하는 IDL (target/idl/counter.json)
{
  "version": "0.1.0",
  "name": "counter",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [
        { "name": "counter", "isMut": true, "isSigner": false },
        { "name": "authority", "isMut": true, "isSigner": true },
        { "name": "systemProgram", "isMut": false, "isSigner": false }
      ],
      "args": []
    },
    {
      "name": "increment",
      "accounts": [
        { "name": "counter", "isMut": true, "isSigner": false },
        { "name": "authority", "isMut": false, "isSigner": true }
      ],
      "args": [
        { "name": "amount", "type": "u64" }
      ]
    }
  ],
  "accounts": [
    {
      "name": "CounterAccount",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "count", "type": "u64" },
          { "name": "authority", "type": "publicKey" }
        ]
      }
    }
  ]
}
```

이 IDL을 기반으로 TypeScript 클라이언트가 자동으로 타입 안전한 함수를 제공합니다:

```typescript
// IDL 기반 자동 생성 타입 (타입 안전!)
await program.methods
  .increment(new BN(5))          // amount: u64
  .accounts({
    counter: counterPubkey,
    authority: wallet.publicKey,
  })
  .rpc();

// 컴파일 타임에 타입 검사:
// - 인자 타입 검사 (u64여야 함)
// - 계정 이름 검사 (counter, authority여야 함)
```

---

## Anchor 생태계

```text
Anchor 생태계:
┌──────────────────────────────────────────────────────┐
│  anchor-lang    → Rust 매크로 및 트레이트            │
│  anchor-spl     → SPL Token, Token-2022 통합         │
│  anchor-client  → Rust 클라이언트                    │
│  @coral-xyz/anchor → TypeScript 클라이언트 (핵심!)  │
└──────────────────────────────────────────────────────┘

주요 버전:
  0.29.x → 현재 많은 프로젝트에서 사용
  0.30.x → 최신 안정 버전 (2024)
  (버전 간 API 변경이 있으므로 Anchor.toml 버전 확인 중요)
```

다음 장에서는 `anchor init`으로 프로젝트를 생성하고 전체 디렉토리 구조를 살펴봅니다.
