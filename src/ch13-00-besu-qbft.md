# Besu로 4노드 프라이빗 Ethereum 배포

지금까지는 체인과 컨트랙트의 실행 규칙을 코드로 보았다. 이번 장에서는 서로 독립된 Besu 프로세스 네 개를 연결하고 QBFT 합의로 블록을 만든다. 한 노드를 끄면 계속 진행되고 두 노드를 끄면 멈추는지도 확인한다.

이 실습은 실제 P2P 통신과 합의를 사용하지만 모든 컨테이너가 한 컴퓨터에서 실행된다. 운영 환경의 다중 호스트·다중 가용 영역 배포와는 다르다.

```text
                   Docker network
        ┌─────────────────────────────────┐
RPC     │                                 │
:8545 ─▶│ node-1 ◀─────── bootnode        │
:8546 ─▶│ node-2 ─┐                       │
:8547 ─▶│ node-3 ─┼─ QBFT validator 4개   │
:8548 ─▶│ node-4 ─┘                       │
        └─────────────────────────────────┘
```

## 준비물

macOS나 Linux에서 Docker와 `curl`을 사용한다. 저장소 루트에서 실습 디렉터리로 이동한다.

```bash
cd practice/besu-qbft
docker compose version
curl --version
```

예제는 검증한 버전인 `hyperledger/besu:26.7.0` 이미지를 고정해서 사용한다. 학습 도중 버전을 임의로 바꾸면 genesis 설정이나 명령 옵션도 함께 검토해야 한다.

## 배포 파일 읽기

실습에는 네 종류의 설정이 있다.

```text
practice/besu-qbft/
├── compose.yaml           # Besu 컨테이너 4개와 내부 네트워크
├── config.toml            # 모든 노드가 공유하는 실행 옵션
├── qbftConfigFile.json    # genesis와 검증자 키 생성 규칙
└── scripts/
    ├── init.sh            # genesis·키·bootnode 주소 생성
    ├── status.sh          # JSON-RPC로 상태 조회
    └── reset.sh           # 컨테이너와 생성 데이터를 삭제
```

`qbftConfigFile.json`의 핵심은 다음 값이다.

- `chainId`: 이 사설망의 체인 ID `1337`
- `blockperiodseconds`: 목표 블록 간격 2초
- `count`: 최초 검증자 수 4
- `zeroBaseFee`와 `min-gas-price`: 로컬 실습 트랜잭션의 수수료를 0으로 허용

`init.sh`는 Besu의 `operator generate-blockchain-config` 명령으로 `genesis.json`과 키 네 쌍을 만든다. genesis의 `extraData`에는 최초 검증자 네 명이 들어간다. 생성된 개인 키와 체인 데이터는 `.gitignore` 대상이며 저장소에 커밋하지 않는다.

각 노드는 같은 genesis를 읽지만 데이터 디렉터리와 검증자 키는 따로 가진다. 2~4번 노드는 고정된 Docker 내부 IP를 포함한 1번 노드의 enode 주소를 bootnode로 사용해 최초 피어를 찾는다. JSON-RPC 포트는 호스트의 `127.0.0.1`에만 공개한다.

## 네트워크 시작

먼저 genesis와 검증자 키를 만든다.

```bash
./scripts/init.sh
```

네 노드를 백그라운드로 시작한다.

```bash
docker compose up -d
docker compose ps
```

최초 실행은 이미지를 내려받기 때문에 시간이 더 걸릴 수 있다. 잠시 뒤 상태 스크립트를 실행한다.

```bash
./scripts/status.sh
```

출력에는 세 가지가 보여야 한다.

1. `eth_blockNumber`의 16진수 블록 높이가 시간이 지나며 증가한다.
2. `net_peerCount`가 `0x3`이다. 자기 자신을 제외한 세 노드와 연결됐다는 뜻이다.
3. `qbft_getValidatorsByBlockNumber`의 결과에 주소 네 개가 있다.

블록 높이만 다시 확인하려면 JSON-RPC를 직접 호출한다.

