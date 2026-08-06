use serde::{Deserialize, Serialize};

/// Request parameters for verifying a Solana Pay invoice transaction on-chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceVerificationRequest {
    /// The unique reference public key generated for this specific order
    pub reference_key: String,
    /// The merchant's receiving wallet address
    pub expected_recipient: String,
    /// The required payment amount (in atomic units or floating-point UI amount)
    pub expected_amount: f64,
    /// Optional expected SPL Token Mint address (e.g. USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v)
    pub expected_spl_mint: Option<String>,
    /// Array of raw transaction signatures found for this reference key
    pub signatures: Vec<String>,
    /// Raw JSON payload from getTransaction or getParsedTransaction RPC call
    pub transaction_rpc_json: Option<String>,
}

/// Response returned after verifying an invoice payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceVerificationResponse {
    pub is_valid: bool,
    pub order_reference: String,
    pub verified_signature: Option<String>,
    pub sender_address: Option<String>,
    pub recipient_address: String,
    pub amount_received: f64,
    pub spl_mint_used: Option<String>,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub status: String,
    pub verification_notes: Vec<String>,
}

/// Request to inspect Token-2022 extensions and assess security/compliance risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token2022RiskRequest {
    pub mint_address: String,
    pub raw_account_data_base64: Option<String>,
    pub has_transfer_fee: bool,
    pub transfer_fee_basis_points: u16,
    pub maximum_fee: u64,
    pub has_transfer_hook: bool,
    pub transfer_hook_program_id: Option<String>,
    pub has_permanent_delegate: bool,
    pub permanent_delegate_address: Option<String>,
    pub is_non_transferable: bool,
    pub has_confidential_transfers: bool,
}

/// Risk assessment result returned by Token-2022 inspector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token2022RiskResponse {
    pub mint_address: String,
    pub risk_level: String, // "SAFE", "MEDIUM_RISK", "HIGH_RISK", "BLOCKED"
    pub risk_score: u8,     // 0 to 100
    pub allows_merchant_custody: bool,
    pub fee_deduction_estimate: f64,
    pub active_extensions: Vec<String>,
    pub security_warnings: Vec<String>,
}

/// Request to evaluate spend rules and transaction security before signing/executing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendFirewallRequest {
    pub recipient_address: String,
    pub amount_usdc: f64,
    pub token_mint: String,
    pub current_daily_spent_usdc: f64,
    pub max_daily_limit_usdc: f64,
    pub allowed_recipients: Vec<String>,
    pub allowed_mints: Vec<String>,
    pub requester_role: String,
}

/// Evaluation result from spend firewall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendFirewallResponse {
    pub approved: bool,
    pub reason: String,
    pub remaining_daily_quota_usdc: f64,
    pub requires_human_approval: bool,
    pub security_checks_passed: Vec<String>,
}

/// Request to verify Durable Nonce account protection against blockhash expiry (~90s).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableNonceRequest {
    pub nonce_account_pubkey: String,
    pub nonce_authority: String,
    pub is_advance_nonce_first_instruction: bool,
}

/// Response returned from Durable Nonce verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableNonceResponse {
    pub is_valid: bool,
    pub nonce_account: String,
    pub nonce_authority: String,
    pub blockhash_expiry_prevented: bool,
    pub status: String,
    pub verification_notes: Vec<String>,
}
