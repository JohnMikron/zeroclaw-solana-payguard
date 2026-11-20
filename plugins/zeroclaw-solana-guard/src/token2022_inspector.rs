use crate::types::{Token2022RiskRequest, Token2022RiskResponse};
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub struct Token2022Inspector;

impl Token2022Inspector {
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

    pub fn inspect_risk(req: &Token2022RiskRequest) -> Token2022RiskResponse {
        let mut warnings = Vec::new();
        let mut extensions = Vec::new();
        let mut risk_score: u8 = 0;
        let mut allows_merchant_custody = true;

        // 1. Validate Mint Public Key Syntax
        if Pubkey::from_str(&req.mint_address).is_err() {
            return Token2022RiskResponse {
                mint_address: req.mint_address.clone(),
                risk_level: "BLOCKED".to_string(),
                risk_score: 100,
                allows_merchant_custody: false,
                fee_deduction_estimate: 0.0,
                active_extensions: vec![],
                security_warnings: vec![format!(
                    "INVALID_MINT_ADDRESS: '{}' is not a valid Solana public key",
                    req.mint_address
                )],
            };
        }

        // 2. Real Binary Account Inspection if raw_account_data_base64 is provided
        let mut is_non_transferable = req.is_non_transferable;
        let mut has_permanent_delegate = req.has_permanent_delegate;
        let mut permanent_delegate_address = req.permanent_delegate_address.clone();
        let mut has_transfer_hook = req.has_transfer_hook;
        let mut transfer_hook_program_id = req.transfer_hook_program_id.clone();
        let mut has_transfer_fee = req.has_transfer_fee;
        let mut transfer_fee_basis_points = req.transfer_fee_basis_points;
        let mut has_confidential_transfers = req.has_confidential_transfers;

        if let Some(ref b64_str) = req.raw_account_data_base64 {
            let data = match Self::decode_base64(b64_str) {
                Ok(bytes) => bytes,
                Err(err) => {
                    return Token2022RiskResponse {
                        mint_address: req.mint_address.clone(),
                        risk_level: "BLOCKED".to_string(),
                        risk_score: 100,
                        allows_merchant_custody: false,
                        fee_deduction_estimate: 0.0,
                        active_extensions: vec![],
                        security_warnings: vec![format!(
                            "CORRUPT_ACCOUNT_DATA: Failed to decode base64 mint account data: {}",
                            err
                        )],
                    };
                }
            };

            if data.len() < 82 {
                return Token2022RiskResponse {
                    mint_address: req.mint_address.clone(),
                    risk_level: "BLOCKED".to_string(),
                    risk_score: 100,
                    allows_merchant_custody: false,
                    fee_deduction_estimate: 0.0,
                    active_extensions: vec![],
                    security_warnings: vec![format!(
                        "INVALID_ACCOUNT_SIZE: Mint account data must be at least 82 bytes, received {}",
                        data.len()
                    )],
                };
            }

            warnings.push("Performed real binary inspection of on-chain Token-2022 TLV account data".to_string());

            // Token-2022 Base mint is 82 bytes + 83 zero-padding = 165 bytes.
            // If data.len() > 165, check data[165] == 1 (AccountType::Mint) and start TLV loop at offset = 166.
            if data.len() > 165 {
                if data[165] != 1 {
                    return Token2022RiskResponse {
                        mint_address: req.mint_address.clone(),
                        risk_level: "BLOCKED".to_string(),
                        risk_score: 100,
                        allows_merchant_custody: false,
                        fee_deduction_estimate: 0.0,
                        active_extensions: vec![],
                        security_warnings: vec![format!(
                            "INVALID_ACCOUNT_TYPE: Expected AccountType::Mint (1) at byte 165, received {}",
                            data[165]
                        )],
                    };
                }

                let mut offset = 166; // TLV starts after AccountType::Mint at byte 165

                // Reset caller-supplied flags to inspect ground truth from binary data
                is_non_transferable = false;
                has_permanent_delegate = false;
                permanent_delegate_address = None;
                has_transfer_hook = false;
                transfer_hook_program_id = None;
                has_transfer_fee = false;
                transfer_fee_basis_points = 0;
                has_confidential_transfers = false;

                while offset + 4 <= data.len() {
                    let ext_type = u16::from_le_bytes([data[offset], data[offset + 1]]);
                    let ext_len = u16::from_le_bytes([data[offset + 2], data[offset + 3]]) as usize;
                    let val_start = offset + 4;
                    let val_end = val_start + ext_len;

                    if val_end > data.len() {
                        warnings.push("Truncated TLV extension entry detected in account data".to_string());
                        break;
                    }

                    let val_bytes = &data[val_start..val_end];

                    match ext_type {
                        // ExtensionType 9: NonTransferable
                        9 => {
                            is_non_transferable = true;
                        }
                        // ExtensionType 12: PermanentDelegate
                        12 => {
                            has_permanent_delegate = true;
                            if val_bytes.len() >= 32 {
                                let mut pk_bytes = [0u8; 32];
                                pk_bytes.copy_from_slice(&val_bytes[..32]);
                                permanent_delegate_address = Some(Pubkey::new_from_array(pk_bytes).to_string());
                            }
                        }
                        // ExtensionType 14: TransferHook
                        14 => {
                            has_transfer_hook = true;
                            if val_bytes.len() >= 64 {
                                let mut pk_bytes = [0u8; 32];
                                pk_bytes.copy_from_slice(&val_bytes[32..64]);
                                transfer_hook_program_id = Some(Pubkey::new_from_array(pk_bytes).to_string());
                            }
                        }
                        // ExtensionType 1: TransferFeeConfig
                        1 => {
                            has_transfer_fee = true;
                            if val_bytes.len() >= 108 {
                                // newer_transfer_fee.transfer_fee_basis_points at offset 106..108
                                transfer_fee_basis_points = u16::from_le_bytes([val_bytes[106], val_bytes[107]]);
                            } else if val_bytes.len() >= 90 {
                                // older_transfer_fee.transfer_fee_basis_points at offset 88..90
                                transfer_fee_basis_points = u16::from_le_bytes([val_bytes[88], val_bytes[89]]);
                            }
                        }
                        // ExtensionType 4 / 5: ConfidentialTransfers
                        4 | 5 => {
                            has_confidential_transfers = true;
                        }
                        _ => {}
                    }

                    offset = val_end;
                }
            }
        } else {
            warnings.push("NOTICE: Inspection based on caller parameters (no raw_account_data_base64 supplied)".to_string());
        }

        // 3. Evaluate Non-Transferable Extension
        if is_non_transferable {
            extensions.push("NonTransferable".to_string());
            warnings.push("CRITICAL: Token is Non-Transferable (Soulbound). Merchant cannot accept or move funds.".to_string());
            risk_score = risk_score.saturating_add(100);
            allows_merchant_custody = false;
        }

        // 4. Evaluate Permanent Delegate Extension
        if has_permanent_delegate {
            extensions.push("PermanentDelegate".to_string());
            if let Some(ref delegate) = permanent_delegate_address {
                if Pubkey::from_str(delegate).is_ok() {
                    warnings.push(format!(
                        "WARNING: Permanent Delegate verified ({}) — Issuer can burn or transfer tokens without merchant signature.",
                        delegate
                    ));
                } else {
                    warnings.push(format!("WARNING: Malformed Permanent Delegate address '{}'.", delegate));
                }
            } else {
                warnings.push("WARNING: Permanent Delegate enabled without specific address.".to_string());
            }
            risk_score = risk_score.saturating_add(40);
        }

        // 5. Evaluate Transfer Hook Extension
        if has_transfer_hook {
            extensions.push("TransferHook".to_string());
            if let Some(ref hook_pid) = transfer_hook_program_id {
                if Pubkey::from_str(hook_pid).is_ok() {
                    warnings.push(format!(
                        "SECURITY NOTICE: Custom Transfer Hook program attached ({}) — External code executes on every transfer.",
                        hook_pid
                    ));
                } else {
                    warnings.push(format!("SECURITY NOTICE: Malformed Transfer Hook program ID '{}'.", hook_pid));
                }
            } else {
                warnings.push("SECURITY NOTICE: Custom Transfer Hook program attached.".to_string());
            }
            risk_score = risk_score.saturating_add(30);
        }

        // 6. Evaluate Transfer Fee Extension
        let mut fee_estimate = 0.0;
        if has_transfer_fee {
            extensions.push("TransferFee".to_string());
            let basis_pts = transfer_fee_basis_points.min(10000); // Cap at 100% (10,000 bps)
            fee_estimate = (basis_pts as f64) / 100.0; // Percentage
            warnings.push(format!(
                "FEE DEDUCTION: Token-2022 Transfer Fee active ({} bps = {:.2}% fee per transfer).",
                basis_pts, fee_estimate
            ));
            if basis_pts > 500 {
                // > 5% fee is high risk
                risk_score = risk_score.saturating_add(35);
            } else {
                risk_score = risk_score.saturating_add(10);
            }
        }

        // 7. Evaluate Confidential Transfers Extension
        if has_confidential_transfers {
            extensions.push("ConfidentialTransfers".to_string());
            warnings.push("COMPLIANCE NOTE: Token uses Zero-Knowledge Confidential Transfers — requires compliance viewing key for audit.".to_string());
            risk_score = risk_score.saturating_add(15);
        }

        let risk_level = match risk_score {
            0..=15 => "SAFE",
            16..=45 => "MEDIUM_RISK",
            46..=75 => "HIGH_RISK",
            _ => "BLOCKED",
        }
        .to_string();

        Token2022RiskResponse {
            mint_address: req.mint_address.clone(),
            risk_level,
            risk_score: risk_score.min(100),
            allows_merchant_custody,
            fee_deduction_estimate: fee_estimate,
            active_extensions: extensions,
            security_warnings: warnings,
        }
    }
}
