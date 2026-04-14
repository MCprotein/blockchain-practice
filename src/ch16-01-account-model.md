# Account 모델: Solana에서는 모든 것이 Account다

## 핵심 개념 이해

Solana를 처음 배울 때 가장 먼저, 가장 깊이 이해해야 할 개념이 바로 **Account 모델**입니다. 이더리움에서 넘어온 개발자들이 가장 많이 혼란스러워하는 부분이기도 합니다.

> **"Solana에서는 모든 것이 Account다"**

SOL을 보유한 지갑? Account입니다.
스마트 컨트랙트(프로그램)? Account입니다.
프로그램이 저장하는 데이터? Account입니다.
시스템 프로그램(SOL 전송 기능)? Account입니다.

이더리움은 EOA(Externally Owned Account)와 Contract Account 두 종류만 있었다면, Solana는 훨씬 더 유연하고 일관된 단일 Account 구조를 사용합니다.

---

## Account의 구조

모든 Solana Account는 다음 필드를 가집니다:

```
┌─────────────────────────────────────────────────────┐
│                    Account                          │
├─────────────────┬───────────────────────────────────┤
│ lamports        │ u64 - SOL 잔액 (1 SOL = 10^9 lamports) │
├─────────────────┼───────────────────────────────────┤
│ data            │ Vec<u8> - 임의의 바이트 배열        │
├─────────────────┼───────────────────────────────────┤
│ owner           │ Pubkey - 이 계정을 "소유"하는 프로그램 │
├─────────────────┼───────────────────────────────────┤
│ executable      │ bool - 프로그램인지 여부            │
├─────────────────┼───────────────────────────────────┤
│ rent_epoch      │ u64 - 렌트 관련 (현재는 거의 무시)  │
└─────────────────┴───────────────────────────────────┘
```

### lamports (잔액)

```rust
// 1 SOL = 1,000,000,000 lamports (10억)
// lamport는 Solana의 최소 단위
// Leslie Lamport (분산 시스템 대가)의 이름에서 유래

let balance_in_sol = account.lamports as f64 / 1_000_000_000.0;
println!("잔액: {} SOL", balance_in_sol);

// JavaScript에서:
const lamports = await connection.getBalance(publicKey);
const sol = lamports / LAMPORTS_PER_SOL; // LAMPORTS_PER_SOL = 1e9
```

### data (바이트 배열)

Account의 `data` 필드는 그냥 바이트 배열(`Vec<u8>`)입니다. 이 바이트 배열에 무엇을 어떻게 저장할지는 **owner 프로그램**이 결정합니다.

```
지갑 Account:        data = [] (비어있음)
Token Account:       data = [mint(32), owner(32), amount(8), ...] (165 bytes)
Mint Account:        data = [mint_authority(36), supply(8), decimals(1), ...] (82 bytes)
커스텀 프로그램 데이터: data = 프로그램이 원하는 구조로 직렬화
```

### owner (소유 프로그램)

`owner`는 이 계정의 `data`를 읽고 쓸 수 있는 프로그램의 주소입니다.

```
중요 규칙:
1. 프로그램은 자신이 owner인 계정의 data만 수정 가능
2. 어떤 프로그램도 다른 프로그램 소유 계정의 data를 직접 수정 불가
3. 단, lamports 감소는 서명자가 가능 (전송)

예시:
- 일반 지갑: owner = System Program (11111111...)
- Token Account: owner = Token Program (TokenkegQfe...)
- 내가 만든 프로그램 데이터: owner = 내 프로그램 주소
```

### executable (실행 가능 여부)

```rust
// executable = true → 프로그램 Account (스마트 컨트랙트)
// executable = false → 데이터 Account (일반 계정)

// 프로그램 Account의 data에는 BPF 바이트코드가 저장됨
// executable Account는 수정 불가 (업그레이드 가능 프로그램은 별도 메커니즘 사용)
```

---

## Account의 세 가지 유형

### 1. 데이터 계정 (Data Account)

프로그램이 상태를 저장하는 계정입니다. `executable = false`.

```
┌─────────────────────────────────────┐
│         데이터 계정 예시             │
├────────────────┬────────────────────┤
│ lamports       │ 2,000,000          │  ← 렌트 면제 보증금
│ data           │ { score: 100,      │  ← 프로그램이 정의한 구조
│                │   player: "Alice" }│
│ owner          │ 내 게임 프로그램    │  ← 이 프로그램만 data 수정 가능
│ executable     │ false              │
│ rent_epoch     │ 0                  │
└────────────────┴────────────────────┘
```

