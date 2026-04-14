# 20-1: Hyperledger Besu - 엔터프라이즈 이더리움 클라이언트

## Hyperledger Besu란

Hyperledger Besu는 Linux Foundation의 Hyperledger 프로젝트 중 하나로, Java로 작성된 엔터프라이즈급 Ethereum 클라이언트다. 2019년에 PegaSys(ConsenSys의 자회사)가 개발하여 Hyperledger에 기증했다.

"엔터프라이즈급"이라는 말의 의미:
- 이더리움 메인넷과 완전히 호환 (같은 EVM, 같은 JSON-RPC API)
- 기업용 추가 기능: 프라이빗 트랜잭션, 권한 관리, 모니터링
- 안정성과 지원 체계 (엔터프라이즈 버전 존재)

Besu는 Geth(Go Ethereum)와 동일한 이더리움 프로토콜을 구현하지만, 기업 환경에 필요한 기능을 추가로 제공한다.

## 주요 특징

### 1. 퍼블릭/프라이빗 네트워크 모두 지원

Besu 노드 하나로 이더리움 메인넷에 연결하거나, 완전히 독립된 프라이빗 네트워크를 구성할 수 있다.

```bash
# 이더리움 메인넷 클라이언트로 실행
besu --network=mainnet --sync-mode=SNAP

# 완전히 새로운 프라이빗 네트워크 시작
besu --genesis-file=genesis.json --network-id=1337
```

같은 소프트웨어, 같은 Solidity 코드를 두 환경 모두에서 사용할 수 있다는 것이 핵심이다.

### 2. 합의 알고리즘

Besu는 여러 합의 알고리즘을 지원한다:

#### IBFT 2.0 (Istanbul Byzantine Fault Tolerant)
- 프라이빗/컨소시엄 체인에서 가장 많이 사용
- 검증자(validator) 집합이 블록을 생성하고 합의
- 즉시 확정성(finality): 한 번 확정된 블록은 번복 불가
- BFT(Byzantine Fault Tolerant): 검증자 1/3 미만이 악의적으로 행동해도 안전

#### QBFT (Quorum Byzantine Fault Tolerant)
- IBFT 2.0의 개선 버전
- EIP 표준 준수, 더 나은 성능
- 새 프로젝트에 권장됨

#### Clique (Proof of Authority)
- 간단한 PoA 구현
- 개발/테스트 환경에 적합
- 블록 생성자가 순번대로 돌아가며 서명

#### Ethash (Proof of Work)
- 이더리움 원래 합의 방식 (이제 Merge로 deprecated)
- 테스트 목적으로만 사용

platform은 **IBFT 2.0**을 사용한다.

### 3. 프라이빗 트랜잭션 (Tessera 연동)

Besu는 Tessera라는 별도 프라이버시 관리자와 연동하여 프라이빗 트랜잭션을 지원한다.

```text
일반 트랜잭션: 모든 노드가 데이터를 볼 수 있음
프라이빗 트랜잭션: 지정된 참여자만 payload를 복호화 가능
```

예시: A 농장의 생산 원가 데이터를 B 유통업자와만 공유하고, C 소매업자는 볼 수 없게 하는 것이 가능하다.

platform은 현재 Tessera를 사용하지 않지만, 확장 시 도입 가능한 옵션이다.

### 4. 권한 관리

Besu는 온체인/오프체인 권한 관리를 지원한다:

```json
{
  "permissioning": {
    "nodes-allowlist": [
      "enode://abc123@192.168.1.1:30303",
      "enode://def456@192.168.1.2:30303"
    ],
    "accounts-allowlist": [
      "0x627306090abaB3A6e1400e9345bC60c78a8BEf57"
    ]
  }
}
```

- **노드 권한**: 특정 노드만 네트워크 참여 가능
- **계정 권한**: 특정 주소만 트랜잭션 제출 가능

platform에서는 계정 권한 관리로 승인된 서비스 계정만 컨트랙트에 쓸 수 있도록 제한한다.

## platform이 Besu를 사용하는 이유

### 식품 공급망 데이터의 기밀성

농업인의 재배 방법, 수확량, 약품 사용 이력 등은 비즈니스 기밀이다. 이더리움 메인넷에 올리면 경쟁사도 볼 수 있다.

