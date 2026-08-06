# ZeroClaw Solana PayGuard 🦞⚡
> **Solana POS Terminal, BRL/USDC Merchant Reconciler & Prompt-Injection-Proof Treasury Guardian for ZeroClaw AI Agent Runtime**

[![Bounty](https://img.shields.io/badge/Superteam%20Brasil-Solana%20Bounty-green.svg)](https://earn.superteam.fun)
[![ZeroClaw](https://img.shields.io/badge/ZeroClaw-v0.1.0-blue.svg)](https://github.com/zeroclaw-labs/zeroclaw)
[![Rust](https://img.shields.io/badge/Rust-1.93+-orange.svg)](https://www.rust-lang.org/)
[![WASM](https://img.shields.io/badge/WASM-wasip2-purple.svg)](https://bytecodealliance.org/)
[![License](https://img.shields.io/badge/License-MIT-lightgrey.svg)](LICENSE)

---

## 🌟 Overview

**ZeroClaw Solana PayGuard** is a complete, self-hosted, privacy-preserving AI agent use-case built on **ZeroClaw** (Rust AI agent runtime) and **Solana**.

It solves a real-world merchant problem: enabling family shops, e-commerce stores, and cross-border businesses (with focus on Brazil PIX-to-USDC merchant flows) to accept instant Solana Pay payments on WhatsApp & Telegram, automatically reconcile invoices on-chain via ZeroClaw SOP crons, and guard merchant treasuries against prompt-injection attacks using hard-coded WASM firewalls and Human Approval Checkpoints.

---

## ✨ Features

- **🛍️ Solana Pay & Blink POS Terminal (Tier 1)**: Instant invoice generation with dynamic 32-byte Ed25519 reference keys and BRL/USDC rate calculation via Jupiter REST API & Switchboard Crossbar REST API.
- **🇧🇷 Brazil PIX Reconciliation**: Reconciles local BRL payment requests with instant Solana USDC settlement.
- **🦀 Custom WASM Plugin (`zeroclaw-solana-guard`) [Tier 3]**: Pure Rust plugin compiled to `wasm32-wasip2` using ZeroClaw `wit/v0` interface. Includes:
  - `solana_verify_invoice`: On-chain reference signature verification & SPL token decimal parsing.
  - `solana_token2022_inspect`: Token-2022 extension risk inspection (Transfer Fee, Permanent Delegate, Non-transferable).
  - `solana_spend_firewall`: Hard-coded spend cap & recipient address firewall.
  - `solana_verify_durable_nonce`: Blockhash expiration protection for SOP approval delays.
- **⏱️ SOP Engine Integration**:
  - `sop_solana_invoice_watcher`: 30-second cron poller verifying reference key transactions on Solana RPC.
  - `sop_refund_approval_gate`: ZeroClaw Human Approval Checkpoint pausing payouts until store owner confirms.
- **🛡️ 100% Prompt-Injection Proof**: Tested against direct chat overrides, memo injections, and role impersonations (fails closed every time).

---

## 📁 Repository Architecture

```text
.
├── Cargo.toml                             # Root Workspace Manifest
├── .gitignore                             # Git Exclusions File
├── LICENSE                                # MIT License File
├── README.md                              # Quickstart Operator Guide
├── config/
│   ├── config.toml                        # ZeroClaw Agent Configuration
│   └── secrets.example.json               # Encrypted-at-rest Secret Store Template
├── plugins/
│   └── zeroclaw-solana-guard/             # Tier 3 WASM Plugin (Rust + wit/v0)
│       ├── Cargo.toml
│       ├── wit/v0/plugin.wit              # WIT v0 Interface Specification
│       ├── src/                           # Plugin Source Code
│       │   ├── lib.rs                     # WIT Bindings & Export Shim
│       │   ├── invoice_verifier.rs        # Signature & Amount Verifier
│       │   ├── token2022_inspector.rs     # Token-2022 Risk Inspector
│       │   ├── spend_firewall.rs          # Spend Cap & Address Firewall
│       │   ├── durable_nonce.rs           # Durable Nonce Verifier
│       │   └── types.rs                   # Data Models & JSON Schemas
│       └── tests/
│           └── plugin_tests.rs            # Host-Run Unit Tests (8/8 Passed)
├── skills/
│   ├── solana_pay_terminal.md             # Solana Pay & QR Generator Skill
│   ├── solana_treasury_guardian.md        # Treasury Guardian & Custody Skill
│   └── pix_usdc_reconciler.md             # Brazil PIX & USDC Reconciler Skill
├── sops/
│   ├── invoice_watcher_sop.json           # Cron Poller SOP
│   ├── refund_approval_sop.json           # Human Approval Checkpoint SOP
│   └── daily_treasury_reconcile_sop.json  # End-of-Day Reconciliation SOP
├── scripts/
│   ├── simulate_merchant_flow.py          # Full POS Lifecycle E2E Devnet Runner
│   ├── test_prompt_injection.py           # Prompt Injection Attack Security Audit
│   └── run_telegram_bot_agent.py          # Live Telegram Bot Channel Daemon
└── docs/
    ├── THREAT_MODEL_AND_CUSTODY.md        # Custody Tier Audit & Threat Matrix
    ├── SUPERTEAM_SHOWCASE_WRITEUP.md      # Submission & Showcase Writeup
    └── internal/                          # Operator Guides & Posts (gitignored)
        ├── SHOWCASE_VIDEO_SCRIPT.md       # 3-Minute Video Showcase Script
        └── BUILD_IN_PUBLIC_X_THREADS.md   # Build-in-Public X Posts (Tiebreaker)
```

---

## 🚀 Quickstart Operator Guide (<15 Minutes)

### 1. Prerequisites
- [Rust Toolchain 1.93+](https://rustup.rs/) with `wasm32-wasip2` target:
  ```bash
  rustup target add wasm32-wasip2
  ```
- Python 3.10+

### 2. Build WASM Plugin & Run Unit Tests
```bash
# Run offline host unit tests (8/8 passed)
cargo test

# Build release WASM component
cargo build --target wasm32-wasip2 --release
```

### 3. Run E2E Simulation & Security Audit Suite
```bash
# Execute full E2E merchant payment workflow simulation
python scripts/simulate_merchant_flow.py

# Execute prompt-injection security audit suite (3/3 attacks blocked)
python scripts/test_prompt_injection.py
```

### 4. Start ZeroClaw Agent Runtime Host
```bash
cargo run --release --features plugins-wasm-cranelift -- --config config/config.toml
```

---

## 🔒 Custody Tier & Security Declaration

- **Custody Tier T1 (Build & Verify / Zero Key Access)**: Default operating mode. The agent holds NO private keys. Customer funds land directly in merchant treasury address (`4zMMC...`).
- **SOP Approval Gate**: Outbound transfers or refund requests pause execution at `sop_refund_approval_gate` until confirmed by store owner.
- **Spend Firewall**: Hard-coded in WASM (`solana_spend_firewall`) enforcing $500.00 USDC daily limit.

---

## 📚 Documentation & Audit Reports

- **Showcase & Operator Writeup**: See [`docs/SUPERTEAM_SHOWCASE_WRITEUP.md`](docs/SUPERTEAM_SHOWCASE_WRITEUP.md)
- **Threat Model & Security Audit**: See [`docs/THREAT_MODEL_AND_CUSTODY.md`](docs/THREAT_MODEL_AND_CUSTODY.md)

---

## 📄 License
Licensed under [MIT License](LICENSE).
