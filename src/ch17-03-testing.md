# Anchor 테스트: TypeScript로 Solana 프로그램 테스트하기

## Solana 테스트는 TypeScript로 작성한다

Anchor 프로그램의 테스트는 **TypeScript**로 작성합니다. Node.js 백엔드 개발자에게 매우 친숙한 환경입니다. 테스트 프레임워크는 Mocha + Chai를 사용하며, Jest와 유사한 패턴으로 작성합니다.

```
이더리움 테스트 스택:          Solana(Anchor) 테스트 스택:
Solidity (프로그램)            Rust + Anchor (프로그램)
Foundry (Forge) 또는           @coral-xyz/anchor (클라이언트)
Hardhat (테스트)               Mocha + Chai (테스트 러너)
JavaScript/TypeScript          TypeScript
```

---

## 패키지 설정

```json
// package.json (anchor init이 자동 생성)
{
  "scripts": {
    "lint:fix": "prettier */*.js \"*/**/*{.js,.ts}\" -w",
    "lint": "prettier */*.js \"*/**/*{.js,.ts}\" --check"
  },
  "dependencies": {
    "@coral-xyz/anchor": "^0.30.1"
  },
  "devDependencies": {
    "chai": "^4.3.4",
    "mocha": "^9.0.3",
    "ts-mocha": "^10.0.0",
    "@types/bn.js": "^5.1.0",
    "@types/chai": "^4.3.0",
    "@types/mocha": "^9.0.0",
    "typescript": "^4.3.5"
  }
}
```

```json
// tsconfig.json
{
  "compilerOptions": {
    "types": ["mocha", "chai"],
    "typeRoots": ["./node_modules/@types"],
    "lib": ["es2015"],
    "module": "commonjs",
    "target": "es6",
    "esModuleInterop": true
  }
}
```

---

## AnchorProvider 설정

`AnchorProvider`는 Solana RPC 연결과 지갑(서명자)을 묶어주는 객체입니다. NestJS의 `ConfigService`나 `DataSource` 초기화와 유사한 역할입니다.

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, web3, BN } from "@coral-xyz/anchor";
import { MyProject } from "../target/types/my_project";  // 자동 생성 타입
import { assert, expect } from "chai";

describe("my-project", () => {
  // ============================================================
  // 1. Provider 설정
  // ============================================================

  // anchor test 실행 시 환경변수에서 자동 설정:
  // - ANCHOR_PROVIDER_URL: 클러스터 URL
  // - ANCHOR_WALLET: 키페어 파일 경로
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // 접근 가능한 객체들:
  const connection = provider.connection;     // RPC 연결
  const wallet = provider.wallet;             // 기본 지갑 (payer)
  const payer = provider.wallet as anchor.Wallet;

  // ============================================================
  // 2. Program 인스턴스 생성
  // ============================================================

  // workspace: Anchor.toml에서 읽어온 프로그램 ID + IDL을 자동 로드
  const program = anchor.workspace.MyProject as Program<MyProject>;

  // 또는 수동으로:
  // const program = new Program<MyProject>(
  //   idl as MyProject,
  //   PROGRAM_ID,
  //   provider,
  // );

  console.log("프로그램 ID:", program.programId.toBase58());
  console.log("지갑 주소:", wallet.publicKey.toBase58());
});
```

---

## 핵심 패턴: program.methods 체인

Anchor TypeScript 클라이언트의 핵심 API입니다:

```typescript
// 기본 패턴:
const txSignature = await program.methods
  .functionName(arg1, arg2)    // Instruction 이름 + 인자
  .accounts({                   // 계정 목록
    accountName: publicKey,
  })
  .signers([keypair])           // 추가 서명자 (payer 외)
  .rpc();                       // 트랜잭션 전송 + 확정 대기

// 변형 패턴들:
// .rpc()                     → 전송 후 서명 반환
// .transaction()             → Transaction 객체 반환 (나중에 전송)
// .instruction()             → Instruction 객체 반환 (직접 조합)
// .simulate()                → 시뮬레이션만 (전송 안 함)
// .prepare()                 → 서명 전 Transaction 준비
```

### BN (BigNumber) 사용

Solana의 `u64`는 JavaScript의 `number` 범위를 초과하므로 `BN`을 사용합니다:

```typescript
import { BN } from "@coral-xyz/anchor";

// u64 타입 인자
await program.methods
  .mintPoints(new BN(1000))    // u64 → BN
  .accounts({...})
  .rpc();