데이터 계정의 예:
- 사용자의 토큰 잔액을 저장하는 Token Account
- NFT 메타데이터를 저장하는 Account
- 게임 플레이어의 점수를 저장하는 Account
- DEX의 유동성 풀 정보를 저장하는 Account

### 2. 프로그램 계정 (Program Account)

스마트 컨트랙트 코드가 저장된 계정입니다. `executable = true`.

```
┌─────────────────────────────────────┐
│         프로그램 계정                │
├────────────────┬────────────────────┤
│ lamports       │ 1,141,440          │  ← 렌트 면제 보증금
│ data           │ [BPF 바이트코드]   │  ← 컴파일된 프로그램
│ owner          │ BPF Loader         │  ← BPF Loader가 소유
│ executable     │ true               │  ← 실행 가능!
│ rent_epoch     │ 0                  │
└────────────────┴────────────────────┘

주소 예시:
- Token Program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
- System Program: 11111111111111111111111111111111
```

### 3. 네이티브 계정 (Native Program Account)

Solana 런타임에 내장된 특별 프로그램들입니다.

```
System Program (11111111111111111111111111111111)
  → 새 계정 생성, SOL 전송, 프로그램 배포

Token Program (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA)
  → SPL 토큰 발행, 전송, 소각

Token-2022 Program (TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb)
  → 확장 기능이 추가된 새 토큰 프로그램

BPF Loader (BPFLoaderUpgradeab1e11111111111111111111111)
  → 프로그램 배포 및 업그레이드 관리

Sysvar 계정들 (읽기 전용 시스템 정보)
  → Clock: 현재 시간/슬롯 정보
  → Rent: 렌트 정책 정보
  → EpochSchedule: 에폭 스케줄 정보
```

---

## 이더리움 vs Solana 상태 모델 심층 비교

### NestJS 관점으로 이해하기

```typescript
// 이더리움 방식: 컨트랙트가 자체 저장소를 가짐
// NestJS 비유: 서비스가 자체 인메모리 Map으로 상태 관리

@Injectable()
class EthereumStyleUserService {
  // 컨트랙트 storage 변수처럼, 서비스 안에 데이터가 있음
  private balances: Map<string, number> = new Map();
  private totalSupply: number = 0;

  transfer(from: string, to: string, amount: number) {
    // 같은 서비스 안의 데이터를 직접 수정
    this.balances.set(from, this.balances.get(from)! - amount);
    this.balances.set(to, (this.balances.get(to) || 0) + amount);
  }
}
```

```typescript
// Solana 방식: 프로그램은 로직만, 데이터는 외부 Account에
// NestJS 비유: 서비스는 로직만 갖고, 외부 DB(Account)를 읽고 씀

@Injectable()
class SolanaStyleTokenProgram {
  // 프로그램 자체에 상태 없음!

  transfer(
    fromAccount: TokenAccount,  // 외부에서 전달받은 Account
    toAccount: TokenAccount,    // 외부에서 전달받은 Account
    amount: number
  ) {
    // 외부 Account의 데이터를 수정
    fromAccount.balance -= amount;
    toAccount.balance += amount;
    // 변경사항은 각 Account에 저장됨
  }
}

// 실제 Solana에서:
// 트랜잭션을 보낼 때 어떤 Account를 사용할지 미리 명시해야 함
// → Sealevel 병렬 처리의 기반!
```

### 이더리움 ERC20 vs Solana SPL Token 비교

```
이더리움 ERC20:
┌─────────────────────────────────────────┐
│           ERC20 컨트랙트                │
│  address: 0xA0b86991c6218b36c1d19D4...  │
│                                         │
│  mapping balances:                      │
│    0xUser1 → 100 USDC                   │
│    0xUser2 → 50 USDC                    │
│    0xUser3 → 200 USDC                   │
│                                         │
│  uint256 totalSupply: 350               │
│  string name: "USD Coin"                │
│  uint8 decimals: 6                      │
└─────────────────────────────────────────┘

Solana SPL Token:
┌──────────────┐
│  Mint 계정   │  ← 토큰 자체 정보 (ERC20 컨트랙트의 메타데이터)
│  supply: 350 │
│  decimals: 6 │
│  authority   │
└──────┬───────┘
       │ "이 Mint에 속하는 Token Account들"
       ├──────────────────────────────┐
       │                              │
┌──────▼──────┐               ┌──────▼──────┐
│ User1의     │               │ User2의     │
│ Token 계정  │               │ Token 계정  │
│ owner: User1│               │ owner: User2│
│ amount: 100 │               │ amount: 50  │
│ mint: USDC  │               │ mint: USDC  │
└─────────────┘               └─────────────┘
```

