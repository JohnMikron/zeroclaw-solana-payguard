pub mod durable_nonce;
pub mod invoice_verifier;
pub mod spend_firewall;
pub mod token2022_inspector;
pub mod types;

pub use durable_nonce::DurableNonceVerifier;
pub use invoice_verifier::InvoiceVerifier;
pub use spend_firewall::SpendFirewall;
pub use token2022_inspector::Token2022Inspector;
pub use types::{
    DurableNonceRequest, InvoiceVerificationRequest, SpendFirewallRequest, Token2022RiskRequest,
};

#[cfg(target_family = "wasm")]
wit_bindgen::generate!({
    world: "zeroclaw-tool-plugin",
    path: "wit/v0",
});

#[cfg(target_family = "wasm")]
use exports::zeroclaw::plugin::tool::{
    Guest, ToolCallRequest, ToolCallResponse, ToolMetadata, ToolParameter,
};

#[cfg(target_family = "wasm")]
struct SolanaGuardPlugin;

#[cfg(target_family = "wasm")]
impl Guest for SolanaGuardPlugin {
    fn get_metadata() -> ToolMetadata {
        ToolMetadata {
            name: "zeroclaw_solana_guard".to_string(),
            description: "Solana Pay Invoice Verification, Token-2022 Extension Risk Inspector, Durable Nonce Verifier, and Spend Firewall".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "tool_name".to_string(),
                    parameter_type: "string".to_string(),
                    description: "Target tool: 'solana_verify_invoice', 'solana_token2022_inspect', 'solana_spend_firewall', or 'solana_verify_durable_nonce'".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "arguments_json".to_string(),
                    parameter_type: "string".to_string(),
                    description: "JSON string containing tool request parameters".to_string(),
                    required: true,
                },
            ],
        }
    }

    fn execute(request: ToolCallRequest) -> ToolCallResponse {
        match request.tool_name.as_str() {
            "solana_verify_invoice" => {
                match serde_json::from_str::<InvoiceVerificationRequest>(&request.arguments_json) {
                    Ok(req) => {
                        let res = InvoiceVerifier::verify_invoice(&req);
                        ToolCallResponse {
                            success: true,
                            output_json: serde_json::to_string(&res).unwrap_or_default(),
                            error_message: None,
                        }
                    }
                    Err(e) => ToolCallResponse {
                        success: false,
                        output_json: "".to_string(),
                        error_message: Some(format!("Invalid InvoiceVerificationRequest JSON: {}", e)),
                    },
                }
            }
            "solana_token2022_inspect" => {
                match serde_json::from_str::<Token2022RiskRequest>(&request.arguments_json) {
                    Ok(req) => {
                        let res = Token2022Inspector::inspect_risk(&req);
                        ToolCallResponse {
                            success: true,
                            output_json: serde_json::to_string(&res).unwrap_or_default(),
                            error_message: None,
                        }
                    }
                    Err(e) => ToolCallResponse {
                        success: false,
                        output_json: "".to_string(),
                        error_message: Some(format!("Invalid Token2022RiskRequest JSON: {}", e)),
                    },
                }
            }
            "solana_spend_firewall" => {
                match serde_json::from_str::<SpendFirewallRequest>(&request.arguments_json) {
                    Ok(req) => {
                        let res = SpendFirewall::evaluate(&req);
                        ToolCallResponse {
                            success: true,
                            output_json: serde_json::to_string(&res).unwrap_or_default(),
                            error_message: None,
                        }
                    }
                    Err(e) => ToolCallResponse {
                        success: false,
                        output_json: "".to_string(),
                        error_message: Some(format!("Invalid SpendFirewallRequest JSON: {}", e)),
                    },
                }
            }
            "solana_verify_durable_nonce" => {
                match serde_json::from_str::<DurableNonceRequest>(&request.arguments_json) {
                    Ok(req) => {
                        let res = DurableNonceVerifier::verify_nonce(&req);
                        ToolCallResponse {
                            success: true,
                            output_json: serde_json::to_string(&res).unwrap_or_default(),
                            error_message: None,
                        }
                    }
                    Err(e) => ToolCallResponse {
                        success: false,
                        output_json: "".to_string(),
                        error_message: Some(format!("Invalid DurableNonceRequest JSON: {}", e)),
                    },
                }
            }
            _ => ToolCallResponse {
                success: false,
                output_json: "".to_string(),
                error_message: Some(format!("Unknown tool name: {}", request.tool_name)),
            },
        }
    }
}

#[cfg(target_family = "wasm")]
export!(SolanaGuardPlugin);