// BN 연산:
const a = new BN(100);
const b = new BN(50);
console.log(a.add(b).toString());   // "150"
console.log(a.sub(b).toString());   // "50"
console.log(a.mul(b).toString());   // "5000"
console.log(a.gt(b));               // true
console.log(a.toNumber());          // 100 (안전한 범위일 때만)
```

---

## 계정 데이터 읽기: program.account

```typescript
// 단일 계정 읽기
const accountData = await program.account.userProfile.fetch(accountPubkey);
console.log("score:", accountData.score.toString());
console.log("owner:", accountData.owner.toBase58());

// 여러 계정 읽기 (배치)
const accounts = await program.account.userProfile.fetchMultiple([
  pubkey1,
  pubkey2,
  pubkey3,
]);

// 조건으로 계정 검색 (주의: 전체 계정 스캔, 비쌈)
const allProfiles = await program.account.userProfile.all();
console.log("전체 프로필 수:", allProfiles.length);

// 필터로 검색 (memcmp: 메모리 비교)
const authorProfiles = await program.account.userProfile.all([
  {
    memcmp: {
      offset: 8,                              // discriminator(8) 건너뜀
      bytes: wallet.publicKey.toBase58(),     // owner 필드 값
    },
  },
]);
```

---

## PDA 주소 계산

```typescript
import { PublicKey } from "@solana/web3.js";

// Rust의 find_program_address와 동일한 결과
const [profilePda, bump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("profile"),
    wallet.publicKey.toBuffer(),
  ],
  program.programId
);

console.log("PDA:", profilePda.toBase58());
console.log("Bump:", bump);

// 여러 seeds 조합:
const [escrowPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("escrow"),
    Buffer.from(tradeId.toString()),
    buyer.toBuffer(),
    seller.toBuffer(),
  ],
  program.programId
);
```

---

## 에러 테스트: AnchorError

```typescript
import { AnchorError } from "@coral-xyz/anchor";

it("권한 없는 사용자는 메모를 수정할 수 없어야 함", async () => {
  const attacker = web3.Keypair.generate();

  try {
    await program.methods
      .updateMemo("해킹 시도")
      .accounts({
        memo: memoPda,
        author: attacker.publicKey,   // 잘못된 author
      })
      .signers([attacker])
      .rpc();

    // 여기까지 오면 테스트 실패
    assert.fail("에러가 발생해야 합니다");

  } catch (err) {
    // AnchorError 타입 검사
    expect(err).to.be.instanceOf(AnchorError);

    const anchorErr = err as AnchorError;

    // 에러 코드 확인 (Rust의 #[error_code] enum)
    expect(anchorErr.error.errorCode.code).to.equal("Unauthorized");
    expect(anchorErr.error.errorCode.number).to.equal(6002);

    // 에러 메시지 확인
    expect(anchorErr.error.errorMessage).to.include("권한이 없습니다");

    // 프로그램 확인
    expect(anchorErr.program.equals(program.programId)).to.be.true;
  }
});

// 또는 chai의 rejectedWith 패턴:
it("빈 메모는 생성 불가", async () => {
  await expect(
    program.methods
      .createMemo("")
      .accounts({ memo: memoPda, author: wallet.publicKey, systemProgram: web3.SystemProgram.programId })
      .rpc()
  ).to.be.rejectedWith(AnchorError, "EmptyContent");
});
```

---

## anchor test 명령어와 동작 방식

```bash
# 전체 테스트 실행 (가장 많이 사용)
anchor test

# 내부 동작:
# 1. anchor build         → 프로그램 컴파일
# 2. solana-test-validator 시작 (백그라운드)
# 3. anchor deploy        → 로컬 검증자에 배포
# 4. yarn run ts-mocha ... → 테스트 실행
# 5. 검증자 종료

# 이미 실행 중인 검증자 사용 (빌드 캐시 활용)
anchor test --skip-local-validator

# 빌드 건너뛰기 (코드 변경 없을 때)
anchor test --skip-build

# 특정 테스트 파일만
anchor test tests/memo.ts

# 로컬 검증자 직접 실행 (개발 중 유지)
solana-test-validator --reset   # 초기화 후 시작

