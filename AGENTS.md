# Repository instructions

This repository contains a Korean mdBook titled “Rust로 이해하는 블록체인과 스마트 컨트랙트”. The target reader is new to Rust and blockchain; do not assume experience with a particular programming language.

## Content contract

- Keep chapter prose in Korean. Code, identifiers, commands, and original technical terms may remain English.
- Explain Rust concepts directly from their ownership, type, and execution rules. Cross-language comparisons are optional and must not be required to understand the text.
- Preserve the learning path: Rust essentials → blockchain model and mini-chain → Ethereum/Solidity → Solana/Anchor → Besu/QBFT operations.
- Prefer a precise explanation, one useful example, and a failure case over encyclopedic coverage.
- Do not reintroduce volatile ecosystem rankings, vendor-specific platform analysis, or unrelated infrastructure chapters into the core path.
- Distinguish hashes, encryption, signatures, consensus, and immutability accurately.
- State tool and library versions only when the claim is checked against current official documentation.

## Navigation and files

`src/SUMMARY.md` is the single source of truth for navigation. Every published chapter must be listed there. Chapter files use `chXX-00-description.md`; appendices use `appendix-X-description.md`.

The runnable Rust exercise is the Cargo workspace crate under `practice/mini-chain`. Keep it synchronized with `src/ch05-00-mini-chain.md`. The runnable private-network exercise is under `practice/besu-qbft`; keep it synchronized with `src/ch13-00-besu-qbft.md`.

## Code fences

- `rust`: standalone code that `mdbook test` can compile and run
- `rust,compile_fail`: intentionally invalid Rust
- `rust,ignore`: external dependencies, multi-file context, or reading-only Rust
- `solidity`, `bash`, `json`, `toml`, `yaml`: examples for their own toolchains or configuration formats
- `text`: output, directory trees, and diagrams

Never leave an untyped code fence. Do not claim ignored or non-Rust snippets were executed by `mdbook test`.

## Validation

Run from the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash -n practice/besu-qbft/scripts/*.sh
BOOTNODE_ENODE=enode://placeholder@127.0.0.1:30303 docker compose -f practice/besu-qbft/compose.yaml config --quiet
mdbook build
mdbook test
git diff --check
```

Completion requires all eight checks to pass. When Besu practice files change and Docker is available, also start the four nodes and verify block production, peer count, validator count, and the one-node/two-node failure cases. If Foundry or Anchor is unavailable, report Solidity and Anchor snippets as documentation-reviewed but not locally executed.

## Editing rules

- Keep diffs small and remove stale files when restructuring the table of contents.
- Do not commit `book/` or `target/`.
- Prefer official Rust, Ethereum, Solidity, Foundry, OpenZeppelin, Solana, Anchor, and Besu documentation for technical claims.
- Treat smart-contract examples as security-sensitive: show checked arithmetic, explicit authority checks, external-call boundaries, and failure tests.
