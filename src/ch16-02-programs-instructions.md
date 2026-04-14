# 프로그램과 Instruction: Solana 스마트 컨트랙트

## 프로그램 = 스마트 컨트랙트

Solana에서 **프로그램(Program)**은 이더리움의 스마트 컨트랙트와 동일한 개념입니다. 용어만 다를 뿐, 블록체인 위에서 실행되는 코드입니다.

그러나 결정적인 차이가 있습니다:

```
이더리움 스마트 컨트랙트:     Solana 프로그램:
┌────────────────────┐        ┌────────────────────┐
│  Counter Contract  │        │  Counter Program   │
│                    │        │                    │
│  uint count = 0;   │        │  (상태 없음!)      │
│                    │        │                    │
│  function inc() {  │        │  fn increment(     │
│    count++;        │        │    accounts,        │
│  }                 │        │    data             │
│                    │        │  ) {               │
│  function get() {  │        │    // 외부 account  │
│    return count;   │        │    // 의 data 수정  │
│  }                 │        │  }                 │
└────────────────────┘        └────────────────────┘
  컨트랙트 자체가 상태 보유      프로그램은 순수 로직만
```

**Solana 프로그램은 완전히 stateless**입니다. 상태는 항상 별도의 Account에 저장됩니다. 이는 마치 Rust의 순수 함수처럼, 같은 입력(accounts + data)이 주어지면 항상 같은 결과를 냅니다.

---

## Instruction 구조

Solana에서 프로그램을 호출하는 단위는 **Instruction**입니다. 이더리움에서 컨트랙트 함수를 호출하는 것과 유사하지만, 구조가 더 명시적입니다.

```rust
// Instruction의 세 가지 구성요소

pub struct Instruction {
    /// 호출할 프로그램의 주소
    pub program_id: Pubkey,

    /// 이 Instruction이 읽거나 쓸 Account 목록
    pub accounts: Vec<AccountMeta>,

    /// 프로그램에 전달할 직렬화된 데이터
    pub data: Vec<u8>,
}

pub struct AccountMeta {
    /// Account의 주소
    pub pubkey: Pubkey,

    /// 이 트랜잭션에서 서명자인가?
    pub is_signer: bool,

    /// 이 Instruction에서 데이터를 수정할 수 있는가?
    pub is_writable: bool,
}
```

### 왜 accounts를 미리 선언해야 하는가?

```typescript
// 이더리움: 어떤 storage에 접근할지 런타임에 결정됨
contract.transfer(to, amount); // 내부적으로 어떤 storage 건드리는지 미리 알 수 없음

// Solana: 어떤 Account를 사용할지 미리 명시
const instruction = new TransactionInstruction({
  programId: COUNTER_PROGRAM_ID,
  accounts: [
    { pubkey: counterAccount, isSigner: false, isWritable: true },
    { pubkey: user.publicKey,  isSigner: true,  isWritable: false },
  ],
  data: Buffer.from([0]), // increment 명령
});
```

미리 선언하는 이유: Sealevel이 어떤 트랜잭션들이 병렬 실행 가능한지 분석하기 위해서입니다. 서로 다른 Account를 건드리는 트랜잭션은 동시에 실행될 수 있습니다.

---

## Transaction 구조

**Transaction**은 하나 이상의 Instruction을 묶은 것입니다. 이더리움과 달리 **여러 Instruction이 하나의 Transaction에 포함될 수 있으며, 이는 원자적(atomic)으로 실행됩니다**.

```
Transaction
├── 헤더 (서명자 수, 읽기 전용 계정 수 등)
├── 계정 주소 목록 (중복 제거된 모든 Account)
├── recent_blockhash (재생 공격 방지)
├── Instruction 목록
│   ├── Instruction 1: System Program → 새 Account 생성
│   ├── Instruction 2: Token Program → 토큰 초기화
│   └── Instruction 3: 내 프로그램 → 사용자 등록
└── 서명 목록
```

### Atomic 실행의 중요성

