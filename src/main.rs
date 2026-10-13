use clap::{Parser, Subcommand};
use ezeval::fuzzer::BoundaryFuzzer;
use ezeval::guardrails::SafetyGuardrail;
use ezeval::runner::{EvalRunner, TestCase};
use serde_json::json;

#[derive(Parser)]
#[command(
    name = "ezeval",
    version = "0.1.0",
    about = "ezEval: High-Throughput Boundary Fuzzing & Safety Guardrails in Rust (EzLabs)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate synthetic edge-case and boundary fuzz mutations
    Fuzz {
        #[arg(long, default_value = "all")]
        category: String,
    },

    /// Inspect a string or payload for credential leaks and hazardous commands
    Check {
        /// Text or payload snippet to scan
        text: String,
    },

    /// Run synthetic evaluation benchmark suite
    Benchmark,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fuzz { category } => {
            let mutations = BoundaryFuzzer::generate_all_mutations();
            println!("=== ezEval Boundary Fuzzer (Rust v0.1.0) ===");
            println!(
                "Generated {} synthetic boundary mutations:\n",
                mutations.len()
            );
            for m in mutations {
                if category == "all" || m.category == category {
                    let val_str = m.value.to_string();
                    let preview = if val_str.chars().count() > 40 {
                        format!("{}...", val_str.chars().take(40).collect::<String>())
                    } else {
                        val_str
                    };
                    println!("[{}] {} ➔ {}", m.category.to_uppercase(), m.name, preview);
                    println!("  Description: {}", m.description);
                }
            }
            println!("\nNative Rust Fuzzer Ready for High-Throughput Ingestion.");
        }
        Commands::Check { text } => {
            let guard = SafetyGuardrail::new();
            let violations = guard.inspect(&text);
            let preview = if text.chars().count() > 80 {
                format!("{}...", text.chars().take(80).collect::<String>())
            } else {
                text
            };
            println!("=== ezEval Safety Scan (Rust v0.1.0) ===");
            println!("Target: {}", preview);
            if violations.is_empty() {
                println!("✅ Status: CLEAN. Zero safety guardrail violations detected.");
            } else {
                println!(
                    "🚨 Status: VIOLATIONS DETECTED ({} hits):",
                    violations.len()
                );
                for v in violations {
                    println!(
                        "  - [{:?}] {}: {} (Snippet: {})",
                        v.severity, v.rule_id, v.description, v.matched_snippet
                    );
                }
            }
        }
        Commands::Benchmark => {
            let runner = EvalRunner::new();
            let schema = json!({
                "type": "object",
                "properties": {
                    "username": { "type": "string", "minLength": 3 },
                    "age": { "type": "integer", "minimum": 0, "maximum": 120 }
                },
                "required": ["username", "age"],
                "additionalProperties": false
            });

            let test_cases = vec![
                TestCase {
                    id: "valid_admin".to_string(),
                    payload: json!({ "username": "admin_user", "age": 30 }),
                },
                TestCase {
                    id: "missing_age".to_string(),
                    payload: json!({ "username": "alice" }),
                },
                TestCase {
                    id: "negative_age".to_string(),
                    payload: json!({ "username": "bob", "age": -5 }),
                },
                TestCase {
                    id: "leaked_bearer".to_string(),
                    payload: json!({ "username": "charlie", "age": 25, "token": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.token" }),
                },
                TestCase {
                    id: "rm_command".to_string(),
                    payload: json!({ "username": "dave", "age": 40, "cmd": "rm -rf /" }),
                },
            ];

            let report = runner.run_suite("agent_user_tool_benchmark", test_cases, Some(schema));
            println!("{}", report.to_markdown_report());
        }
    }
}
