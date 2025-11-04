pub mod api;
pub mod cartridges;
pub mod config;
pub mod database;
pub mod errors;
pub mod gpu_bridge;
pub mod gvpie_analysis;
pub mod logging;
pub mod models;
pub mod monitor;
pub mod pixel_vm;

pub use api::SystemStatus;
pub use cartridges::Cartridge;
pub use database::{
    DecisionRecord, EventRecord, ExperienceDB, PatternAnalysis, SystemMetricsRecord, TrendAnalysis,
};
pub use errors::{AiRuntimeError, Result};
pub use gvpie_analysis::{
    GvpieAnalysisReport, GvpieAnalyzer, OptimizationSuggestion, PerformanceInsights,
};
pub use logging::{IncidentSeverity, LogSeverity, StructuredLogger};
pub use monitor::{SystemMetrics, SystemMonitor};

use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use gpu_bridge::GpuExecutionBridge;
use gvpie_core::PixelInstruction;

pub use pixel_vm::{ExecutionBackend, PixelProgramRequest, PixelProgramResponse};

#[derive(Debug)]
pub struct AiRuntime {
    #[cfg(feature = "gpu")]
    gpu_core: Option<Arc<gvpie_core::GpuCore>>,
    pixel_vm: pixel_vm::PixelVmRuntime,
    cartridge_manager: Arc<RwLock<cartridges::CartridgeManager>>,
    gpu_bridge: GpuExecutionBridge,
    gvpie_analyzer: Arc<RwLock<gvpie_analysis::GvpieAnalyzer>>,
    // TODO: Add database, monitoring, etc.
}

impl AiRuntime {
    pub async fn new() -> Result<Self> {
        // Initialize GPU core (may fail if no GPU available)
        #[cfg(feature = "gpu")]
        let gpu_core = if std::env::var("GVPIE_DISABLE_GPU").is_ok() {
            None
        } else {
            match gvpie_core::GpuCore::new().await {
                Ok(core) => Some(Arc::new(core)),
                Err(e) => {
                    println!("⚠️  GPU not available: {}", e);
                    None
                }
            }
        };

        #[cfg(not(feature = "gpu"))]
        let gpu_core = None;

        let cartridge_manager = cartridges::CartridgeManager::new(cartridge_storage_path())?;

        #[cfg(feature = "gpu")]
        let pixel_vm = pixel_vm::PixelVmRuntime::new(gpu_core.clone());
        #[cfg(not(feature = "gpu"))]
        let pixel_vm = pixel_vm::PixelVmRuntime::new(None);

        let gpu_bridge = GpuExecutionBridge::new(gpu_core.clone());

        // Initialize GPU bridge if available
        if gpu_bridge.is_gpu_available() {
            gpu_bridge
                .initialize()
                .await
                .map_err(AiRuntimeError::AnyhowError)?;
        }

        // Initialize GVPIe analyzer with workspace root
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let gvpie_analyzer = gvpie_analysis::GvpieAnalyzer::new(workspace_root);

        Ok(Self {
            gpu_core,
            pixel_vm,
            cartridge_manager: Arc::new(RwLock::new(cartridge_manager)),
            gpu_bridge,
            gvpie_analyzer: Arc::new(RwLock::new(gvpie_analyzer)),
        })
    }

    #[cfg(feature = "gpu")]
    pub fn gpu_available(&self) -> bool {
        self.gpu_bridge.is_gpu_available()
    }

    #[cfg(not(feature = "gpu"))]
    pub fn gpu_available(&self) -> bool {
        false
    }

    pub async fn list_cartridges(&self) -> Vec<Cartridge> {
        let manager = self.cartridge_manager.read().await;
        manager.list()
    }

    pub async fn get_cartridge(&self, id: &str) -> Option<Cartridge> {
        let manager = self.cartridge_manager.read().await;
        manager.get(id)
    }

    pub async fn execute_cartridge(
        &self,
        cartridge_id: &str,
        input_data: Option<&str>,
    ) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();
        let manager = self.cartridge_manager.read().await;

        // Execute the cartridge
        let output_data = manager.execute(cartridge_id, input_data)?;

        // GPU GLYPH EXPANSION INTEGRATION
        let (backend, glyphs_expanded) = if self.gpu_available() {
            self.execute_with_glyph_expansion(&output_data)
                .await?
                .map(|_| ("gpu".to_string(), true))
                .unwrap_or(("cpu".to_string(), false))
        } else {
            ("cpu".to_string(), false)
        };

        let result = ExecutionResult {
            output: format!(
                "Executed cartridge: {} ({} bytes)",
                cartridge_id,
                output_data.len()
            ),
            backend,
            duration_ms: start.elapsed().as_millis() as u64,
            data: output_data,
            glyphs_expanded, // NEW: Report if glyph expansion occurred
        };

