# Anchor 프로젝트 구조

## anchor init으로 프로젝트 생성

```bash
# 새 Anchor 프로젝트 생성
anchor init my-project
cd my-project

# 또는 JavaScript 테스트 대신 TypeScript로 (기본값)
anchor init my-project --javascript  # JS 사용 시 (비권장)
```

생성 직후 출력:
```text
yarn install v1.22.19
[1/4] Resolving packages...
[2/4] Fetching packages...
[3/4] Linking dependencies...
[4/4] Building fresh packages...
Done in 12.34s.
my-project initialized
```

---

## 디렉토리 구조 전체 해설

```text
my-project/
├── Anchor.toml              ← 프로젝트 설정 파일 (NestJS의 nest-cli.json)
├── Cargo.toml               ← Rust 워크스페이스 설정
├── package.json             ← Node.js 의존성 (테스트용)
├── tsconfig.json            ← TypeScript 설정
├── .gitignore
│
├── programs/                ← Solana 프로그램 (스마트 컨트랙트) 디렉토리
│   └── my-project/
│       ├── Cargo.toml       ← 프로그램별 Rust 의존성
│       └── src/
│           └── lib.rs       ← 프로그램 소스코드 (핵심!)
│
├── tests/                   ← TypeScript 테스트 (Mocha 기반)
│   └── my-project.ts
│
├── migrations/              ← 배포 스크립트
│   └── deploy.ts
│
├── app/                     ← 프론트엔드 앱 (선택사항, 기본 비어있음)
│
└── target/                  ← 빌드 결과물 (git ignore)
    ├── deploy/
    │   └── my_project.so    ← 컴파일된 프로그램 (BPF 바이트코드)
    ├── idl/
    │   └── my_project.json  ← 자동 생성 IDL
    └── types/
        └── my_project.ts    ← 자동 생성 TypeScript 타입
```

---

## Anchor.toml 상세 설명

```toml
# Anchor.toml - NestJS의 nest-cli.json + .env 역할

[features]
resolution = true     # IDL 계정 해석 활성화
skip-lint = false     # Anchor 린트 규칙 적용

[programs.localnet]
# 프로그램 이름 = 프로그램 ID (배포 주소)
my_project = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

[programs.devnet]
my_project = "실제_배포된_프로그램_ID"

[registry]
url = "https://api.apr.dev"

[provider]
# 사용할 클러스터
cluster = "Localnet"   # Localnet | Devnet | Mainnet

# 서명에 사용할 키페어 파일 경로
wallet = "~/.config/solana/id.json"

[scripts]
# anchor test 실행 시 호출되는 스크립트
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```

### 클러스터별 설정 전환

```bash
# 현재 Solana CLI 클러스터 확인
solana config get

# Localnet으로 전환 (개발 중)
solana config set --url localhost
# Anchor.toml의 cluster = "Localnet"과 일치해야 함

# Devnet으로 전환 (테스트 배포)
solana config set --url devnet
# Anchor.toml의 cluster = "Devnet"으로 변경
```

---

## NestJS vs Anchor 구조 상세 비교

Node.js 백엔드 개발자에게 친숙한 NestJS 패턴과 Anchor를 1:1 대응으로 설명합니다.

### 1. 모듈 선언: @Module vs declare_id!

```typescript
// NestJS: 앱 모듈 선언
@Module({
  imports: [UsersModule, AuthModule],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
```

```rust,ignore
// Anchor: 프로그램 ID 선언 (프로그램의 "신원증명")
use anchor_lang::prelude::*;

// 이 프로그램이 배포된 주소 (네트워크의 고유 식별자)
// anchor build 후 target/deploy/my_project-keypair.json에서 자동 생성
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
```

`declare_id!`는 프로그램이 자신의 주소를 알고 있게 하여, 내부에서 `crate::ID`로 참조할 수 있게 합니다. PDA 생성 시 필수입니다.

### 2. 컨트롤러: @Controller vs #[program]

```typescript
// NestJS: HTTP 요청을 받는 컨트롤러
@Controller('users')
export class UserController {
  @Post()
  create(@Body() dto: CreateUserDto) {
    return this.userService.create(dto);
  }

  @Put(':id')
  update(@Param('id') id: string, @Body() dto: UpdateUserDto) {
    return this.userService.update(id, dto);
  }

  @Delete(':id')
  remove(@Param('id') id: string) {
    return this.userService.remove(id);
  }
}
```

