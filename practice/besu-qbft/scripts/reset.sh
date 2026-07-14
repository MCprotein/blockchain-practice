#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export BOOTNODE_ENODE=${BOOTNODE_ENODE:-enode://placeholder@127.0.0.1:30303}

cd "$ROOT_DIR"
docker compose down --volumes --remove-orphans
rm -rf generated nodes .env

echo "체인 데이터와 로컬 검증자 키를 삭제했습니다."
