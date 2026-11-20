use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

use zeroclaw_solana_guard::durable_nonce::DurableNonceVerifier;
use zeroclaw_solana_guard::invoice_verifier::InvoiceVerifier;
use zeroclaw_solana_guard::spend_firewall::SpendFirewall;
use zeroclaw_solana_guard::token2022_inspector::Token2022Inspector;
use zeroclaw_solana_guard::types::{
    DurableNonceRequest, InvoiceVerificationRequest, SpendFirewallRequest, Token2022RiskRequest,
};

fn print_usage_and_exit() -> ! {
    eprintln!(
        "Usage: payguard_cli --tool <name> [--input <json_or_path_or_dash>]\n\n\
         Supported tools:\n  \
           solana_verify_invoice (or verify_invoice)\n  \
           solana_token2022_inspect (or token2022_inspect)\n  \
           solana_spend_firewall (or spend_firewall)\n  \
           solana_verify_durable_nonce (or verify_durable_nonce)\n\n\
         Input:\n  \
           Passed via --input as raw JSON string, file path, '-' for stdin, or piped via stdin."
    );
    let err_json = serde_json::json!({
        "error": "Missing or invalid arguments. Usage: payguard_cli --tool <name> [--input <json_or_stdin>]"
    });
    println!("{}", err_json);
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut tool_name: Option<String> = None;
    let mut input_raw: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--tool" {
            if i + 1 < args.len() {
                tool_name = Some(args[i + 1].clone());
                i += 2;
                continue;
            } else {
                print_usage_and_exit();
            }
        } else if let Some(val) = arg.strip_prefix("--tool=") {
            tool_name = Some(val.to_string());
            i += 1;
            continue;
        } else if arg == "--input" {
            if i + 1 < args.len() {
                input_raw = Some(args[i + 1].clone());
                i += 2;
                continue;
            } else {
                print_usage_and_exit();
            }
        } else if let Some(val) = arg.strip_prefix("--input=") {
            input_raw = Some(val.to_string());
            i += 1;
            continue;
        } else if arg == "-h" || arg == "--help" {
            print_usage_and_exit();
        } else {
            // Positional fallback: if tool not set, first positional is tool
            if tool_name.is_none() {
                tool_name = Some(arg.clone());
            } else if input_raw.is_none() {
                input_raw = Some(arg.clone());
            }
            i += 1;
        }
    }

    let tool = match tool_name {
        Some(t) => t,
        None => print_usage_and_exit(),
    };

    // Resolve JSON input from argument, file, or stdin
    let input_str = match input_raw {
        Some(ref val) if val == "-" => {
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                let err = serde_json::json!({ "error": format!("Failed to read stdin: {}", e) });
                println!("{}", err);
                process::exit(1);
            }
            buf
        }
        Some(ref val) => {
            let p = Path::new(val);
            if p.is_file() {
                match fs::read_to_string(p) {
                    Ok(s) => s,
                    Err(e) => {
                        let err = serde_json::json!({ "error": format!("Failed to read input file '{}': {}", val, e) });
                        println!("{}", err);
                        process::exit(1);
                    }
                }
            } else {
                val.clone()
            }
        }
        None => {
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                let err = serde_json::json!({ "error": format!("Failed to read stdin: {}", e) });
                println!("{}", err);
                process::exit(1);
            }
            buf
        }
    };

    let result_json = match tool.as_str() {
        "solana_verify_invoice" | "verify_invoice" | "invoice_verifier" => {
            match serde_json::from_str::<InvoiceVerificationRequest>(&input_str) {
                Ok(req) => {
                    let res = InvoiceVerifier::verify_invoice(&req);
                    serde_json::to_string(&res).unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e))
                }
                Err(e) => {
                    let err = serde_json::json!({ "error": format!("Invalid InvoiceVerificationRequest JSON: {}", e) });
                    println!("{}", err);
                    process::exit(1);
                }
            }
        }
        "solana_token2022_inspect" | "token2022_inspect" | "token2022_inspector" => {
            match serde_json::from_str::<Token2022RiskRequest>(&input_str) {
                Ok(req) => {
                    let res = Token2022Inspector::inspect_risk(&req);
                    serde_json::to_string(&res).unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e))
                }
                Err(e) => {
                    let err = serde_json::json!({ "error": format!("Invalid Token2022RiskRequest JSON: {}", e) });
                    println!("{}", err);
                    process::exit(1);
                }
            }
        }
        "solana_spend_firewall" | "spend_firewall" => {
            match serde_json::from_str::<SpendFirewallRequest>(&input_str) {
                Ok(req) => {
                    let res = SpendFirewall::evaluate(&req);
                    serde_json::to_string(&res).unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e))
                }
                Err(e) => {
                    let err = serde_json::json!({ "error": format!("Invalid SpendFirewallRequest JSON: {}", e) });
                    println!("{}", err);
                    process::exit(1);
                }
            }
        }
        "solana_verify_durable_nonce" | "verify_durable_nonce" | "durable_nonce" => {
            match serde_json::from_str::<DurableNonceRequest>(&input_str) {
                Ok(req) => {
                    let res = DurableNonceVerifier::verify_nonce(&req);
                    serde_json::to_string(&res).unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e))
                }
                Err(e) => {
                    let err = serde_json::json!({ "error": format!("Invalid DurableNonceRequest JSON: {}", e) });
                    println!("{}", err);
                    process::exit(1);
                }
            }
        }
        _ => {
            let err = serde_json::json!({
                "error": format!("Unknown tool '{}'. Supported tools: solana_verify_invoice, solana_token2022_inspect, solana_spend_firewall, solana_verify_durable_nonce", tool)
            });
            println!("{}", err);
            process::exit(1);
        }
    };

    println!("{}", result_json);
}
