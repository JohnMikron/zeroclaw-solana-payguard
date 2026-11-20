# Skill: Solana Pay POS Terminal

## Description
Teaches ZeroClaw agent how to act as a point-of-sale terminal for local & online merchants. Generates Solana Pay URLs with unique reference keys, handles BRL/USDC conversion using resilient price feeds, and formats dynamic payment requests.

## Workflow

### Step 1: Parse Order Parameters
- Extract `table_number` or `order_id`, `amount`, and `currency` (USDC, BRL, or SOL).
- If currency is BRL (Brazilian Real):
  - Query Switchboard Crossbar REST API: `https://crossbar.switchboard.xyz/v1/fetch`, AwesomeAPI (`https://economia.awesomeapi.com.br/last/USD-BRL`), or Coinbase Spot API (`https://api.coinbase.com/v2/prices/USDC-BRL/spot`). (Note: Jupiter Price API is reserved for on-chain crypto token swaps like SOL/USDC, while fiat BRL/USDC conversion utilizes dedicated FX oracles).
  - Calculate USDC equivalent based on live rate (e.g. 1 USD = 5.95 BRL).

### Step 2: Derive Solana Pay Reference Key
- Generate a unique 32-byte Ed25519 public key (the `reference` key) using random seed or order hash.
- Store `order_id`, `reference_key`, `expected_amount_usdc`, and `created_timestamp` in ZeroClaw memory database (`solana_orders` table).

### Step 3: Construct Solana Pay URL
- Build standard Solana Pay URI format:
  `solana:<MERCHANT_TREASURY_PUBKEY>?amount=<AMOUNT_USDC>&spl-token=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU&reference=<REFERENCE_KEY>&label=MerchantPOS&message=Invoice%20%23<ORDER_ID>`

### Step 4: Output Terminal Receipt & QR Payload
- Output clean, ultra-compact markdown response (<150 tokens) for WhatsApp / Telegram channel:

```markdown
🧾 *Invoice #412 Created*
💰 *Amount*: 25.00 USDC (R$ 140.00 BRL)
🔑 *Ref Key*: `4tWDzxRH...9mvy`
📲 *Scan / Pay Link*: [Click to Pay via Wallet](solana:DLoBeytd5Km1WuiZ6eyymvGF1YtSiCwPzCU6t8ctgipK?amount=25.00&spl-token=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU&reference=4tWDzxRH9wQ5sH9MbMRwgWWLMYBVMUx7x8dgLCdT9mvy)

⏳ *Status*: Awaiting on-chain payment...
```
