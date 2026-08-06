use crate::types::{SpendFirewallRequest, SpendFirewallResponse};
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub struct SpendFirewall;

impl SpendFirewall {
    pub fn evaluate(req: &SpendFirewallRequest) -> SpendFirewallResponse {
        let mut passed_checks = Vec::new();

        // 1. Verify Recipient Address Syntax
        if Pubkey::from_str(&req.recipient_address).is_err() {
            return SpendFirewallResponse {
                approved: false,
                reason: format!("INVALID_RECIPIENT_ADDRESS: '{}' is not a valid Solana public key", req.recipient_address),
                remaining_daily_quota_usdc: req.max_daily_limit_usdc - req.current_daily_spent_usdc,
                requires_human_approval: true,
                security_checks_passed: vec![],
            };
        }
        passed_checks.push("Recipient Public Key Syntax Validated".to_string());

        // 2. Check Allowed Mints
        if !req.allowed_mints.contains(&req.token_mint) && !req.allowed_mints.contains(&"*".to_string()) {
            return SpendFirewallResponse {
                approved: false,
                reason: format!("UNALLOWED_MINT: Token mint '{}' is not on approved merchant list", req.token_mint),
                remaining_daily_quota_usdc: req.max_daily_limit_usdc - req.current_daily_spent_usdc,
                requires_human_approval: true,
                security_checks_passed: passed_checks,
            };
        }
        passed_checks.push("Token Mint Allowlist Verified".to_string());

        // 3. Check Daily Spend Cap
        let proposed_total = req.current_daily_spent_usdc + req.amount_usdc;
        let remaining_quota = (req.max_daily_limit_usdc - req.current_daily_spent_usdc).max(0.0);

        if proposed_total > req.max_daily_limit_usdc {
            return SpendFirewallResponse {
                approved: false,
                reason: format!(
                    "DAILY_SPEND_CAP_EXCEEDED: Transaction of {:.2} USDC exceeds remaining daily quota of {:.2} USDC (Cap: {:.2} USDC)",
                    req.amount_usdc, remaining_quota, req.max_daily_limit_usdc
                ),
                remaining_daily_quota_usdc: remaining_quota,
                requires_human_approval: true,
                security_checks_passed: passed_checks,
            };
        }
        passed_checks.push(format!(
            "Daily Spend Limit Check Passed ({:.2} / {:.2} USDC remaining)",
            remaining_quota - req.amount_usdc, req.max_daily_limit_usdc
        ));

        // 4. Check Recipient Allowlist & Human Approval Checkpoint
        let is_whitelisted = req.allowed_recipients.contains(&req.recipient_address)
            || req.allowed_recipients.contains(&"*".to_string());

        let requires_approval = if !is_whitelisted {
            true
        } else if req.amount_usdc > 100.0 {
            // Any single transaction > 100 USDC requires human checkpoint approval
            true
        } else {
            false
        };

        if !is_whitelisted {
            passed_checks.push("UNRECOGNIZED_DESTINATION: Destination requires Store Owner SOP Approval Gate".to_string());
        } else {
            passed_checks.push("Destination Address Whitelist Verified".to_string());
        }

        let remaining_after = remaining_quota - req.amount_usdc;

        let is_approved = !requires_approval;

        SpendFirewallResponse {
            approved: is_approved,
            reason: if requires_approval {
                "REQUIRES_HUMAN_APPROVAL_GATE: High value or unwhitelisted recipient trigger ZeroClaw SOP Checkpoint".to_string()
            } else {
                "TRANSACTION_APPROVED: All firewall checks passed".to_string()
            },
            remaining_daily_quota_usdc: remaining_after,
            requires_human_approval: requires_approval,
            security_checks_passed: passed_checks,
        }
    }
}