```typescript
// 예시: NFT 민팅 트랜잭션
// 세 Instruction이 모두 성공하거나, 모두 실패해야 함

const mintTx = new Transaction()
  .add(
    // 1. NFT Mint Account 생성 (System Program)
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: mintAccount.publicKey,
      lamports: mintRent,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM_ID,
    })
  )
  .add(
    // 2. Mint 초기화 (Token Program)
    createInitializeMintInstruction(
      mintAccount.publicKey,
      0, // decimals
      payer.publicKey, // mintAuthority
      payer.publicKey  // freezeAuthority
    )
  )
  .add(
    // 3. NFT 메타데이터 생성 (Metaplex Program)
    createCreateMetadataAccountV3Instruction(...)
  );

// 만약 3번이 실패하면, 1번과 2번도 롤백됨!
await sendAndConfirmTransaction(connection, mintTx, [payer, mintAccount]);
```

이더리움에서 여러 컨트랙트 호출을 원자적으로 처리하려면 별도의 Multicall 컨트랙트가 필요했지만, Solana는 이것이 기본 기능입니다.

---

## 직렬화: Borsh

Solana의 Instruction `data` 필드와 Account `data` 필드는 모두 바이트 배열입니다. 이 바이트 배열을 만들고 해석하는 직렬화 방식으로 **Borsh**를 사용합니다.

### Borsh vs JSON 비교

```
JSON 직렬화:
{"action": "increment", "amount": 5}
→ 33 bytes, 파싱 오버헤드 있음

Borsh 직렬화:
[0, 5, 0, 0, 0, 0, 0, 0, 0]  (action=0, amount=5 as u64 little-endian)
→ 9 bytes, 파싱이 매우 빠름

이름의 유래: Binary Object Representation Serializer for Hashing
특징:
- 결정론적 (같은 데이터 → 항상 같은 바이트)
- 스키마 없이 역직렬화 가능
- 매우 빠르고 간결
- Rust, JavaScript, Python 등 다양한 언어 지원
```

```rust
// Rust에서 Borsh 사용
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterInstruction {
    pub action: u8,   // 0 = increment, 1 = decrement, 2 = reset
    pub amount: u64,
}

// 직렬화
let ix = CounterInstruction { action: 0, amount: 5 };
let bytes = ix.try_to_vec().unwrap();
// bytes = [0, 5, 0, 0, 0, 0, 0, 0, 0]

// 역직렬화
let ix_parsed = CounterInstruction::try_from_slice(&bytes).unwrap();
assert_eq!(ix_parsed.action, 0);
assert_eq!(ix_parsed.amount, 5);
```

```typescript
// TypeScript에서 Borsh 사용
import * as borsh from 'borsh';

class CounterInstruction {
  action: number;
  amount: bigint;

  constructor(fields: { action: number; amount: bigint }) {
    this.action = fields.action;
    this.amount = fields.amount;
  }
}

const schema = new Map([
  [CounterInstruction, {
    kind: 'struct',
    fields: [
      ['action', 'u8'],
      ['amount', 'u64'],
    ],
  }],
]);

// 직렬화
const instruction = new CounterInstruction({ action: 0, amount: BigInt(5) });
const bytes = borsh.serialize(schema, instruction);

// 역직렬화
const parsed = borsh.deserialize(schema, CounterInstruction, Buffer.from(bytes));
```

---

## Native 프로그램 작성 (Anchor 없이)

Anchor 프레임워크 없이 Raw Rust로 카운터 프로그램을 작성해보겠습니다. 이를 통해 Solana 프로그램의 기본 구조를 이해할 수 있습니다.

### 프로젝트 구조

```
counter/
├── Cargo.toml
└── src/
    └── lib.rs
```

### Cargo.toml

```toml
[package]
name = "counter"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
solana-program = "1.18"
borsh = { version = "0.10", features = ["derive"] }
borsh-derive = "0.10"
```

### src/lib.rs - 완전한 카운터 프로그램