# 별도 터미널에서:
anchor deploy
anchor test --skip-local-validator --skip-build
```

---

## 전체 테스트 예제: 메모 프로그램

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorError, BN, web3 } from "@coral-xyz/anchor";
import { MemoProgram } from "../target/types/memo_program";
import { assert, expect } from "chai";

describe("memo-program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MemoProgram as Program<MemoProgram>;
  const author = provider.wallet as anchor.Wallet;

  // 테스트 전체에서 공유할 PDA
  let memoPda: anchor.web3.PublicKey;
  let memoBump: number;

  // ============================================================
  // before: 각 테스트 전 공통 설정
  // ============================================================
  before(async () => {
    // PDA 주소 계산
    [memoPda, memoBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("memo"), author.publicKey.toBuffer()],
      program.programId
    );
    console.log("메모 PDA:", memoPda.toBase58());
    console.log("작성자:", author.publicKey.toBase58());
  });

  // ============================================================
  // 테스트 1: 메모 생성
  // ============================================================
  it("새 메모를 생성할 수 있다", async () => {
    const content = "안녕하세요, Solana!";

    const txSig = await program.methods
      .createMemo(content)
      .accounts({
        memo: memoPda,
        author: author.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log("트랜잭션:", txSig);

    // 계정 데이터 확인
    const memoAccount = await program.account.memo.fetch(memoPda);

    assert.equal(memoAccount.content, content);
    assert.isTrue(memoAccount.author.equals(author.publicKey));
    assert.isAbove(memoAccount.createdAt.toNumber(), 0);
    assert.equal(memoAccount.bump, memoBump);
  });

  // ============================================================
  // 테스트 2: 메모 조회
  // ============================================================
  it("메모를 조회할 수 있다", async () => {
    const memoAccount = await program.account.memo.fetch(memoPda);

    expect(memoAccount.content).to.equal("안녕하세요, Solana!");
    expect(memoAccount.author.toBase58()).to.equal(author.publicKey.toBase58());
  });

  // ============================================================
  // 테스트 3: 메모 업데이트
  // ============================================================
  it("작성자가 메모를 수정할 수 있다", async () => {
    const newContent = "수정된 메모 내용";

    await program.methods
      .updateMemo(newContent)
      .accounts({
        memo: memoPda,
        author: author.publicKey,
      })
      .rpc();

    const memoAccount = await program.account.memo.fetch(memoPda);
    expect(memoAccount.content).to.equal(newContent);
    expect(memoAccount.updatedAt.toNumber()).to.be.gte(
      memoAccount.createdAt.toNumber()
    );
  });

  // ============================================================
  // 테스트 4: 에러 케이스 - 빈 내용
  // ============================================================
  it("빈 내용으로 메모를 생성하면 에러가 발생한다", async () => {
    // 다른 사용자로 새 PDA 계산
    const newAuthor = web3.Keypair.generate();

    // Devnet에서 SOL 에어드롭 (로컬에서는 자동)
    const airdropSig = await provider.connection.requestAirdrop(
      newAuthor.publicKey,
      2 * web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    const [newMemoPda] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("memo"), newAuthor.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .createMemo("")
        .accounts({
          memo: newMemoPda,
          author: newAuthor.publicKey,
          systemProgram: web3.SystemProgram.programId,
        })
        .signers([newAuthor])
        .rpc();

      assert.fail("에러가 발생해야 합니다");
    } catch (err) {
      expect(err).to.be.instanceOf(AnchorError);
      const anchorErr = err as AnchorError;
      expect(anchorErr.error.errorCode.code).to.equal("EmptyContent");
    }
  });

  // ============================================================
  // 테스트 5: 에러 케이스 - 권한 없는 수정
  // ============================================================
  it("다른 사용자는 메모를 수정할 수 없다", async () => {
    const attacker = web3.Keypair.generate();

    // 공격자에게 SOL 지급
    const sig = await provider.connection.requestAirdrop(
      attacker.publicKey,
      web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    try {
      await program.methods
        .updateMemo("해킹!")
        .accounts({
          memo: memoPda,
          author: attacker.publicKey,   // 잘못된 author
        })
        .signers([attacker])
        .rpc();

      assert.fail("에러가 발생해야 합니다");
    } catch (err) {
      // has_one = author 제약조건 위반
      // → ConstraintHasOne 에러 또는 커스텀 Unauthorized 에러
      expect(err).to.be.instanceOf(AnchorError);
    }
  });

  // ============================================================
  // 테스트 6: 이벤트 리스닝
  // ============================================================
  it("메모 생성 시 이벤트가 발행된다", async () => {
    return new Promise<void>(async (resolve, reject) => {
      // 이벤트 리스너 등록
      const listener = program.addEventListener(
        "MemoCreated",
        (event, slot) => {
          try {
            expect(event.author.toBase58()).to.equal(
              author.publicKey.toBase58()
            );
            expect(event.timestamp.toNumber()).to.be.above(0);
            console.log("이벤트 수신! slot:", slot);
            program.removeEventListener(listener);
            resolve();
          } catch (e) {
            reject(e);
          }
        }
      );

      // 트랜잭션 전송 (이벤트 트리거)
      // 새 author로 테스트 (기존 PDA는 이미 초기화됨)
      const newAuthor = web3.Keypair.generate();
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          newAuthor.publicKey,
          web3.LAMPORTS_PER_SOL
        )
      );
      const [newPda] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("memo"), newAuthor.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .createMemo("이벤트 테스트")
        .accounts({
          memo: newPda,
          author: newAuthor.publicKey,
          systemProgram: web3.SystemProgram.programId,
        })
        .signers([newAuthor])
        .rpc();
    });
  });

  // ============================================================
  // 테스트 7: 계정 삭제 및 렌트 반환
  // ============================================================
  it("메모를 삭제하면 렌트가 반환된다", async () => {
    // 삭제 전 잔액
    const balanceBefore = await provider.connection.getBalance(
      author.publicKey
    );

    await program.methods
      .deleteMemo()
      .accounts({
        memo: memoPda,
        author: author.publicKey,
      })
      .rpc();

    // 삭제 후 잔액 (렌트 반환으로 증가)
    const balanceAfter = await provider.connection.getBalance(author.publicKey);
    expect(balanceAfter).to.be.greaterThan(balanceBefore);

    // 계정이 삭제되었는지 확인
    const deletedAccount = await provider.connection.getAccountInfo(memoPda);
    expect(deletedAccount).to.be.null;
  });

  // ============================================================
  // 테스트 8: 전체 계정 목록 조회
  // ============================================================
  it("모든 메모 계정을 조회할 수 있다", async () => {
    const allMemos = await program.account.memo.all();
    console.log(`현재 메모 수: ${allMemos.length}`);

    // 특정 작성자의 메모만 필터링
    const myMemos = await program.account.memo.all([
      {
        memcmp: {
          offset: 8,  // discriminator 건너뜀
          bytes: author.publicKey.toBase58(),
        },
      },
    ]);
    console.log(`내 메모 수: ${myMemos.length}`);
  });
});
```

