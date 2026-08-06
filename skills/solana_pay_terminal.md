# Skill: Solana Pay POS Terminal

## Description
Teaches ZeroClaw agent how to act as a point-of-sale terminal for local & online merchants. Generates Solana Pay URLs with unique reference keys, handles BRL/USDC conversion using resilient price feeds, and formats dynamic payment requests.

## Workflow

### Step 1: Parse Order Parameters
- Extract `table_number` or `order_id`, `amount`, and `currency` (USDC, BRL, or SOL).
- If currency is BRL (Brazilian Real):
  - Query Jupiter Price REST API: `https://api.jup.ag/price/v2?ids=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
  - Fallback: Query Switchboard Crossbar REST API: `https://crossbar.switchboard.xyz/v1/fetch` (unauthenticated production-ready feed resilient past Pyth July 31 2026 deprecation).
  - Calculate USDC equivalent based on live rate (e.g. 1 USD = 5.60 BRL).

### Step 2: Derive Solana Pay Reference Key
- Generate a unique 32-byte Ed25519 public key (the `reference` key) using random seed or order hash.
- Store `order_id`, `reference_key`, `expected_amount_usdc`, and `created_timestamp` in ZeroClaw memory database (`solana_orders` table).

### Step 3: Construct Solana Pay URL
- Build standard Solana Pay URI format:
  `solana:<MERCHANT_TREASURY_PUBKEY>?amount=<AMOUNT_USDC>&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&reference=<REFERENCE_KEY>&label=MerchantPOS&message=Invoice%20%23<ORDER_ID>`

### Step 4: Output Terminal Receipt & QR Payload
- Output clean, ultra-compact markdown response (<150 tokens) for WhatsApp / Telegram channel:

```markdown
🧾 *Invoice #412 Created*
💰 *Amount*: 25.00 USDC (R$ 140.00 BRL)
🔑 *Ref Key*: `9xQeWvG8...3bapC8G`
📲 *Scan / Pay Link*: [Click to Pay via Wallet](solana:4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU?amount=25.00&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&reference=9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G)

⏳ *Status*: Awaiting on-chain payment...
```
