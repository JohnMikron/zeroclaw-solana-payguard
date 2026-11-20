use crate::types::{InvoiceVerificationRequest, InvoiceVerificationResponse};
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub struct InvoiceVerifier;

impl InvoiceVerifier {
    pub fn verify_invoice(req: &InvoiceVerificationRequest) -> InvoiceVerificationResponse {
        let mut notes = Vec::new();

        // 0. Guard against invalid expected_amount
        if req.expected_amount <= 0.0 || !req.expected_amount.is_finite() {
            return InvoiceVerificationResponse {
                is_valid: false,
                order_reference: req.reference_key.clone(),
                verified_signature: None,
                sender_address: None,
                recipient_address: req.expected_recipient.clone(),
                amount_received: 0.0,
                spl_mint_used: req.expected_spl_mint.clone(),
                slot: 0,
                block_time: None,
                status: "INVALID_AMOUNT".to_string(),
                verification_notes: vec!["Expected amount must be a positive finite number".to_string()],
            };
        }

        // 1. Validate Public Keys
        let reference_pubkey = match Pubkey::from_str(&req.reference_key) {
            Ok(pk) => pk,
            Err(_) => {
                return InvoiceVerificationResponse {
                    is_valid: false,
                    order_reference: req.reference_key.clone(),
                    verified_signature: None,
                    sender_address: None,
                    recipient_address: req.expected_recipient.clone(),
                    amount_received: 0.0,
                    spl_mint_used: req.expected_spl_mint.clone(),
                    slot: 0,
                    block_time: None,
                    status: "INVALID_REFERENCE_PUBKEY".to_string(),
                    verification_notes: vec!["Reference key is not a valid Solana pubkey".to_string()],
                };
            }
        };

        let recipient_pubkey = match Pubkey::from_str(&req.expected_recipient) {
            Ok(pk) => pk,
            Err(_) => {
                return InvoiceVerificationResponse {
                    is_valid: false,
                    order_reference: req.reference_key.clone(),
                    verified_signature: None,
                    sender_address: None,
                    recipient_address: req.expected_recipient.clone(),
                    amount_received: 0.0,
                    spl_mint_used: req.expected_spl_mint.clone(),
                    slot: 0,
                    block_time: None,
                    status: "INVALID_RECIPIENT_PUBKEY".to_string(),
                    verification_notes: vec!["Expected recipient is not a valid Solana pubkey".to_string()],
                };
            }
        };

        if let Some(ref mint) = req.expected_spl_mint {
            if Pubkey::from_str(mint).is_err() {
                return InvoiceVerificationResponse {
                    is_valid: false,
                    order_reference: req.reference_key.clone(),
                    verified_signature: None,
                    sender_address: None,
                    recipient_address: req.expected_recipient.clone(),
                    amount_received: 0.0,
                    spl_mint_used: req.expected_spl_mint.clone(),
                    slot: 0,
                    block_time: None,
                    status: "INVALID_MINT_PUBKEY".to_string(),
                    verification_notes: vec!["Expected SPL mint is not a valid Solana pubkey".to_string()],
                };
            }
        }

        notes.push(format!("Reference key verified: {}", reference_pubkey));
        notes.push(format!("Recipient target verified: {}", recipient_pubkey));

        // 2. Process Signatures
        if req.signatures.is_empty() {
            return InvoiceVerificationResponse {
                is_valid: false,
                order_reference: req.reference_key.clone(),
                verified_signature: None,
                sender_address: None,
                recipient_address: req.expected_recipient.clone(),
                amount_received: 0.0,
                spl_mint_used: req.expected_spl_mint.clone(),
                slot: 0,
                block_time: None,
                status: "NO_SIGNATURES_FOUND".to_string(),
                verification_notes: vec!["No transactions currently found for reference key".to_string()],
            };
        }

        let selected_sig = req.signatures[0].clone();
        notes.push(format!("Signature selected for verification: {}", selected_sig));

        // 3. Require On-Chain RPC Transaction JSON - Return PENDING if missing
        let json_str = match req.transaction_rpc_json {
            Some(ref s) => s,
            None => {
                notes.push("Awaiting on-chain RPC transaction payload for verification".to_string());
                return InvoiceVerificationResponse {
                    is_valid: false,
                    order_reference: req.reference_key.clone(),
                    verified_signature: Some(selected_sig),
                    sender_address: None,
                    recipient_address: req.expected_recipient.clone(),
                    amount_received: 0.0,
                    spl_mint_used: req.expected_spl_mint.clone(),
                    slot: 0,
                    block_time: None,
                    status: "PENDING_RPC_CONFIRMATION".to_string(),
                    verification_notes: notes,
                };
            }
        };

        // Parse RPC payload
        let parsed: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(val) => val,
            Err(e) => {
                notes.push(format!("Failed to parse RPC transaction JSON: {}", e));
                return InvoiceVerificationResponse {
                    is_valid: false,
                    order_reference: req.reference_key.clone(),
                    verified_signature: Some(selected_sig),
                    sender_address: None,
                    recipient_address: req.expected_recipient.clone(),
                    amount_received: 0.0,
                    spl_mint_used: req.expected_spl_mint.clone(),
                    slot: 0,
                    block_time: None,
                    status: "RPC_PARSE_ERROR".to_string(),
                    verification_notes: notes,
                };
            }
        };

