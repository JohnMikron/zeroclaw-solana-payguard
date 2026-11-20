use zeroclaw_solana_guard::durable_nonce::DurableNonceVerifier;
use zeroclaw_solana_guard::invoice_verifier::InvoiceVerifier;
use zeroclaw_solana_guard::spend_firewall::SpendFirewall;
use zeroclaw_solana_guard::token2022_inspector::Token2022Inspector;
use zeroclaw_solana_guard::types::{
    DurableNonceRequest, InvoiceVerificationRequest, SpendFirewallRequest, Token2022RiskRequest,
};

#[test]
fn test_verify_invoice_valid_payment() {
    let req = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 25.0,
        expected_spl_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        signatures: vec!["5VERIFIED_SIG_11111111111111111111111111111111111111111111111111111111111111111".to_string()],
        transaction_rpc_json: Some(r#"{
            "slot": 284910200,
            "blockTime": 1785261000,
            "meta": {
                "err": null,
                "preTokenBalances": [
                    {
                        "accountIndex": 1,
                        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "owner": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "uiTokenAmount": {
                            "amount": "0",
                            "decimals": 6,
                            "uiAmount": 0.0,
                            "uiAmountString": "0"
                        }
                    }
                ],
                "postTokenBalances": [
                    {
                        "accountIndex": 1,
                        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "owner": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "uiTokenAmount": {
                            "amount": "25000000",
                            "decimals": 6,
                            "uiAmount": 25.0,
                            "uiAmountString": "25.0"
                        }
                    }
                ]
            },
            "transaction": {
                "message": {
                    "accountKeys": [
                        "CustomerWallet111111111111111111111111111111",
                        "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G"
                    ]
                }
            }
        }"#.to_string()),
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    assert!(res.is_valid);
    assert_eq!(res.status, "PAID_AND_VERIFIED");
    assert_eq!(res.amount_received, 25.0);
    assert_eq!(res.slot, 284910200);
}

#[test]
fn test_verify_invoice_free_money_exploit_prevented() {
    // Attack scenario: Attacker points to a transaction where merchant's account
    // had 25.0 USDC before and still has 25.0 USDC after (net delta = 0).
    let req = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 25.0,
        expected_spl_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        signatures: vec!["5EXPLOIT_SIG_11111111111111111111111111111111111111111111111111111111111111111".to_string()],
        transaction_rpc_json: Some(r#"{
            "slot": 284910200,
            "blockTime": 1785261000,
            "meta": {
                "err": null,
                "preTokenBalances": [
                    {
                        "accountIndex": 1,
                        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "owner": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "uiTokenAmount": {
                            "amount": "25000000",
                            "decimals": 6,
                            "uiAmount": 25.0,
                            "uiAmountString": "25.0"
                        }
                    }
                ],
                "postTokenBalances": [
                    {
                        "accountIndex": 1,
                        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "owner": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "uiTokenAmount": {
                            "amount": "25000000",
                            "decimals": 6,
                            "uiAmount": 25.0,
                            "uiAmountString": "25.0"
                        }
                    }
                ]
            },
            "transaction": {
                "message": {
                    "accountKeys": [
                        "CustomerWallet111111111111111111111111111111",
                        "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G"
                    ]
                }
            }
        }"#.to_string()),
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    // Must be rejected because net received is 0
    assert!(!res.is_valid);
    assert_eq!(res.status, "UNDERPAID");
    assert_eq!(res.amount_received, 0.0);
}

#[test]
fn test_verify_invoice_rejects_failed_transaction() {
    let req = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 25.0,
        expected_spl_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        signatures: vec!["5FAILED_SIG_111111111111111111111111111111111111111111111111111111111111111111".to_string()],
        transaction_rpc_json: Some(r#"{
            "slot": 284910200,
            "blockTime": 1785261000,
            "meta": {
                "err": { "InstructionError": [0, "Custom"] },
                "postTokenBalances": [
                    {
                        "accountIndex": 1,
                        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "owner": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "uiTokenAmount": { "amount": "25000000", "decimals": 6, "uiAmount": 25.0 }
                    }
                ]
            },
            "transaction": {
                "message": {
                    "accountKeys": [
                        "CustomerWallet111111111111111111111111111111",
                        "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G"
                    ]
                }
            }
        }"#.to_string()),
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    assert!(!res.is_valid);
    assert_eq!(res.status, "TRANSACTION_FAILED");
}

