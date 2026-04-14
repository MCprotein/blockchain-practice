# 미니프로젝트: Anchor로 포인트 시스템 구현

## 프로젝트 개요

지금까지 배운 Solana와 Anchor의 모든 개념을 종합하여 **포인트 시스템(Point System)**을 구현합니다. 실제 서비스에서 자주 쓰이는 충성도 포인트, 게임 점수, 리워드 토큰 등의 패턴을 Solana 위에서 구현합니다.

### 요구사항

```text
기능:
1. initialize    - 포인트 시스템 초기화 (관리자 설정)
2. register_user - 사용자 등록 (포인트 계정 생성)
3. mint_points   - 관리자가 사용자에게 포인트 발행
4. transfer_points - 사용자 간 포인트 전송
5. burn_points   - 포인트 소각

규칙:
- 관리자만 mint 가능
- 사용자는 자신의 포인트만 전송/소각 가능
- 전송 시 잔액 부족 에러 처리
- 각 작업마다 이벤트 발행

계정 구조:
- PointSystem: 전체 시스템 상태 (글로벌 PDA)
- UserPoint: 사용자별 포인트 잔액 (사용자별 PDA)
```

---

## 프로젝트 생성

```bash
anchor init sol-point-system
cd sol-point-system

# 빌드 의존성 확인
anchor --version   # 0.30.1
solana --version   # 1.18+
```

---

## 계정 구조 설계

```text
┌────────────────────────────────────────────────────────┐
│             포인트 시스템 계정 구조                     │
│                                                        │
│  PointSystem Account (PDA: ["point-system"])           │
│  ┌─────────────────────────────────────────────────┐   │
│  │ admin: Pubkey          ← 관리자 주소             │   │
│  │ total_supply: u64      ← 총 발행량               │   │
│  │ total_burned: u64      ← 총 소각량               │   │
│  │ user_count: u64        ← 등록 사용자 수          │   │
│  │ bump: u8               ← PDA bump               │   │
│  └─────────────────────────────────────────────────┘   │
│                                                        │
│  UserPoint Account (PDA: ["user-point", user_pubkey])  │
│  ┌─────────────────────────────────────────────────┐   │
│  │ owner: Pubkey          ← 사용자 주소             │   │
│  │ balance: u64           ← 포인트 잔액             │   │
│  │ total_earned: u64      ← 누적 획득량             │   │
│  │ total_spent: u64       ← 누적 사용량             │   │
│  │ bump: u8               ← PDA bump               │   │
│  └─────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────┘
```

---

## 프로그램 코드: programs/sol-point-system/src/lib.rs

