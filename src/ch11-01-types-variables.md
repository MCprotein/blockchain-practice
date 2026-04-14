# Chapter 11-1: 타입과 변수

## Solidity 타입 시스템 개요

Solidity는 정적 타입 언어다. TypeScript처럼 컴파일 시점에 타입을 확인하며, 모든 변수는 선언 시 타입을 명시해야 한다. 단, TypeScript와 달리 타입 추론이 제한적이므로 거의 항상 명시적으로 타입을 작성한다.

## 값 타입 (Value Types)

값 타입은 변수에 값 자체가 저장되며, 다른 변수에 할당할 때 복사된다.

### bool

```solidity
bool public isActive = true;
bool public isPaused = false;

function toggle() public {
    isActive = !isActive;
}
```

TypeScript의 `boolean`과 동일하다. `&&`, `||`, `!` 연산자 사용 가능.

### uint와 int — 정수형

Solidity의 정수형은 크기를 직접 지정한다. 8비트 단위로 8~256까지 지정 가능하다.

```solidity
uint8   smallNumber = 255;        // 0 ~ 255
uint16  mediumNumber = 65535;     // 0 ~ 65,535
uint32  tokenId = 4294967295;     // 0 ~ 4,294,967,295
uint128 halfMax = type(uint128).max;
uint256 bigNumber = 1e18;         // 10^18 (1 ETH in wei)

// uint는 uint256의 별칭
uint sameAsUint256 = 100;

// 부호 있는 정수
int8  temp = -10;    // -128 ~ 127
int256 debt = -1000; // int는 int256의 별칭
```

**TypeScript와의 비교:**

| TypeScript | Solidity | 범위 |
|-----------|---------|------|
| `number` | `uint8` | 0~255 |
| `number` | `uint16` | 0~65535 |
| `number` | `uint256` | 0~2^256-1 |
| `number` | `int256` | -2^255 ~ 2^255-1 |
| `bigint` | `uint256` | 유사하지만 Solidity는 언어 기본형 |

TypeScript의 `number`는 IEEE 754 64비트 부동소수점이라 정밀도 문제가 있다. Solidity는 정수만 지원하므로 이런 문제가 없다. **Solidity에는 소수점이 없다.** 이더(ETH)도 wei 단위(10^18)의 정수로 표현한다.

```solidity
// ETH 금액 계산 예시
uint256 oneEther = 1 ether;           // 1000000000000000000 (10^18 wei)
uint256 halfEther = 0.5 ether;        // 500000000000000000
uint256 oneGwei = 1 gwei;             // 1000000000 (10^9 wei)

// 퍼센트 계산: 소수점 대신 basis point (1/10000) 사용
uint256 feeRate = 250;    // 2.50%
uint256 amount = 10000;
uint256 fee = amount * feeRate / 10000;  // 250 (2.5%)
```

### address

Ethereum 주소(20바이트, 40자리 16진수)를 저장하는 타입이다.

```solidity
address public owner;
address payable public treasury;  // ETH를 받을 수 있는 주소

// address 리터럴 (체크섬 형식)
address constant ZERO_ADDRESS = address(0);
address constant BURN_ADDRESS = 0x000000000000000000000000000000000000dEaD;

function example() public {
    // address 비교
    require(msg.sender != address(0), "Invalid address");
    
    // 잔액 조회
    uint256 balance = address(this).balance;        // 이 컨트랙트의 ETH 잔액
    uint256 ownerBalance = owner.balance;           // owner 주소의 ETH 잔액
    
    // ETH 전송 (payable 주소에만 가능)
    address payable recipient = payable(msg.sender);
    recipient.transfer(1 ether);    // 실패 시 revert
    recipient.send(1 ether);        // 실패 시 false 반환 (권장하지 않음)
    
    // call 방식 (권장)
    (bool success, ) = recipient.call{value: 1 ether}("");
    require(success, "Transfer failed");
}
```

`address`와 `address payable`의 차이: `payable` 주소만 `transfer`, `send`를 직접 호출할 수 있다. 일반 `address`는 `call`을 사용해야 한다.

### bytes — 고정 크기 바이트 배열