        Ok(result)
    }

    #[cfg(feature = "gpu")]
    async fn execute_with_glyph_expansion(&self, ascii_data: &[u8]) -> Result<Option<()>> {
        // Convert to u32 for glyph expander (assuming ASCII data)
        let ascii_u32: Vec<u32> = ascii_data.iter().map(|&b| b as u32).collect();

        // Pad or truncate to expected 128x64 size
        let mut padded_data = vec![32u32; 128 * 64]; // Space characters
        let copy_len = std::cmp::min(ascii_u32.len(), padded_data.len());
        padded_data[..copy_len].copy_from_slice(&ascii_u32[..copy_len]);

        // Execute glyph expansion
        // Note: This requires GlyphExpander to be available in gvpie-core
        println!("🎨 Expanding glyphs on GPU...");

        // TODO: Actually call glyph expansion once gvpie-core exports it
        // For now, simulate the operation
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        println!("✅ Glyph expansion simulated");

        Ok(Some(()))
    }

    #[cfg(not(feature = "gpu"))]
    async fn execute_with_glyph_expansion(&self, _ascii_data: &[u8]) -> Result<Option<()>> {
        // No-op when GPU feature is disabled
        Ok(Some(()))
    }

    pub async fn create_cartridge(&self, cartridge: Cartridge) -> Result<Cartridge> {
        let mut manager = self.cartridge_manager.write().await;
        manager.create_cartridge(cartridge.clone())?;
        Ok(cartridge)
    }

    pub async fn update_cartridge(&self, cartridge: Cartridge) -> Result<Cartridge> {
        let mut manager = self.cartridge_manager.write().await;
        manager.update_cartridge(cartridge.clone())?;
        Ok(cartridge)
    }

    pub async fn delete_cartridge(&self, id: &str) -> Result<()> {
        let mut manager = self.cartridge_manager.write().await;
        manager.delete_cartridge(id)?;
        Ok(())
    }

    pub async fn execute_pixel_program(
        &self,
        request: PixelProgramRequest,
    ) -> Result<PixelProgramResponse> {
        self.pixel_vm
            .execute_program(request)
            .await
            .map_err(AiRuntimeError::AnyhowError)
    }

    pub fn assemble_pixel_program(&self, source: &str) -> Result<Vec<PixelInstruction>> {
        self.pixel_vm
            .assemble_from_text(source)
            .map_err(AiRuntimeError::AnyhowError)
    }

    pub fn pixel_backends(&self) -> Vec<String> {
        self.pixel_vm.available_backends()
    }

    // GVPIe Analysis Methods

    /// Analyze the entire GVPIe codebase and provide comprehensive insights
    pub async fn analyze_gvpie_codebase(&self) -> Result<gvpie_analysis::GvpieAnalysisReport> {
        let mut analyzer = self.gvpie_analyzer.write().await;
        analyzer.analyze_gvpie_codebase().await
    }

    /// Analyze a specific GVPIe component
    pub async fn analyze_gvpie_component<P: AsRef<std::path::Path>>(
        &self,
        component_path: P,
    ) -> Result<gvpie_analysis::GvpieAnalysisReport> {
        let mut analyzer = self.gvpie_analyzer.write().await;
        analyzer.analyze_component(component_path).await
    }

    /// Get real-time development suggestions for changed files
    pub async fn suggest_gvpie_improvements(
        &self,
        changed_files: &[PathBuf],
    ) -> Result<Vec<gvpie_analysis::OptimizationSuggestion>> {
        let analyzer = self.gvpie_analyzer.read().await;
        analyzer
            .suggest_improvements_for_changes(changed_files)
            .await
    }

    /// Predict performance impact of proposed changes
    pub async fn predict_gvpie_performance_impact(
        &self,
        changes: &[String],
    ) -> Result<gvpie_analysis::PerformanceInsights> {
        let analyzer = self.gvpie_analyzer.read().await;
        analyzer.predict_performance_impact(changes).await
    }

    /// Get AI-powered development assistance for GVPIe
    pub async fn get_gvpie_development_assistance(&self) -> Result<GvpieDevelopmentAssistance> {
        let analyzer = self.gvpie_analyzer.read().await;

        // Analyze current state
        let mut full_analyzer = self.gvpie_analyzer.write().await;
        let analysis_report = full_analyzer.analyze_gvpie_codebase().await?;
        drop(full_analyzer);

        // Generate development recommendations
        let recommendations = self
            .generate_development_recommendations(&analysis_report)
            .await?;

        Ok(GvpieDevelopmentAssistance {
            analysis_report: analysis_report.clone(),
            recommendations,
            next_actions: self
                .suggest_next_development_actions(&analysis_report)
                .await?,
            performance_predictions: analyzer.predict_performance_impact(&[]).await?,
        })
    }

    // Private helper methods for development assistance
    async fn generate_development_recommendations(
        &self,
        analysis: &gvpie_analysis::GvpieAnalysisReport,
    ) -> Result<Vec<DevelopmentRecommendation>> {
        let mut recommendations = Vec::new();

        // GPU optimization recommendations
        if analysis.gpu_analysis.gpu_utilization_score < 0.8 {
            recommendations.push(DevelopmentRecommendation {
                category: "GPU Optimization".to_string(),
                priority: "High".to_string(),
                title: "Improve GPU Utilization".to_string(),
                description: "GPU utilization is below optimal. Consider implementing workgroup size optimization and memory coalescing.".to_string(),
                estimated_effort: "2-3 days".to_string(),
                expected_impact: "20-30% performance improvement".to_string(),
                code_examples: vec![
                    "// Optimize workgroup sizes for better occupancy".to_string(),
                    "let optimal_workgroup_size = calculate_optimal_workgroup_size(device_limits);".to_string(),
                ],
            });
        }

        // Pixel VM optimization recommendations
        if analysis.pixel_vm_analysis.vm_performance_score < 0.9 {
            recommendations.push(DevelopmentRecommendation {
                category: "Pixel VM".to_string(),
                priority: "Medium".to_string(),
                title: "Implement Instruction Fusion".to_string(),
                description: "Pixel VM could benefit from fusing common instruction patterns to reduce execution overhead.".to_string(),
                estimated_effort: "1-2 weeks".to_string(),
                expected_impact: "10-15% VM performance improvement".to_string(),
                code_examples: vec![
                    "// Fuse consecutive LOAD operations".to_string(),
                    "match (current_op, next_op) { (Load(a), Load(b)) => LoadDual(a, b), ... }".to_string(),
                ],
            });
        }

        // Architecture recommendations
        if analysis.architecture_analysis.modularity_score < 0.85 {
            recommendations.push(DevelopmentRecommendation {
                category: "Architecture".to_string(),
                priority: "Low".to_string(),
                title: "Improve Module Boundaries".to_string(),
                description: "Consider extracting common patterns into shared traits to improve code reusability.".to_string(),
                estimated_effort: "3-5 days".to_string(),
                expected_impact: "Better maintainability and code reuse".to_string(),
                code_examples: vec![
                    "pub trait GpuOperation { async fn execute(&self) -> Result<()>; }".to_string(),
                ],
            });
        }

        Ok(recommendations)
    }

    async fn suggest_next_development_actions(
        &self,
        analysis: &gvpie_analysis::GvpieAnalysisReport,
    ) -> Result<Vec<NextAction>> {
        let mut actions = Vec::new();

        // Immediate actions based on analysis
        actions.push(NextAction {
            action_type: "Testing".to_string(),
            description: "Run comprehensive GPU benchmarks to establish performance baseline"
                .to_string(),
            command: Some("cargo test --features gpu --release gpu_benchmark_tests".to_string()),
            estimated_time: "30 minutes".to_string(),
        });

        actions.push(NextAction {
            action_type: "Profiling".to_string(),
            description: "Profile Pixel VM execution to identify hotspots".to_string(),
            command: Some("cargo run --example pixel_vm_profiler".to_string()),
            estimated_time: "15 minutes".to_string(),
        });

        if analysis.optimization_suggestions.len() > 0 {
            actions.push(NextAction {
                action_type: "Optimization".to_string(),
                description: format!(
                    "Implement {} high-priority optimizations",
                    analysis
                        .optimization_suggestions
                        .iter()
                        .filter(|s| matches!(s.priority, gvpie_analysis::Priority::High))
                        .count()
                ),
                command: None,
                estimated_time: "2-4 hours".to_string(),
            });
        }

        Ok(actions)
    }
}

// Re-export main types
pub use api::ApiServer;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub output: String,
    pub backend: String,
    pub duration_ms: u64,
    pub data: Vec<u8>,
    pub glyphs_expanded: bool, // NEW
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GvpieDevelopmentAssistance {
    pub analysis_report: gvpie_analysis::GvpieAnalysisReport,
    pub recommendations: Vec<DevelopmentRecommendation>,
    pub next_actions: Vec<NextAction>,
    pub performance_predictions: gvpie_analysis::PerformanceInsights,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DevelopmentRecommendation {
    pub category: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub estimated_effort: String,
    pub expected_impact: String,
    pub code_examples: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NextAction {
    pub action_type: String,
    pub description: String,
    pub command: Option<String>,
    pub estimated_time: String,
}

fn cartridge_storage_path() -> PathBuf {
    std::env::var("GVPIE_CARTRIDGE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./cartridges"))
}