**핵심 차이**: Solana에서 각 사용자는 토큰마다 별도의 Token Account를 생성해야 합니다. 이것이 Solana에서 새 토큰을 받으려면 먼저 "Associated Token Account"를 만들어야 하는 이유입니다.

---

## 렌트 (Rent): 계정 유지 비용

Solana에서 Account를 유지하려면 **렌트**를 지불해야 합니다. 이는 검증자들의 RAM 사용에 대한 비용입니다.

### 렌트 작동 방식

```
과거 방식 (현재는 deprecated):
- 매 에폭마다 account.lamports에서 렌트 차감
- lamports가 0이 되면 계정 삭제

현재 방식: Rent-Exempt (렌트 면제)
- 충분한 lamports를 보증금으로 예치하면 렌트 영구 면제
- 최소 lamports = 데이터 크기에 따라 계산
- 계정을 닫을 때 보증금을 돌려받음
```

### Rent-Exempt 계산

```rust
// 렌트 면제 최소 lamports 계산
// 기준: ~0.00000348 SOL per byte per year × 2년 = 면제

// 대략적인 계산:
// 128 bytes 계정 = 약 890,880 lamports = 0.00089 SOL
// 0 bytes 계정   = 약 890,880 lamports (기본 오버헤드)

// JavaScript로 계산:
const { Connection, LAMPORTS_PER_SOL } = require('@solana/web3.js');
const connection = new Connection('https://api.devnet.solana.com');

async function calculateRentExempt(dataSize: number): Promise<number> {
  const rentExemptBalance = await connection.getMinimumBalanceForRentExemption(dataSize);
  console.log(`${dataSize} bytes 계정의 렌트 면제 최소: ${rentExemptBalance} lamports`);
  console.log(`= ${rentExemptBalance / LAMPORTS_PER_SOL} SOL`);
  return rentExemptBalance;
}

// 실제 값 예시:
// 0 bytes  → 890,880 lamports (0.00089 SOL)
// 165 bytes (Token Account) → 2,039,280 lamports (0.00204 SOL)
// 200 bytes → 2,277,120 lamports (0.00228 SOL)
```

### Rent-Exempt 조건

```
계정 lamports ≥ 2년치 렌트 → 렌트 면제 (영구 유지)
계정 lamports < 2년치 렌트 → 매 에폭마다 차감 (결국 삭제)
```

실제로 Solana 생태계에서는 모든 계정을 **rent-exempt 상태로 생성**하는 것이 표준입니다.

---

## ASCII 아트: Account 전체 구조 시각화

```
Solana 네트워크의 전체 Account 구조:

┌─────────────────────────────────────────────────────────────────┐
│                        Solana 상태 (State)                      │
│                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐                     │
│  │  System Program │    │  Token Program  │   Native Programs   │
│  │  executable=true│    │  executable=true│                     │
│  │  data=[bytecode]│    │  data=[bytecode]│                     │
│  └────────┬────────┘    └────────┬────────┘                     │
│           │ owns                 │ owns                         │
│           ▼                      ▼                              │
│  ┌────────────────┐   ┌──────────────────┐   ┌──────────────┐  │
│  │  Alice 지갑    │   │  Alice USDC      │   │  Mint 계정   │  │
│  │  lamports:5SOL │   │  Token Account   │   │  USDC 정보   │  │
│  │  data:[]       │   │  amount: 100     │   │  supply:1000 │  │
│  │  exec: false   │   │  exec: false     │   │  exec: false │  │
│  └────────────────┘   └──────────────────┘   └──────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              내 게임 프로그램                             │   │
│  │  address: GameProg111...   executable=true               │   │
│  │  data: [BPF 바이트코드]                                  │   │
│  └───────────────────────────┬──────────────────────────────┘   │
│                              │ owns                              │
│                              ▼                                  │
│         ┌────────────────────────────────────┐                  │
│         │  Alice의 게임 데이터 계정           │                  │
│         │  owner: GameProg111...             │                  │
│         │  data: {score: 9500, level: 42}    │                  │
│         │  executable: false                 │                  │
│         └────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 코드 예제: JavaScript/TypeScript로 Account 데이터 읽기

```typescript
import {
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  AccountInfo,
} from '@solana/web3.js';

const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