```solidity
bytes1  singleByte = 0xFF;
bytes4  selector = 0x12345678;    // 함수 선택자 (4바이트)
bytes32 hash = keccak256("hello"); // 해시값 저장에 자주 사용

// 비트 연산
bytes1 a = 0x0F;
bytes1 b = 0xF0;
bytes1 result = a | b;  // 0xFF
```

`bytes32`는 특히 자주 쓰인다. 해시 저장, role 정의 등에 활용된다.

## 참조 타입 (Reference Types)

참조 타입은 데이터의 위치(storage, memory, calldata)를 함께 지정해야 한다.

### string

```solidity
string public name = "MyToken";
string private description;

function setDescription(string memory newDesc) public {
    description = newDesc;
}

function getDescription() public view returns (string memory) {
    return description;
}
```

Solidity의 `string`은 UTF-8로 인코딩된 동적 길이 바이트 배열이다. TypeScript의 `string`과 달리 인덱스 접근(`s[0]`)이나 길이 확인(`s.length`)이 직접 되지 않는다. 문자열 연산이 필요하면 `bytes`로 변환하거나 라이브러리를 사용한다.

```solidity
// 문자열 길이 확인 (bytes로 변환 필요)
string memory s = "hello";
uint256 len = bytes(s).length;  // 5

// 문자열 연결 (abi.encodePacked 사용)
string memory greeting = string(abi.encodePacked("Hello, ", name, "!"));
```

### 배열 (Array)

```solidity
// 고정 크기 배열
uint256[3] public fixedArray = [1, 2, 3];

// 동적 배열 (상태 변수)
uint256[] public dynamicArray;
address[] public voters;

function addVoter(address voter) public {
    voters.push(voter);           // 요소 추가
}

function removeLastVoter() public {
    voters.pop();                  // 마지막 요소 제거
}

function getVoterCount() public view returns (uint256) {
    return voters.length;
}

// 메모리 배열 (함수 내부)
function createTempArray(uint256 size) public pure returns (uint256[] memory) {
    uint256[] memory temp = new uint256[](size);  // 동적 크기
    for (uint256 i = 0; i < size; i++) {
        temp[i] = i * 2;
    }
    return temp;
}
```

**TypeScript Array와의 차이:**
- Storage 배열은 `push`, `pop`만 지원 (중간 삽입 없음)
- 메모리 배열은 크기를 미리 지정해야 함
- 배열 삭제(`delete arr[i]`)는 해당 인덱스를 기본값으로 초기화할 뿐 크기는 줄지 않음

### struct — 구조체

```solidity
struct User {
    address wallet;
    string username;
    uint256 balance;
    bool isActive;
    uint256 createdAt;
}

// 상태 변수로 저장
User public admin;
User[] public allUsers;
mapping(address => User) public users;

function createUser(string memory username) public {
    // 구조체 초기화 방법 1: 필드명 지정
    users[msg.sender] = User({
        wallet: msg.sender,
        username: username,
        balance: 0,
        isActive: true,
        createdAt: block.timestamp
    });
    
    // 구조체 초기화 방법 2: 순서대로
    allUsers.push(User(msg.sender, username, 0, true, block.timestamp));
}

function deactivateUser() public {
    // storage 참조로 직접 수정
    User storage user = users[msg.sender];
    user.isActive = false;
}
```

TypeScript의 `interface`나 `type`과 유사하다. 단, `User storage user`로 참조를 가져오면 실제 storage를 직접 수정하고, `User memory user`로 가져오면 복사본을 수정하므로 원본이 바뀌지 않는다.

### mapping — 키-값 저장소

```solidity
// mapping(키 타입 => 값 타입)
mapping(address => uint256) public balances;
mapping(address => bool) public whitelist;
mapping(uint256 => string) public tokenURIs;

// 중첩 매핑 (ERC-20의 allowance 구조)
mapping(address => mapping(address => uint256)) public allowances;

function getBalance(address account) public view returns (uint256) {
    return balances[account];  // 없는 키는 기본값(0) 반환
}

function allow(address spender, uint256 amount) public {
    allowances[msg.sender][spender] = amount;
}
```

mapping은 별도의 챕터(11-03)에서 더 자세히 다룬다.

## 상태 변수 vs 로컬 변수 vs 전역 변수

### 상태 변수 (State Variable)

컨트랙트 최상위 레벨에 선언되며 블록체인의 storage에 영구 저장된다.

