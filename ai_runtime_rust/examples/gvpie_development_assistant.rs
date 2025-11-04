//! GVPIe Development Assistant Example
//!
//! This example demonstrates how to use the AI Runtime as a co-developer for GVPIe,
//! providing real-time code analysis, optimization suggestions, and development assistance.

use ai_runtime::{AiRuntime, GvpieDevelopmentAssistance};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    println!("🚀 Starting GVPIe Development Assistant");
    println!("=====================================");

    // Initialize AI Runtime
    let runtime = AiRuntime::new().await?;
    println!("✅ AI Runtime initialized");

    // Demonstrate comprehensive codebase analysis
    println!("\n📊 Analyzing GVPIe Codebase...");
    let analysis_report = runtime.analyze_gvpie_codebase().await?;

    println!("📈 Analysis Results:");
    println!(
        "  • Architecture Score: {:.1}%",
        analysis_report.architecture_analysis.modularity_score * 100.0
    );
    println!(
        "  • GPU Utilization: {:.1}%",
        analysis_report.gpu_analysis.gpu_utilization_score * 100.0
    );
    println!(
        "  • Pixel VM Performance: {:.1}%",
        analysis_report.pixel_vm_analysis.vm_performance_score * 100.0
    );
    println!(
        "  • Optimization Opportunities: {}",
        analysis_report.optimization_suggestions.len()
    );
    println!(
        "  • Security Findings: {}",
        analysis_report.security_findings.len()
    );

    // Show top optimization suggestions
    println!("\n🔧 Top Optimization Suggestions:");
    for (i, suggestion) in analysis_report
        .optimization_suggestions
        .iter()
        .take(3)
        .enumerate()
    {
        println!(
            "  {}. [{}] {} - {}",
            i + 1,
            format!("{:?}", suggestion.priority),
            format!("{:?}", suggestion.category),
            suggestion.description
        );
        if let Some(code) = &suggestion.suggested_code {
            println!("     💡 Suggested implementation:");
            for line in code.lines().take(3) {
                println!("        {}", line.trim());
            }
            if code.lines().count() > 3 {
                println!("        ...");
            }
        }
    }

    // Demonstrate component-specific analysis
    println!("\n🔍 Analyzing GPU Components...");
    let gpu_analysis = runtime
        .analyze_gvpie_component("gvpie-core/src/gpu")
        .await?;
    println!(
        "  • GPU Component Score: {:.1}%",
        gpu_analysis.gpu_analysis.compute_shader_efficiency * 100.0
    );

    // Demonstrate change-based suggestions
    println!("\n📝 Simulating File Changes...");
    let changed_files = vec![
        PathBuf::from("gvpie-core/src/gpu/scheduler.rs"),
        PathBuf::from("gvpie-core/src/pixel_language/executor.rs"),
    ];

    let suggestions = runtime.suggest_gvpie_improvements(&changed_files).await?;
    println!(
        "  • Generated {} suggestions for changed files",
        suggestions.len()
    );

    for suggestion in suggestions.iter().take(2) {
        println!(
            "    - {}: {}",
            format!("{:?}", suggestion.category),
            suggestion.description
        );
    }

    // Demonstrate performance prediction
    println!("\n⚡ Predicting Performance Impact...");
    let changes = vec![
        "Optimized GPU workgroup sizes".to_string(),
        "Implemented instruction fusion in Pixel VM".to_string(),
    ];

    let performance_insights = runtime.predict_gvpie_performance_impact(&changes).await?;
    println!(
        "  • Predicted GPU/CPU Balance: {:.1}%",
        performance_insights.gpu_cpu_balance * 100.0
    );
    println!(
        "  • Performance Hotspots: {}",
        performance_insights.hotspots.len()
    );
    println!(
        "  • Scaling Prediction (1K users): {:.1}%",
        performance_insights
            .predicted_scalability
            .predicted_1k_users
    );

    // Get comprehensive development assistance
    println!("\n🧠 Getting AI Development Assistance...");
    let assistance = runtime.get_gvpie_development_assistance().await?;

    println!("📋 Development Recommendations:");
    for (i, rec) in assistance.recommendations.iter().take(3).enumerate() {
        println!("  {}. [{}] {}", i + 1, rec.priority, rec.title);
        println!("     📖 {}", rec.description);
        println!(
            "     ⏱️  Effort: {} | Impact: {}",
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
            println!("     💻 Command: {}", command);
        }
    }

    // Demonstrate real-time monitoring simulation
    println!("\n🔄 Simulating Real-time Development Monitoring...");
    for i in 1..=3 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        println!("  [{}] Monitoring development session... ✓", i);

        if i == 2 {
            println!("  🚨 Detected potential performance regression in GPU scheduler");
            println!("  💡 Suggestion: Run GPU benchmarks to validate changes");
        }
    }

    println!("\n✨ GVPIe Development Assistant Demo Complete!");
    println!("🎉 The AI Runtime is ready to accelerate your GVPIe development!");

    println!("\n📚 Available API Endpoints:");
    println!("  • GET  /api/gvpie/analyze - Full codebase analysis");
    println!("  • GET  /api/gvpie/analyze/:component - Component analysis");
    println!("  • POST /api/gvpie/suggestions - Get optimization suggestions");
    println!("  • GET  /api/gvpie/assistance - Comprehensive development assistance");
    println!("  • POST /api/gvpie/predict-performance - Performance impact prediction");

    Ok(())
}

/// Helper function to display analysis results in a formatted way
fn display_analysis_summary(assistance: &GvpieDevelopmentAssistance) {
    println!("📊 Analysis Summary:");
    println!(
        "  Architecture: {:.1}%",
        assistance
            .analysis_report
            .architecture_analysis
            .modularity_score
            * 100.0
    );
    println!(
        "  GPU Performance: {:.1}%",
        assistance
            .analysis_report
            .gpu_analysis
            .gpu_utilization_score
            * 100.0
    );
    println!(
        "  VM Performance: {:.1}%",
        assistance
            .analysis_report
            .pixel_vm_analysis
            .vm_performance_score
            * 100.0
    );

    let high_priority_suggestions = assistance
        .analysis_report
        .optimization_suggestions
        .iter()
        .filter(|s| matches!(s.priority, ai_runtime::gvpie_analysis::Priority::High))
        .count();

    println!("  High Priority Issues: {}", high_priority_suggestions);
}