        let slot_found = parsed.get("slot").and_then(|s| s.as_u64()).unwrap_or(0);
        let time_found = parsed.get("blockTime").and_then(|b| b.as_i64());

        // Extract sender address safely across different JSON structures (strings or object arrays)
        let sender_found = parsed
            .pointer("/transaction/message/accountKeys/0")
            .and_then(|key| {
                if let Some(s) = key.as_str() {
                    Some(s.to_string())
                } else {
                    key.get("pubkey").and_then(|p| p.as_str()).map(|s| s.to_string())
                }
            });

        // 4. Verify Transaction Status (meta.err must be null / None)
        if let Some(err) = parsed.pointer("/meta/err") {
            if !err.is_null() {
                notes.push(format!("Transaction failed on-chain: {:?}", err));
                return InvoiceVerificationResponse {
                    is_valid: false,
                    order_reference: req.reference_key.clone(),
                    verified_signature: Some(selected_sig),
                    sender_address: sender_found,
                    recipient_address: req.expected_recipient.clone(),
                    amount_received: 0.0,
                    spl_mint_used: req.expected_spl_mint.clone(),
                    slot: slot_found,
                    block_time: time_found,
                    status: "TRANSACTION_FAILED".to_string(),
                    verification_notes: notes,
                };
            }
        }

        // 5. Verify Reference Key actually exists in transaction.message.accountKeys
        let mut reference_found = false;
        let mut account_keys_list = Vec::new();

        if let Some(account_keys) = parsed.pointer("/transaction/message/accountKeys").and_then(|k| k.as_array()) {
            for key in account_keys {
                let pubkey_str = if let Some(s) = key.as_str() {
                    s.to_string()
                } else if let Some(pk) = key.get("pubkey").and_then(|p| p.as_str()) {
                    pk.to_string()
                } else {
                    continue;
                };

                if pubkey_str == req.reference_key {
                    reference_found = true;
                }
                account_keys_list.push(pubkey_str);
            }
        }

        if !reference_found {
            notes.push(format!(
                "Security check failed: Reference key '{}' not present in transaction accountKeys",
                req.reference_key
            ));
            return InvoiceVerificationResponse {
                is_valid: false,
                order_reference: req.reference_key.clone(),
                verified_signature: Some(selected_sig),
                sender_address: sender_found,
                recipient_address: req.expected_recipient.clone(),
                amount_received: 0.0,
                spl_mint_used: req.expected_spl_mint.clone(),
                slot: slot_found,
                block_time: time_found,
                status: "REFERENCE_KEY_NOT_FOUND".to_string(),
                verification_notes: notes,
            };
        }
        notes.push("Reference key presence verified in transaction accountKeys".to_string());