```rust,ignore
// Anchor: Solana Instruction을 받는 프로그램 모듈
#[program]
pub mod my_project {
    use super::*;

    // @Post() create() 에 해당
    pub fn create_user(ctx: Context<CreateUser>, name: String) -> Result<()> {
        // ctx.accounts.* 로 계정 접근
        let user = &mut ctx.accounts.user;
        user.name = name;
        user.owner = ctx.accounts.signer.key();
        Ok(())
    }

    // @Put(':id') update() 에 해당
    pub fn update_user(ctx: Context<UpdateUser>, new_name: String) -> Result<()> {
        ctx.accounts.user.name = new_name;
        Ok(())
    }

    // @Delete(':id') remove() 에 해당 (계정 닫기)
    pub fn delete_user(ctx: Context<DeleteUser>) -> Result<()> {
        // #[account(close = signer)] 제약조건이 자동으로 처리
        Ok(())
    }
}
```

### 3. DTO 검증: class-validator vs #[derive(Accounts)]

```typescript
// NestJS: 요청 데이터 검증
import { IsString, IsNotEmpty, MaxLength } from 'class-validator';

export class CreateUserDto {
  @IsString()
  @IsNotEmpty()
  @MaxLength(32)
  name: string;
}

// 파이프라인이 자동으로 검증 실행
// ValidationPipe가 요청 전에 DTO 검증
```

```rust,ignore
// Anchor: 트랜잭션 계정 검증
#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(
        init,                    // 새 계정 생성
        payer = signer,          // 비용 지불자
        space = 8 + User::LEN,  // 할당 공간
    )]
    pub user: Account<'info, User>,  // 역직렬화 + owner 검증 자동

    #[account(mut)]              // 잔액 변경 허용
    pub signer: Signer<'info>,   // 서명자 검증 자동

    pub system_program: Program<'info, System>,  // System Program 검증 자동
}
// Anchor가 트랜잭션 실행 전에 모든 검증 자동 실행
```

### 4. 엔티티: TypeORM Entity vs #[account]

```typescript
// NestJS + TypeORM: 데이터베이스 엔티티
@Entity('users')
export class User {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ length: 32 })
  name: string;

  @Column({ type: 'bigint' })
  score: number;

  @ManyToOne(() => Organization)
  org: Organization;
}
```

```rust,ignore
// Anchor: Solana Account에 저장될 데이터 구조
#[account]  // ← @Entity()에 해당
pub struct User {
    pub owner: Pubkey,   // 32 bytes - 소유자 주소 (FK처럼 참조)
    pub name: String,    // 4 + len bytes
    pub score: u64,      // 8 bytes
    pub bump: u8,        // 1 byte - PDA bump
}

impl User {
    pub const LEN: usize =
        32 +    // owner: Pubkey
        4 + 32 + // name: String (4 = length prefix, 32 = max chars)
        8 +     // score: u64
        1;      // bump: u8
    // 총 77 bytes
    // + 8 bytes (Anchor discriminator) = 85 bytes 실제 할당
}
```

### 5. 서비스/비즈니스 로직: @Injectable Service vs 프로그램 함수

```typescript
// NestJS: 비즈니스 로직을 담당하는 서비스
@Injectable()
export class UserService {
  async addScore(userId: string, points: number): Promise<void> {
    const user = await this.userRepo.findOne(userId);
    if (!user) throw new NotFoundException();
    if (user.score + points > MAX_SCORE) throw new BadRequestException();
    user.score += points;
    await this.userRepo.save(user);
  }
}
```

```rust,ignore
// Anchor: 같은 역할의 프로그램 함수
pub fn add_score(ctx: Context<AddScore>, points: u64) -> Result<()> {
    let user = &mut ctx.accounts.user;

    // 검증 로직
    require!(
        user.score.checked_add(points).is_some(),
        MyError::ScoreOverflow
    );

    // 비즈니스 로직
    user.score = user.score.checked_add(points).unwrap();

    // 이벤트 발행 (이더리움의 emit과 동일)
    emit!(ScoreAdded {
        user: user.owner,
        points,
        new_score: user.score,
    });

    Ok(())
}
```

---

## 핵심 매크로 상세 설명

### declare_id!

```rust,ignore
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// 이 매크로가 하는 일:
// 1. ID 상수 생성: pub const ID: Pubkey = Pubkey::new_from_array([...])
// 2. check_id() 함수 생성: 프로그램 ID 검증용
// 3. 런타임이 올바른 프로그램을 호출했는지 검증

// PDA 생성 시 활용:
let (pda, bump) = Pubkey::find_program_address(
    &[b"seed"],
    &crate::ID,  // declare_id!로 선언된 프로그램 ID 참조
);
```

### #[program]