async function readAccountInfo(address: string) {
  const publicKey = new PublicKey(address);

  // 기본 계정 정보 읽기
  const accountInfo: AccountInfo<Buffer> | null =
    await connection.getAccountInfo(publicKey);

  if (!accountInfo) {
    console.log('계정이 존재하지 않습니다.');
    return;
  }

  console.log('=== Account 정보 ===');
  console.log(`lamports: ${accountInfo.lamports} (${accountInfo.lamports / LAMPORTS_PER_SOL} SOL)`);
  console.log(`data 크기: ${accountInfo.data.length} bytes`);
  console.log(`owner: ${accountInfo.owner.toBase58()}`);
  console.log(`executable: ${accountInfo.executable}`);
  console.log(`rentEpoch: ${accountInfo.rentEpoch}`);
}

// SOL 잔액 조회
async function getBalance(address: string) {
  const publicKey = new PublicKey(address);
  const lamports = await connection.getBalance(publicKey);
  console.log(`잔액: ${lamports / LAMPORTS_PER_SOL} SOL`);
}

// 여러 계정 한 번에 조회 (배치 최적화)
async function getMultipleAccounts(addresses: string[]) {
  const publicKeys = addresses.map(addr => new PublicKey(addr));
  const accounts = await connection.getMultipleAccountsInfo(publicKeys);

  accounts.forEach((account, index) => {
    if (account) {
      console.log(`계정 ${index}: ${account.lamports} lamports, owner: ${account.owner.toBase58()}`);
    } else {
      console.log(`계정 ${index}: 존재하지 않음`);
    }
  });
}

// 렌트 면제 최소 잔액 계산
async function checkRentExempt(dataSize: number) {
  const minBalance = await connection.getMinimumBalanceForRentExemption(dataSize);
  console.log(`${dataSize}bytes 계정의 렌트 면제 최소: ${minBalance / LAMPORTS_PER_SOL} SOL`);
}

// 실행
(async () => {
  // System Program 주소 (잘 알려진 주소)
  const SYSTEM_PROGRAM = '11111111111111111111111111111111';
  const TOKEN_PROGRAM = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';

  await readAccountInfo(SYSTEM_PROGRAM);
  await readAccountInfo(TOKEN_PROGRAM);
  await checkRentExempt(165); // Token Account 크기
  await checkRentExempt(200); // 커스텀 데이터 계정
})();
```

### 실행 결과 예시:

```
=== Account 정보 (System Program) ===
lamports: 1000000000000 (1000 SOL)
data 크기: 14 bytes
owner: NativeLoader1111111111111111111111111111111
executable: true
rentEpoch: 0

=== Account 정보 (Token Program) ===
lamports: 1141440 (0.00114144 SOL)
data 크기: 36 bytes (업그레이드 가능한 프로그램 포인터)
owner: BPFLoaderUpgradeab1e11111111111111111111111
executable: true
rentEpoch: 0

165bytes 계정의 렌트 면제 최소: 0.00203928 SOL
200bytes 계정의 렌트 면제 최소: 0.00228288 SOL
```

---

## Rust에서 Account 데이터 역직렬화

```rust
use solana_program::{
    account_info::AccountInfo,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

// 커스텀 데이터 구조 정의
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PlayerData {
    pub score: u64,
    pub level: u32,
    pub player_name: String,
    pub is_initialized: bool,
}

// Account에서 데이터 읽기
pub fn read_player_data(account: &AccountInfo) -> Result<PlayerData, Box<dyn std::error::Error>> {
    // account.data는 RefCell<&mut [u8]>
    let data = account.data.borrow();

    // Borsh로 역직렬화
    let player_data = PlayerData::try_from_slice(&data)?;
    println!("플레이어: {}, 점수: {}, 레벨: {}",
        player_data.player_name,
        player_data.score,
        player_data.level
    );

    Ok(player_data)
}

// Account에 데이터 쓰기
pub fn write_player_data(
    account: &AccountInfo,
    data: &PlayerData,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut account_data = account.data.borrow_mut();
    data.serialize(&mut *account_data)?;
    Ok(())
}
```

---

## 핵심 정리

| 개념 | 이더리움 | Solana |
|------|---------|--------|
| 기본 단위 | Wei (10^-18 ETH) | Lamport (10^-9 SOL) |
| 계정 유형 | EOA, Contract | 모두 Account |
| 데이터 위치 | 컨트랙트 내부 storage | 별도 Account의 data 필드 |
| 프로그램 상태 | stateful | stateless |
| 계정 유지 비용 | 없음 | Rent (rent-exempt 가능) |
| 계정 생성 | 자동 (첫 트랜잭션) | 명시적 생성 필요 |

다음 장에서는 프로그램(스마트 컨트랙트)이 어떻게 작동하고, Instruction과 Transaction이 어떻게 구성되는지 살펴봅니다.