#[test]
fn test_verify_invoice_rejects_missing_reference_key() {
    let req = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 25.0,
        expected_spl_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        signatures: vec!["5SIG_NO_REF_KEY_1111111111111111111111111111111111111111111111111111111111111".to_string()],
        transaction_rpc_json: Some(r#"{
            "slot": 284910200,
            "blockTime": 1785261000,
            "meta": {
                "err": null,
                "postTokenBalances": [
                    {
                        "accountIndex": 1,
                        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "owner": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "uiTokenAmount": { "amount": "25000000", "decimals": 6, "uiAmount": 25.0 }
                    }
                ]
            },
            "transaction": {
                "message": {
                    "accountKeys": [
                        "CustomerWallet111111111111111111111111111111",
                        "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
                    ]
                }
            }
        }"#.to_string()),
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    assert!(!res.is_valid);
    assert_eq!(res.status, "REFERENCE_KEY_NOT_FOUND");
}

#[test]
fn test_verify_invoice_skips_different_owner() {
    let req = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 25.0,
        expected_spl_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        signatures: vec!["5WRONG_OWNER_SIG_1111111111111111111111111111111111111111111111111111111111".to_string()],
        transaction_rpc_json: Some(r#"{
            "slot": 284910200,
            "blockTime": 1785261000,
            "meta": {
                "err": null,
                "postTokenBalances": [
                    {
                        "accountIndex": 1,
                        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "owner": "AttackerWallet111111111111111111111111111111",
                        "uiTokenAmount": { "amount": "25000000", "decimals": 6, "uiAmount": 25.0 }
                    }
                ]
            },
            "transaction": {
                "message": {
                    "accountKeys": [
                        "CustomerWallet111111111111111111111111111111",
                        "AttackerWallet111111111111111111111111111111",
                        "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G"
                    ]
                }
            }
        }"#.to_string()),
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    assert!(!res.is_valid);
    assert_eq!(res.status, "UNDERPAID");
    assert_eq!(res.amount_received, 0.0);
}

#[test]
fn test_verify_invoice_eliminates_sol_fallback_when_spl_expected() {
    let req = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 25.0,
        expected_spl_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        signatures: vec!["5SOL_ONLY_SIG_11111111111111111111111111111111111111111111111111111111111111".to_string()],
        transaction_rpc_json: Some(r#"{
            "slot": 284910200,
            "blockTime": 1785261000,
            "meta": {
                "err": null,
                "preBalances": [1000000000, 0, 0],
                "postBalances": [0, 50000000000, 0],
                "preTokenBalances": [],
                "postTokenBalances": []
            },
            "transaction": {
                "message": {
                    "accountKeys": [
                        "CustomerWallet111111111111111111111111111111",
                        "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                        "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G"
                    ]
                }
            }
        }"#.to_string()),
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    // When SPL token is expected, sending native SOL must NOT satisfy the invoice
    assert!(!res.is_valid);
    assert_eq!(res.status, "UNDERPAID");
    assert_eq!(res.amount_received, 0.0);
}

#[test]
fn test_verify_invoice_pending_without_rpc_data() {
    let req = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 25.0,
        expected_spl_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        signatures: vec!["5VERIFIED_SIG_11111111111111111111111111111111111111111111111111111111111111111".to_string()],
        transaction_rpc_json: None,
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    assert!(!res.is_valid);
    assert_eq!(res.status, "PENDING_RPC_CONFIRMATION");
    assert_eq!(res.amount_received, 0.0);
}

