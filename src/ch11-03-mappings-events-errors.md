# Chapter 11-3: Mapping, 이벤트, 에러 처리

## mapping — 키-값 저장소

### 기본 선언과 사용

`mapping`은 Solidity에서 가장 중요한 자료구조다. 해시 테이블 기반으로 구현되어 있어 O(1) 조회가 가능하다.

```solidity
// mapping(키 타입 => 값 타입) 가시성 변수명;
mapping(address => uint256) public balances;
mapping(uint256 => address) public tokenOwners;
mapping(address => bool) public whitelist;
mapping(bytes32 => uint256) public roleTimestamps;
```

**기본 사용법:**

```solidity
contract MappingBasics {
    mapping(address => uint256) private _balances;
    
    function set(address account, uint256 amount) public {
        _balances[account] = amount;        // 쓰기
    }
    
    function get(address account) public view returns (uint256) {
        return _balances[account];           // 읽기
    }
    
    function addAmount(address account, uint256 amount) public {
        _balances[account] += amount;        // 기존 값에 더하기
    }
    
    function deleteEntry(address account) public {
        delete _balances[account];           // 기본값(0)으로 초기화
    }
}
```

### JavaScript Map과의 결정적 차이

```javascript
// JavaScript Map
const balances = new Map();
balances.set('alice', 100);
console.log(balances.get('bob'));  // undefined
console.log(balances.has('bob')); // false

// 순회 가능
for (const [addr, amount] of balances) {
    console.log(addr, amount);
}
balances.size; // 크기 확인 가능
```

```solidity
// Solidity mapping
mapping(address => uint256) balances;
balances[alice] = 100;
balances[bob];    // 0 (undefined가 아니라 기본값!)
// balances.has(bob) 같은 메서드 없음
// for...of 순회 불가능
// .size/.length 없음
// 저장된 키 목록 조회 불가능
```

**Solidity mapping의 3가지 핵심 제약:**

1. **순회 불가능** — 어떤 키가 저장되어 있는지 알 수 없다
2. **기본값 존재** — 없는 키를 읽으면 0/false/address(0) 등 기본값 반환
3. **삭제해도 완전 제거 아님** — `delete`는 기본값으로 초기화만 함

**"키가 존재하는지" 확인하는 패턴:**

```solidity
// 방법 1: 별도 bool mapping으로 추적
mapping(address => uint256) public balances;
mapping(address => bool) public exists;

function register(address user) public {
    exists[user] = true;
    balances[user] = 0;
}

// 방법 2: struct의 sentinel 값으로 구분
struct User {
    uint256 balance;
    bool registered;  // 등록 여부 추적
}
mapping(address => User) public users;

function isRegistered(address user) public view returns (bool) {
    return users[user].registered;
}
```

**순회가 필요한 경우 — 별도 배열로 키 추적:**

```solidity
contract IterableMapping {
    mapping(address => uint256) public balances;
    address[] public holders;  // 키(주소) 목록을 별도 배열로 유지
    mapping(address => bool) private _isHolder;
    
    function deposit(address user, uint256 amount) public {
        if (!_isHolder[user]) {
            holders.push(user);
            _isHolder[user] = true;
        }
        balances[user] += amount;
    }
    
    // 이제 순회 가능
    function getTotalBalance() public view returns (uint256 total) {
        for (uint256 i = 0; i < holders.length; i++) {
            total += balances[holders[i]];
        }
    }
    
    function getHolderCount() public view returns (uint256) {
        return holders.length;
    }
}
```

주의: 배열을 유지하는 비용이 추가된다. 배열이 커지면 `getTotalBalance` 같은 루프 함수는 가스 한도를 초과할 수 있다.

### 중첩 mapping (Nested Mapping)

```solidity
// ERC-20의 allowance 구조
// owner => spender => 허용량
mapping(address => mapping(address => uint256)) private _allowances;

function approve(address spender, uint256 amount) public returns (bool) {
    _allowances[msg.sender][spender] = amount;
    emit Approval(msg.sender, spender, amount);
    return true;
}

function allowance(address owner, address spender) public view returns (uint256) {
    return _allowances[owner][spender];
}
```

더 복잡한 중첩:

```solidity
// NFT 마켓플레이스: 판매자 => 토큰ID => 가격
mapping(address => mapping(uint256 => uint256)) public listings;

// 역할 관리: 역할 해시 => 계정 => 권한 여부
mapping(bytes32 => mapping(address => bool)) private _roles;

function grantRole(bytes32 role, address account) public {
    _roles[role][account] = true;
}

function hasRole(bytes32 role, address account) public view returns (bool) {
    return _roles[role][account];
}
```

