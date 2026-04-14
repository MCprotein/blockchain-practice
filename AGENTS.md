# AGENTS.md — Rust + Blockchain 완전 정복

Agent and Codex instructions for the blockchain-practice repository.

## Project Overview

An **mdBook-based Korean-language guidebook** for learning Rust and blockchain development, targeting experienced Node.js/TypeScript backend developers. The book follows a 4-week curriculum with 5 mini-projects, covering Rust fundamentals, Ethereum/Solidity, Solana/Anchor, and production integration patterns.

The curriculum is intentionally interleaved: readers should learn Rust syntax and blockchain concepts in parallel, not finish all Rust theory before touching blockchain. Preserve this rhythm when reorganizing or adding chapters.

This repository is maintained with both Claude Code and OpenAI Codex. AI-assisted content must still follow the Korean prose, TypeScript comparison, code-fence, build, and test rules below.

- **Live site**: https://mcprotein.github.io/blockchain-practice/
- **Content language**: Korean (한국어)
- **Build tool**: mdBook
- **Deployment**: GitHub Actions → GitHub Pages (auto-deploy on push to `main`)

## Build Commands

```bash
# Build static site (output: book/ directory)
mdbook build

# Local development server with live reload
mdbook serve --open

# Run doctests on Rust code blocks in markdown
mdbook test

# Check for broken links (if mdbook-linkcheck is installed)
mdbook build 2>&1 | grep -i warning
```

All commands must be run from the **project root** (where `book.toml` lives).

## File Structure

```
blockchain-practice/
├── book.toml              # mdBook config — title, language, theme, CSS
├── src/                   # All chapter content (Markdown)
│   ├── SUMMARY.md         # Table of contents — drives site navigation
│   ├── introduction.md    # Preface
│   ├── ch00-XX-*.md       # Orientation: vocabulary and code-reading map
│   ├── ch01-XX-*.md       # Ch 1: Getting started with Rust
│   ├── ch02-XX-*.md       # Ch 2: Ownership and borrowing
│   ├── ch03-XX-*.md       # Ch 3: Structs, enums, pattern matching
│   ├── ch04-XX-*.md       # Ch 4: Error handling
│   ├── ch05-XX-*.md       # Ch 5: Traits and generics
│   ├── ch06-XX-*.md       # Ch 6: Collections and iterators
│   ├── ch07-XX-*.md       # Ch 7: Async programming
│   ├── ch08-XX-*.md       # Mini-project: Blockchain in Rust
│   ├── ch09-XX-*.md       # Ch 9: Blockchain fundamentals
│   ├── ch10-XX-*.md       # Ch 10: Ethereum architecture
│   ├── ch11-XX-*.md       # Ch 11: Solidity basics
│   ├── ch12-XX-*.md       # Ch 12: Foundry toolchain
│   ├── ch13-XX-*.md       # Ch 13: Token standards (ERC-20, ERC-721)
│   ├── ch14-XX-*.md       # Ch 14: Advanced smart contracts
│   ├── ch15-XX-*.md       # Mini-project: Token Vault
│   ├── ch16-XX-*.md       # Ch 16: Solana architecture
│   ├── ch17-XX-*.md       # Ch 17: Anchor framework
│   ├── ch18-XX-*.md       # Mini-project: Solana token program
│   ├── ch19-XX-*.md       # Ch 19: Rust + Ethereum (Alloy)
│   ├── ch20-XX-*.md       # Ch 20: Private chains (Hyperledger Besu)
│   ├── ch21-XX-*.md       # Mini-project: Mini Trace
│   ├── ch22-XX-*.md       # Ch 22: Platform project analysis
│   └── appendix-*.md      # Appendices
├── theme/
│   └── custom.css         # Custom CSS overrides (light theme, font sizes)
├── practice/              # Mini-project Rust source code (workspace crates)
├── Cargo.toml             # Rust workspace root
├── Cargo.lock             # Locked dependencies
└── .github/
    └── workflows/
        └── deploy.yml     # CI/CD: build + deploy to GitHub Pages
```

## Conventions

### Content Language

All prose content must be written in **Korean**. Code identifiers, commands, and technical terms use their original English form. When introducing a technical term, include the English term alongside the Korean: e.g., 소유권(Ownership), 빌림(Borrowing).

### Chapter File Naming

Format: `chXX-YY-description.md`

| Segment | Meaning | Example |
|---------|---------|---------|
| `XX` | Two-digit chapter number | `01`, `09`, `22` |
| `YY` | Two-digit section number (`00` = chapter intro) | `00`, `01`, `03` |
| `description` | Lowercase English, hyphen-separated | `getting-started`, `evm-gas` |

Appendices follow the pattern: `appendix-a-ecosystem.md`, `appendix-b-nodejs-to-rust.md`.

### SUMMARY.md Registration

`src/SUMMARY.md` is the **single source of truth** for site navigation. Every chapter file must be listed here or it will not appear on the built site.

