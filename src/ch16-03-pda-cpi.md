# PDA와 CPI: 프로그램 간 상호작용

## PDA (Program Derived Address)

### 왜 PDA가 필요한가?

Solana에서 일반 Account는 개인키(private key)로 서명해야 트랜잭션을 보낼 수 있습니다. 그런데 프로그램이 "자신을 대신해서" Account를 제어하고 싶을 때 문제가 생깁니다.

```text
시나리오: 에스크로(Escrow) 프로그램
─────────────────────────────────────
Alice가 100 SOL을 에스크로에 예치
→ Bob이 조건 이행 시 자동으로 SOL 받음

문제: 에스크로 Account의 SOL을 누가 전송하는가?
- Alice도 아니고 (이미 에스크로에 넣었으므로)
- Bob도 아니고 (조건 확인 전에 가져가면 안 되므로)
- 에스크로 프로그램이 직접 전송해야 함!

하지만 프로그램은 private key가 없으므로 서명 불가!
→ PDA가 해결책
```

**PDA는 프로그램이 "소유"하고 "서명"할 수 있는 특수 주소입니다.**

---

### PDA 생성 원리

일반 Keypair는 ed25519 타원 곡선 위에 존재하는 점(point)입니다. PDA는 의도적으로 **곡선 밖**에 있는 주소를 사용합니다.

```text
ed25519 타원 곡선:

    y
    │     ·  ·  ·
    │  ·           ·
    │ ·               ·   ← 곡선 위 점 = 일반 Keypair (private key 존재)
    │·                 ·
    ──────────────────────── x
    │·                 ·
    │ ·               ·
    │  ·           ·
    │     ·  ·  ·
    │
    │                    × ← 곡선 밖 점 = PDA (private key 없음!)

find_program_address(seeds, program_id):
  1. SHA256(seeds + program_id + bump) 계산
  2. 결과가 곡선 위에 있으면 bump 감소 후 재시도
  3. 결과가 곡선 밖이면 → 이것이 PDA!
  bump는 255부터 시작해서 감소 (canonical bump = 최초 성공 값)
```

```rust,ignore
// PDA 생성 코드
use solana_program::pubkey::Pubkey;

// find_program_address: canonical bump를 자동으로 찾아줌
let seeds = &[
    b"user-data",          // 문자열 시드
    user_pubkey.as_ref(),  // 사용자 주소 시드
];
let (pda, bump) = Pubkey::find_program_address(seeds, &program_id);

// 반환값:
// pda: 생성된 PDA 주소 (Pubkey)
// bump: 곡선 밖으로 밀어낸 값 (0~255)

// create_program_address: bump를 직접 지정 (검증용)
let pda_verify = Pubkey::create_program_address(
    &[b"user-data", user_pubkey.as_ref(), &[bump]],
    &program_id,
)?;
assert_eq!(pda, pda_verify);
```

```typescript
// TypeScript에서 PDA 찾기
import { PublicKey } from '@solana/web3.js';

const PROGRAM_ID = new PublicKey("MyProg111...");
const userPubkey = new PublicKey("User111...");

// seeds는 Buffer 또는 Uint8Array 배열
const [pda, bump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("user-data"),  // 문자열 시드
    userPubkey.toBuffer(),     // 주소 시드
  ],
  PROGRAM_ID
);

console.log("PDA:", pda.toBase58());
console.log("Bump:", bump);

// 같은 seeds + program_id → 항상 같은 PDA (결정론적!)
// 즉, 클라이언트와 프로그램이 독립적으로 같은 주소를 계산할 수 있음
```

---

### PDA의 실제 사용 패턴

#### 패턴 1: 사용자별 데이터 계정

```rust,ignore
// 각 사용자마다 고유한 데이터 계정을 결정론적으로 생성
let seeds = &[
    b"player",
    user_pubkey.as_ref(),
    &[bump],
];
// → 같은 user는 항상 같은 PDA 주소
// → 클라이언트가 사전에 주소 계산 가능 (DB 조회 불필요!)
```

#### 패턴 2: 글로벌 상태 계정

```rust,ignore
// 프로그램 전체의 설정/상태를 저장
let seeds = &[b"global-config", &[bump]];
// → 프로그램당 하나의 고정된 설정 계정
```

#### 패턴 3: 에스크로 금고

```rust,ignore
// 특정 거래의 SOL/토큰을 보관하는 금고
let seeds = &[
    b"escrow",
    trade_id.as_ref(),
    &[bump],
];
// → 거래 ID마다 고유한 에스크로 계정
```

---

### PDA Rust 코드 예제: 사용자 프로필 시스템

