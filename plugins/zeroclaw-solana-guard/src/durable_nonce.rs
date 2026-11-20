use crate::types::{DurableNonceRequest, DurableNonceResponse};
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub struct DurableNonceVerifier;

impl DurableNonceVerifier {
    /// Decodes standard Base64 string into bytes without external crates.
    fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
        let input = input.trim();
        let mut out = Vec::new();
        let mut buf: u32 = 0;
        let mut bits = 0;

        for &b in input.as_bytes() {
            let val = match b {
                b'A'..=b'Z' => b - b'A',
                b'a'..=b'z' => b - b'a' + 26,
                b'0'..=b'9' => b - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' => continue,
                b' ' | b'\n' | b'\r' | b'\t' => continue,
                _ => return Err(format!("Invalid base64 byte: 0x{:02x}", b)),
            };
            buf = (buf << 6) | (val as u32);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((buf >> bits) as u8);
                buf &= (1 << bits) - 1;
            }
        }
        Ok(out)
    }

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

        if nonce_account == nonce_authority {
            return DurableNonceResponse {
                is_valid: false,
                nonce_account: req.nonce_account_pubkey.clone(),
                nonce_authority: req.nonce_authority.clone(),
                blockhash_expiry_prevented: false,
                status: "INVALID_NONCE_CONFIGURATION".to_string(),
                verification_notes: vec!["Nonce account cannot be identical to its nonce authority".to_string()],
            };
        }

        notes.push(format!("Validated Nonce Account: {}", nonce_account));
        notes.push(format!("Validated Nonce Authority: {}", nonce_authority));

        // 3. Inspect On-Chain Raw Nonce Account Data (if provided)
        if let Some(ref b64_data) = req.raw_nonce_account_data_base64 {
            match Self::decode_base64(b64_data) {
                Ok(bytes) => {
                    if bytes.len() < 80 {
                        notes.push(format!(
                            "Invalid nonce account size: expected at least 80 bytes, received {}",
                            bytes.len()
                        ));
                        return DurableNonceResponse {
                            is_valid: false,
                            nonce_account: req.nonce_account_pubkey.clone(),
                            nonce_authority: req.nonce_authority.clone(),
                            blockhash_expiry_prevented: false,
                            status: "INVALID_NONCE_ACCOUNT_SIZE".to_string(),
                            verification_notes: notes,
                        };
                    }

                    // Solana Nonce Account layout:
                    // 0..4: version (u32)
                    // 4..8: state (u32: 0=Uninitialized, 1=Initialized)
                    // 8..40: authority (32 bytes)
                    // 40..72: blockhash (32 bytes)
                    // 72..80: fee_calculator (8 bytes)
                    let state = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                    if state != 1 {
                        notes.push(format!("Nonce account is not initialized (state={})", state));
                        return DurableNonceResponse {
                            is_valid: false,
                            nonce_account: req.nonce_account_pubkey.clone(),
                            nonce_authority: req.nonce_authority.clone(),
                            blockhash_expiry_prevented: false,
                            status: "NONCE_ACCOUNT_NOT_INITIALIZED".to_string(),
                            verification_notes: notes,
                        };
                    }

                    let mut auth_bytes = [0u8; 32];
                    auth_bytes.copy_from_slice(&bytes[8..40]);
                    let onchain_authority = Pubkey::new_from_array(auth_bytes).to_string();

                    if onchain_authority != req.nonce_authority {
                        notes.push(format!(
                            "Nonce authority mismatch: expected {}, on-chain account has {}",
                            req.nonce_authority, onchain_authority
                        ));
                        return DurableNonceResponse {
                            is_valid: false,
                            nonce_account: req.nonce_account_pubkey.clone(),
                            nonce_authority: req.nonce_authority.clone(),
                            blockhash_expiry_prevented: false,
                            status: "NONCE_AUTHORITY_MISMATCH".to_string(),
                            verification_notes: notes,
                        };
                    }

                    notes.push("On-chain Nonce Account state and authority verified against binary payload".to_string());
                }
                Err(err) => {
                    notes.push(format!("Failed to decode base64 nonce account data: {}", err));
                    return DurableNonceResponse {
                        is_valid: false,
                        nonce_account: req.nonce_account_pubkey.clone(),
                        nonce_authority: req.nonce_authority.clone(),
                        blockhash_expiry_prevented: false,
                        status: "INVALID_NONCE_ACCOUNT_DATA".to_string(),
                        verification_notes: notes,
                    };
                }
            }
        }

        // 4. Real Transaction Payload Inspection for Instruction 0 AdvanceNonceAccount
        if let Some(ref tx_json_str) = req.transaction_rpc_json {
            let parsed_tx: Result<serde_json::Value, _> = serde_json::from_str(tx_json_str);
            match parsed_tx {
                Ok(parsed) => {
                    let mut advance_nonce_verified = false;

                    // Extract accountKeys to resolve indices
                    let mut account_keys = Vec::new();
                    if let Some(keys) = parsed.pointer("/transaction/message/accountKeys").and_then(|k| k.as_array()) {
                        for k in keys {
                            if let Some(s) = k.as_str() {
                                account_keys.push(s.to_string());
                            } else if let Some(pk) = k.get("pubkey").and_then(|p| p.as_str()) {
                                account_keys.push(pk.to_string());
                            }
                        }
                    }

                    // Inspect Instruction 0
                    if let Some(ix0) = parsed.pointer("/transaction/message/instructions/0") {
                        // Check parsed format: {"program": "system", "parsed": {"type": "advanceNonce", "info": {...}}}
                        if let Some(program) = ix0.get("program").and_then(|p| p.as_str()) {
                            if program == "system" {
                                if let Some(ix_type) = ix0.pointer("/parsed/type").and_then(|t| t.as_str()) {
                                    if ix_type == "advanceNonce" {
                                        let nonce_acc = ix0.pointer("/parsed/info/nonceAccount").and_then(|a| a.as_str());
                                        if nonce_acc == Some(&req.nonce_account_pubkey) {
                                            advance_nonce_verified = true;
                                        }
                                    }
                                }
                            }
                        }

                        // Check compiled/raw format: {"programIdIndex": X, "accounts": [nonce_idx, ...], "data": ...}
                        if !advance_nonce_verified {
                            let system_pid = "11111111111111111111111111111111";
                            let pid_idx_opt = ix0.get("programIdIndex").and_then(|i| i.as_u64());
                            let pid_matches = if let Some(pid_idx) = pid_idx_opt {
                                account_keys.get(pid_idx as usize).map(|s| s.as_str()) == Some(system_pid)
                            } else if let Some(pid) = ix0.get("programId").and_then(|p| p.as_str()) {
                                pid == system_pid
                            } else {
                                ix0.get("program").and_then(|p| p.as_str()) == Some("system")
                            };

                            if pid_matches {
                                if let Some(acc_indices) = ix0.get("accounts").and_then(|a| a.as_array()) {
                                    if let Some(first_acc_idx) = acc_indices.first().and_then(|i| i.as_u64()) {
                                        if let Some(first_key) = account_keys.get(first_acc_idx as usize) {
                                            if first_key == &req.nonce_account_pubkey {
                                                // Verify instruction data u32 LE == 4 (SystemInstruction::AdvanceNonceAccount)
                                                let data_valid = if let Some(data_str) = ix0.get("data").and_then(|d| d.as_str()) {
                                                    let bytes = bs58::decode(data_str).into_vec().ok()
                                                        .or_else(|| Self::decode_base64(data_str).ok());
                                                    match bytes {
                                                        Some(b) if b.len() >= 4 => {
                                                            u32::from_le_bytes([b[0], b[1], b[2], b[3]]) == 4
                                                        }
                                                        _ => false,
                                                    }
                                                } else if let Some(arr) = ix0.get("data").and_then(|d| d.as_array()) {
                                                    if arr.len() >= 4 {
                                                        let b0 = arr[0].as_u64().unwrap_or(255) as u8;
                                                        let b1 = arr[1].as_u64().unwrap_or(255) as u8;
                                                        let b2 = arr[2].as_u64().unwrap_or(255) as u8;
                                                        let b3 = arr[3].as_u64().unwrap_or(255) as u8;
                                                        u32::from_le_bytes([b0, b1, b2, b3]) == 4
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    false
                                                };

                                                if data_valid {
                                                    advance_nonce_verified = true;
                                                } else {
                                                    notes.push("Instruction 0 data does not match SystemInstruction::AdvanceNonceAccount (u32 LE == 4)".to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !advance_nonce_verified {
                        notes.push(
                            "CRITICAL ERROR: Transaction payload inspection failed: Instruction 0 is NOT AdvanceNonceAccount for this nonce account".to_string()
                        );
                        return DurableNonceResponse {
                            is_valid: false,
                            nonce_account: req.nonce_account_pubkey.clone(),
                            nonce_authority: req.nonce_authority.clone(),
                            blockhash_expiry_prevented: false,
                            status: "MISSING_ADVANCE_NONCE_INSTRUCTION".to_string(),
                            verification_notes: notes,
                        };
                    }
                    notes.push("AdvanceNonceAccount verified as Instruction 0 from transaction message".to_string());
                }
                Err(e) => {
                    notes.push(format!("Failed to parse transaction RPC JSON: {}", e));
                    return DurableNonceResponse {
                        is_valid: false,
                        nonce_account: req.nonce_account_pubkey.clone(),
                        nonce_authority: req.nonce_authority.clone(),
                        blockhash_expiry_prevented: false,
                        status: "RPC_PARSE_ERROR".to_string(),
                        verification_notes: notes,
                    };
                }
            }
        } else {
            // Fallback: Validate caller assertion if no transaction payload provided
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
            notes.push("AdvanceNonceAccount asserted by caller (no raw transaction payload provided)".to_string());
        }

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