```rust,ignore
use anchor_lang::prelude::*;

declare_id!("PoiNt1111111111111111111111111111111111111111");

// ============================================================
// 에러 코드
// ============================================================
#[error_code]
pub enum PointError {
    #[msg("관리자 권한이 필요합니다")]
    AdminRequired,

    #[msg("포인트 잔액이 부족합니다")]
    InsufficientBalance,

    #[msg("발행량은 0보다 커야 합니다")]
    ZeroAmount,

    #[msg("이미 등록된 사용자입니다")]
    AlreadyRegistered,

    #[msg("산술 오버플로가 발생했습니다")]
    ArithmeticOverflow,

    #[msg("자기 자신에게 전송할 수 없습니다")]
    SelfTransfer,
}

// ============================================================
// 이벤트
// ============================================================
#[event]
pub struct SystemInitialized {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct UserRegistered {
    pub user: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PointsMinted {
    pub recipient: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub total_supply: u64,
    pub timestamp: i64,
}

#[event]
pub struct PointsTransferred {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct PointsBurned {
    pub user: Pubkey,
    pub amount: u64,
    pub remaining: u64,
    pub timestamp: i64,
}

// ============================================================
// 계정 구조체
// ============================================================

/// 포인트 시스템 전역 상태
#[account]
pub struct PointSystem {
    pub admin: Pubkey,         // 32
    pub total_supply: u64,     // 8
    pub total_burned: u64,     // 8
    pub user_count: u64,       // 8
    pub bump: u8,              // 1
}

impl PointSystem {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 1;  // 57 bytes
}

/// 사용자별 포인트 계정
#[account]
pub struct UserPoint {
    pub owner: Pubkey,         // 32
    pub balance: u64,          // 8
    pub total_earned: u64,     // 8
    pub total_spent: u64,      // 8
    pub bump: u8,              // 1
}

impl UserPoint {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 1;  // 57 bytes
}

// ============================================================
// 프로그램 로직
// ============================================================
#[program]
pub mod sol_point_system {
    use super::*;

    // ----------------------------------------------------------
    // 1. initialize: 포인트 시스템 초기화
    // ----------------------------------------------------------
    /// 포인트 시스템을 배포 후 한 번만 실행
    /// admin 계정을 설정하고 글로벌 상태 계정 생성
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let system = &mut ctx.accounts.point_system;
        let clock = Clock::get()?;

        system.admin = ctx.accounts.admin.key();
        system.total_supply = 0;
        system.total_burned = 0;
        system.user_count = 0;
        system.bump = ctx.bumps.point_system;

        emit!(SystemInitialized {
            admin: system.admin,
            timestamp: clock.unix_timestamp,
        });

        msg!("포인트 시스템 초기화 완료. 관리자: {}", system.admin);
        Ok(())
    }

    // ----------------------------------------------------------
    // 2. register_user: 사용자 등록
    // ----------------------------------------------------------
    /// 새 사용자를 등록하고 UserPoint PDA 계정 생성
    /// 누구나 자신의 계정을 생성할 수 있음
    pub fn register_user(ctx: Context<RegisterUser>) -> Result<()> {
        let user_point = &mut ctx.accounts.user_point;
        let system = &mut ctx.accounts.point_system;
        let clock = Clock::get()?;

        user_point.owner = ctx.accounts.user.key();
        user_point.balance = 0;
        user_point.total_earned = 0;
        user_point.total_spent = 0;
        user_point.bump = ctx.bumps.user_point;

        // 사용자 수 증가
        system.user_count = system.user_count
            .checked_add(1)
            .ok_or(PointError::ArithmeticOverflow)?;

        emit!(UserRegistered {
            user: user_point.owner,
            timestamp: clock.unix_timestamp,
        });

        msg!("사용자 등록: {}", user_point.owner);
        Ok(())
    }

    // ----------------------------------------------------------
    // 3. mint_points: 포인트 발행 (관리자 전용)
    // ----------------------------------------------------------
    /// 관리자가 사용자에게 포인트를 발행
    /// has_one = admin 제약조건으로 관리자 검증
    pub fn mint_points(ctx: Context<MintPoints>, amount: u64) -> Result<()> {
        require!(amount > 0, PointError::ZeroAmount);

        let system = &mut ctx.accounts.point_system;
        let user_point = &mut ctx.accounts.user_point;
        let clock = Clock::get()?;

        // 잔액 증가 (오버플로 방지)
        user_point.balance = user_point.balance
            .checked_add(amount)
            .ok_or(PointError::ArithmeticOverflow)?;

        user_point.total_earned = user_point.total_earned
            .checked_add(amount)
            .ok_or(PointError::ArithmeticOverflow)?;

        // 총 공급량 증가
        system.total_supply = system.total_supply
            .checked_add(amount)
            .ok_or(PointError::ArithmeticOverflow)?;

        emit!(PointsMinted {
            recipient: user_point.owner,
            amount,
            new_balance: user_point.balance,
            total_supply: system.total_supply,
            timestamp: clock.unix_timestamp,
        });

        msg!(
            "포인트 발행: {} → {} (잔액: {})",
            amount,
            user_point.owner,
            user_point.balance
        );
        Ok(())
    }

    // ----------------------------------------------------------
    // 4. transfer_points: 포인트 전송
    // ----------------------------------------------------------
    /// 사용자가 다른 사용자에게 포인트 전송
    pub fn transfer_points(ctx: Context<TransferPoints>, amount: u64) -> Result<()> {
        require!(amount > 0, PointError::ZeroAmount);

        let from_point = &mut ctx.accounts.from_point;
        let to_point = &mut ctx.accounts.to_point;

        // 자기 자신에게 전송 방지
        require!(
            from_point.owner != to_point.owner,
            PointError::SelfTransfer
        );

        // 잔액 확인
        require!(
            from_point.balance >= amount,
            PointError::InsufficientBalance
        );

        let clock = Clock::get()?;

        // 송신자 잔액 감소
        from_point.balance = from_point.balance
            .checked_sub(amount)
            .ok_or(PointError::ArithmeticOverflow)?;
        from_point.total_spent = from_point.total_spent
            .checked_add(amount)
            .ok_or(PointError::ArithmeticOverflow)?;

        // 수신자 잔액 증가
        to_point.balance = to_point.balance
            .checked_add(amount)
            .ok_or(PointError::ArithmeticOverflow)?;
        to_point.total_earned = to_point.total_earned
            .checked_add(amount)
            .ok_or(PointError::ArithmeticOverflow)?;

        emit!(PointsTransferred {
            from: from_point.owner,
            to: to_point.owner,
            amount,
            timestamp: clock.unix_timestamp,
        });

        msg!(
            "포인트 전송: {} → {} ({}pt)",
            from_point.owner,
            to_point.owner,
            amount
        );
        Ok(())
    }

    // ----------------------------------------------------------
    // 5. burn_points: 포인트 소각
    // ----------------------------------------------------------
    /// 사용자가 자신의 포인트를 소각
    pub fn burn_points(ctx: Context<BurnPoints>, amount: u64) -> Result<()> {
        require!(amount > 0, PointError::ZeroAmount);

        let system = &mut ctx.accounts.point_system;
        let user_point = &mut ctx.accounts.user_point;
        let clock = Clock::get()?;

        require!(
            user_point.balance >= amount,
            PointError::InsufficientBalance
        );

        user_point.balance = user_point.balance
            .checked_sub(amount)
            .ok_or(PointError::ArithmeticOverflow)?;
        user_point.total_spent = user_point.total_spent
            .checked_add(amount)
            .ok_or(PointError::ArithmeticOverflow)?;

        system.total_burned = system.total_burned
            .checked_add(amount)
            .ok_or(PointError::ArithmeticOverflow)?;

        emit!(PointsBurned {
            user: user_point.owner,
            amount,
            remaining: user_point.balance,
            timestamp: clock.unix_timestamp,
        });

        msg!(
            "포인트 소각: {} ({}pt, 잔액: {})",
            user_point.owner,
            amount,
            user_point.balance
        );
        Ok(())
    }
}

// ============================================================
// 계정 검증 구조체
// ============================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + PointSystem::LEN,
        seeds = [b"point-system"],
        bump,
    )]
    pub point_system: Account<'info, PointSystem>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterUser<'info> {
    #[account(
        mut,
        seeds = [b"point-system"],
        bump = point_system.bump,
    )]
    pub point_system: Account<'info, PointSystem>,

    #[account(
        init,
        payer = user,
        space = 8 + UserPoint::LEN,
        seeds = [b"user-point", user.key().as_ref()],
        bump,
    )]
    pub user_point: Account<'info, UserPoint>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintPoints<'info> {
    #[account(
        mut,
        seeds = [b"point-system"],
        bump = point_system.bump,
        has_one = admin @ PointError::AdminRequired,
    )]
    pub point_system: Account<'info, PointSystem>,

    #[account(
        mut,
        seeds = [b"user-point", user_point.owner.as_ref()],
        bump = user_point.bump,
    )]
    pub user_point: Account<'info, UserPoint>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferPoints<'info> {
    #[account(
        mut,
        seeds = [b"user-point", sender.key().as_ref()],
        bump = from_point.bump,
        has_one = sender @ PointError::AdminRequired,
    )]
    pub from_point: Account<'info, UserPoint>,

    #[account(
        mut,
        seeds = [b"user-point", to_point.owner.as_ref()],
        bump = to_point.bump,
    )]
    pub to_point: Account<'info, UserPoint>,

    pub sender: Signer<'info>,
}

#[derive(Accounts)]
pub struct BurnPoints<'info> {
    #[account(
        mut,
        seeds = [b"point-system"],
        bump = point_system.bump,
    )]
    pub point_system: Account<'info, PointSystem>,

    #[account(
        mut,
        seeds = [b"user-point", user.key().as_ref()],
        bump = user_point.bump,
        has_one = user @ PointError::AdminRequired,
    )]
    pub user_point: Account<'info, UserPoint>,

    pub user: Signer<'info>,
}
```

