//! GVPIe Development Assistant CLI
//!
//! A command-line tool that provides AI-powered development assistance for GVPIe.
//!
//! Usage:
//!   cargo run --bin gvpie_dev_assistant analyze
//!   cargo run --bin gvpie_dev_assistant suggest --files src/gpu/mod.rs
//!   cargo run --bin gvpie_dev_assistant assist

use ai_runtime::AiRuntime;
use std::path::PathBuf;

#[derive(Debug)]
enum Command {
    Analyze,
    Suggest { files: Vec<String> },
    Assist,
    Component { path: String },
    Predict { changes: Vec<String> },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(false)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let command = parse_args(&args)?;

    println!("🤖 GVPIe AI Development Assistant");
    println!("==================================");

    // Initialize AI Runtime
    let runtime = AiRuntime::new().await?;

    match command {
        Command::Analyze => {
            println!("📊 Analyzing entire GVPIe codebase...");
            let report = runtime.analyze_gvpie_codebase().await?;

            println!("\n✅ Analysis Complete!");
            println!("📈 Scores:");
            println!(
                "  • Architecture: {:.1}%",
                report.architecture_analysis.modularity_score * 100.0
            );
            println!(
                "  • GPU Utilization: {:.1}%",
                report.gpu_analysis.gpu_utilization_score * 100.0
            );
            println!(
                "  • Pixel VM: {:.1}%",
                report.pixel_vm_analysis.vm_performance_score * 100.0
            );

            println!("\n🔧 Top Optimizations:");
            for (i, suggestion) in report.optimization_suggestions.iter().take(5).enumerate() {
                println!(
                    "  {}. [{}] {}",
                    i + 1,
                    format!("{:?}", suggestion.priority),
                    suggestion.description
                );
            }

            if !report.security_findings.is_empty() {
                println!("\n🛡️  Security Findings:");
                for finding in report.security_findings.iter().take(3) {
                    println!(
                        "  • [{}] {}",
                        format!("{:?}", finding.severity),
                        finding.description
                    );
                }
            }
        }

        Command::Suggest { files } => {
            println!("💡 Getting suggestions for {} files...", files.len());
            let paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
            let suggestions = runtime.suggest_gvpie_improvements(&paths).await?;

            println!("\n✅ Generated {} suggestions:", suggestions.len());
            for (i, suggestion) in suggestions.iter().enumerate() {
                println!(
                    "  {}. [{}] {}",
                    i + 1,
                    format!("{:?}", suggestion.priority),
                    suggestion.description
                );

                if let Some(code) = &suggestion.suggested_code {
                    println!("     💻 Suggested code:");
                    for line in code.lines().take(2) {
                        println!("        {}", line.trim());
                    }
                    if code.lines().count() > 2 {
                        println!("        ...");
                    }
                }
            }
        }

        Command::Assist => {
            println!("🧠 Getting comprehensive development assistance...");
            let assistance = runtime.get_gvpie_development_assistance().await?;

            println!("\n📋 Development Recommendations:");
            for (i, rec) in assistance.recommendations.iter().enumerate() {
                println!("  {}. [{}] {}", i + 1, rec.priority, rec.title);
                println!("     📖 {}", rec.description);
                println!(
                    "     ⏱️  {} | 🎯 {}",
                    rec.estimated_effort, rec.expected_impact
                );
            }

            println!("\n🎯 Next Actions:");
            for (i, action) in assistance.next_actions.iter().enumerate() {
                println!(
                    "  {}. {} ({})",
                    i + 1,
                    action.description,
                    action.estimated_time
                );
                if let Some(command) = &action.command {
                    println!("     💻 {}", command);
                }
            }
        }

        Command::Component { path } => {
            println!("🔍 Analyzing component: {}", path);
            let report = runtime.analyze_gvpie_component(&path).await?;

            println!("\n✅ Component Analysis Complete!");
            println!("📊 Component Scores:");
            println!(
                "  • Architecture: {:.1}%",
                report.architecture_analysis.modularity_score * 100.0
            );
            println!(
                "  • GPU Performance: {:.1}%",
                report.gpu_analysis.compute_shader_efficiency * 100.0
            );

            if !report.optimization_suggestions.is_empty() {
                println!("\n🔧 Component-Specific Optimizations:");
                for (i, suggestion) in report.optimization_suggestions.iter().take(3).enumerate() {
                    println!("  {}. {}", i + 1, suggestion.description);
                }
            }
        }

        Command::Predict { changes } => {
            println!(
                "⚡ Predicting performance impact of {} changes...",
                changes.len()
            );
            let insights = runtime.predict_gvpie_performance_impact(&changes).await?;

            println!("\n📈 Performance Predictions:");
            println!(
                "  • GPU/CPU Balance: {:.1}%",
                insights.gpu_cpu_balance * 100.0
            );
            println!(
                "  • Scaling (1K users): {:.1}%",
                insights.predicted_scalability.predicted_1k_users
            );
            println!(
                "  • Scaling (10K users): {:.1}%",
                insights.predicted_scalability.predicted_10k_users
            );

            if !insights
                .predicted_scalability
                .scaling_bottlenecks
                .is_empty()
            {
                println!("\n🚨 Potential Bottlenecks:");
                for bottleneck in &insights.predicted_scalability.scaling_bottlenecks {
                    println!("  • {}", bottleneck);
                }
            }
        }
    }

    println!("\n🎉 Analysis complete! Use the insights to accelerate your GVPIe development.");
    Ok(())
}

fn parse_args(args: &[String]) -> Result<Command, Box<dyn std::error::Error>> {
    if args.len() < 2 {
        return Err("Usage: gvpie_dev_assistant <command> [options]".into());
    }

    match args[1].as_str() {
        "analyze" => Ok(Command::Analyze),
        "suggest" => {
            let files = if args.len() > 3 && args[2] == "--files" {
                args[3..].to_vec()
            } else {
                vec!["src/".to_string()] // Default to src directory
            };
            Ok(Command::Suggest { files })
        }
        "assist" => Ok(Command::Assist),
        "component" => {
            if args.len() < 3 {
                return Err("Usage: gvpie_dev_assistant component <path>".into());
            }
            Ok(Command::Component {
                path: args[2].clone(),
            })
        }
        "predict" => {
            let changes = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                vec!["General optimizations".to_string()]
            };
            Ok(Command::Predict { changes })
        }
        _ => Err(format!(
            "Unknown command: {}. Available: analyze, suggest, assist, component, predict",
            args[1]
        )
        .into()),
    }
}