---

## 유용한 테스트 유틸리티

```typescript
// 트랜잭션 상세 정보 확인
async function getTransactionDetails(sig: string) {
  const tx = await provider.connection.getTransaction(sig, {
    commitment: "confirmed",
    maxSupportedTransactionVersion: 0,
  });

  if (tx?.meta?.logMessages) {
    console.log("프로그램 로그:");
    tx.meta.logMessages.forEach(log => console.log(" ", log));
  }

  if (tx?.meta?.fee) {
    console.log(`수수료: ${tx.meta.fee} lamports`);
  }
}

// 계정 존재 여부 확인
async function accountExists(pubkey: web3.PublicKey): Promise<boolean> {
  const info = await provider.connection.getAccountInfo(pubkey);
  return info !== null;
}

// SOL 잔액 확인 (SOL 단위)
async function getBalanceSOL(pubkey: web3.PublicKey): Promise<number> {
  const lamports = await provider.connection.getBalance(pubkey);
  return lamports / web3.LAMPORTS_PER_SOL;
}

// 여러 트랜잭션 동시 전송 (성능 테스트용)
async function sendParallelTransactions(count: number) {
  const promises = Array.from({ length: count }, (_, i) =>
    program.methods
      .someInstruction(i)
      .accounts({...})
      .rpc()
  );

  const results = await Promise.allSettled(promises);
  const succeeded = results.filter(r => r.status === "fulfilled").length;
  const failed = results.filter(r => r.status === "rejected").length;
  console.log(`성공: ${succeeded}, 실패: ${failed}`);
}
```

---

## 테스트 실행 및 결과 해석

```bash
$ anchor test

# 출력 예시:
  memo-program
    새 메모를 생성할 수 있다
      트랜잭션: 5KBNvBW2w...
      ✓ 새 메모를 생성할 수 있다 (1234ms)
    메모를 조회할 수 있다
      ✓ 메모를 조회할 수 있다 (456ms)
    작성자가 메모를 수정할 수 있다
      ✓ 작성자가 메모를 수정할 수 있다 (789ms)
    빈 내용으로 메모를 생성하면 에러가 발생한다
      ✓ 빈 내용으로 메모를 생성하면 에러가 발생한다 (321ms)
    다른 사용자는 메모를 수정할 수 없다
      ✓ 다른 사용자는 메모를 수정할 수 없다 (654ms)
    메모 생성 시 이벤트가 발행된다
      이벤트 수신! slot: 42
      ✓ 메모 생성 시 이벤트가 발행된다 (987ms)
    메모를 삭제하면 렌트가 반환된다
      ✓ 메모를 삭제하면 렌트가 반환된다 (543ms)
    모든 메모 계정을 조회할 수 있다
      현재 메모 수: 3
      내 메모 수: 1
      ✓ 모든 메모 계정을 조회할 수 있다 (234ms)

  8 passing (5s)
```

다음 장에서는 배운 모든 것을 종합하여 포인트 시스템 미니 프로젝트를 구현합니다.