```rust
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    program::invoke,
    sysvar::Sysvar,
};

// ============================================================
// 1. 데이터 구조 정의
// ============================================================

/// 카운터 계정에 저장될 데이터
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,
    pub authority: Pubkey,  // 카운터를 초기화한 사람
    pub is_initialized: bool,
}

impl CounterAccount {
    pub const LEN: usize =
        8 +   // count: u64
        32 +  // authority: Pubkey
        1;    // is_initialized: bool
    // = 41 bytes
}

/// Instruction 타입 정의
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    /// 카운터 초기화
    /// accounts: [counter_account(writable), authority(signer), system_program]
    Initialize,

    /// 카운터 증가
    /// accounts: [counter_account(writable), authority(signer)]
    Increment { amount: u64 },

    /// 카운터 감소
    /// accounts: [counter_account(writable), authority(signer)]
    Decrement { amount: u64 },

    /// 카운터 리셋
    /// accounts: [counter_account(writable), authority(signer)]
    Reset,
}

// ============================================================
// 2. 엔트리포인트 등록
// ============================================================

// 이 매크로가 프로그램의 진입점을 등록합니다
// 이더리움의 fallback 함수와 유사한 역할
entrypoint!(process_instruction);

// ============================================================
// 3. 메인 처리 함수
// ============================================================

pub fn process_instruction(
    program_id: &Pubkey,        // 이 프로그램의 주소
    accounts: &[AccountInfo],   // 트랜잭션에서 전달된 계정들
    instruction_data: &[u8],    // 직렬화된 명령 데이터
) -> ProgramResult {

    // Instruction 역직렬화
    let instruction = CounterInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Instruction 종류에 따라 라우팅
    match instruction {
        CounterInstruction::Initialize => {
            process_initialize(program_id, accounts)
        }
        CounterInstruction::Increment { amount } => {
            process_increment(accounts, amount)
        }
        CounterInstruction::Decrement { amount } => {
            process_decrement(accounts, amount)
        }
        CounterInstruction::Reset => {
            process_reset(accounts)
        }
    }
}

// ============================================================
// 4. 각 Instruction 처리 함수
// ============================================================

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    // 계정 목록 파싱 (순서가 중요!)
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // 검증: authority가 서명했는가?
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 검증: counter_account가 이미 초기화되었는가?
    if counter_account.data_len() > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // 렌트 면제를 위한 최소 lamports 계산
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(CounterAccount::LEN);

    // System Program에 CPI로 계정 생성
    // (CPI = Cross-Program Invocation, 프로그램이 다른 프로그램 호출)
    invoke(
        &system_instruction::create_account(
            authority.key,          // 비용 지불자
            counter_account.key,    // 새로 만들 계정
            required_lamports,      // 렌트 면제 보증금
            CounterAccount::LEN as u64,  // 데이터 크기
            program_id,             // 소유자 = 이 프로그램
        ),
        &[
            authority.clone(),
            counter_account.clone(),
            system_program.clone(),
        ],
    )?;

    // 초기 데이터 저장
    let counter_data = CounterAccount {
        count: 0,
        authority: *authority.key,
        is_initialized: true,
    };

    let mut account_data = counter_account.data.borrow_mut();
    counter_data.serialize(&mut *account_data)?;

    msg!("카운터 초기화 완료! authority: {}", authority.key);
    Ok(())
}

fn process_increment(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;

    // 서명 검증
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 현재 데이터 읽기
    let mut counter_data =
        CounterAccount::try_from_slice(&counter_account.data.borrow())?;

    // 권한 검증: 초기화한 사람만 수정 가능
    if counter_data.authority != *authority.key {
        return Err(ProgramError::IllegalOwner);
    }

    // 초기화 여부 확인
    if !counter_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    // 오버플로 방지하며 증가
    counter_data.count = counter_data.count
        .checked_add(amount)
        .ok_or(ProgramError::InvalidArgument)?;

    // 수정된 데이터 저장
    counter_data.serialize(&mut *counter_account.data.borrow_mut())?;

    msg!("카운터 증가: {} → {}", counter_data.count - amount, counter_data.count);
    Ok(())
}

fn process_decrement(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut counter_data =
        CounterAccount::try_from_slice(&counter_account.data.borrow())?;

    if counter_data.authority != *authority.key {
        return Err(ProgramError::IllegalOwner);
    }

    // 언더플로 방지
    counter_data.count = counter_data.count
        .checked_sub(amount)
        .ok_or(ProgramError::InvalidArgument)?;

    counter_data.serialize(&mut *counter_account.data.borrow_mut())?;

    msg!("카운터 감소: {}", counter_data.count);
    Ok(())
}

fn process_reset(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut counter_data =
        CounterAccount::try_from_slice(&counter_account.data.borrow())?;

    if counter_data.authority != *authority.key {
        return Err(ProgramError::IllegalOwner);
    }

    counter_data.count = 0;
    counter_data.serialize(&mut *counter_account.data.borrow_mut())?;

    msg!("카운터 리셋!");
    Ok(())
}
```

---

## 이더리움 컨트랙트 호출과의 비교