Besu 프라이빗 체인에서는:
- 네트워크 참여자를 통제
- 필요시 Tessera로 특정 참여자만 데이터 열람 가능
- 규제 기관에게는 감사 접근권 부여 가능

### EVM 호환 - 같은 Solidity 컨트랙트 사용

가장 큰 장점 중 하나다. Besu는 완전한 EVM 구현을 포함하므로, 이더리움 메인넷용으로 작성된 Solidity 코드를 그대로 배포할 수 있다.

```text
이더리움 메인넷:  TraceRecord.sol 배포 → 동작
Besu 프라이빗:   TraceRecord.sol 배포 → 동일하게 동작
```

Foundry로 컴파일하고, Alloy로 상호작용하는 코드도 변경이 거의 없다. RPC 엔드포인트 URL만 바꾸면 된다.

### 가스 비용 0으로 설정

`genesis.json`에서 가스 가격을 0으로 설정할 수 있다:

```json
{
  "config": {
    "chainId": 1337,
    "berlinBlock": 0,
    "ibft2": {
      "blockperiodseconds": 2,
      "epochlength": 30000,
      "requesttimeoutseconds": 4
    }
  },
  "gasLimit": "0x1fffffffffffff",
  "difficulty": "0x1",
  "alloc": {
    "0xfe3b557e8fb62b89f4916b721be55ceb828dbd73": {
      "privateKey": "...",
      "balance": "0xad78ebc5ac6200000"
    }
  }
}
```

그리고 `min-gas-price=0` 옵션으로 가스 가격 0인 트랜잭션을 허용한다:

```bash
besu \
  --genesis-file=genesis.json \
  --min-gas-price=0 \
  --rpc-http-enabled \
  --rpc-http-cors-origins="all"
```

이렇게 하면 플랫폼 운영사가 가스 비용을 부담하지 않아도 되고, 사용자에게도 추가 비용이 없다.

## Docker로 로컬 Besu 네트워크 실행하기

개발 환경에서 Besu IBFT 2.0 네트워크를 Docker로 구성하는 방법 개요:

### 디렉토리 구조

```text
besu-network/
├── docker-compose.yml
├── genesis.json
├── node1/
│   ├── data/
│   └── key              # 노드 개인키
├── node2/
│   ├── data/
│   └── key
├── node3/
│   ├── data/
│   └── key
└── node4/
    ├── data/
    └── key
```

### genesis.json (IBFT 2.0)