#[test]
fn test_verify_invoice_invalid_reference_key() {
    let req = InvoiceVerificationRequest {
        reference_key: "INVALID_PUBKEY".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 25.0,
        expected_spl_mint: None,
        signatures: vec![],
        transaction_rpc_json: None,
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    assert!(!res.is_valid);
    assert_eq!(res.status, "INVALID_REFERENCE_PUBKEY");
}

#[test]
fn test_token2022_risk_non_transferable() {
    let req = Token2022RiskRequest {
        mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        raw_account_data_base64: None,
        has_transfer_fee: false,
        transfer_fee_basis_points: 0,
        maximum_fee: 0,
        has_transfer_hook: false,
        transfer_hook_program_id: None,
        has_permanent_delegate: false,
        permanent_delegate_address: None,
        is_non_transferable: true,
        has_confidential_transfers: false,
    };

    let res = Token2022Inspector::inspect_risk(&req);

    assert_eq!(res.risk_level, "BLOCKED");
    assert!(!res.allows_merchant_custody);
    assert!(res.active_extensions.contains(&"NonTransferable".to_string()));
}

#[test]
fn test_token2022_risk_real_binary_payload() {
    // Construct 170-byte raw mint data with NonTransferable extension:
    // Base mint is 82 bytes + 83 zero-padding = 165 bytes
    // Byte 165: 1 (AccountType::Mint)
    // TLV starts at offset 166: type=9 (u16=9, len=0)
    let mut raw_data = vec![0u8; 165];
    raw_data.push(1); // AccountType::Mint at byte 165
    raw_data.extend_from_slice(&9u16.to_le_bytes()); // ExtType 9: NonTransferable
    raw_data.extend_from_slice(&0u16.to_le_bytes()); // ExtLen 0

    // Custom base64 encode
    let mut b64_str = String::new();
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in raw_data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        b64_str.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        b64_str.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            b64_str.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            b64_str.push('=');
        }
        if chunk.len() > 2 {
            b64_str.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            b64_str.push('=');
        }
    }

    // Caller claims is_non_transferable: false, but the binary data has the extension!
    let req = Token2022RiskRequest {
        mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        raw_account_data_base64: Some(b64_str),
        has_transfer_fee: false,
        transfer_fee_basis_points: 0,
        maximum_fee: 0,
        has_transfer_hook: false,
        transfer_hook_program_id: None,
        has_permanent_delegate: false,
        permanent_delegate_address: None,
        is_non_transferable: false,
        has_confidential_transfers: false,
    };

    let res = Token2022Inspector::inspect_risk(&req);

    // Binary inspection overrides caller-supplied false
    assert_eq!(res.risk_level, "BLOCKED");
    assert!(!res.allows_merchant_custody);
    assert!(res.active_extensions.contains(&"NonTransferable".to_string()));
}

#[test]
fn test_spend_firewall_approves_within_cap() {
    let req = SpendFirewallRequest {
        recipient_address: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        amount_usdc: 50.0,
        token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        current_daily_spent_usdc: 100.0,
        max_daily_limit_usdc: 500.0,
        allowed_recipients: vec!["4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string()],
        allowed_mints: vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()],
        requester_role: "merchant_agent".to_string(),
    };

    let res = SpendFirewall::evaluate(&req);

    assert!(res.approved);
    assert!(!res.hard_blocked);
    assert!(!res.requires_human_approval);
    assert_eq!(res.remaining_daily_quota_usdc, 350.0);
}

#[test]
fn test_spend_firewall_blocks_exceeding_cap() {
    let req = SpendFirewallRequest {
        recipient_address: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        amount_usdc: 450.0,
        token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        current_daily_spent_usdc: 100.0,
        max_daily_limit_usdc: 500.0,
        allowed_recipients: vec!["4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string()],
        allowed_mints: vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()],
        requester_role: "merchant_agent".to_string(),
    };

    let res = SpendFirewall::evaluate(&req);

    assert!(!res.approved);
    assert!(res.hard_blocked);
    assert!(!res.requires_human_approval);
    assert!(res.reason.contains("DAILY_SPEND_CAP_EXCEEDED"));
}