---

## TypeScript 테스트 코드: tests/sol-point-system.ts

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program, BN, AnchorError, web3 } from "@coral-xyz/anchor";
import { SolPointSystem } from "../target/types/sol_point_system";
import { assert, expect } from "chai";

describe("sol-point-system", () => {
  // ============================================================
  // 설정
  // ============================================================
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SolPointSystem as Program<SolPointSystem>;

  // 역할별 키페어
  const admin = provider.wallet as anchor.Wallet;
  const alice = web3.Keypair.generate();
  const bob = web3.Keypair.generate();
  const attacker = web3.Keypair.generate();

  // PDA 주소들
  let systemPda: web3.PublicKey;
  let alicePointPda: web3.PublicKey;
  let bobPointPda: web3.PublicKey;

  // ============================================================
  // before: 테스트 전 환경 준비
  // ============================================================
  before(async () => {
    // PDA 계산
    [systemPda] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("point-system")],
      program.programId
    );

    [alicePointPda] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user-point"), alice.publicKey.toBuffer()],
      program.programId
    );

    [bobPointPda] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user-point"), bob.publicKey.toBuffer()],
      program.programId
    );

    // Alice, Bob, Attacker에게 SOL 에어드롭
    const airdropTargets = [alice.publicKey, bob.publicKey, attacker.publicKey];
    for (const target of airdropTargets) {
      const sig = await provider.connection.requestAirdrop(
        target,
        2 * web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);
    }

    console.log("Admin:", admin.publicKey.toBase58());
    console.log("Alice:", alice.publicKey.toBase58());
    console.log("Bob:", bob.publicKey.toBase58());
    console.log("System PDA:", systemPda.toBase58());
    console.log("Alice Point PDA:", alicePointPda.toBase58());
    console.log("Bob Point PDA:", bobPointPda.toBase58());
  });

  // ============================================================
  // 테스트 1: 시스템 초기화
  // ============================================================
  it("포인트 시스템을 초기화할 수 있다", async () => {
    const txSig = await program.methods
      .initialize()
      .accounts({
        pointSystem: systemPda,
        admin: admin.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log("초기화 tx:", txSig);

    const systemAccount = await program.account.pointSystem.fetch(systemPda);
    assert.isTrue(systemAccount.admin.equals(admin.publicKey));
    assert.equal(systemAccount.totalSupply.toNumber(), 0);
    assert.equal(systemAccount.totalBurned.toNumber(), 0);
    assert.equal(systemAccount.userCount.toNumber(), 0);
  });

  // ============================================================
  // 테스트 2: 사용자 등록
  // ============================================================
  it("Alice가 사용자 등록을 할 수 있다", async () => {
    await program.methods
      .registerUser()
      .accounts({
        pointSystem: systemPda,
        userPoint: alicePointPda,
        user: alice.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceAccount = await program.account.userPoint.fetch(alicePointPda);
    assert.isTrue(aliceAccount.owner.equals(alice.publicKey));
    assert.equal(aliceAccount.balance.toNumber(), 0);

    const systemAccount = await program.account.pointSystem.fetch(systemPda);
    assert.equal(systemAccount.userCount.toNumber(), 1);
  });

  it("Bob도 사용자 등록을 할 수 있다", async () => {
    await program.methods
      .registerUser()
      .accounts({
        pointSystem: systemPda,
        userPoint: bobPointPda,
        user: bob.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([bob])
      .rpc();

    const systemAccount = await program.account.pointSystem.fetch(systemPda);
    assert.equal(systemAccount.userCount.toNumber(), 2);
  });

  // ============================================================
  // 테스트 3: 포인트 발행 (mint)
  // ============================================================
  it("관리자가 Alice에게 포인트를 발행할 수 있다", async () => {
    const mintAmount = new BN(1000);

    await program.methods
      .mintPoints(mintAmount)
      .accounts({
        pointSystem: systemPda,
        userPoint: alicePointPda,
        admin: admin.publicKey,
      })
      .rpc();

    const aliceAccount = await program.account.userPoint.fetch(alicePointPda);
    assert.equal(aliceAccount.balance.toNumber(), 1000);
    assert.equal(aliceAccount.totalEarned.toNumber(), 1000);

    const systemAccount = await program.account.pointSystem.fetch(systemPda);
    assert.equal(systemAccount.totalSupply.toNumber(), 1000);
  });

  it("관리자가 Bob에게도 포인트를 발행할 수 있다", async () => {
    await program.methods
      .mintPoints(new BN(500))
      .accounts({
        pointSystem: systemPda,
        userPoint: bobPointPda,
        admin: admin.publicKey,
      })
      .rpc();

    const bobAccount = await program.account.userPoint.fetch(bobPointPda);
    assert.equal(bobAccount.balance.toNumber(), 500);

    const systemAccount = await program.account.pointSystem.fetch(systemPda);
    assert.equal(systemAccount.totalSupply.toNumber(), 1500);
  });

  it("관리자가 아닌 사용자는 포인트를 발행할 수 없다", async () => {
    try {
      await program.methods
        .mintPoints(new BN(9999))
        .accounts({
          pointSystem: systemPda,
          userPoint: alicePointPda,
          admin: attacker.publicKey,   // 공격자가 admin 사칭
        })
        .signers([attacker])
        .rpc();

      assert.fail("에러가 발생해야 합니다");
    } catch (err) {
      expect(err).to.be.instanceOf(AnchorError);
      const anchorErr = err as AnchorError;
      expect(anchorErr.error.errorCode.code).to.equal("AdminRequired");
      console.log("관리자 권한 검증 성공!");
    }
  });

  // ============================================================
  // 테스트 4: 포인트 전송
  // ============================================================
  it("Alice가 Bob에게 포인트를 전송할 수 있다", async () => {
    const transferAmount = new BN(300);

    await program.methods
      .transferPoints(transferAmount)
      .accounts({
        fromPoint: alicePointPda,
        toPoint: bobPointPda,
        sender: alice.publicKey,
      })
      .signers([alice])
      .rpc();

    const aliceAccount = await program.account.userPoint.fetch(alicePointPda);
    const bobAccount = await program.account.userPoint.fetch(bobPointPda);

    assert.equal(aliceAccount.balance.toNumber(), 700);   // 1000 - 300
    assert.equal(aliceAccount.totalSpent.toNumber(), 300);
    assert.equal(bobAccount.balance.toNumber(), 800);     // 500 + 300
    assert.equal(bobAccount.totalEarned.toNumber(), 800); // 500 + 300

    console.log("전송 후 Alice 잔액:", aliceAccount.balance.toNumber());
    console.log("전송 후 Bob 잔액:", bobAccount.balance.toNumber());
  });

  it("잔액 부족 시 전송이 실패한다", async () => {
    try {
      await program.methods
        .transferPoints(new BN(9999))   // Alice 잔액(700)보다 많음
        .accounts({
          fromPoint: alicePointPda,
          toPoint: bobPointPda,
          sender: alice.publicKey,
        })
        .signers([alice])
        .rpc();

      assert.fail("에러가 발생해야 합니다");
    } catch (err) {
      expect(err).to.be.instanceOf(AnchorError);
      const anchorErr = err as AnchorError;
      expect(anchorErr.error.errorCode.code).to.equal("InsufficientBalance");
      console.log("잔액 부족 검증 성공!");
    }
  });

  it("자기 자신에게 전송하면 에러가 발생한다", async () => {
    try {
      await program.methods
        .transferPoints(new BN(100))
        .accounts({
          fromPoint: alicePointPda,
          toPoint: alicePointPda,   // 자기 자신
          sender: alice.publicKey,
        })
        .signers([alice])
        .rpc();

      assert.fail("에러가 발생해야 합니다");
    } catch (err) {
      expect(err).to.be.instanceOf(AnchorError);
      const anchorErr = err as AnchorError;
      expect(anchorErr.error.errorCode.code).to.equal("SelfTransfer");
    }
  });

  // ============================================================
  // 테스트 5: 포인트 소각
  // ============================================================
  it("Bob이 자신의 포인트를 소각할 수 있다", async () => {
    const burnAmount = new BN(200);
    const bobBefore = await program.account.userPoint.fetch(bobPointPda);
    const systemBefore = await program.account.pointSystem.fetch(systemPda);

    await program.methods
      .burnPoints(burnAmount)
      .accounts({
        pointSystem: systemPda,
        userPoint: bobPointPda,
        user: bob.publicKey,
      })
      .signers([bob])
      .rpc();

    const bobAfter = await program.account.userPoint.fetch(bobPointPda);
    const systemAfter = await program.account.pointSystem.fetch(systemPda);

    assert.equal(
      bobAfter.balance.toNumber(),
      bobBefore.balance.toNumber() - 200
    );
    assert.equal(
      systemAfter.totalBurned.toNumber(),
      systemBefore.totalBurned.toNumber() + 200
    );

    console.log("소각 후 Bob 잔액:", bobAfter.balance.toNumber());
    console.log("총 소각량:", systemAfter.totalBurned.toNumber());
  });

  // ============================================================
  // 테스트 6: 전체 상태 확인
  // ============================================================
  it("최종 시스템 상태를 확인한다", async () => {
    const system = await program.account.pointSystem.fetch(systemPda);
    const alice = await program.account.userPoint.fetch(alicePointPda);
    const bob = await program.account.userPoint.fetch(bobPointPda);

    console.log("\n========== 최종 포인트 현황 ==========");
    console.log(`총 발행량:  ${system.totalSupply.toNumber()} pt`);
    console.log(`총 소각량:  ${system.totalBurned.toNumber()} pt`);
    console.log(`유통량:     ${system.totalSupply.sub(system.totalBurned).toNumber()} pt`);
    console.log(`사용자 수:  ${system.userCount.toNumber()} 명`);
    console.log(`Alice 잔액: ${alice.balance.toNumber()} pt (획득: ${alice.totalEarned}, 사용: ${alice.totalSpent})`);
    console.log(`Bob 잔액:   ${bob.balance.toNumber()} pt (획득: ${bob.totalEarned}, 사용: ${bob.totalSpent})`);
    console.log("======================================\n");

    // 불변식 검증: 유통량 = 모든 사용자 잔액 합계
    const totalBalance = alice.balance.add(bob.balance);
    const circulation = system.totalSupply.sub(system.totalBurned);
    assert.equal(
      totalBalance.toString(),
      circulation.toString(),
      "유통량이 잔액 합계와 일치해야 함"
    );
  });

  // ============================================================
  // 테스트 7: 이벤트 수신
  // ============================================================
  it("포인트 발행 시 이벤트가 발행된다", async () => {
    return new Promise<void>(async (resolve, reject) => {
      const listener = program.addEventListener(
        "PointsMinted",
        (event, slot) => {
          try {
            expect(event.amount.toNumber()).to.equal(50);
            console.log(
              `이벤트 수신: ${event.amount} pt → ${event.recipient.toBase58().slice(0, 8)}...`
            );
            program.removeEventListener(listener);
            resolve();
          } catch (e) {
            reject(e);
          }
        }
      );

      setTimeout(() => {
        program.removeEventListener(listener);
        reject(new Error("이벤트 타임아웃"));
      }, 10000);

      await program.methods
        .mintPoints(new BN(50))
        .accounts({
          pointSystem: systemPda,
          userPoint: alicePointPda,
          admin: admin.publicKey,
        })
        .rpc();
    });
  });
});
```

---

## 단계별 실행 가이드

### Step 1: 프로젝트 설정

```bash
# 프로젝트 생성 및 이동
anchor init sol-point-system
cd sol-point-system

# lib.rs에 위의 프로그램 코드 붙여넣기
# programs/sol-point-system/src/lib.rs

# 테스트 파일 작성
# tests/sol-point-system.ts
```

### Step 2: 빌드

```bash
anchor build

# 빌드 성공 확인:
# target/deploy/sol_point_system.so
# target/idl/sol_point_system.json
# target/types/sol_point_system.ts

# 프로그램 ID 확인
anchor keys list
# sol_point_system: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# lib.rs의 declare_id!와 Anchor.toml의 ID를 빌드된 ID로 업데이트
anchor keys sync
```

### Step 3: 로컬 테스트 실행

```bash
# 전체 테스트 실행 (검증자 자동 시작/종료)
anchor test

# 개발 중 빠른 반복:
# 터미널 1: 검증자 유지
solana-test-validator --reset

# 터미널 2: 배포 후 테스트
anchor build && anchor deploy && anchor test --skip-local-validator --skip-build
```

### Step 4: Devnet 배포 및 테스트

```bash
# Devnet으로 전환
solana config set --url devnet

# SOL 에어드롭 (Devnet 무료)
solana airdrop 2

# Anchor.toml 수정
# [provider]
# cluster = "Devnet"

# Devnet에 배포
anchor deploy --provider.cluster devnet

# Devnet에서 테스트
anchor test --provider.cluster devnet
```

### Step 5: 배포된 프로그램 탐색

```bash
# Solana Explorer에서 확인
# https://explorer.solana.com/address/<PROGRAM_ID>?cluster=devnet

# CLI로 프로그램 계정 확인
solana program show <PROGRAM_ID>

# 생성된 계정 확인 (Python 코드 예시)
# 또는 TypeScript로:
npx ts-node -e "
const anchor = require('@coral-xyz/anchor');
const { PublicKey } = require('@solana/web3.js');
const connection = new anchor.web3.Connection('https://api.devnet.solana.com');
const programId = new PublicKey(process.env.PROGRAM_ID);
connection.getAccountInfo(programId).then((account) => {
  console.log(account ? '프로그램 계정 존재' : '프로그램 계정 없음');
});
"
```

---

## 확장 아이디어

이 기본 포인트 시스템에 추가할 수 있는 기능들:

```rust,ignore
// 1. 만료 기능: expires_at: i64 필드 추가
pub fn is_expired(&self, current_time: i64) -> bool {
    self.expires_at > 0 && current_time > self.expires_at
}

// 2. 등급 시스템
pub fn get_tier(&self) -> &str {
    match self.total_earned.try_into().unwrap_or(0u64) {
        0..=999 => "Bronze",
        1000..=4999 => "Silver",
        5000..=9999 => "Gold",
        _ => "Diamond",
    }
}

// 3. 전송 수수료 (일부를 treasury로)
// 4. 일일 발행 한도
// 5. 화이트리스트 발행자 (admin 외 추가 발행자)
// 6. 잠금(lock) 기능 - 일정 기간 전송 불가
```

---

## 배운 내용 정리

이 미니프로젝트를 통해 다음을 실습했습니다:

```text
Solana/Anchor 핵심 개념:
✓ declare_id! - 프로그램 ID 관리
✓ #[program] - Instruction 라우팅
✓ #[account] - 계정 데이터 구조
✓ #[derive(Accounts)] - 계정 검증
✓ PDA (Program Derived Address) - 결정론적 계정 주소
✓ seeds + bump - PDA 생성 및 검증
✓ has_one - 계정 관계 검증
✓ #[error_code] - 커스텀 에러
✓ #[event] - 이벤트 발행
✓ checked_add/sub - 안전한 산술 연산

TypeScript 테스트:
✓ AnchorProvider 설정
✓ program.methods 체인
✓ PDA 주소 계산
✓ 계정 데이터 조회
✓ AnchorError 처리
✓ 이벤트 리스닝
✓ 잔액 검증
```

이제 여러분은 Solana 위에서 실제 동작하는 dApp 백엔드를 구축할 수 있습니다. 다음 단계로는 SPL Token 통합, Metaplex를 이용한 NFT, 또는 실제 DEX 프로토콜 구현에 도전해보세요.