```bash
curl --silent http://127.0.0.1:8545 \
  --header 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

노드가 시작되지 않거나 블록이 늘지 않으면 먼저 로그를 본다.

```bash
docker compose logs --tail=100 node-1
```

검증자 키가 `nodes/node-N/data/key`에 없으면 Besu가 새 키를 생성한다. 이 키는 genesis에 등록된 검증자와 다르므로 합의에 참여하지 못한다. 이런 상태라면 `./scripts/reset.sh` 후 다시 초기화한다.

## 합의의 장애 허용 한계 확인

QBFT는 블록을 확정할 때 검증자 3분의 2 이상의 서명이 필요하다. 네 검증자 중 한 대만 멈추면 세 대가 남으므로 블록을 계속 만든다.

```bash
docker compose stop node-4
./scripts/status.sh
sleep 6
./scripts/status.sh
```

두 출력의 블록 높이가 증가해야 한다. 이제 검증자 한 대를 더 끈다.

```bash
docker compose stop node-3
./scripts/status.sh
sleep 6
./scripts/status.sh
```

두 대만 남으면 3분의 2 서명 조건을 만족하지 못한다. 이때 노드 프로세스와 RPC는 살아 있지만 블록 높이는 증가하지 않는다. 서버가 실행 중이라는 사실과 체인이 진행된다는 사실은 같지 않다.

노드를 복구한다. 합의가 멈춘 동안 QBFT의 라운드 제한 시간은 실패할 때마다 두 배로 늘어난다. 일부 노드만 시작하면 각 노드의 라운드가 어긋나 복구가 늦어진다. 네 노드를 함께 재시작해 타이머를 초기화한다. 체인 데이터는 보존된다.

```bash
docker compose start node-3 node-4
docker compose restart
sleep 6
./scripts/status.sh
```

## 컨트랙트 배포 연결하기

8장에서 만든 Foundry 프로젝트를 이 네트워크에 배포해 보자. 이 실습망은 London 규칙에 맞춰 만들었으므로 `foundry.toml`에도 같은 EVM 버전을 지정한다.

```toml
[profile.default]
evm_version = "london"
```

학습용 새 계정을 만들고 개인 키는 셸 변수로만 전달한다.

```bash
cast wallet new
export PRIVATE_KEY='<방금 만든 학습용 개인 키>'
forge create src/Counter.sol:Counter \
  --rpc-url http://127.0.0.1:8545 \
  --private-key "$PRIVATE_KEY" \
  --legacy \
  --gas-price 0 \
  --broadcast
unset PRIVATE_KEY
```

수수료가 0인 개발망에서는 `value`가 0인 트랜잭션을 보낼 때 계정 잔액이 필요 없다. 이 설정은 실습 편의를 위한 것이며 실제 운영망의 수수료 정책으로 그대로 가져가면 안 된다.

## 종료와 초기화

컨테이너만 내리고 체인 데이터는 보존하려면 다음 명령을 쓴다.

```bash
docker compose down
```

다음 시작에서 같은 체인을 이어 간다.

```bash
docker compose up -d
```

genesis, 검증자 키, 블록 데이터까지 모두 지우려면 초기화 스크립트를 실행한다.

```bash
./scripts/reset.sh
```

`reset.sh`를 실행하면 이전 체인과 계정 상태는 사라진다.

## 운영 환경으로 옮기기 전에

이 구성은 학습용이다. `host-allowlist`와 CORS를 넓게 열었고 키를 로컬 파일에 저장한다. Docker 네트워크로 격리됐다는 사실도 접근 제어와 방화벽을 대신하지 않는다.

운영 환경에서는 최소한 다음 항목을 별도로 설계한다.

- 검증자를 서로 다른 호스트와 장애 영역에 배치
- P2P·RPC 방화벽, TLS, 인증, API별 접근 통제
- 외부 서명기나 HSM을 이용한 검증자 키 관리와 백업
- 고정된 피어 주소, 노드·합의·디스크 모니터링, 장애 복구 절차
- permissioning, 검증자 추가·제거, 버전 업그레이드 절차

핵심은 컨테이너 수가 아니라 독립된 장애 영역과 키 관리다. 이 장을 마쳤다면 “노드 네 개를 실행했다”를 넘어 어떤 조건에서 합의가 계속되고 멈추는지 설명해야 한다.

## 확인 문제

1. 네 노드가 같은 genesis를 사용해야 하는 이유는 무엇인가?
2. 한 노드를 껐을 때 `peerCount`와 블록 높이는 각각 어떻게 바뀌는가?
3. 두 노드가 남아 RPC 요청에 답해도 체인이 멈추는 이유는 무엇인가?
4. 이 Docker Compose 구성이 운영용 고가용성 배포가 아닌 이유는 무엇인가?