#[test]
fn test_spend_firewall_blocks_nan_and_negative() {
    // Test NaN
    let req_nan = SpendFirewallRequest {
        recipient_address: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        amount_usdc: f64::NAN,
        token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        current_daily_spent_usdc: 0.0,
        max_daily_limit_usdc: 500.0,
        allowed_recipients: vec!["4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string()],
        allowed_mints: vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()],
        requester_role: "merchant_agent".to_string(),
    };
    let res_nan = SpendFirewall::evaluate(&req_nan);
    assert!(!res_nan.approved);
    assert!(res_nan.hard_blocked);
    assert!(!res_nan.requires_human_approval);
    assert!(res_nan.reason.contains("INVALID_AMOUNT"));

    // Test Negative
    let req_neg = SpendFirewallRequest {
        recipient_address: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        amount_usdc: -50.0,
        token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        current_daily_spent_usdc: 0.0,
        max_daily_limit_usdc: 500.0,
        allowed_recipients: vec!["4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string()],
        allowed_mints: vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()],
        requester_role: "merchant_agent".to_string(),
    };
    let res_neg = SpendFirewall::evaluate(&req_neg);
    assert!(!res_neg.approved);
    assert!(res_neg.hard_blocked);
    assert!(!res_neg.requires_human_approval);
    assert!(res_neg.reason.contains("INVALID_AMOUNT"));

    // Test Infinity
    let req_inf = SpendFirewallRequest {
        recipient_address: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        amount_usdc: f64::INFINITY,
        token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        current_daily_spent_usdc: 0.0,
        max_daily_limit_usdc: 500.0,
        allowed_recipients: vec!["4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string()],
        allowed_mints: vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()],
        requester_role: "merchant_agent".to_string(),
    };
    let res_inf = SpendFirewall::evaluate(&req_inf);
    assert!(!res_inf.approved);
    assert!(res_inf.hard_blocked);
    assert!(!res_inf.requires_human_approval);
    assert!(res_inf.reason.contains("INVALID_AMOUNT"));
}

#[test]
fn test_spend_firewall_wildcard_bypass_blocked() {
    // Attack scenario: allowed_recipients contains "*", attacker sends to unauthorized pubkey
    let req = SpendFirewallRequest {
        recipient_address: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        amount_usdc: 50.0,
        token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        current_daily_spent_usdc: 0.0,
        max_daily_limit_usdc: 500.0,
        allowed_recipients: vec!["*".to_string()],
        allowed_mints: vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()],
        requester_role: "merchant_agent".to_string(),
    };

    let res = SpendFirewall::evaluate(&req);

    // Wildcard "*" must NOT grant unauthenticated bypass
    assert!(!res.approved);
    assert!(!res.hard_blocked);
    assert!(res.requires_human_approval);
}

#[test]
fn test_spend_firewall_enforces_hardcoded_wasm_limit() {
    // Attack scenario: Prompt injection injects max_daily_limit_usdc = 10000.0
    let req = SpendFirewallRequest {
        recipient_address: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        amount_usdc: 600.0,
        token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        current_daily_spent_usdc: 0.0,
        max_daily_limit_usdc: 10000.0, // Injected high limit
        allowed_recipients: vec!["4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string()],
        allowed_mints: vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()],
        requester_role: "merchant_agent".to_string(),
    };

    let res = SpendFirewall::evaluate(&req);

    // Must be blocked because WASM enforces hardcoded cap of 500.0 USDC
    assert!(!res.approved);
    assert!(res.hard_blocked);
    assert!(!res.requires_human_approval);
    assert!(res.reason.contains("DAILY_SPEND_CAP_EXCEEDED"));
}

#[test]
fn test_verify_durable_nonce_valid() {
    let req = DurableNonceRequest {
        nonce_account_pubkey: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        nonce_authority: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        is_advance_nonce_first_instruction: true,
        transaction_rpc_json: None,
        raw_nonce_account_data_base64: None,
    };

    let res = DurableNonceVerifier::verify_nonce(&req);

    assert!(res.is_valid);
    assert!(res.blockhash_expiry_prevented);
    assert_eq!(res.status, "DURABLE_NONCE_VALIDATED");
}

#[test]
fn test_verify_durable_nonce_missing_instruction() {
    let req = DurableNonceRequest {
        nonce_account_pubkey: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        nonce_authority: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        is_advance_nonce_first_instruction: false,
        transaction_rpc_json: None,
        raw_nonce_account_data_base64: None,
    };

    let res = DurableNonceVerifier::verify_nonce(&req);

    assert!(!res.is_valid);
    assert!(!res.blockhash_expiry_prevented);
    assert_eq!(res.status, "MISSING_ADVANCE_NONCE_INSTRUCTION");
}