```rust,ignore
#[program]
pub mod my_program {
    use super::*;

    // 이 매크로가 하는 일:
    // 1. entrypoint 자동 등록
    // 2. 함수 이름 기반 8바이트 discriminator 생성
    //    sha256("global:function_name")[..8]
    // 3. Instruction 디스패치 자동화
    // 4. Context<T> 자동 주입

    pub fn my_instruction(ctx: Context<MyAccounts>, arg: u64) -> Result<()> {
        // Result<()>는 Anchor의 에러 타입
        // Ok(()) 반환 시 성공, Err(e) 반환 시 트랜잭션 롤백
        Ok(())
    }
}

// 생성되는 discriminator 예시:
// "global:initialize" → sha256 → 앞 8바이트
// [175, 175, 109, 31, 13, 152, 155, 237]
```

### #[derive(Accounts)]

```rust,ignore
#[derive(Accounts)]
#[instruction(amount: u64)]  // Instruction 인자에 접근 필요할 때
pub struct Transfer<'info> {
    // 이 매크로가 하는 일:
    // 1. 각 필드를 순서대로 AccountInfo에서 파싱
    // 2. 타입에 맞는 검증 실행 (Account<T>, Signer, Program 등)
    // 3. 제약조건(constraint) 검증

    #[account(
        mut,
        has_one = owner,        // from.owner == owner.key()
        constraint = from.amount >= amount @ MyError::InsufficientFunds
    )]
    pub from: Account<'info, Wallet>,

    #[account(mut)]
    pub to: Account<'info, Wallet>,

    pub owner: Signer<'info>,
}
```

### #[account]

```rust,ignore
#[account]
pub struct GameState {
    pub admin: Pubkey,
    pub total_players: u32,
    pub prize_pool: u64,
    pub is_active: bool,
    pub name: String,
}

// #[account] 매크로가 하는 일:
// 1. BorshSerialize, BorshDeserialize 자동 구현
// 2. AccountSerialize, AccountDeserialize 구현
//    → 앞 8바이트 discriminator 자동 추가/검증
// 3. Owner 트레이트 구현
//    → 이 구조체는 현재 프로그램만 소유할 수 있음

// discriminator = sha256("account:GameState")[..8]
// 이를 통해 타입 안전성 보장: 잘못된 타입의 계정 전달 시 에러
```

---

## 프로젝트 초기 파일 내용

### programs/my-project/src/lib.rs (초기 템플릿)

```rust,ignore
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod my_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
```

### programs/my-project/Cargo.toml

```toml
[package]
name = "my-project"
version = "0.1.0"
description = "Created with Anchor"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "my_project"

[features]
default = []
cpi = []    # 다른 프로그램에서 이 프로그램을 CPI로 호출할 때 활성화
no-entrypoint = []  # 라이브러리로만 사용할 때
no-idl = []
no-log-ix-name = []
is-upgrade-authority-signer = []

[dependencies]
anchor-lang = "0.30.1"
# SPL Token 사용 시:
# anchor-spl = "0.30.1"
```

### tests/my-project.ts (초기 템플릿)

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MyProject } from "../target/types/my_project";

describe("my-project", () => {
  // 테스트 환경 설정
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MyProject as Program<MyProject>;

  it("Is initialized!", async () => {
    const tx = await program.methods.initialize().rpc();
    console.log("Transaction signature", tx);
  });
});
```

---

## 빌드 및 배포 명령어

```bash
# 빌드 (Rust → BPF 바이트코드)
anchor build
# 결과: target/deploy/my_project.so
# 결과: target/idl/my_project.json
# 결과: target/types/my_project.ts

# 프로그램 ID 확인 및 업데이트
anchor keys list
# my_project: Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS

# Anchor.toml과 lib.rs의 ID가 다르면 동기화
anchor keys sync

# 로컬 테스트 검증자 실행 (별도 터미널)
solana-test-validator

# 로컬에 배포
anchor deploy

# 테스트 실행 (로컬 검증자 자동 시작 + 배포 + 테스트)
anchor test

# Devnet에 배포
anchor deploy --provider.cluster devnet
```

---

## 전체 흐름 한눈에 보기

```text
개발 워크플로:

[lib.rs 작성]
     │
     ▼
[anchor build]
     │
     ├─ target/deploy/my_project.so (BPF 바이트코드)
     ├─ target/idl/my_project.json  (ABI)
     └─ target/types/my_project.ts  (TypeScript 타입)
     │
     ▼
[anchor test]
     │
     ├─ 로컬 검증자 시작
     ├─ 프로그램 배포
     └─ tests/*.ts 실행 (Mocha + Chai)
          │
          └─ program.methods.xxx().accounts({}).rpc()
               → 트랜잭션 전송 → 프로그램 실행 → 결과 확인
```

다음 장에서는 `#[derive(Accounts)]`의 계정 타입과 제약조건을 상세히 알아봅니다.