## 이벤트 (Event)

### 이벤트의 역할

이벤트는 블록체인의 로그(log)에 데이터를 기록하는 메커니즘이다. 상태 변수보다 훨씬 저렴하게 데이터를 기록할 수 있으며, 외부(프론트엔드, 백엔드 서버)에서 구독할 수 있다.

**이벤트 vs 상태 변수 비교:**
- 상태 변수 저장: ~20,000 가스 (새 슬롯)
- 이벤트 로그: ~375 가스 (기본) + 8 가스/byte

단, 이벤트 로그는 컨트랙트 내부에서 읽을 수 없다. 오직 외부에서만 조회 가능하다.

### 이벤트 선언과 발행

```solidity
contract EventExample {
    // 이벤트 선언
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event Mint(address indexed to, uint256 amount, uint256 timestamp);
    
    // 이벤트 발행 (emit)
    function transfer(address to, uint256 amount) public {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        balances[msg.sender] -= amount;
        balances[to] += amount;
        emit Transfer(msg.sender, to, amount);  // emit 키워드로 발행
    }
}
```

### indexed 파라미터

이벤트 파라미터에 `indexed`를 붙이면 해당 값으로 이벤트를 필터링할 수 있다. 이벤트당 최대 3개의 `indexed` 파라미터를 가질 수 있다.

```solidity
event Transfer(
    address indexed from,   // 인덱싱됨 - 필터링 가능
    address indexed to,     // 인덱싱됨 - 필터링 가능
    uint256 value           // 인덱싱 안됨 - 데이터로만 저장
);

// indexed가 있으면 특정 주소의 Transfer만 필터링 가능:
// 예: alice가 보낸 모든 Transfer 이벤트 조회 가능
```

**indexed vs non-indexed 비교:**
- `indexed`: Bloom 필터에 저장 → 빠른 검색 가능, 최대 32바이트 (32바이트 초과 시 keccak256 해시)
- non-indexed: ABI 인코딩되어 data 필드에 저장 → 검색 불가하지만 임의 크기 가능

### Node.js EventEmitter와의 비교

```typescript
// Node.js EventEmitter
const EventEmitter = require('events');
const emitter = new EventEmitter();

// 이벤트 리스너 등록
emitter.on('Transfer', ({ from, to, value }) => {
    console.log(`${from} -> ${to}: ${value}`);
});

// 이벤트 발생
emitter.emit('Transfer', { from: 'alice', to: 'bob', value: 100 });
```

```solidity
// Solidity - 이벤트 발행
event Transfer(address indexed from, address indexed to, uint256 value);

function transfer(address to, uint256 value) public {
    emit Transfer(msg.sender, to, value);
}
```

```typescript
// ethers.js - Solidity 이벤트 구독 (프론트엔드/백엔드)
const contract = new ethers.Contract(address, abi, provider);

// 실시간 구독
contract.on('Transfer', (from, to, value, event) => {
    console.log(`Transfer: ${from} -> ${to}: ${value}`);
    console.log('Block:', event.blockNumber);
    console.log('TxHash:', event.transactionHash);
});

// 과거 이벤트 조회
const filter = contract.filters.Transfer(aliceAddress); // alice가 보낸 것만
const events = await contract.queryFilter(filter, fromBlock, toBlock);
events.forEach(e => {
    console.log(e.args.from, e.args.to, e.args.value);
});
```

### 이벤트 설계 패턴

```solidity
contract GoodEventDesign {
    // 규칙 1: 중요한 상태 변경은 항상 이벤트 발행
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    
    // 규칙 2: indexed는 "누가/무엇이" 필터링이 필요한 파라미터에
    event TokenMinted(
        address indexed to,      // "누가 받았나" 필터링 가능
        uint256 indexed tokenId, // "어떤 토큰" 필터링 가능
        string metadataURI       // 메타데이터 (indexed 불필요, 32byte 초과 가능)
    );
    
    // 규칙 3: 이전 값과 새 값 모두 기록
    event PriceUpdated(
        uint256 indexed tokenId,
        uint256 oldPrice,
        uint256 newPrice
    );
    
    // 규칙 4: 타임스탬프는 이벤트에 포함할 필요 없음 (블록에 이미 있음)
    // block.timestamp는 이벤트 구독자가 event.blockNumber로 조회 가능
}
```

## 에러 처리

### require()

가장 일반적인 조건 검증. 조건이 false면 트랜잭션을 revert하고 메시지를 반환한다.