```rust,ignore
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserProfile {
    pub owner: Pubkey,
    pub username: String,
    pub score: u64,
    pub bump: u8,           // PDA bump 저장 (나중에 invoke_signed에 사용)
}

impl UserProfile {
    pub const MAX_USERNAME_LEN: usize = 32;
    pub const LEN: usize = 32 + 4 + Self::MAX_USERNAME_LEN + 8 + 1;
    // owner(32) + string_len(4) + username(32) + score(8) + bump(1)
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // username 파싱 (첫 바이트가 길이, 나머지가 UTF-8)
    let username_len = data[0] as usize;
    let username = std::str::from_utf8(&data[1..1 + username_len])
        .map_err(|_| ProgramError::InvalidInstructionData)?
        .to_string();

    let accounts_iter = &mut accounts.iter();
    let profile_account = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // PDA 검증: 클라이언트가 올바른 PDA를 전달했는가?
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[b"profile", user.key.as_ref()],
        program_id,
    );

    if expected_pda != *profile_account.key {
        msg!("잘못된 PDA 주소");
        return Err(ProgramError::InvalidArgument);
    }

    // 렌트 계산
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(UserProfile::LEN);

    // PDA Account 생성 (invoke_signed 사용!)
    // 일반 invoke와 달리, PDA seeds를 제공하여 프로그램이 서명
    invoke_signed(
        &system_instruction::create_account(
            user.key,
            profile_account.key,
            required_lamports,
            UserProfile::LEN as u64,
            program_id,
        ),
        &[user.clone(), profile_account.clone(), system_program.clone()],
        // signer_seeds: PDA를 "서명"하는 seeds
        &[&[b"profile", user.key.as_ref(), &[bump]]],
    )?;

    // 프로필 데이터 저장
    let profile = UserProfile {
        owner: *user.key,
        username,
        score: 0,
        bump,
    };
    profile.serialize(&mut *profile_account.data.borrow_mut())?;

    msg!("프로필 생성 완료!");
    Ok(())
}
```

---

## CPI (Cross-Program Invocation)

### CPI란?

CPI는 하나의 프로그램이 다른 프로그램을 호출하는 기능입니다. 이더리움의 external call 또는 NestJS에서 서비스가 다른 서비스를 호출하는 것과 유사합니다.

```text
이더리움 external call:
─────────────────────────────
MyContract.foo() {
    // 다른 컨트랙트 호출
    IERC20(tokenAddress).transferFrom(from, to, amount);
}

NestJS 서비스 간 호출:
─────────────────────────────
@Injectable()
class OrderService {
  constructor(private paymentService: PaymentService) {}

  async createOrder() {
    await this.paymentService.charge(amount);  // 다른 서비스 호출
  }
}

Solana CPI:
─────────────────────────────
// 내 프로그램에서 Token Program의 transfer 호출
invoke(
    &token_instruction::transfer(...),
    &[from_account, to_account, authority],
)?;
```

### invoke() vs invoke_signed()

```rust,ignore
// invoke(): 일반 CPI (PDA 서명 불필요)
// 사용 케이스: user가 서명자인 경우
invoke(
    &system_instruction::transfer(from_pubkey, to_pubkey, lamports),
    &[from_account.clone(), to_account.clone(), system_program.clone()],
)?;

// invoke_signed(): PDA가 서명자인 CPI
// 사용 케이스: 프로그램 자신의 PDA에서 자산 이동
invoke_signed(
    &system_instruction::transfer(pda_pubkey, to_pubkey, lamports),
    &[pda_account.clone(), to_account.clone(), system_program.clone()],
    &[&[b"vault", &[bump]]],  // PDA seeds로 서명
)?;
```

---

### CPI 예제 1: System Program에 CPI로 SOL 전송

```rust,ignore
// 시나리오: 내 프로그램의 PDA(금고)에서 SOL을 사용자에게 전송

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction,
};

pub fn withdraw_from_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let vault_account = next_account_info(accounts_iter)?;   // PDA 금고
    let recipient = next_account_info(accounts_iter)?;        // 받는 사람
    let system_program = next_account_info(accounts_iter)?;   // System Program

    // PDA 검증
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault"],
        program_id,
    );
    assert_eq!(vault_pda, *vault_account.key);

    // PDA(금고)에서 recipient에게 SOL 전송
    // vault_account는 private key가 없으므로 invoke_signed 필요
    invoke_signed(
        &system_instruction::transfer(
            vault_account.key,  // from (PDA)
            recipient.key,      // to
            amount,
        ),
        &[
            vault_account.clone(),
            recipient.clone(),
            system_program.clone(),
        ],
        // PDA의 seeds로 서명 (private key 없이도 서명 가능!)
        &[&[b"vault", &[bump]]],
    )?;

    Ok(())
}
```

### CPI 예제 2: Token Program에 CPI로 토큰 전송

```rust,ignore
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_lang::prelude::*;

// Anchor를 사용한 Token Program CPI
pub fn transfer_tokens<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    authority: &Signer<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        token_program.to_account_info(),
        cpi_accounts,
    );

    token::transfer(cpi_ctx, amount)?;
    Ok(())
}

// PDA가 authority인 경우 (invoke_signed 케이스)
pub fn transfer_tokens_with_pda<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    pda_authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
    seeds: &[&[u8]],  // PDA seeds
) -> Result<()> {
    let cpi_accounts = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: pda_authority.clone(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        token_program.to_account_info(),
        cpi_accounts,
        &[seeds],  // signer_seeds
    );

    token::transfer(cpi_ctx, amount)?;
    Ok(())
}
```

