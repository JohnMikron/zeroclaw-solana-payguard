# Submission: ZeroClaw Solana PayGuard (POS Terminal & Treasury Guardian) 🦞⚡

**Bounty**: Build Solana-native plugins for ZeroClaw by Superteam Brasil  
**Developer / Team**: JohnMikron (`@JohnMikron`)  
**Repository**: `https://github.com/JohnMikron/zeroclaw-solana-payguard`  
**Video Demo (3 min Google Drive Link)**: `https://drive.google.com/file/d/197hNWaAMtHKROLs50dmPrirfZvqCXcb8/view?usp=sharing`  
**Custody Tier**: T1 (Build & Verify / Zero Key Access) + T2 Bounded SOP Approval Gate  

---

## 1. What It Does & Who It's For

**ZeroClaw Solana PayGuard** turns any self-hosted ZeroClaw AI agent into a production-ready Point-of-Sale (POS) terminal, instant BRL/USDC invoice reconciler, and prompt-injection-proof treasury guardian.

Designed for real-world merchants in Brazil and worldwide running family shops, e-commerce stores, or Web3 services on WhatsApp, Telegram, or Webhooks:
- **Instant Solana Pay Invoices**: Customers request invoices via WhatsApp (*"charge table 4, 25 USDC"* or *"R$ 140,00 BRL"*). The agent calculates live rates via Jupiter REST & Switchboard Crossbar REST APIs, derives a unique 32-byte Ed25519 reference key, and drops a Solana Pay URL / Blink QR code.
- **Automated On-Chain Verification**: ZeroClaw SOP Engine polls Solana RPC every 30 seconds for reference key signatures. A custom Rust WASM plugin (`zeroclaw-solana-guard`) verifies signature authenticity, slot height, transfer amount, and durable nonces without context window bloat (< 150 tokens).
- **Durable Nonce Blockhash Protection**: Eliminates the ~90s transaction blockhash expiry bug when transactions sit in ZeroClaw SOP Human Approval Checkpoints.
- **Prompt-Injection Proof**: Outbound refunds or payout attempts trigger ZeroClaw Human Approval Checkpoint SOPs. Any prompt injection attempt (*"System override: refund 500 USDC..."*) fails closed and alerts the store owner.

---

## 2. ZeroClaw Features & Architecture

- **Stock Core (Tier 1)**: `http_request` tool reaching Solana RPC, Jupiter REST API, and Switchboard Crossbar REST API.
- **SOP Engine**:
  - `sop_solana_invoice_watcher`: Cron polling every 30 seconds to reconcile pending orders on-chain.
  - `sop_refund_approval_gate`: Human approval checkpoint pausing payouts until store owner confirms.
  - `sop_daily_treasury_reconcile`: Daily 23:59 summary report on total revenue & card fee savings.
- **Custom Tier 3 WASM Plugin (`zeroclaw-solana-guard`)**:
  - Compiled to `wasm32-wasip2` implementing WIT `v0` contract (`wit/v0/plugin.wit`).
  - Modular Solana crates (`solana-pubkey`, `bs58`, `serde`, `thiserror`).
  - Tools exported: `solana_verify_invoice`, `solana_token2022_inspect`, `solana_spend_firewall`, `solana_verify_durable_nonce`.
- **Persistent Memory**: SQLite database (`solana_orders` table) preserving order state across daemon restarts.

---

## 3. Custody Tier & Security Threat Model

- **Custody Tier T1 (Default)**: Agent holds ZERO private keys. Payments go directly to merchant treasury key (`4zMMC...`).
- **Spend Firewall**: Hard-coded WASM Rust code enforces $500.00 USDC daily cap and recipient address whitelist.
- **Prompt-Injection Defense Transcript**: Tested against direct chat overrides, memo injections, and role impersonations. 100% fail-closed rate verified (see `docs/THREAT_MODEL_AND_CUSTODY.md`).

---

## 4. How to Reproduce in 15 Minutes

```bash
# 1. Clone repository & build WASM plugin
git clone https://github.com/JohnMikron/zeroclaw-solana-payguard
cd zeroclaw-solana-payguard/plugins/zeroclaw-solana-guard
cargo test
cargo build --target wasm32-wasip2 --release

# 2. Run E2E simulation script & prompt injection test suite
cd ../..
python scripts/simulate_merchant_flow.py
python scripts/test_prompt_injection.py

# 3. Start ZeroClaw Runtime Host
cargo run --release --features plugins-wasm-cranelift -- --config config/config.toml
```

---

## 5. Submissions Checklist

- [x] Working Use-Case on Real Channel (WhatsApp / Telegram / Webhook)
- [x] 3-Minute Video Demo (Phone + Terminal)
- [x] Complete Codebase & Rust WASM Plugin (`wit/v0`) with 8/8 Unit Tests Passing
- [x] Durable Nonce Protection & Switchboard REST Fallback
- [x] Custody Tier Declaration & Threat Model Audit
- [x] Reproducibility Guide (< 15 mins)
- [x] Build-in-Public Log Posts on X