#[test]
fn test_verify_durable_nonce_real_rpc_inspection() {
    // Test transaction payload with AdvanceNonceAccount as instruction 0
    let valid_tx_json = r#"{
        "transaction": {
            "message": {
                "accountKeys": [
                    "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G",
                    "SysvarRecentB1ockHashes11111111111111111111",
                    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                    "11111111111111111111111111111111"
                ],
                "instructions": [
                    {
                        "program": "system",
                        "parsed": {
                            "type": "advanceNonce",
                            "info": {
                                "nonceAccount": "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G",
                                "nonceAuthority": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
                            }
                        }
                    }
                ]
            }
        }
    }"#;

    let req = DurableNonceRequest {
        nonce_account_pubkey: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        nonce_authority: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        is_advance_nonce_first_instruction: true,
        transaction_rpc_json: Some(valid_tx_json.to_string()),
        raw_nonce_account_data_base64: None,
    };

    let res = DurableNonceVerifier::verify_nonce(&req);
    assert!(res.is_valid);
    assert_eq!(res.status, "DURABLE_NONCE_VALIDATED");

    // Attack scenario: Caller claims is_advance_nonce_first_instruction: true,
    // but the transaction payload has a different first instruction!
    let spoofed_tx_json = r#"{
        "transaction": {
            "message": {
                "accountKeys": [
                    "AttackerWallet111111111111111111111111111111"
                ],
                "instructions": [
                    {
                        "program": "spl-token",
                        "parsed": {
                            "type": "transfer",
                            "info": {}
                        }
                    }
                ]
            }
        }
    }"#;

    let spoofed_req = DurableNonceRequest {
        nonce_account_pubkey: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        nonce_authority: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        is_advance_nonce_first_instruction: true, // Caller asserts true
        transaction_rpc_json: Some(spoofed_tx_json.to_string()),
        raw_nonce_account_data_base64: None,
    };

    let spoofed_res = DurableNonceVerifier::verify_nonce(&spoofed_req);
    assert!(!spoofed_res.is_valid);
    assert_eq!(spoofed_res.status, "MISSING_ADVANCE_NONCE_INSTRUCTION");
}

#[test]
fn test_verify_invoice_invalid_expected_amount() {
    let req_zero = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: 0.0,
        expected_spl_mint: None,
        signatures: vec!["5VERIFIED_SIG_11111111111111111111111111111111111111111111111111111111111111111".to_string()],
        transaction_rpc_json: None,
    };
    let res_zero = InvoiceVerifier::verify_invoice(&req_zero);
    assert!(!res_zero.is_valid);
    assert_eq!(res_zero.status, "INVALID_AMOUNT");

    let req_neg = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: -10.0,
        expected_spl_mint: None,
        signatures: vec![],
        transaction_rpc_json: None,
    };
    let res_neg = InvoiceVerifier::verify_invoice(&req_neg);
    assert!(!res_neg.is_valid);
    assert_eq!(res_neg.status, "INVALID_AMOUNT");

    let req_nan = InvoiceVerificationRequest {
        reference_key: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        expected_recipient: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        expected_amount: f64::NAN,
        expected_spl_mint: None,
        signatures: vec![],
        transaction_rpc_json: None,
    };
    let res_nan = InvoiceVerifier::verify_invoice(&req_nan);
    assert!(!res_nan.is_valid);
    assert_eq!(res_nan.status, "INVALID_AMOUNT");
}

#[test]
fn test_verify_durable_nonce_fail_closed_on_bad_account_data() {
    // 1. Under 80 bytes
    let short_data = "AAAA"; // 3 bytes
    let req_short = DurableNonceRequest {
        nonce_account_pubkey: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        nonce_authority: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        is_advance_nonce_first_instruction: true,
        transaction_rpc_json: None,
        raw_nonce_account_data_base64: Some(short_data.to_string()),
    };
    let res_short = DurableNonceVerifier::verify_nonce(&req_short);
    assert!(!res_short.is_valid);
    assert_eq!(res_short.status, "INVALID_NONCE_ACCOUNT_SIZE");

    // 2. Corrupt base64
    let req_corrupt = DurableNonceRequest {
        nonce_account_pubkey: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        nonce_authority: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        is_advance_nonce_first_instruction: true,
        transaction_rpc_json: None,
        raw_nonce_account_data_base64: Some("not-valid-base64!!!".to_string()),
    };
    let res_corrupt = DurableNonceVerifier::verify_nonce(&req_corrupt);
    assert!(!res_corrupt.is_valid);
    assert_eq!(res_corrupt.status, "INVALID_NONCE_ACCOUNT_DATA");
}

