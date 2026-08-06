use crate::types::{DurableNonceRequest, DurableNonceResponse};
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub struct DurableNonceVerifier;

impl DurableNonceVerifier {
    pub fn verify_nonce(req: &DurableNonceRequest) -> DurableNonceResponse {
        let mut notes = Vec::new();

        // 1. Verify Nonce Account Address
        let nonce_account = match Pubkey::from_str(&req.nonce_account_pubkey) {
            Ok(pk) => pk,
            Err(_) => {
                return DurableNonceResponse {
                    is_valid: false,
                    nonce_account: req.nonce_account_pubkey.clone(),
                    nonce_authority: req.nonce_authority.clone(),
                    blockhash_expiry_prevented: false,
                    status: "INVALID_NONCE_ACCOUNT_PUBKEY".to_string(),
                    verification_notes: vec!["Nonce account is not a valid Solana public key".to_string()],
                };
            }
        };

        // 2. Verify Nonce Authority
        let nonce_authority = match Pubkey::from_str(&req.nonce_authority) {
            Ok(pk) => pk,
            Err(_) => {
                return DurableNonceResponse {
                    is_valid: false,
                    nonce_account: req.nonce_account_pubkey.clone(),
                    nonce_authority: req.nonce_authority.clone(),
                    blockhash_expiry_prevented: false,
                    status: "INVALID_NONCE_AUTHORITY_PUBKEY".to_string(),
                    verification_notes: vec!["Nonce authority is not a valid Solana public key".to_string()],
                };
            }
        };

        notes.push(format!("Validated Nonce Account: {}", nonce_account));
        notes.push(format!("Validated Nonce Authority: {}", nonce_authority));

        // 3. Verify First Instruction is AdvanceNonceAccount
        if !req.is_advance_nonce_first_instruction {
            notes.push("CRITICAL ERROR: AdvanceNonceAccount must be the 1st instruction in transaction message".to_string());
            return DurableNonceResponse {
                is_valid: false,
                nonce_account: req.nonce_account_pubkey.clone(),
                nonce_authority: req.nonce_authority.clone(),
                blockhash_expiry_prevented: false,
                status: "MISSING_ADVANCE_NONCE_INSTRUCTION".to_string(),
                verification_notes: notes,
            };
        }

        notes.push("AdvanceNonceAccount verified as Instruction 0".to_string());
        notes.push("Durable Nonce active: Transaction protected from ~90s blockhash expiration during SOP Human Checkpoint delay".to_string());

        DurableNonceResponse {
            is_valid: true,
            nonce_account: req.nonce_account_pubkey.clone(),
            nonce_authority: req.nonce_authority.clone(),
            blockhash_expiry_prevented: true,
            status: "DURABLE_NONCE_VALIDATED".to_string(),
            verification_notes: notes,
        }
    }
}