```solidity
contract Example {
    uint256 public totalSupply;     // 상태 변수 (storage)
    address private _owner;         // 상태 변수 (storage)
    string public name;             // 상태 변수 (storage)
}
```

쓰기(write) 비용이 비싸다. `SSTORE` opcode는 약 20,000 가스 (새 슬롯 쓰기) 또는 5,000 가스 (기존 슬롯 업데이트).

### 로컬 변수 (Local Variable)

함수 내부에 선언되며 함수 실행 중에만 존재한다. EVM 스택이나 메모리를 사용하므로 훨씬 저렴하다.

```solidity
function calculate(uint256 a, uint256 b) public pure returns (uint256) {
    uint256 sum = a + b;       // 로컬 변수 (stack)
    uint256 product = a * b;   // 로컬 변수 (stack)
    return sum + product;
}
```

### 전역 변수 (Global Variable)

EVM이 제공하는 특수 변수들이다. 선언 없이 어디서든 사용할 수 있다.

```solidity
// 블록 관련
block.timestamp   // 현재 블록의 Unix 타임스탬프 (uint256)
block.number      // 현재 블록 번호 (uint256)
block.coinbase    // 현재 블록을 채굴한 주소 (address payable)
block.gaslimit    // 현재 블록의 가스 한도 (uint256)
block.basefee     // 현재 블록의 기본 가스비 (uint256, EIP-1559)

// 트랜잭션 관련
msg.sender    // 현재 함수 호출자 주소
msg.value     // 전송된 ETH (wei)
msg.data      // 전체 calldata (bytes)
msg.sig       // 함수 선택자 (bytes4, msg.data의 처음 4바이트)

tx.origin     // 트랜잭션 최초 발신자 (EOA만 가능)
tx.gasprice   // 트랜잭션의 가스 가격

// 가스
gasleft()     // 남은 가스량 (함수)
```

## 가시성 (Visibility)

### 상태 변수 가시성

```solidity
contract Visibility {
    uint256 public pubVar;      // 외부 읽기 가능, 자동 getter 생성
    uint256 private privVar;    // 이 컨트랙트만 접근 가능
    uint256 internal intVar;    // 이 컨트랙트 + 상속 컨트랙트
    // external은 상태 변수에 사용 불가
}
```

`public` 상태 변수는 Solidity가 자동으로 getter 함수를 생성한다:

```solidity
uint256 public totalSupply = 1000;
// 위 선언이 아래 함수를 자동 생성:
// function totalSupply() external view returns (uint256) { return totalSupply; }
```

### 함수 가시성

```solidity
contract FunctionVisibility {
    // public: 외부, 내부 모두 호출 가능
    function publicFunc() public {}
    
    // external: 외부에서만 호출 가능 (내부에서 this.externalFunc()로는 가능)
    function externalFunc() external {}
    
    // internal: 이 컨트랙트 + 상속 컨트랙트에서 호출 가능
    function internalFunc() internal {}
    
    // private: 이 컨트랙트에서만 호출 가능
    function privateFunc() private {}
}
```

**성능 팁:** 외부에서만 호출할 함수는 `external`로 선언하는 게 `public`보다 약간 가스를 절약한다. `external` 함수의 파라미터는 calldata에서 직접 읽으므로 복사 비용이 없다.

## 상수: constant와 immutable

```solidity
contract Constants {
    // constant: 컴파일 타임 상수 (리터럴만 가능)
    uint256 public constant MAX_SUPPLY = 1_000_000 * 10**18;
    string public constant NAME = "MyToken";
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    
    // immutable: 배포 시 한 번만 설정 가능
    address public immutable deployer;
    uint256 public immutable deployedAt;
    
    constructor() {
        deployer = msg.sender;        // 생성자에서만 설정 가능
        deployedAt = block.timestamp; // 런타임 값 사용 가능
    }
}
```

**constant vs immutable 비교:**

| | constant | immutable |
|--|---------|-----------|
| 값 설정 시점 | 컴파일 시 | 배포 시 (생성자) |
| 런타임 값 사용 | 불가 | 가능 |
| 가스 비용 | 가장 저렴 | 저렴 |
| 사용 사례 | 고정 한도, 역할 해시 | 배포자 주소, 배포 시각 |

`constant`와 `immutable` 모두 storage를 사용하지 않아서 일반 상태 변수보다 훨씬 저렴하다.