        // 6. Calculate Net Balance Delta using Atomic Units (u64)
        let amount_found: f64;

        if let Some(ref expected_mint) = req.expected_spl_mint {
            // STRICT SPL TOKEN VERIFICATION: No fallback to native SOL
            let mut detected_decimals: u32 = 6; // Standard USDC default

            // Helper to extract atomic token amount and decimals
            let parse_token_amount = |tb: &serde_json::Value| -> (u64, u32) {
                let dec = tb
                    .pointer("/uiTokenAmount/decimals")
                    .and_then(|d| d.as_u64())
                    .unwrap_or(6) as u32;

                let amt = if let Some(amt_str) = tb.pointer("/uiTokenAmount/amount").and_then(|a| a.as_str()) {
                    amt_str.parse::<u64>().unwrap_or(0)
                } else if let Some(amt_num) = tb.pointer("/uiTokenAmount/amount").and_then(|a| a.as_u64()) {
                    amt_num
                } else if let Some(ui_amt) = tb.pointer("/uiTokenAmount/uiAmount").and_then(|a| a.as_f64()) {
                    let factor = 10u64.pow(dec);
                    (ui_amt * (factor as f64)).round() as u64
                } else if let Some(amt_str) = tb.pointer("/uiTokenAmount/uiAmountString").and_then(|a| a.as_str()) {
                    if let Ok(ui_amt) = amt_str.parse::<f64>() {
                        let factor = 10u64.pow(dec);
                        (ui_amt * (factor as f64)).round() as u64
                    } else {
                        0
                    }
                } else {
                    0
                };
                (amt, dec)
            };

            // Helper to check if a token balance entry belongs strictly to the expected recipient
            let is_recipient_entry = |tb: &serde_json::Value| -> bool {
                // 1. Check mint matches
                if let Some(m) = tb.get("mint").and_then(|v| v.as_str()) {
                    if m != expected_mint {
                        return false;
                    }
                }

                // 2. Check owner matches strictly
                if let Some(owner) = tb.get("owner").and_then(|o| o.as_str()) {
                    return owner == req.expected_recipient;
                }

                // If owner tag is omitted, check if accountIndex resolves to expected_recipient
                if let Some(idx) = tb.get("accountIndex").and_then(|i| i.as_u64()) {
                    if let Some(acc_key) = account_keys_list.get(idx as usize) {
                        return acc_key == &req.expected_recipient;
                    }
                }

                false
            };

            // Sum all recipient pre-balances independently
            let mut total_pre_atomic: u64 = 0;
            if let Some(pre_array) = parsed.pointer("/meta/preTokenBalances").and_then(|v| v.as_array()) {
                for tb in pre_array {
                    if is_recipient_entry(tb) {
                        let (amt, dec) = parse_token_amount(tb);
                        detected_decimals = dec;
                        total_pre_atomic = total_pre_atomic.saturating_add(amt);
                    }
                }
            }

            // Sum all recipient post-balances independently
            let mut total_post_atomic: u64 = 0;
            let mut recipient_token_entry_found = false;

            if let Some(post_array) = parsed.pointer("/meta/postTokenBalances").and_then(|v| v.as_array()) {
                for tb in post_array {
                    if is_recipient_entry(tb) {
                        recipient_token_entry_found = true;
                        let (post_amt, dec) = parse_token_amount(tb);
                        detected_decimals = dec;
                        total_post_atomic = total_post_atomic.saturating_add(post_amt);
                    }
                }
            }

            if !recipient_token_entry_found {
                notes.push("No postTokenBalances entries found matching expected recipient and SPL mint".to_string());
            }

            // Net balance delta strictly for recipient token accounts
            let net_atomic_delta = total_post_atomic.saturating_sub(total_pre_atomic);
            let factor = 10u64.pow(detected_decimals);
            amount_found = (net_atomic_delta as f64) / (factor as f64);

            let expected_atomic: u64 = (req.expected_amount * (factor as f64)).round() as u64;

            notes.push(format!(
                "SPL Token Net Delta: {} atomic units ({:.6} UI, expected {} atomic units)",
                net_atomic_delta, amount_found, expected_atomic
            ));

            if net_atomic_delta < expected_atomic || net_atomic_delta == 0 {
                notes.push(format!(
                    "Underpaid error: Expected {} atomic units ({:.6} UI), net delta received {} atomic units ({:.6} UI)",
                    expected_atomic, req.expected_amount, net_atomic_delta, amount_found
                ));
                return InvoiceVerificationResponse {
                    is_valid: false,
                    order_reference: req.reference_key.clone(),
                    verified_signature: Some(selected_sig),
                    sender_address: sender_found,
                    recipient_address: req.expected_recipient.clone(),
                    amount_received: amount_found,
                    spl_mint_used: req.expected_spl_mint.clone(),
                    slot: slot_found,
                    block_time: time_found,
                    status: "UNDERPAID".to_string(),
                    verification_notes: notes,
                };
            }
        } else {
            // Native SOL verification: Net lamports delta for recipient accountIndex
            let recipient_idx_opt = account_keys_list.iter().position(|k| k == &req.expected_recipient);

            if let Some(recipient_idx) = recipient_idx_opt {
                let pre_lamports = parsed
                    .pointer(&format!("/meta/preBalances/{}", recipient_idx))
                    .and_then(|b| b.as_u64())
                    .unwrap_or(0);

                let post_lamports = parsed
                    .pointer(&format!("/meta/postBalances/{}", recipient_idx))
                    .and_then(|b| b.as_u64())
                    .unwrap_or(0);

                let net_lamports = post_lamports.saturating_sub(pre_lamports);
                amount_found = (net_lamports as f64) / 1_000_000_000.0;
                let expected_lamports = (req.expected_amount * 1_000_000_000.0).round() as u64;

                notes.push(format!(
                    "Native SOL Net Delta: {} lamports ({:.9} SOL, expected {} lamports)",
                    net_lamports, amount_found, expected_lamports
                ));

                if net_lamports < expected_lamports || net_lamports == 0 {
                    notes.push(format!(
                        "Underpaid error: Expected {} lamports ({:.9} SOL), net delta received {} lamports ({:.9} SOL)",
                        expected_lamports, req.expected_amount, net_lamports, amount_found
                    ));
                    return InvoiceVerificationResponse {
                        is_valid: false,
                        order_reference: req.reference_key.clone(),
                        verified_signature: Some(selected_sig),
                        sender_address: sender_found,
                        recipient_address: req.expected_recipient.clone(),
                        amount_received: amount_found,
                        spl_mint_used: None,
                        slot: slot_found,
                        block_time: time_found,
                        status: "UNDERPAID".to_string(),
                        verification_notes: notes,
                    };
                }
            } else {
                notes.push("Recipient address not found in transaction accountKeys for native SOL transfer".to_string());
                return InvoiceVerificationResponse {
                    is_valid: false,
                    order_reference: req.reference_key.clone(),
                    verified_signature: Some(selected_sig),
                    sender_address: sender_found,
                    recipient_address: req.expected_recipient.clone(),
                    amount_received: 0.0,
                    spl_mint_used: None,
                    slot: slot_found,
                    block_time: time_found,
                    status: "UNDERPAID".to_string(),
                    verification_notes: notes,
                };
            }
        }

        notes.push("Payment amount matches or exceeds required invoice value".to_string());

        InvoiceVerificationResponse {
            is_valid: true,
            order_reference: req.reference_key.clone(),
            verified_signature: Some(selected_sig),
            sender_address: sender_found,
            recipient_address: req.expected_recipient.clone(),
            amount_received: amount_found,
            spl_mint_used: req.expected_spl_mint.clone(),
            slot: slot_found,
            block_time: time_found,
            status: "PAID_AND_VERIFIED".to_string(),
            verification_notes: notes,
        }
    }
}