#[test]
fn test_verify_durable_nonce_compiled_instruction_advance_nonce_check() {
    let nonce_pk = "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G";
    let auth_pk = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

    // Valid: data u32 LE == 4 (SystemInstruction::AdvanceNonceAccount)
    let valid_data_bs58 = bs58::encode(&[4u8, 0, 0, 0]).into_string();
    let tx_compiled_valid = format!(r#"{{
        "transaction": {{
            "message": {{
                "accountKeys": [
                    "{}",
                    "SysvarRecentB1ockHashes11111111111111111111",
                    "{}",
                    "11111111111111111111111111111111"
                ],
                "instructions": [
                    {{
                        "programIdIndex": 3,
                        "accounts": [0, 1, 2],
                        "data": "{}"
                    }}
                ]
            }}
        }}
    }}"#, nonce_pk, auth_pk, valid_data_bs58);

    let req_valid = DurableNonceRequest {
        nonce_account_pubkey: nonce_pk.to_string(),
        nonce_authority: auth_pk.to_string(),
        is_advance_nonce_first_instruction: true,
        transaction_rpc_json: Some(tx_compiled_valid),
        raw_nonce_account_data_base64: None,
    };
    let res_valid = DurableNonceVerifier::verify_nonce(&req_valid);
    assert!(res_valid.is_valid);
    assert_eq!(res_valid.status, "DURABLE_NONCE_VALIDATED");

    // Invalid: data u32 LE == 2 (SystemInstruction::Transfer instead of AdvanceNonceAccount)
    let transfer_data_bs58 = bs58::encode(&[2u8, 0, 0, 0]).into_string();
    let tx_compiled_invalid = format!(r#"{{
        "transaction": {{
            "message": {{
                "accountKeys": [
                    "{}",
                    "SysvarRecentB1ockHashes11111111111111111111",
                    "{}",
                    "11111111111111111111111111111111"
                ],
                "instructions": [
                    {{
                        "programIdIndex": 3,
                        "accounts": [0, 1, 2],
                        "data": "{}"
                    }}
                ]
            }}
        }}
    }}"#, nonce_pk, auth_pk, transfer_data_bs58);

    let req_invalid = DurableNonceRequest {
        nonce_account_pubkey: nonce_pk.to_string(),
        nonce_authority: auth_pk.to_string(),
        is_advance_nonce_first_instruction: true,
        transaction_rpc_json: Some(tx_compiled_invalid),
        raw_nonce_account_data_base64: None,
    };
    let res_invalid = DurableNonceVerifier::verify_nonce(&req_invalid);
    assert!(!res_invalid.is_valid);
    assert_eq!(res_invalid.status, "MISSING_ADVANCE_NONCE_INSTRUCTION");
}

#[test]
fn test_token2022_risk_invalid_account_type() {
    // 166 bytes data: 165 bytes padding + byte 165 has value 2 (AccountType::Account instead of Mint)
    let mut raw_data = vec![0u8; 165];
    raw_data.push(2); // Invalid AccountType (not 1)

    // Base64 encode
    let mut b64_str = String::new();
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in raw_data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        b64_str.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        b64_str.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            b64_str.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            b64_str.push('=');
        }
        if chunk.len() > 2 {
            b64_str.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            b64_str.push('=');
        }
    }

    let req = Token2022RiskRequest {
        mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        raw_account_data_base64: Some(b64_str),
        has_transfer_fee: false,
        transfer_fee_basis_points: 0,
        maximum_fee: 0,
        has_transfer_hook: false,
        transfer_hook_program_id: None,
        has_permanent_delegate: false,
        permanent_delegate_address: None,
        is_non_transferable: false,
        has_confidential_transfers: false,
    };

    let res = Token2022Inspector::inspect_risk(&req);
    assert_eq!(res.risk_level, "BLOCKED");
    assert!(!res.allows_merchant_custody);
    assert!(res.security_warnings.iter().any(|w| w.contains("INVALID_ACCOUNT_TYPE")));
}
