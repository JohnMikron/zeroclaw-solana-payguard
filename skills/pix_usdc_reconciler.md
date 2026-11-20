# Skill: Brazil PIX & Solana USDC Invoice Reconciler

## Description
Enables Brazilian merchants to accept payments seamlessly using local BRL / PIX dynamic QR codes alongside instant Solana USDC settlement.

## Context & Problem
Local Brazilian merchants operating via WhatsApp require instant BRL invoicing, but want settlement in yield-bearing or stable Solana USDC without custody risk or high credit card fees (3-5%).

## Workflow
1. Customer initiates chat on WhatsApp: *"Invoice table 4 for R$ 100,00"*.
2. Agent calculates USDC equivalent (e.g. 100.00 BRL / 5.60 BRL per USD = 17.85 USDC).
3. Agent generates dual payment invoice:
   - **Solana Pay Link**: `solana:DLoBeytd5Km1WuiZ6eyymvGF1YtSiCwPzCU6t8ctgipK?amount=17.85&spl-token=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU&reference=<REF>`
   - **PIX Copy & Paste Payload**: Static/dynamic BR Code for local BRL banking apps.
4. SOP Cron Poller monitors reference key on Solana RPC.
5. On payment receipt, agent posts receipt to WhatsApp store owner channel:
   *"Invoice #412 paid ✓ (17.85 USDC received on Solana Devnet). Order closed."*
