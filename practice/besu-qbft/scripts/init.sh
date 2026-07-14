#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
IMAGE="hyperledger/besu:26.7.0"

if ! command -v docker >/dev/null 2>&1; then
  echo "오류: Docker를 설치하고 실행한 뒤 다시 시도하세요." >&2
  exit 1
fi

if [[ -f "$ROOT_DIR/generated/genesis.json" && -f "$ROOT_DIR/.env" ]]; then
  node_count=0
  for node_dir in "$ROOT_DIR"/nodes/node-*/data; do
    if [[ -f "$node_dir/key" && -f "$node_dir/key.pub" ]]; then
      node_count=$((node_count + 1))
    fi
  done

  if [[ "$node_count" -eq 4 ]]; then
    echo "이미 초기화되었습니다. 새 키가 필요하면 ./scripts/reset.sh를 먼저 실행하세요."
    exit 0
  fi
fi

rm -rf "$ROOT_DIR/generated" "$ROOT_DIR/nodes" "$ROOT_DIR/.env"
mkdir -p "$ROOT_DIR/generated" "$ROOT_DIR/nodes"

docker run --rm \
  --user "$(id -u):$(id -g)" \
  --mount "type=bind,source=$ROOT_DIR,target=/work" \
  "$IMAGE" \
  operator generate-blockchain-config \
  --config-file=/work/qbftConfigFile.json \
  --to=/work/generated \
  --private-key-file-name=key

node_number=1
for key_dir in "$ROOT_DIR"/generated/keys/*; do
  [[ -d "$key_dir" ]] || continue

  data_dir="$ROOT_DIR/nodes/node-$node_number/data"
  mkdir -p "$data_dir"
  cp "$key_dir/key" "$key_dir/key.pub" "$data_dir/"
  node_number=$((node_number + 1))
done

if [[ "$node_number" -ne 5 ]]; then
  echo "오류: 검증자 키 4쌍을 만들지 못했습니다." >&2
  exit 1
fi

public_key=$(tr -d '\r\n' < "$ROOT_DIR/nodes/node-1/data/key.pub")
public_key=${public_key#0x}
printf 'BOOTNODE_ENODE=enode://%s@172.31.239.11:30303\n' "$public_key" > "$ROOT_DIR/.env"

echo "초기화 완료: genesis.json과 검증자 키 4쌍을 만들었습니다."
echo "다음 명령: docker compose up -d"
