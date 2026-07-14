#!/usr/bin/env bash
set -euo pipefail

RPC_URL=${RPC_URL:-http://127.0.0.1:8545}

if ! command -v curl >/dev/null 2>&1; then
  echo "오류: curl이 필요합니다." >&2
  exit 1
fi

rpc() {
  local method=$1
  local params=$2

  curl --fail --silent --show-error \
    --max-time 5 \
    --header "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" \
    "$RPC_URL"
}

echo "RPC: $RPC_URL"
echo "블록 높이:"
rpc "eth_blockNumber" "[]"
echo
echo "연결된 피어 수:"
rpc "net_peerCount" "[]"
echo
echo "현재 검증자 목록:"
rpc "qbft_getValidatorsByBlockNumber" '["latest"]'
echo
