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
                "postBalances": [100000000, 25000000000],
                "postTokenBalances": [
                    {
                        "uiTokenAmount": {
                            "uiAmount": 25.0,
                            "uiAmountString": "25.0"
                        }
                    }
                ]
            },
            "transaction": { "message": { "accountKeys": ["CustomerWallet111111111111111111111111111111"] } }
        }"#.to_string()),
    };

    let res = InvoiceVerifier::verify_invoice(&req);

    assert!(res.is_valid);
    assert_eq!(res.status, "PAID_AND_VERIFIED");
    assert_eq!(res.amount_received, 25.0);
    assert_eq!(res.slot, 284910200);
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
    assert!(res.requires_human_approval);
    assert!(res.reason.contains("DAILY_SPEND_CAP_EXCEEDED"));
}

#[test]
fn test_verify_durable_nonce_valid() {
    let req = DurableNonceRequest {
        nonce_account_pubkey: "9xQeWvG816bUx9EPjFWdd5AufqSSqeM2qN1xzybapC8G".to_string(),
        nonce_authority: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
        is_advance_nonce_first_instruction: true,
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
    };

    let res = DurableNonceVerifier::verify_nonce(&req);

    assert!(!res.is_valid);
    assert!(!res.blockhash_expiry_prevented);
    assert_eq!(res.status, "MISSING_ADVANCE_NONCE_INSTRUCTION");
}