```json
{
  "config": {
    "chainId": 1337,
    "berlinBlock": 0,
    "londonBlock": 0,
    "ibft2": {
      "blockperiodseconds": 2,
      "epochlength": 30000,
      "requesttimeoutseconds": 4
    }
  },
  "nonce": "0x0",
  "timestamp": "0x58ee40ba",
  "extraData": "0xf87aa00000000000000000000000000000000000000000000000000000000000000000f854940000000000000000000000000000000000000001940000000000000000000000000000000000000002940000000000000000000000000000000000000003940000000000000000000000000000000000000004c080a00000000000000000000000000000000000000000000000000000000000000000880000000000000000",
  "gasLimit": "0x1fffffffffffff",
  "difficulty": "0x1",
  "mixHash": "0x63746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365",
  "coinbase": "0x0000000000000000000000000000000000000000",
  "alloc": {
    "0xfe3b557e8fb62b89f4916b721be55ceb828dbd73": {
      "balance": "0xad78ebc5ac6200000"
    },
    "0x627306090abaB3A6e1400e9345bC60c78a8BEf57": {
      "balance": "0xad78ebc5ac6200000"
    }
  },
  "number": "0x0",
  "gasUsed": "0x0",
  "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
}
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  bootnode:
    image: hyperledger/besu:latest
    container_name: besu-bootnode
    command: >
      --genesis-file=/config/genesis.json
      --node-private-key-file=/config/node1/key
      --rpc-http-enabled
      --rpc-http-api=ETH,NET,IBFT,WEB3
      --host-allowlist="*"
      --rpc-http-cors-origins="all"
      --min-gas-price=0
      --p2p-port=30303
      --rpc-http-port=8545
    volumes:
      - ./genesis.json:/config/genesis.json
      - ./node1:/config/node1
      - ./node1/data:/var/lib/besu
    ports:
      - "8545:8545"
      - "30303:30303"
    networks:
      - besu-network

  validator1:
    image: hyperledger/besu:latest
    container_name: besu-validator1
    depends_on:
      - bootnode
    command: >
      --genesis-file=/config/genesis.json
      --node-private-key-file=/config/node2/key
      --bootnodes=enode://<bootnode-enode>@bootnode:30303
      --min-gas-price=0
      --p2p-port=30303
      --rpc-http-enabled
      --rpc-http-port=8546
    volumes:
      - ./genesis.json:/config/genesis.json
      - ./node2:/config/node2
      - ./node2/data:/var/lib/besu
    ports:
      - "8546:8546"
    networks:
      - besu-network

  validator2:
    image: hyperledger/besu:latest
    container_name: besu-validator2
    depends_on:
      - bootnode
    command: >
      --genesis-file=/config/genesis.json
      --node-private-key-file=/config/node3/key
      --bootnodes=enode://<bootnode-enode>@bootnode:30303
      --min-gas-price=0
      --p2p-port=30303
    volumes:
      - ./genesis.json:/config/genesis.json
      - ./node3:/config/node3
      - ./node3/data:/var/lib/besu
    networks:
      - besu-network

  validator3:
    image: hyperledger/besu:latest
    container_name: besu-validator3
    depends_on:
      - bootnode
    command: >
      --genesis-file=/config/genesis.json
      --node-private-key-file=/config/node4/key
      --bootnodes=enode://<bootnode-enode>@bootnode:30303
      --min-gas-price=0
      --p2p-port=30303
    volumes:
      - ./genesis.json:/config/genesis.json
      - ./node4:/config/node4
      - ./node4/data:/var/lib/besu
    networks:
      - besu-network

networks:
  besu-network:
    driver: bridge
```

### 네트워크 시작

```bash
# 검증자 키 생성
besu operator generate-blockchain-config \
  --config-file=ibftConfigFile.json \
  --to=networkFiles \
  --private-key-file-name=key

# 네트워크 시작
docker-compose up -d

# 상태 확인
curl -X POST \
  http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"ibft_getValidatorsByBlockNumber","params":["latest"],"id":1}'
```

## IBFT 2.0 합의 상세

IBFT 2.0(Istanbul Byzantine Fault Tolerant 2.0)은 프라이빗 Ethereum 네트워크에서 가장 많이 사용되는 합의 알고리즘이다.

### 검증자(Validator)란

검증자는 블록을 제안하고 서명할 권한을 가진 노드다. IBFT 2.0에서는 검증자 집합이 미리 정의되어 있으며, 동적으로 추가/제거가 가능하다.

```text
검증자 집합 = {V1, V2, V3, V4}
일반 노드 = 블록체인을 동기화하지만 블록 생성에 참여 안 함
```

### 블록 생성 라운드

각 블록 생성은 "라운드"를 통해 이루어진다:

```text
라운드 1:
  1. 제안자(Proposer) 선택: 블록 높이에 따라 순번 결정
     (높이 % 검증자 수 = 제안자 인덱스)
  
  2. 블록 제안: 제안자가 새 블록을 네트워크에 브로드캐스트
  
  3. PREPARE 단계: 다른 검증자들이 제안 받고 PREPARE 메시지 전송
  
  4. COMMIT 단계: 2/3 초과 PREPARE 수신 시 COMMIT 메시지 전송
  
  5. 블록 확정: 2/3 초과 COMMIT 수신 시 블록이 체인에 추가됨
```

### 2/3 이상 동의 필요

BFT 알고리즘의 핵심이다:

```text
검증자 수: N
허용 가능한 결함 노드: f < N/3

필요한 동의: > 2N/3

예시 (검증자 4개):
  4 * 2/3 = 2.67 → 3개 이상 동의 필요
  즉, 1개 노드가 고장나거나 악의적이어도 합의 가능
  
예시 (검증자 7개):
  7 * 2/3 = 4.67 → 5개 이상 동의 필요
  즉, 2개 노드까지 허용
```

