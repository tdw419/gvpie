use gvpie_bootstrap::analysis::PplAnalysisOrchestrator;
use gvpie_bootstrap::pixel_language::executor::PixelInstruction;
use gvpie_bootstrap::pixel_language::ops::PixelOp;
use gvpie_bootstrap::pixel_language::programs;

#[test]
fn test_ppl_analysis_orchestrator() {
    pollster::block_on(async {
        // 1. Create the orchestrator
        let orchestrator = PplAnalysisOrchestrator::new().await.unwrap();

        // 2. Define the source program to be analyzed
        let source_program = vec![
            PixelInstruction { r: PixelOp::LoadSource as u32, g: 0, b: 0, a: 0 },
            PixelInstruction { r: PixelOp::Halt as u32, g: 0, b: 0, a: 0 },
            PixelInstruction { r: PixelOp::Halt as u32, g: 0, b: 0, a: 0 },
        ];

        // 3. Get the analyzer program
        let analyzer_program = programs::complexity_analyzer();

        // 4. Run the analysis
        let result = orchestrator.run_analysis(&analyzer_program, &source_program).await.unwrap();

        // 5. Assert that the result is correct
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 3); // The source program has 3 instructions
    });
}