---

### 이더리움 external call과의 비교

```text
이더리움 external call:
─────────────────────────────────────────────────────────────
// Re-entrancy 공격 가능!
// 악의적인 컨트랙트가 callback으로 재진입할 수 있음
contract Vulnerable {
    mapping(address => uint) balances;

    function withdraw() external {
        uint amount = balances[msg.sender];
        (bool success,) = msg.sender.call{value: amount}(""); // ← 위험!
        balances[msg.sender] = 0;  // 이미 늦음
    }
}

Solana CPI:
─────────────────────────────────────────────────────────────
// Re-entrancy가 구조적으로 불가능!
// 이유 1: 프로그램은 자신이 owner인 계정만 수정 가능
// 이유 2: 호출 스택에서 같은 프로그램의 재진입 금지
// 이유 3: 계정 잠금(borrow) 메커니즘

invoke(
    &token::transfer(...),
    accounts,
)?;
// CPI 완료 후에만 다음 코드 실행
// 중간에 재진입 불가
```

---

### PDA + CPI 종합 예제: 에스크로 프로그램

```rust,ignore
// 완전한 에스크로 흐름:
// 1. Alice가 SOL을 에스크로 PDA에 예치
// 2. 조건 확인 후 프로그램이 Bob에게 자동 전송

#[derive(BorshSerialize, BorshDeserialize)]
pub struct EscrowData {
    pub depositor: Pubkey,   // Alice
    pub recipient: Pubkey,   // Bob
    pub amount: u64,
    pub condition_met: bool,
    pub bump: u8,
}

// Step 1: Alice가 에스크로 생성 및 SOL 예치
pub fn create_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    recipient: Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let escrow_account = next_account_info(accounts_iter)?;
    let alice = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let (escrow_pda, bump) = Pubkey::find_program_address(
        &[b"escrow", alice.key.as_ref()],
        program_id,
    );

    // PDA 생성 (System Program에 CPI)
    invoke_signed(
        &system_instruction::create_account(
            alice.key,
            escrow_account.key,
            amount + rent_minimum,  // 예치금 + 렌트
            EscrowData::LEN as u64,
            program_id,
        ),
        &[alice.clone(), escrow_account.clone(), system_program.clone()],
        &[&[b"escrow", alice.key.as_ref(), &[bump]]],
    )?;

    // Alice의 SOL을 에스크로로 전송 (Alice가 서명자이므로 invoke 사용)
    invoke(
        &system_instruction::transfer(alice.key, escrow_account.key, amount),
        &[alice.clone(), escrow_account.clone(), system_program.clone()],
    )?;

    let data = EscrowData {
        depositor: *alice.key,
        recipient,
        amount,
        condition_met: false,
        bump,
    };
    data.serialize(&mut *escrow_account.data.borrow_mut())?;

    Ok(())
}

// Step 2: 조건 이행 후 Bob에게 SOL 전송
pub fn release_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let escrow_account = next_account_info(accounts_iter)?;
    let bob = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let mut data = EscrowData::try_from_slice(&escrow_account.data.borrow())?;

    // 수취인 검증
    if data.recipient != *bob.key {
        return Err(ProgramError::InvalidArgument);
    }

    // 에스크로 PDA에서 Bob에게 SOL 전송 (PDA가 서명자이므로 invoke_signed)
    invoke_signed(
        &system_instruction::transfer(
            escrow_account.key,
            bob.key,
            data.amount,
        ),
        &[escrow_account.clone(), bob.clone(), system_program.clone()],
        // PDA seeds로 서명! private key 없이도 가능
        &[&[b"escrow", data.depositor.as_ref(), &[data.bump]]],
    )?;

    Ok(())
}
```

---

## PDA와 CPI 핵심 정리

```text
PDA 요약:
┌────────────────────────────────────────────────────┐
│  PDA = seeds + program_id → 곡선 밖의 결정론적 주소 │
│                                                    │
│  특징:                                             │
│  • private key 없음 → 탈취 불가                    │
│  • 프로그램만 서명 가능 (invoke_signed)             │
│  • 결정론적 → 클라이언트가 미리 계산 가능           │
│  • 사용자별 데이터 계정, 금고, 에스크로 등에 활용   │
└────────────────────────────────────────────────────┘

CPI 요약:
┌────────────────────────────────────────────────────┐
│  invoke()        = 일반 호출 (user가 서명자)        │
│  invoke_signed() = PDA가 서명자인 호출              │
│                                                    │
│  이더리움 대비 장점:                                │
│  • Re-entrancy 공격 구조적 불가                    │
│  • 계정 접근 권한이 명확                           │
│  • Atomic 실행 보장                                │
└────────────────────────────────────────────────────┘
```

다음 장부터는 Anchor 프레임워크를 배워 이 모든 보일러플레이트를 대폭 줄이겠습니다.