### 즉시 확정성(Immediate Finality)

IBFT 2.0의 중요한 특성이다. 블록이 한 번 체인에 추가되면 절대로 번복되지 않는다.

이더리움 PoS에서는 "확정(finality)"에 2-3 에포크(~13분)가 필요하다. 그 전까지는 이론적으로 체인 재구성이 가능하다.

IBFT 2.0에서는:
```text
블록 추가 = 즉시 확정
영수증을 받으면 = 영원히 확정
```

platform 같은 비즈니스 애플리케이션에서 이것은 매우 중요하다. "이 트랜잭션이 정말로 확정됐나요?"를 걱정할 필요가 없다.

### 타임아웃과 라운드 변경

제안자가 응답하지 않거나 잘못된 블록을 제안하면, 타임아웃 후 다음 라운드로 넘어간다:

```json
"ibft2": {
  "blockperiodseconds": 2,        // 블록 생성 주기 (초)
  "epochlength": 30000,           // 검증자 재선출 주기 (블록 수)
  "requesttimeoutseconds": 4      // 라운드 타임아웃 (초)
}
```

- `blockperiodseconds: 2`: 2초마다 새 블록 시도
- `requesttimeoutseconds: 4`: 4초 안에 합의 실패 시 다음 라운드

### 검증자 추가/제거

IBFT 2.0은 온체인 투표로 검증자를 동적으로 관리한다:

```bash
# 새 검증자 추가 제안 (기존 검증자가 투표)
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "ibft_proposeValidatorVote",
    "params": ["0xNew_Validator_Address", true],
    "id": 1
  }'

# 현재 검증자 목록 조회
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "ibft_getValidatorsByBlockNumber",
    "params": ["latest"],
    "id": 1
  }'
```

과반수(>50%) 검증자가 동의하면 해당 주소가 검증자로 추가된다.

## Besu 모니터링

Besu는 Prometheus 메트릭을 기본 지원한다:

```bash
besu \
  --metrics-enabled \
  --metrics-host=0.0.0.0 \
  --metrics-port=9545
```

Grafana + Prometheus로 대시보드를 구성할 수 있다. 주요 메트릭:
- 블록 생성 시간
- 피어 연결 수
- 트랜잭션 풀 크기
- 가스 사용량

## Alloy에서 Besu 연결

Besu 프라이빗 체인에 연결하는 것은 이더리움 메인넷 연결과 코드상 거의 동일하다:

```rust,ignore
use alloy::providers::{Provider, ProviderBuilder};

// 이더리움 메인넷 (코드)
let mainnet = ProviderBuilder::new()
    .on_http("https://mainnet.infura.io/v3/KEY".parse()?);

// Besu 프라이빗 체인 (코드) - URL만 다름
let besu = ProviderBuilder::new()
    .on_http("http://besu-node:8545".parse()?);

// 체인 ID 확인 (Besu는 genesis.json에서 설정한 ID)
let chain_id = besu.get_chain_id().await?;
println!("Besu 체인 ID: {}", chain_id); // 1337

// 검증자 목록 조회 (Besu 전용 API)
// raw JSON-RPC 호출
let validators: serde_json::Value = besu
    .raw_request(
        "ibft_getValidatorsByBlockNumber".into(),
        serde_json::json!(["latest"]),
    )
    .await?;
```

## 요약

Hyperledger Besu:
- Java 기반 엔터프라이즈 Ethereum 클라이언트
- **EVM 호환**: 같은 Solidity, 같은 Alloy 코드 사용
- **IBFT 2.0**: 빠른 BFT 합의, 즉시 확정성
- **가스비 0**: `--min-gas-price=0`으로 무료 트랜잭션
- **권한 관리**: 노드/계정 화이트리스트
- **프라이빗 트랜잭션**: Tessera 연동으로 선택적 공개

platform이 Besu를 선택한 이유: EVM 호환성 + 빠른 확정 + 가스비 0 + 데이터 프라이버시

다음 장에서는 퍼블릭 vs 프라이빗 체인을 더 깊이 비교하고, "왜 DB 대신 블록체인?"이라는 질문에 답한다.
