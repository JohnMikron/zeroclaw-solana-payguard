use crate::types::{InvoiceVerificationRequest, InvoiceVerificationResponse};
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub struct InvoiceVerifier;

impl InvoiceVerifier {
    pub fn verify_invoice(req: &InvoiceVerificationRequest) -> InvoiceVerificationResponse {
        let mut notes = Vec::new();

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

        // Extract amount: Robust iteration over postTokenBalances
        let mut amount_found = 0.0;

        if let Some(token_balances) = parsed.pointer("/meta/postTokenBalances").and_then(|v| v.as_array()) {
            for tb in token_balances {
                // If expected mint is provided, match mint
                if let Some(ref mint) = req.expected_spl_mint {
                    if let Some(tb_mint) = tb.get("mint").and_then(|m| m.as_str()) {
                        if tb_mint != mint {
                            continue;
                        }
                    }
                }

                // Check owner if present
                if let Some(owner) = tb.get("owner").and_then(|o| o.as_str()) {
                    if owner != req.expected_recipient {
                        // Skip balances not belonging to expected recipient if owner tag is explicit
                        if token_balances.len() > 1 && owner != req.expected_recipient {
                            continue;
                        }
                    }
                }

                if let Some(amt) = tb.pointer("/uiTokenAmount/uiAmount").and_then(|a| a.as_f64()) {
                    if amt > amount_found {
                        amount_found = amt;
                    }
                } else if let Some(amt_str) = tb.pointer("/uiTokenAmount/uiAmountString").and_then(|a| a.as_str()) {
                    if let Ok(parsed_amt) = amt_str.parse::<f64>() {
                        if parsed_amt > amount_found {
                            amount_found = parsed_amt;
                        }
                    }
                }
            }
        }

        // Fallback to postBalances (Native SOL) if no SPL token balance matched
        if amount_found == 0.0 {
            if let Some(balances) = parsed.pointer("/meta/postBalances").and_then(|v| v.as_array()) {
                if balances.len() >= 2 {
                    if let Some(lamports) = balances[1].as_f64() {
                        amount_found = lamports / 1_000_000_000.0;
                    }
                }
            }
        }

        notes.push("Successfully extracted on-chain transaction metadata from RPC payload".to_string());

        // 4. Verify Amount vs Requirement
        let is_valid_amount = amount_found >= (req.expected_amount - 0.000001);
        if !is_valid_amount {
            notes.push(format!(
                "Underpaid error: Expected {}, received {}",
                req.expected_amount, amount_found
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