When adding a new chapter:
1. Create the file in `src/` using the naming convention above.
2. Add it to `src/SUMMARY.md` in the correct position.
3. Run `mdbook build` to verify no errors.

SUMMARY.md entry format:
```markdown
- [Chapter Title in Korean](./chXX-YY-description.md)
  - [Section Title in Korean](./chXX-YY-description.md)
```

### TypeScript Comparison Requirement

The target reader is a **4-year Node.js/NestJS backend developer** who is new to Rust and blockchain. Every chapter introducing a Rust concept must include a TypeScript/Node.js comparison showing the equivalent pattern.

Required comparisons by topic:
- Type system: TypeScript interfaces/types ↔ Rust structs/enums/traits
- Error handling: try/catch + Promise rejection ↔ Result<T, E> + ? operator
- Async: async/await + Event Loop ↔ async/await + Tokio runtime
- Modules: npm packages ↔ Cargo crates
- OOP patterns: class + interface ↔ struct + impl + trait
- Dependency injection (NestJS) ↔ Rust module system

Comparison code block pattern:
```markdown
```typescript
// TypeScript: ...
```

```rust,ignore
// Rust: ...
```
```

### Code Examples

- Code examples should be runnable when they are presented as standalone code. When an example is intentionally partial, compile-failing, output-only, or dependent on project setup, mark it explicitly with the correct code-fence tag.
- Standalone Rust examples should compile with `rustc`, `mdbook test`, or as part of a Cargo project.
- Use `rust,ignore` for Rust snippets that require external crates, omitted module context, project setup, Anchor/Alloy/Tokio scaffolding, or are meant only for reading.
- Use `rust,compile_fail` for examples whose purpose is to demonstrate compiler errors.
- Use `text` for command output, directory trees, diagrams, and compiler output. Do not leave code fences untyped.
- Solidity examples should target a recent Solidity version (^0.8.x).
- Anchor/Solana examples should target the Anchor version used in `practice/`.
- Do not include placeholder comments like `// TODO` or `// ...` in final examples.

### CSS and Theming

- Theme is set to `light` in `book.toml` (`default-theme = "light"`).
- Custom styles live exclusively in `theme/custom.css`.
- Do not modify mdBook default theme files unless absolutely necessary.

## Testing

### Primary test: `mdbook build`

```bash
mdbook build
```

A passing build produces **0 errors**. The goal is also **0 warnings** — warnings about missing files or broken internal links indicate a SUMMARY.md registration problem.

### Doctest: `mdbook test`

```bash
mdbook test
```

Rust code blocks annotated with ` ```rust ` are executed as doctests. Blocks that should not be tested must be annotated with ` ```rust,ignore `, ` ```rust,no_run `, or ` ```rust,compile_fail ` as appropriate. All non-code outputs and diagrams must use ` ```text `.

### Manual verification

After `mdbook serve --open`, verify:
- Navigation sidebar renders all chapters
- Internal links between chapters resolve correctly
- Code blocks display with correct syntax highlighting
- Korean characters render without mojibake

## Key Constraints

1. **Korean content only** — do not write chapter prose in English.
2. **TypeScript comparisons required** — every Rust concept chapter needs a TS equivalent example.
3. **SUMMARY.md must stay in sync** — adding a file without updating SUMMARY.md breaks navigation.
4. **No commits of `book/`** — the build output directory is gitignored.
5. **No commits of `.omc/`** — AI agent state files are gitignored.
6. **Code-fence tags must be accurate** — use `text`, `bash`, `typescript`, `solidity`, `rust,ignore`, `rust,no_run`, or `rust,compile_fail` instead of untyped fences.
7. **`mdbook build` must produce 0 errors** — treat any build error as a blocking issue.
8. **`mdbook test` must pass** — doctest failures indicate bad Rust fence classification or broken standalone examples.

## CI/CD

Workflow file: `.github/workflows/deploy.yml`

- **Trigger**: push to `main` branch, or manual `workflow_dispatch`
- **Runner**: `ubuntu-latest`
- **Steps**:
  1. `actions/checkout@v4`
  2. Install mdBook via `peaceiris/actions-mdbook@v2` (latest version)
  3. `mdbook build`
  4. Upload artifact with `actions/upload-pages-artifact@v3` (path: `./book`)
  5. Deploy with `actions/deploy-pages@v4`
- **Permissions**: `contents: read`, `pages: write`, `id-token: write`
- **Concurrency**: `group: pages`, `cancel-in-progress: false`

GitHub Pages must be configured to **Source: GitHub Actions** in repository Settings > Pages.

## practice/ Directory

The `practice/` directory contains runnable Rust code for the mini-projects. It is a Cargo workspace — each mini-project is a separate crate. When writing or modifying practice code:

- Register new crates in the root `Cargo.toml` workspace members list.
- Keep practice code in sync with the corresponding guidebook chapter.
- Practice code is separate from `mdbook test` — run `cargo test` inside `practice/` to verify it.
