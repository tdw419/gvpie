use gvpie_bootstrap::gpu::executor::GpuPixelMachine;

#[test]
fn test_gpu_executor_simple() {
    std::env::set_var("XDG_RUNTIME_DIR", "/tmp");
    pollster::block_on(async {
        let machine = GpuPixelMachine::new().await.unwrap();
        let result = machine.execute().await.unwrap();
        assert_eq!(result, 42);
    });
}