```solidity
function transfer(address to, uint256 amount) public {
    // 입력값 검증
    require(to != address(0), "ERC20: transfer to zero address");
    require(amount > 0, "ERC20: amount must be positive");
    
    // 상태 검증
    require(balances[msg.sender] >= amount, "ERC20: insufficient balance");
    
    // 실행
    balances[msg.sender] -= amount;
    balances[to] += amount;
    emit Transfer(msg.sender, to, amount);
}
```

가스 효율: `require`가 실패하면 남은 가스를 전부 반환한다 (이미 소비된 가스는 제외).

### revert()

`require`와 동일하게 트랜잭션을 되돌리지만, 더 복잡한 조건 처리에 유용하다.

```solidity
function withdraw(uint256 amount) public {
    if (amount == 0) {
        revert("Cannot withdraw zero");
    }
    if (balances[msg.sender] < amount) {
        revert("Insufficient balance");
    }
    balances[msg.sender] -= amount;
    payable(msg.sender).transfer(amount);
}
```

### assert()

내부 불변식(invariant) 검증에 사용. 절대 false가 되어서는 안 되는 조건에 사용한다. `assert`가 실패하면 남은 가스를 전부 소비한다 (버그를 나타내므로 페널티).

```solidity
function _update(address from, address to, uint256 amount) internal {
    if (from != address(0)) {
        balances[from] -= amount;
    }
    if (to != address(0)) {
        balances[to] += amount;
    }
    
    // 불변식: 총 공급량은 항상 모든 잔액의 합과 같아야 함
    // 이 조건이 false라면 코드에 심각한 버그가 있는 것
    assert(totalSupply == calculateTotalBalance());
}
```

**언제 무엇을 써야 하나:**

| 상황 | 사용 |
|------|------|
| 외부 입력값 검증 | `require` |
| 비즈니스 규칙 검증 | `require` |
| 내부 불변식 검증 | `assert` |
| 복잡한 분기 처리 | `revert` |
| 커스텀 에러 타입 | `revert CustomError()` |

### 커스텀 에러 (Custom Errors, Solidity 0.8.4+)

Solidity 0.8.4부터 타입이 있는 커스텀 에러를 정의할 수 있다. 문자열 에러 메시지보다 가스 효율이 훨씬 좋다.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract CustomErrors {
    // 에러 선언 (컨트랙트 외부 또는 내부에 선언 가능)
    error InsufficientBalance(address user, uint256 available, uint256 required);
    error Unauthorized(address caller, address required);
    error InvalidAmount(uint256 amount);
    error TransferToZeroAddress();
    
    mapping(address => uint256) public balances;
    address public owner;
    
    constructor() {
        owner = msg.sender;
    }
    
    function transfer(address to, uint256 amount) public {
        // 커스텀 에러로 revert (파라미터 포함)
        if (to == address(0)) revert TransferToZeroAddress();
        if (amount == 0) revert InvalidAmount(amount);
        if (balances[msg.sender] < amount) {
            revert InsufficientBalance(msg.sender, balances[msg.sender], amount);
        }
        
        balances[msg.sender] -= amount;
        balances[to] += amount;
    }
    
    function adminAction() public {
        if (msg.sender != owner) revert Unauthorized(msg.sender, owner);
        paused = !paused;
    }
}
```

**커스텀 에러의 장점:**
- 가스 절약: 문자열 저장 불필요 (4바이트 선택자 + ABI 인코딩 파라미터만)
- 타입 안전성: 파라미터가 명확히 정의됨
- 프론트엔드에서 파싱 용이

```typescript
// ethers.js에서 커스텀 에러 처리
try {
    await contract.transfer(toAddress, amount);
} catch (error) {
    if (error.code === 'CALL_EXCEPTION') {
        // 커스텀 에러 파싱
        const decoded = contract.interface.parseError(error.data);
        if (decoded?.name === 'InsufficientBalance') {
            const { user, available, required } = decoded.args;
            console.log(`잔액 부족: ${available} < ${required}`);
        }
    }
}
```

**가스 비용 비교:**

```solidity
// 문자열 에러: ~50+ bytes of data
require(amount > 0, "ERC20: transfer amount must be greater than zero");

// 커스텀 에러: 4 bytes selector + encoded params
if (amount == 0) revert InvalidAmount(amount);
```

## try/catch

외부 컨트랙트 호출 시 에러를 처리하는 구문이다.

```solidity
interface IOracle {
    function getPrice(address token) external view returns (uint256);
}

