use crate::types::{Token2022RiskRequest, Token2022RiskResponse};

pub struct Token2022Inspector;

impl Token2022Inspector {
    pub fn inspect_risk(req: &Token2022RiskRequest) -> Token2022RiskResponse {
        let mut warnings = Vec::new();
        let mut extensions = Vec::new();
        let mut risk_score: u8 = 0;
        let mut allows_merchant_custody = true;

        // 1. Check Non-transferable
        if req.is_non_transferable {
            extensions.push("NonTransferable".to_string());
            warnings.push("CRITICAL: Token is Non-Transferable (Soulbound). Merchant cannot accept or move funds.".to_string());
            risk_score = risk_score.saturating_add(100);
            allows_merchant_custody = false;
        }

        // 2. Check Permanent Delegate
        if req.has_permanent_delegate {
            extensions.push("PermanentDelegate".to_string());
            if let Some(ref delegate) = req.permanent_delegate_address {
                warnings.push(format!(
                    "WARNING: Permanent Delegate present ({}) — Issuer can burn or transfer tokens without merchant signature.",
                    delegate
                ));
            } else {
                warnings.push("WARNING: Permanent Delegate enabled.".to_string());
            }
            risk_score = risk_score.saturating_add(40);
        }

        // 3. Check Transfer Hook
        if req.has_transfer_hook {
            extensions.push("TransferHook".to_string());
            if let Some(ref hook_pid) = req.transfer_hook_program_id {
                warnings.push(format!(
                    "SECURITY NOTICE: Custom Transfer Hook program attached ({}) — External code executes on every transfer.",
                    hook_pid
                ));
            } else {
                warnings.push("SECURITY NOTICE: Custom Transfer Hook program attached.".to_string());
            }
            risk_score = risk_score.saturating_add(30);
        }

        // 4. Check Transfer Fee
        let mut fee_estimate = 0.0;
        if req.has_transfer_fee {
            extensions.push("TransferFee".to_string());
            let basis_pts = req.transfer_fee_basis_points;
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

        // 5. Check Confidential Transfers
        if req.has_confidential_transfers {
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
