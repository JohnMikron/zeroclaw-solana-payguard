use crate::types::{SpendFirewallRequest, SpendFirewallResponse};
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub struct SpendFirewall;

impl SpendFirewall {
    /// Hardware/WASM-enforced maximum daily spend limit ceiling (500.0 USDC).
    /// Prevents prompt-injection attacks from inflating caller-supplied limits.
    pub const HARDCODED_MAX_DAILY_LIMIT_USDC: f64 = 500.0;

    /// Single transaction threshold above which human checkpoint approval is mandatory.
    pub const SINGLE_TRANSACTION_APPROVAL_THRESHOLD_USDC: f64 = 100.0;

    pub fn evaluate(req: &SpendFirewallRequest) -> SpendFirewallResponse {
        let mut passed_checks = Vec::new();

        // 1. Validate Transaction Amount (Must be finite, strictly positive, no NaN or Infinity)
        if !req.amount_usdc.is_finite() || req.amount_usdc <= 0.0 {
            return SpendFirewallResponse {
                approved: false,
                hard_blocked: true,
                reason: format!(
                    "INVALID_AMOUNT: Transaction amount must be a positive finite number (received: {})",
                    req.amount_usdc
                ),
                remaining_daily_quota_usdc: 0.0,
                requires_human_approval: false,
                security_checks_passed: vec![],
            };
        }

        // Validate Current Daily Spent (Must be finite, non-negative)
        if !req.current_daily_spent_usdc.is_finite() || req.current_daily_spent_usdc < 0.0 {
            return SpendFirewallResponse {
                approved: false,
                hard_blocked: true,
                reason: format!(
                    "INVALID_DAILY_SPENT: Current daily spent must be a non-negative finite number (received: {})",
                    req.current_daily_spent_usdc
                ),
                remaining_daily_quota_usdc: 0.0,
                requires_human_approval: false,
                security_checks_passed: vec![],
            };
        }
        passed_checks.push("Transaction Amount Numeric Validity Checked".to_string());

        // 2. Verify Recipient Address Syntax
        if Pubkey::from_str(&req.recipient_address).is_err() {
            let limit = if req.max_daily_limit_usdc.is_finite() && req.max_daily_limit_usdc > 0.0 {
                req.max_daily_limit_usdc.min(Self::HARDCODED_MAX_DAILY_LIMIT_USDC)
            } else {
                Self::HARDCODED_MAX_DAILY_LIMIT_USDC
            };
            return SpendFirewallResponse {
                approved: false,
                hard_blocked: true,
                reason: format!(
                    "INVALID_RECIPIENT_ADDRESS: '{}' is not a valid Solana public key",
                    req.recipient_address
                ),
                remaining_daily_quota_usdc: (limit - req.current_daily_spent_usdc).max(0.0),
                requires_human_approval: false,
                security_checks_passed: passed_checks,
            };
        }
        passed_checks.push("Recipient Public Key Syntax Validated".to_string());

        // 3. Clean up and Sanitize Requester Role
        let cleaned_role = req.requester_role.trim().to_lowercase();
        let recognized_roles = ["merchant_agent", "store_owner", "operator", "system", "admin"];
        let mut role_requires_approval = false;

        if cleaned_role.is_empty() {
            role_requires_approval = true;
            passed_checks.push("Requester role is empty: defaulting to untrusted approval gate".to_string());
        } else if !recognized_roles.contains(&cleaned_role.as_str()) {
            role_requires_approval = true;
            passed_checks.push(format!(
                "UNVERIFIED_REQUESTER_ROLE: Role '{}' requires human checkpoint verification",
                req.requester_role
            ));
        } else {
            passed_checks.push(format!("Requester Role Validated: '{}'", cleaned_role));
        }

        // 4. Check Allowed Mints (Strict Allowlist matching, reject wildcard bypass)
        let is_mint_allowed = req
            .allowed_mints
            .iter()
            .any(|m| m != "*" && m == &req.token_mint);

        let effective_daily_limit = if req.max_daily_limit_usdc.is_finite() && req.max_daily_limit_usdc > 0.0 {
            req.max_daily_limit_usdc.min(Self::HARDCODED_MAX_DAILY_LIMIT_USDC)
        } else {
            Self::HARDCODED_MAX_DAILY_LIMIT_USDC
        };

        if req.max_daily_limit_usdc > Self::HARDCODED_MAX_DAILY_LIMIT_USDC {
            passed_checks.push(format!(
                "WASM Guard: Enforced hardcoded daily spend cap ceiling of {:.2} USDC (caller requested {:.2} USDC)",
                Self::HARDCODED_MAX_DAILY_LIMIT_USDC,
                req.max_daily_limit_usdc
            ));
        }

        let remaining_quota = (effective_daily_limit - req.current_daily_spent_usdc).max(0.0);

        if !is_mint_allowed {
            return SpendFirewallResponse {
                approved: false,
                hard_blocked: true,
                reason: format!(
                    "UNALLOWED_MINT: Token mint '{}' is not on approved merchant list",
                    req.token_mint
                ),
                remaining_daily_quota_usdc: remaining_quota,
                requires_human_approval: false,
                security_checks_passed: passed_checks,
            };
        }
        passed_checks.push("Token Mint Allowlist Verified".to_string());

        // 5. Check Daily Spend Cap against Effective (Hardcoded-Bounded) Daily Limit
        let proposed_total = req.current_daily_spent_usdc + req.amount_usdc;

        if proposed_total > effective_daily_limit {
            return SpendFirewallResponse {
                approved: false,
                hard_blocked: true,
                reason: format!(
                    "DAILY_SPEND_CAP_EXCEEDED: Transaction of {:.2} USDC exceeds remaining daily quota of {:.2} USDC (Cap: {:.2} USDC)",
                    req.amount_usdc, remaining_quota, effective_daily_limit
                ),
                remaining_daily_quota_usdc: remaining_quota,
                requires_human_approval: false,
                security_checks_passed: passed_checks,
            };
        }
        passed_checks.push(format!(
            "Daily Spend Limit Check Passed ({:.2} / {:.2} USDC remaining)",
            remaining_quota - req.amount_usdc,
            effective_daily_limit
        ));

        // 6. Check Recipient Allowlist & Human Approval Checkpoint
        // REMOVED WILDCARD '*' BYPASS: Wildcard entries in allowed_recipients are strictly ignored
        let is_whitelisted = req
            .allowed_recipients
            .iter()
            .any(|r| r != "*" && r == &req.recipient_address);

        let mut requires_approval = role_requires_approval;

        if !is_whitelisted {
            requires_approval = true;
            passed_checks.push("UNRECOGNIZED_DESTINATION: Destination requires Store Owner SOP Approval Gate".to_string());
        } else {
            passed_checks.push("Destination Address Whitelist Verified".to_string());
            if req.amount_usdc > Self::SINGLE_TRANSACTION_APPROVAL_THRESHOLD_USDC {
                // Any single transaction > 100 USDC requires human checkpoint approval
                requires_approval = true;
                passed_checks.push(format!(
                    "HIGH_VALUE_TRANSACTION: Amount {:.2} USDC exceeds {:.2} USDC checkpoint threshold",
                    req.amount_usdc, Self::SINGLE_TRANSACTION_APPROVAL_THRESHOLD_USDC
                ));
            }
        }

        let remaining_after = (remaining_quota - req.amount_usdc).max(0.0);
        let is_approved = !requires_approval;

        SpendFirewallResponse {
            approved: is_approved,
            hard_blocked: false,
            reason: if requires_approval {
                "REQUIRES_HUMAN_APPROVAL_GATE: High value, unwhitelisted recipient, or unverified role triggers ZeroClaw SOP Checkpoint".to_string()
            } else {
                "TRANSACTION_APPROVED: All firewall checks passed".to_string()
            },
            remaining_daily_quota_usdc: remaining_after,
            requires_human_approval: requires_approval,
            security_checks_passed: passed_checks,
        }
    }
}