```
이더리움 (Solidity):
─────────────────────────────────────────────────────
// 컨트랙트 배포 후 영구 주소
address contractAddress = 0xA0b86991c6218b36c1d19D4...;

// ABI로 함수 호출
await contract.increment(5);
// → tx에 함수 선택자 + 인코딩된 인자가 data로 들어감
// → 컨트랙트 내부의 count storage를 직접 수정


Solana (Native Rust):
─────────────────────────────────────────────────────
// 프로그램 ID (배포된 주소)
const PROGRAM_ID = new PublicKey("Counter111...");

// Instruction 생성: 어떤 계정을 사용할지 명시
const ix = new TransactionInstruction({
  programId: PROGRAM_ID,
  accounts: [
    { pubkey: counterPubkey, isSigner: false, isWritable: true },
    { pubkey: user.publicKey,  isSigner: true,  isWritable: false },
  ],
  data: borsh.serialize(schema, new IncrementInstruction({ amount: 5n })),
});

// 트랜잭션 실행
await sendAndConfirmTransaction(connection, new Transaction().add(ix), [user]);
// → 프로그램이 counterPubkey Account의 data를 수정
```

---

## 클라이언트 코드: TypeScript로 카운터 호출

```typescript
import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
  AccountMeta,
} from '@solana/web3.js';
import * as borsh from 'borsh';

const PROGRAM_ID = new PublicKey('여기에_배포된_프로그램_ID');
const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

// Instruction 스키마 (Borsh)
class InitializeInstruction {
  instruction: number = 0;
  constructor() {}
}
class IncrementInstruction {
  instruction: number = 1;
  amount: bigint;
  constructor(amount: bigint) { this.amount = amount; }
}

const schema = new Map([
  [InitializeInstruction, { kind: 'struct', fields: [['instruction', 'u8']] }],
  [IncrementInstruction,  { kind: 'struct', fields: [['instruction', 'u8'], ['amount', 'u64']] }],
]);

async function initializeCounter(
  payer: Keypair,
  counterKeypair: Keypair
): Promise<void> {
  const ix = new TransactionInstruction({
    programId: PROGRAM_ID,
    accounts: [
      { pubkey: counterKeypair.publicKey, isSigner: true,  isWritable: true  },
      { pubkey: payer.publicKey,          isSigner: true,  isWritable: true  },
      { pubkey: SystemProgram.programId,  isSigner: false, isWritable: false },
    ],
    data: Buffer.from(borsh.serialize(schema, new InitializeInstruction())),
  });

  const tx = new Transaction().add(ix);
  const sig = await sendAndConfirmTransaction(connection, tx, [payer, counterKeypair]);
  console.log(`초기화 완료! 서명: ${sig}`);
}

async function incrementCounter(
  payer: Keypair,
  counterPubkey: PublicKey,
  amount: bigint
): Promise<void> {
  const ix = new TransactionInstruction({
    programId: PROGRAM_ID,
    accounts: [
      { pubkey: counterPubkey,   isSigner: false, isWritable: true  },
      { pubkey: payer.publicKey, isSigner: true,  isWritable: false },
    ],
    data: Buffer.from(borsh.serialize(schema, new IncrementInstruction(amount))),
  });

  const tx = new Transaction().add(ix);
  await sendAndConfirmTransaction(connection, tx, [payer]);
  console.log(`카운터 ${amount} 증가!`);
}
```

---

## 핵심 정리

```
Solana 프로그램 실행 흐름:

사용자
  │
  ▼
Transaction
  ├── Instruction 1
  │   ├── program_id: 어떤 프로그램?
  │   ├── accounts:   어떤 계정들?
  │   └── data:       무슨 명령? (Borsh 직렬화)
  └── Instruction 2
      └── ...
  │
  ▼
Solana Runtime (Sealevel)
  ├── 병렬 실행 가능한 트랜잭션 분석
  ├── 프로그램 로드
  └── process_instruction(program_id, accounts, data) 호출
  │
  ▼
프로그램 실행
  ├── Instruction 역직렬화
  ├── 계정 검증 (owner, signer, writable)
  ├── 비즈니스 로직 실행
  └── Account data 수정
```

다음 장에서는 PDA(Program Derived Address)와 CPI(Cross-Program Invocation)를 통해 프로그램들이 어떻게 서로 상호작용하는지 배웁니다.