contract PriceConsumer {
    IOracle public oracle;
    
    constructor(address _oracle) {
        oracle = IOracle(_oracle);
    }
    
    function getSafePrice(address token) public view returns (uint256 price, bool success) {
        try oracle.getPrice(token) returns (uint256 _price) {
            // 성공 케이스
            return (_price, true);
        } catch Error(string memory reason) {
            // require/revert("message") 실패
            emit PriceFetchFailed(token, reason);
            return (0, false);
        } catch Panic(uint256 errorCode) {
            // assert 실패, 오버플로 등 (errorCode로 구분)
            // errorCode: 0x01=assert, 0x11=overflow, 0x12=div by zero
            return (0, false);
        } catch (bytes memory lowLevelData) {
            // 커스텀 에러 또는 기타
            return (0, false);
        }
    }
}
```

**TypeScript try/catch와의 비교:**

```typescript
// TypeScript
async function getSafePrice(token: string): Promise<number | null> {
    try {
        const price = await oracle.getPrice(token);
        return price;
    } catch (error) {
        if (error instanceof InsufficientDataError) {
            console.log('데이터 부족:', error.message);
        }
        return null;
    }
}
```

```solidity
// Solidity
function getSafePrice(address token) public returns (uint256) {
    try oracle.getPrice(token) returns (uint256 price) {
        return price;
    } catch {
        return 0;
    }
}
```

주요 차이: Solidity의 `try/catch`는 **외부 함수 호출**에만 사용 가능하다. 내부 함수 호출이나 산술 연산에는 사용할 수 없다.

## 완전한 예제: 이벤트와 에러가 있는 NFT 마켓플레이스

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IERC721 {
    function ownerOf(uint256 tokenId) external view returns (address);
    function transferFrom(address from, address to, uint256 tokenId) external;
    function getApproved(uint256 tokenId) external view returns (address);
    function isApprovedForAll(address owner, address operator) external view returns (bool);
}

/// @title NFT 마켓플레이스 컨트랙트
contract NFTMarketplace {
    // ============ 커스텀 에러 ============
    error NotTokenOwner(address caller, address owner, uint256 tokenId);
    error NotApproved(uint256 tokenId);
    error ListingNotFound(address nft, uint256 tokenId);
    error ListingAlreadyExists(address nft, uint256 tokenId);
    error PriceTooLow(uint256 provided, uint256 minimum);
    error InsufficientPayment(uint256 sent, uint256 required);
    error WithdrawFailed(address recipient, uint256 amount);
    error InvalidAddress();
    
    // ============ 구조체 ============
    struct Listing {
        address seller;
        uint256 price;
        bool active;
    }
    
    // ============ 상태 변수 ============
    uint256 public constant MINIMUM_PRICE = 0.001 ether;
    uint256 public constant FEE_BPS = 250; // 2.5% (basis points)
    address public feeRecipient;
    
    // nft 주소 => tokenId => 판매 정보
    mapping(address => mapping(uint256 => Listing)) public listings;
    
    // 판매자별 미정산 수익
    mapping(address => uint256) public proceeds;
    
    // ============ 이벤트 ============
    event Listed(
        address indexed nft,
        uint256 indexed tokenId,
        address indexed seller,
        uint256 price
    );
    
    event Sold(
        address indexed nft,
        uint256 indexed tokenId,
        address indexed buyer,
        address seller,
        uint256 price
    );
    
    event Cancelled(
        address indexed nft,
        uint256 indexed tokenId,
        address indexed seller
    );
    
    event PriceUpdated(
        address indexed nft,
        uint256 indexed tokenId,
        uint256 oldPrice,
        uint256 newPrice
    );
    
    event ProceedsWithdrawn(address indexed seller, uint256 amount);
    
    // ============ 생성자 ============
    constructor(address _feeRecipient) {
        if (_feeRecipient == address(0)) revert InvalidAddress();
        feeRecipient = _feeRecipient;
    }
    
    // ============ 판매 등록 ============
    function listItem(
        address nft,
        uint256 tokenId,
        uint256 price
    ) external {
        if (nft == address(0)) revert InvalidAddress();
        if (price < MINIMUM_PRICE) revert PriceTooLow(price, MINIMUM_PRICE);
        
        IERC721 token = IERC721(nft);
        
        // 소유자 확인
        address tokenOwner = token.ownerOf(tokenId);
        if (msg.sender != tokenOwner) {
            revert NotTokenOwner(msg.sender, tokenOwner, tokenId);
        }
        
        // 승인 확인 (마켓플레이스가 전송 권한 필요)
        bool approved = token.getApproved(tokenId) == address(this)
            || token.isApprovedForAll(msg.sender, address(this));
        if (!approved) revert NotApproved(tokenId);
        
        // 중복 등록 확인
        if (listings[nft][tokenId].active) {
            revert ListingAlreadyExists(nft, tokenId);
        }
        
        listings[nft][tokenId] = Listing({
            seller: msg.sender,
            price: price,
            active: true
        });
        
        emit Listed(nft, tokenId, msg.sender, price);
    }
    
    // ============ 구매 ============
    function buyItem(address nft, uint256 tokenId) external payable {
        Listing storage listing = listings[nft][tokenId];
        
        if (!listing.active) revert ListingNotFound(nft, tokenId);
        if (msg.value < listing.price) {
            revert InsufficientPayment(msg.value, listing.price);
        }
        
        address seller = listing.seller;
        uint256 price = listing.price;
        
        // 상태 먼저 변경 (재진입 공격 방지)
        listing.active = false;
        
        // 수수료 계산
        uint256 fee = (price * FEE_BPS) / 10000;
        uint256 sellerProceeds = price - fee;
        
        // 수익 기록
        proceeds[seller] += sellerProceeds;
        proceeds[feeRecipient] += fee;
        
        // NFT 전송 (외부 호출 - 상태 변경 후)
        try IERC721(nft).transferFrom(seller, msg.sender, tokenId) {
            emit Sold(nft, tokenId, msg.sender, seller, price);
        } catch {
            // 전송 실패 시 상태 복구
            listing.active = true;
            proceeds[seller] -= sellerProceeds;
            proceeds[feeRecipient] -= fee;
            revert("NFT transfer failed");
        }
        
        // 초과 지불금 환불
        uint256 excess = msg.value - price;
        if (excess > 0) {
            (bool refundSuccess, ) = payable(msg.sender).call{value: excess}("");
            require(refundSuccess, "Refund failed");
        }
    }
    
    // ============ 등록 취소 ============
    function cancelListing(address nft, uint256 tokenId) external {
        Listing storage listing = listings[nft][tokenId];
        
        if (!listing.active) revert ListingNotFound(nft, tokenId);
        if (listing.seller != msg.sender) {
            revert NotTokenOwner(msg.sender, listing.seller, tokenId);
        }
        
        listing.active = false;
        emit Cancelled(nft, tokenId, msg.sender);
    }
    
    // ============ 가격 수정 ============
    function updatePrice(address nft, uint256 tokenId, uint256 newPrice) external {
        Listing storage listing = listings[nft][tokenId];
        
        if (!listing.active) revert ListingNotFound(nft, tokenId);
        if (listing.seller != msg.sender) {
            revert NotTokenOwner(msg.sender, listing.seller, tokenId);
        }
        if (newPrice < MINIMUM_PRICE) revert PriceTooLow(newPrice, MINIMUM_PRICE);
        
        uint256 oldPrice = listing.price;
        listing.price = newPrice;
        
        emit PriceUpdated(nft, tokenId, oldPrice, newPrice);
    }
    
    // ============ 수익 출금 ============
    function withdrawProceeds() external {
        uint256 amount = proceeds[msg.sender];
        require(amount > 0, "No proceeds to withdraw");
        
        // 재진입 방지: 상태 먼저 변경
        proceeds[msg.sender] = 0;
        
        (bool success, ) = payable(msg.sender).call{value: amount}("");
        if (!success) {
            proceeds[msg.sender] = amount; // 실패 시 복구
            revert WithdrawFailed(msg.sender, amount);
        }
        
        emit ProceedsWithdrawn(msg.sender, amount);
    }
    
    // ============ 조회 함수 ============
    function getListing(
        address nft,
        uint256 tokenId
    ) external view returns (Listing memory) {
        return listings[nft][tokenId];
    }
    
    function getProceeds(address seller) external view returns (uint256) {
        return proceeds[seller];
    }
}
```

이 컨트랙트는 다음을 보여준다:

1. **커스텀 에러**: 파라미터가 있는 타입 안전한 에러
2. **이벤트 설계**: indexed로 필터링 가능한 구조
3. **try/catch**: 외부 컨트랙트 호출 실패 처리
4. **Checks-Effects-Interactions**: 외부 호출 전 상태 변경

## 정리

- **mapping**은 O(1) 해시 테이블이지만 순회 불가, 기본값 존재
- **이벤트**는 블록체인 로그에 저렴하게 기록하고 외부에서 구독 가능
- **require**는 입력 검증, **assert**는 내부 불변식, **revert**는 복잡한 분기
- **커스텀 에러**는 문자열 에러보다 가스 효율적이고 타입 안전함
- **try/catch**는 외부 컨트랙트 호출에만 사용 가능
