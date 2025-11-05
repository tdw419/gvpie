// GPU-Native PixelInstruction Analyzer
// This shader analyzes PixelInstruction patterns directly on GPU

struct AnalysisMetrics {
    total_instructions: atomic<u32>,
    unique_opcodes: atomic<u32>,
    complexity_score: atomic<u32>,
    optimization_opportunities: atomic<u32>,
}

@group(0) @binding(0) var<storage, read> pixel_code: array<u32>;  // RGBA pixels as u32
@group(0) @binding(1) var<storage, read_write> metrics: AnalysisMetrics;
@group(0) @binding(2) var<storage, read_write> opcode_histogram: array<atomic<u32>, 256>;

// Decode RGBA pixel into components
fn decode_pixel(pixel: u32) -> vec4<u32> {
    return vec4<u32>(
        (pixel >> 0u) & 0xFFu,   // R: primary opcode/data
        (pixel >> 8u) & 0xFFu,   // G: data
        (pixel >> 16u) & 0xFFu,  // B: data
        (pixel >> 24u) & 0xFFu   // A: data
    );
}

// Calculate instruction complexity
fn calculate_complexity(r: u32, g: u32, b: u32, a: u32) -> u32 {
    // Simple heuristic: number of non-zero components + value ranges
    var complexity = 0u;
    if (r != 0u) { complexity += 1u; }
    if (g != 0u) { complexity += 1u; }
    if (b != 0u) { complexity += 1u; }
    if (a != 0u) { complexity += 1u; }

    // Add complexity for large values (might be complex operations)
    if (r > 128u || g > 128u || b > 128u) {
        complexity += 1u;
    }

    return complexity;
}

// Detect optimization opportunities
fn is_optimizable(current: vec4<u32>, next: vec4<u32>) -> bool {
    // Example: detect repeated patterns
    if (current.r == next.r && current.g == next.g) {
        return true;  // Could be folded
    }

    // Detect no-ops (all zeros)
    if (current.r == 0u && current.g == 0u && current.b == 0u) {
        return true;  // Could be eliminated
    }

    return false;
}

@compute @workgroup_size(256)
fn analyze_pixel_patterns(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    let total_pixels = arrayLength(&pixel_code);

    if (idx >= total_pixels) {
        return;
    }

    // Decode current pixel instruction
    let current_pixel = pixel_code[idx];
    let current = decode_pixel(current_pixel);

    // Update metrics (atomic operations for thread safety)
    atomicAdd(&metrics.total_instructions, 1u);

    // Track opcode distribution
    atomicAdd(&opcode_histogram[current.r], 1u);

    // Calculate and accumulate complexity
    let complexity = calculate_complexity(current.r, current.g, current.b, current.a);
    atomicAdd(&metrics.complexity_score, complexity);

    // Check for optimization opportunities (with next instruction)
    if (idx + 1u < total_pixels) {
        let next_pixel = pixel_code[idx + 1u];
        let next = decode_pixel(next_pixel);

        if (is_optimizable(current, next)) {
            atomicAdd(&metrics.optimization_opportunities, 1u);
        }
    }
}

// Second pass: analyze unique opcodes (requires histogram from first pass)
@compute @workgroup_size(1)
fn count_unique_opcodes() {
    var unique_count = 0u;
    for (var i = 0u; i < 256u; i++) {
        if (atomicLoad(&opcode_histogram[i]) > 0u) {
            unique_count += 1u;
        }
    }
    atomicStore(&metrics.unique_opcodes, unique_count);
}