## 데이터 위치 (Data Location)

Solidity의 참조 타입(string, bytes, array, struct, mapping)은 반드시 데이터 위치를 지정해야 한다.

### storage

블록체인에 영구 저장. 상태 변수는 기본적으로 storage.

```solidity
mapping(address => uint256) public balances;  // storage (상태 변수)

function updateUser(address addr) internal {
    // storage 참조 - 실제 데이터를 직접 수정
    User storage user = users[addr];
    user.balance += 100;  // storage의 값이 직접 변경됨
}
```

### memory

함수 실행 중에만 존재하는 임시 메모리. 함수 호출이 끝나면 사라진다.

```solidity
function processName(string memory name) public pure returns (string memory) {
    // memory - 복사본, 함수 끝나면 사라짐
    bytes memory nameBytes = bytes(name);
    return string(nameBytes);
}
```

### calldata

함수의 입력 파라미터 영역. 읽기 전용이고 `external` 함수에서 사용 가능. `memory`보다 저렴하다.

```solidity
// calldata - 복사 없이 직접 읽기 (gas 절약)
function processData(bytes calldata data) external pure returns (uint256) {
    return data.length;
}
```

### 언제 무엇을 쓸까?

```solidity
contract DataLocationExample {
    struct Item {
        uint256 id;
        string name;
    }
    
    Item[] public items;
    
    // calldata: external 함수의 읽기 전용 파라미터 (가장 저렴)
    function addItem(string calldata name) external {
        items.push(Item(items.length, name));
    }
    
    // memory: 함수 내부에서 수정하거나 반환할 때
    function getItemCopy(uint256 id) external view returns (Item memory) {
        return items[id];  // storage에서 memory로 복사
    }
    
    // storage: storage 데이터를 직접 수정할 때
    function renameItem(uint256 id, string memory newName) external {
        Item storage item = items[id];  // 참조
        item.name = newName;            // storage 직접 수정
    }
}
```

## TypeScript 타입과 Solidity 타입 대응표

| TypeScript | Solidity | 비고 |
|-----------|---------|------|
| `boolean` | `bool` | 동일 |
| `number` | `uint256` | TS는 부동소수점, SOL은 정수 |
| `bigint` | `uint256` | 범위 다름 (SOL은 2^256-1까지) |
| `string` | `string` | SOL은 인덱스 접근 불가 |
| `Buffer`/`Uint8Array` | `bytes` | 동적 크기 바이트 |
| `string` (hex) | `address` | 20바이트 특수 타입 |
| `T[]` | `T[]` / `T[N]` | SOL은 고정/동적 구분 |
| `Record<K,V>` / `Map<K,V>` | `mapping(K => V)` | SOL은 순회 불가 |
| `interface`/`type` | `struct` | SOL struct는 메서드 없음 |
| `enum` | `enum` | SOL enum은 uint8 기반 |
| `null`/`undefined` | 없음 | SOL은 기본값으로 초기화 |

## 기본값 (Default Values)

Solidity에서 선언만 하고 초기화하지 않으면 타입의 기본값이 자동으로 설정된다:

```solidity
bool public b;           // false
uint256 public n;        // 0
int256 public i;         // 0
address public a;        // address(0) = 0x0000...0000
bytes32 public hash;     // bytes32(0)
string public s;         // "" (빈 문자열)
uint256[] public arr;    // [] (빈 배열)
```

TypeScript에서 `undefined`나 `null`이 없는 것처럼, Solidity에도 `null`이 없다. 대신 기본값으로 초기화된다. 이 특성 때문에 `mapping`에서 없는 키를 조회하면 0이 반환된다.

## 정리

Solidity의 타입 시스템은 TypeScript보다 더 세밀하고 제약이 많다. 특히:

1. **정수 크기를 명시해야 한다** — `uint256`, `uint8` 등으로 메모리 크기 최적화
2. **소수점이 없다** — 금액 계산 시 wei 단위 또는 basis point 활용
3. **데이터 위치를 지정해야 한다** — storage/memory/calldata는 가스 비용에 직결
4. **기본값이 있다** — null/undefined 없이 항상 타입의 기본값으로 초기화

다음 챕터에서는 함수와 제어자(modifier)를 자세히 살펴본다.
